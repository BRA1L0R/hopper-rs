use bytes::BufMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    select,
};

#[cfg(feature = "zerocopy")]
mod linux;

use crate::protocol::connection::{Connection, ConnectionError};

async fn flush(into: &mut Connection, from: &mut Connection) -> Result<(), ConnectionError> {
    into.write_buffer().put(from.read_buffer());
    into.flush().await
}

pub async fn flush_bidirectional(
    mut client: Connection,
    mut server: Connection,
) -> Result<(TcpStream, TcpStream), ConnectionError> {
    flush(&mut client, &mut server).await?;
    flush(&mut server, &mut client).await?;

    Ok((client.into_socket(), server.into_socket()))
}

/// Uses an external transferred counter so in an event of an error or
/// when the future gets dropped by the select data still gets recorded
#[cfg(not(feature = "zerocopy"))]
async fn pipe(mut input: OwnedReadHalf, mut output: OwnedWriteHalf, transferred: &mut u64) {
    // Accomodate the average MTU of tcp connections
    let mut buffer = [0u8; 2048];

    // read from the socket into the buffer, increment the transfer counter
    // and then write all to the other end of the pipe
    loop {
        let size = match input.read(&mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(p) => p,
        };

        *transferred += size as u64; // always safe doing

        if output.write_all(&buffer[..size]).await.is_err() {
            break;
        }
    }
}

#[cfg(all(feature = "zerocopy", not(target_family = "unix")))]
compile_error!("feature zerocopy is only supported on unix systems");

#[cfg(feature = "zerocopy")]
async fn pipe(
    mut input: OwnedReadHalf,
    mut output: OwnedWriteHalf,
    transferred: &mut u64,
) -> std::io::Result<()> {
    use std::{io::ErrorKind, os::fd::AsFd};

    let mut pipe = linux::Pipe::new().unwrap();
    let mut exitflag = false;

    macro_rules! splice_all {
        ($exitflag:tt, $splice:expr) => {
            match $splice {
                Ok(bytes) => bytes,
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => {
                    $exitflag = true;
                    break;
                }
            }
        };
    }

    loop {
        let _ = input.read(&mut []).await?;
        loop {
            splice_all!(exitflag, pipe.splice_into(input.as_ref().as_fd()));
        }

        let _ = output.write(&[]).await?;
        loop {
            let bytes = splice_all!(exitflag, pipe.splice_out(output.as_ref().as_fd()));
            *transferred += bytes as u64;
        }

        if exitflag {
            break Ok(());
        }
    }
}

/// returns the number of bytes transferred
/// `(serverbound, clientbound)`
pub async fn copy_bidirectional(server: TcpStream, client: TcpStream) -> (u64, u64) {
    let mut serverbound = 0;
    let mut clientbound = 0;

    // nagle algo causes ping delay
    server.set_nodelay(true).unwrap();
    client.set_nodelay(true).unwrap();

    let (rs, ws) = server.into_split();
    let (rc, wc) = client.into_split();

    // select ensures that when one pipe finishes
    // the other one gets dropped. Fixes socket leak
    // which kept sockets in a WAIT state forever
    select! {
        _ = pipe(rs, wc, &mut clientbound) => {}
        _ = pipe(rc, ws, &mut serverbound) => {},
    };

    (serverbound, clientbound)
}
