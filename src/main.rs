use std::ffi::CString;
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut master: libc::c_int = 0;
    let mut slave: libc::c_int = 0;

    let res = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    if res < 0 {
        return Err("Failed to open PTY via libc::openpty".into());
    }

    let master_file = unsafe { std::fs::File::from_raw_fd(master) };
    let slave_fd = slave;

    let pid = unsafe { libc::fork() };
    if pid < 0 {
        return Err("Failed to fork process".into());
    }

    if pid == 0 {
        drop(master_file);

        unsafe {
            libc::setsid();
            if libc::login_tty(slave_fd) < 0 {
                libc::_exit(1);
            }
        }

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let c_shell = CString::new(shell)?;
        let c_term = CString::new("TERM=xterm-256color")?;
        let args = [c_shell.clone(), CString::new("-i")?];
        let envs = [c_term];

        let c_args: Vec<*const libc::c_char> = args.iter().map(|arg| arg.as_ptr()).chain(std::iter::once(std::ptr::null())).collect();
        let c_envs: Vec<*const libc::c_char> = envs.iter().map(|env| env.as_ptr()).chain(std::iter::once(std::ptr::null())).collect();

        unsafe {
            libc::execve(c_shell.as_ptr(), c_args.as_ptr(), c_envs.as_ptr());
        }

        process::exit(1);
    } else {
        unsafe {
            libc::close(slave_fd);
        }

        println!("[Parent] PTY successfully opened. Listening for shell output...\n");

        let mut buf = [0u8; 1024];
        let mut master_writer = master_file.try_clone()?;

        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = master_writer.write_all(b"echo 'Hello from ts-term via PTY!'\n");
            let _ = master_writer.write_all(b"ls -la\n");
        });

        let mut master_reader = master_file;

        loop {
            match master_reader.read(&mut buf) {
                Ok(0) => {
                    println!("\n[Parent] EOF reached (shell terminated).");
                    break;
                }
                Ok(n) => {
                    std::io::stdout().write_all(&buf[..n])?;
                    std::io::stdout().flush()?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("\n[Read Error]: {e}");
                    break;
                }
            }
        }
    }

    Ok(())
}