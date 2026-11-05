use std::io;
use std::os::fd::OwnedFd;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

use rustix::event::{poll, PollFd, PollFlags};
use rustix::fs::{open, Mode, OFlags};
use rustix::io::Errno;
use rustix::process::{ioctl_tiocsctty, setsid};
use rustix::pty::{grantpt, openpt, ptsname, unlockpt, OpenptFlags};
use rustix::termios::{
    tcgetattr, tcsetattr, tcsetwinsize, OptionalActions, Termios, Winsize,
};

struct Pty {
    master: OwnedFd,
    child: Child,
}

impl Pty {
    fn spawn(rows: u16, cols: u16) -> io::Result<Self> {
        let master = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY | OpenptFlags::CLOEXEC)?;
        grantpt(&master)?;
        unlockpt(&master)?;
        tcsetwinsize(&master, Self::winsize(rows, cols))?;

        let name = ptsname(&master, Vec::new())?;
        let slave = open(name.as_c_str(), OFlags::RDWR | OFlags::NOCTTY, Mode::empty())?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let mut cmd = Command::new(shell);
        cmd.env("TERM", "xterm-256color")
            .stdin(Stdio::from(slave.try_clone()?))
            .stdout(Stdio::from(slave.try_clone()?))
            .stderr(Stdio::from(slave));

        unsafe {
            cmd.pre_exec(|| {
                setsid()?;
                ioctl_tiocsctty(rustix::stdio::stdin())?;
                Ok(())
            });
        }
        let child = cmd.spawn()?;
        Ok(Self { master, child })
    }

    fn winsize(rows: u16, cols: u16) -> Winsize {
        Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 }
    }

    fn resize(&self, rows: u16, cols: u16) -> io::Result<()> {
        Ok(tcsetwinsize(&self.master, Self::winsize(rows, cols))?)
    }

    fn write_all(&self, mut data: &[u8]) -> io::Result<()> {
        while !data.is_empty() {
            match rustix::io::write(&self.master, data) {
                Ok(n) => data = &data[n..],
                Err(Errno::INTR) => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
}

struct RawMode(Termios);

impl RawMode {
    fn enable() -> io::Result<Self> {
        let stdin = rustix::stdio::stdin();
        let orig = tcgetattr(stdin)?;
        let mut raw = orig.clone();
        raw.make_raw();
        tcsetattr(stdin, OptionalActions::Now, &raw)?;
        Ok(Self(orig))
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = tcsetattr(rustix::stdio::stdin(), OptionalActions::Now, &self.0);
    }
}

fn main() -> io::Result<()> {
    let mut pty = Pty::spawn(24, 80)?;
    let _raw = RawMode::enable()?;
    let stdin = rustix::stdio::stdin();
    let stdout = rustix::stdio::stdout();
    let mut buf = [0u8; 4096];

    'main: loop {
        let mut fds = [
            PollFd::new(&stdin, PollFlags::IN),
            PollFd::new(&pty.master, PollFlags::IN),
        ];
        match poll(&mut fds, None) {
            Ok(_) => {}
            Err(Errno::INTR) => continue,
            Err(e) => return Err(e.into()),
        }
        let in_ev = fds[0].revents();
        let pty_ev = fds[1].revents();

        if in_ev.contains(PollFlags::IN) {
            match rustix::io::read(stdin, &mut buf) {
                Ok(0) => break 'main,
                Ok(n) => {
                    if buf[..n].contains(&0x1d) {
                        break 'main;
                    }
                    pty.write_all(&buf[..n])?;
                }
                Err(Errno::INTR) => {}
                Err(e) => return Err(e.into()),
            }
        }

        if pty_ev.intersects(PollFlags::IN | PollFlags::HUP) {
            match rustix::io::read(&pty.master, &mut buf) {
                Ok(0) => break 'main,
                Ok(n) => {
                    eprintln!("pty: {n} bytes\r");
                    let mut out = &buf[..n];
                    while !out.is_empty() {
                        match rustix::io::write(stdout, out) {
                            Ok(w) => out = &out[w..],
                            Err(Errno::INTR) => {}
                            Err(e) => return Err(e.into()),
                        }
                    }
                }
                Err(Errno::INTR) => {}
                Err(Errno::IO) => break 'main,
                Err(e) => return Err(e.into()),
            }
        }
    }

    let _ = pty.child.kill();
    pty.child.wait()?;
    Ok(())
}