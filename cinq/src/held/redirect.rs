// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Standard output and error pointed at a pipe, so that what libraries write
//! to them is held too
//!
//! A thread reads the pipe into the buffer, so that no writer waits on a
//! full pipe.  Standard output follows standard error only when it is a
//! terminal, since one that is a file or a pipe takes no part in the screen
//! that the problems switch to.  Both ends of each pipe stay out of the
//! processes that cinq starts.

use std::{
    fs::File,
    io::{self, IsTerminal, Write},
    iter,
    os::fd::{AsFd, OwnedFd},
    thread::{self, JoinHandle},
};

use rustix::{
    event::{PollFd, PollFlags, poll},
    io::{FdFlags, fcntl_setfd, read, retry_on_intr},
    pipe::pipe,
    stdio::{dup2_stderr, dup2_stdout},
};

/// Bytes that the thread takes from the pipe at most at once
const CHUNK: usize = 4096;

pub struct Redirect {
    /// Standard error as it was, a terminal
    error: OwnedFd,
    /// Standard output as it was, when it is a terminal as well
    output: Option<OwnedFd>,
    /// Write end of the pipe that ends the thread once it closes
    stop: OwnedFd,
    reader: JoinHandle<()>,
}

impl Redirect {
    /// Points standard error, and standard output when it is a terminal, at
    /// a pipe whose bytes the thread hands to `sink`
    pub fn start(sink: fn(&[u8])) -> io::Result<Self> {
        // Bytes that standard output buffered belong to the terminal
        let _ = io::stdout().flush();
        let error = io::stderr().as_fd().try_clone_to_owned()?;
        let output = io::stdout()
            .is_terminal()
            .then(|| io::stdout().as_fd().try_clone_to_owned())
            .transpose()?;
        let (data, written) = piped()?;
        let (halt, stop) = piped()?;
        let reader = thread::Builder::new()
            .name("held".to_owned())
            .spawn(move || iter::from_fn(|| next(&data, &halt)).for_each(|bytes| sink(&bytes)))?;
        let redirect = Self {
            error,
            output,
            stop,
            reader,
        };
        redirect
            .pointed_at(&written)
            .inspect_err(|_| redirect.restored())
            .map(|()| redirect)
    }

    /// Puts the descriptors back, and waits for the thread to hand over what
    /// the pipe still holds
    pub fn stop(self) {
        // Bytes that standard output buffered meanwhile belong to the buffer
        let _ = io::stdout().flush();
        self.restored();
        // Ends the thread even when something still holds a write end of
        // the pipe of data, such as a process that cinq started meanwhile
        drop(self.stop);
        let _ = self.reader.join();
    }

    /// Returns standard error as it was
    pub fn terminal(&self) -> io::Result<File> {
        self.error.try_clone().map(File::from)
    }

    fn pointed_at(&self, pipe: &OwnedFd) -> io::Result<()> {
        dup2_stderr(pipe)?;
        self.output.as_ref().map_or(Ok(()), |_| dup2_stdout(pipe))?;
        Ok(())
    }

    fn restored(&self) {
        let _ = dup2_stderr(&self.error);
        let _ = self.output.as_ref().map(dup2_stdout);
    }
}

/// Returns the read and write ends of a pipe
fn piped() -> io::Result<(OwnedFd, OwnedFd)> {
    let (read, write) = pipe()?;
    fcntl_setfd(&read, FdFlags::CLOEXEC)?;
    fcntl_setfd(&write, FdFlags::CLOEXEC)?;
    Ok((read, write))
}

/// Waits for bytes on `data` and returns up to a chunk of them, or returns
/// nothing at the end: when every writer has closed it, or when `halt`
/// closes and no bytes are left
fn next(data: &OwnedFd, halt: &OwnedFd) -> Option<Vec<u8>> {
    let mut fds = [
        PollFd::new(data, PollFlags::IN),
        PollFd::new(halt, PollFlags::IN),
    ];
    retry_on_intr(|| poll(&mut fds, None)).ok()?;
    if fds[0].revents().is_empty() {
        return None;
    }
    let mut chunk = [0; CHUNK];
    let size = retry_on_intr(|| read(data, &mut chunk)).ok()?;
    (size > 0).then(|| chunk[..size].to_vec())
}
