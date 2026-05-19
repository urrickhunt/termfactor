use std::io::{self, Read};

#[cfg(unix)]
pub fn read_single_key() -> io::Result<char> {
    use nix::sys::termios::{
        tcgetattr, tcsetattr, LocalFlags, SetArg, SpecialCharacterIndices, Termios,
    };
    use std::os::fd::{AsFd, AsRawFd, BorrowedFd};

    struct TermiosGuard {
        file_descriptor: i32,
        original: Termios,
    }

    impl Drop for TermiosGuard {
        fn drop(&mut self) {
            let _ = tcsetattr(
                unsafe { BorrowedFd::borrow_raw(self.file_descriptor) },
                SetArg::TCSANOW,
                &self.original,
            );
        }
    }

    fn as_io_error(error: nix::Error) -> io::Error {
        io::Error::other(error)
    }

    fn consume_escape_sequence(
        file_descriptor: BorrowedFd<'_>,
        stdin: &mut impl Read,
        raw_mode: &Termios,
    ) -> io::Result<bool> {
        let mut timed_raw_mode = raw_mode.clone();
        timed_raw_mode.control_chars[SpecialCharacterIndices::VMIN as usize] = 0;
        timed_raw_mode.control_chars[SpecialCharacterIndices::VTIME as usize] = 1;
        tcsetattr(file_descriptor, SetArg::TCSANOW, &timed_raw_mode).map_err(as_io_error)?;

        let mut buffer = [0_u8; 1];
        if stdin.read(&mut buffer)? == 0 {
            tcsetattr(file_descriptor, SetArg::TCSANOW, raw_mode).map_err(as_io_error)?;
            return Ok(false);
        }

        while stdin.read(&mut buffer)? != 0 {}

        tcsetattr(file_descriptor, SetArg::TCSANOW, raw_mode).map_err(as_io_error)?;

        Ok(true)
    }

    let stdin = io::stdin();
    let file_descriptor = stdin.as_fd();
    let original = tcgetattr(file_descriptor).map_err(as_io_error)?;
    let guard = TermiosGuard {
        file_descriptor: file_descriptor.as_raw_fd(),
        original: original.clone(),
    };

    let mut raw = original;
    raw.local_flags
        .remove(LocalFlags::ICANON | LocalFlags::ECHO | LocalFlags::ISIG);
    raw.control_chars[SpecialCharacterIndices::VMIN as usize] = 1;
    raw.control_chars[SpecialCharacterIndices::VTIME as usize] = 0;
    tcsetattr(file_descriptor, SetArg::TCSANOW, &raw).map_err(as_io_error)?;

    let mut buffer = [0_u8; 1];
    {
        let mut stdin_lock = stdin.lock();
        stdin_lock.read_exact(&mut buffer)?;

        if buffer[0] == b'\x1b' && consume_escape_sequence(file_descriptor, &mut stdin_lock, &raw)?
        {
            drop(stdin_lock);
            drop(guard);
            return Ok('\0');
        }
    }

    drop(guard);

    Ok(char::from(buffer[0]))
}

#[cfg(windows)]
pub fn read_single_key() -> io::Result<char> {
    use winapi::ctypes::c_void;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_INPUT_HANDLE;
    use winapi::um::wincon::{
        ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT, ENABLE_VIRTUAL_TERMINAL_INPUT,
    };

    struct ConsoleModeGuard {
        handle: *mut c_void,
        original_mode: DWORD,
    }

    impl Drop for ConsoleModeGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = SetConsoleMode(self.handle, self.original_mode);
            }
        }
    }

    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        let mut mode: DWORD = 0;
        if GetConsoleMode(handle, std::ptr::addr_of_mut!(mode)) == 0 {
            return Err(io::Error::last_os_error());
        }

        let guard = ConsoleModeGuard {
            handle,
            original_mode: mode,
        };

        let raw_mode = mode
            & !(ENABLE_ECHO_INPUT
                | ENABLE_LINE_INPUT
                | ENABLE_PROCESSED_INPUT
                | ENABLE_VIRTUAL_TERMINAL_INPUT);
        if SetConsoleMode(handle, raw_mode) == 0 {
            return Err(io::Error::last_os_error());
        }

        let mut buffer = [0_u8; 1];
        io::stdin().read_exact(&mut buffer)?;
        drop(guard);

        Ok(char::from(buffer[0]))
    }
}
