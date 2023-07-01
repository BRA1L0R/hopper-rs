use std::{
    io::{Error, ErrorKind},
    mem::MaybeUninit,
    os::fd::{AsRawFd, RawFd},
};

use libc::{O_NONBLOCK, SPLICE_F_MORE, SPLICE_F_NONBLOCK};

#[repr(C)]
pub struct Pipe {
    read: RawFd,
    write: RawFd,
}

fn os_error(result: isize) -> std::io::Result<usize> {
    match result {
        0.. => Ok(result as usize),
        _ => Err(Error::last_os_error()),
    }
}

impl Pipe {
    const FLAGS: u32 = SPLICE_F_NONBLOCK;

    pub fn new() -> std::io::Result<Pipe> {
        let mut pipes = MaybeUninit::<Self>::uninit();

        if unsafe { libc::pipe2(pipes.as_mut_ptr() as _, O_NONBLOCK) } < 0 {
            return Err(Error::last_os_error());
        };

        Ok(unsafe { pipes.assume_init() })
    }

    pub fn splice_into<I: AsRawFd>(&mut self, input: I) -> std::io::Result<usize> {
        let result = unsafe {
            libc::splice(
                input.as_raw_fd(),
                core::ptr::null_mut(),
                self.write,
                core::ptr::null_mut(),
                65535,
                Self::FLAGS,
            )
        };

        let result = os_error(result)?;

        match result {
            0 => Err(ErrorKind::UnexpectedEof.into()),
            v => Ok(v),
        }
    }

    pub fn splice_out<O: AsRawFd>(&mut self, output: O) -> std::io::Result<usize> {
        let result = unsafe {
            libc::splice(
                self.read,
                core::ptr::null_mut(),
                output.as_raw_fd(),
                core::ptr::null_mut(),
                65535,
                Self::FLAGS,
            )
        };

        let result = os_error(result)?;

        match result {
            0 => Err(ErrorKind::UnexpectedEof.into()),
            v => Ok(v),
        }
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        unsafe {
            let read_res = libc::close(self.read);
            let write_res = libc::close(self.write);

            debug_assert!(read_res >= 0 && write_res >= 0);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Pipe;

    #[test]
    fn new() {
        let pipes = Pipe::new().unwrap();
        assert!(pipes.read != 0);
        assert!(pipes.write != 0);
    }
}
