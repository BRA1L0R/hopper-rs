use bytes::BufMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    select,
};

#[cfg(zerocopy)]
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
#[cfg(not(zerocopy))]
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

#[cfg(zerocopy)]
async fn pipe(
    mut input: OwnedReadHalf,
    mut output: OwnedWriteHalf,
    transferred: &mut u64,
) -> std::io::Result<()> {
    use std::os::fd::AsFd;

    let mut pipe = linux::Pipe::new().unwrap();

    loop {
        let _ = input.read(&mut []).await?;
        while pipe.splice_into(input.as_ref().as_fd()).is_ok() {}

        let _ = output.write(&[]).await?;
        while pipe.splice_out(output.as_ref().as_fd()).is_ok() {}
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
