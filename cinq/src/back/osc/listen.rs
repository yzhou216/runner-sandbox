// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Listening for OSC commands on a UDP port, one packet to a datagram
//!
//! A thread of its own receives the datagrams, since a blocking receive
//! must not take a worker of iced's executor, and hands each command to
//! the stream that [`listen`] returns.  A packet that is not OSC, and an
//! address that cinq does not know, go to standard error once for each
//! sender and address, until [`warn`] has written so many lines that it
//! ends them, so that a flood does not fill the terminal or the memory.

mod notes;
#[cfg(test)]
mod tests;

use std::{
    io,
    net::{IpAddr, SocketAddr, UdpSocket},
    thread,
    time::{Duration, Instant},
};

use iced::futures::{Stream, channel::mpsc};

use super::{
    incoming::{self, Message},
    remote::{Reading, Remote},
};
use crate::warn;
use notes::Notes;

/// Longest that the thread waits for a datagram before it looks whether
/// the stream has gone, which frees the port for the next setting
const LOOK: Duration = Duration::from_millis(250);

/// Most characters of an unknown address that a warning holds
const ADDRESS: usize = 64;

/// Time between tries to bind an address that failed
const RETRY: Duration = Duration::from_secs(2);

/// Time that the thread of the setting before may hold its port after the
/// stream has gone: one wait for a datagram and the time to see the stream
/// closed
const RELEASE: Duration = Duration::from_millis(500);

const DATAGRAM: usize = 65535;

/// Most messages that one datagram gives commands for.  A real remote sends
/// a few together, while a bundle of thousands fits in one datagram, and
/// each of its messages would fire GO again.
const MESSAGES: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Heard {
    Command(IpAddr, Remote),
    /// Packet or message from the host at the address that is no command,
    /// as the warnings tell it
    Ignored(String),
    /// Why the address could not be listened on, as the sidebar tells it
    Failed(String),
    /// Address that failed, and was told to fail, is listened on now
    Listening,
}

/// Returns the commands that arrive at `address`, or one failure when the
/// address cannot be bound.  The channel has no bound for commands, since the
/// interface takes them as fast as they come and a dropped GO would be worse
/// than the memory of a flood.  What was no command goes on at most once a
/// second, since the sidebar keeps one line of it.
pub fn listen(address: SocketAddr) -> impl Stream<Item = Heard> {
    let (sender, receiver) = mpsc::unbounded();
    thread::spawn(move || serve(address, &sender));
    receiver
}

fn serve(address: SocketAddr, sender: &mpsc::UnboundedSender<Heard>) {
    if let Some(socket) = bound(address, sender) {
        receive(&socket, sender);
    }
}

/// Returns the socket on `address`, trying again every [`RETRY`] while the
/// stream lasts, so that a port that another program frees later comes into
/// use without a change of the file.  The first failure is tried again after
/// [`RELEASE`] and not told, since the thread of the setting before may still
/// hold the port, as it does when the setting moves between two addresses of
/// one port.  A later failure is told once, and so is the success that
/// follows it.
fn bound(address: SocketAddr, sender: &mpsc::UnboundedSender<Heard>) -> Option<UdpSocket> {
    let bind = || {
        UdpSocket::bind(address).and_then(|socket| {
            socket.set_read_timeout(Some(LOOK))?;
            Ok(socket)
        })
    };
    let mut failures = 0_u32;
    while !sender.is_closed() {
        match bind() {
            Ok(socket) => {
                if failures > 1 {
                    tell(sender, Heard::Listening);
                }
                return Some(socket);
            }
            Err(error) => {
                failures += 1;
                complain(address, &error, failures, sender);
                thread::sleep(if failures == 1 { RELEASE } else { RETRY });
            }
        }
    }
    None
}

/// Tells of the failure to bind on `address` that is the `failures`th in a
/// row: standard error hears of every one after the first, and the sidebar
/// of the second
fn complain(
    address: SocketAddr,
    error: &io::Error,
    failures: u32,
    sender: &mpsc::UnboundedSender<Heard>,
) {
    if failures > 1 {
        warn::once(&format!(
            "osc_listen: cannot listen on {address}: {error}; trying again every {} seconds",
            RETRY.as_secs()
        ));
    }
    if failures == 2 {
        // The sidebar is narrow and its warnings lie below the lists, so
        // that the line leaves out the address, which the file holds, and
        // words the reason by the kind of the error, without the system's
        // text
        tell(
            sender,
            Heard::Failed(format!(
                "osc_listen: cannot listen, {}; retrying",
                reason(error)
            )),
        );
    }
}

fn tell(sender: &mpsc::UnboundedSender<Heard>, heard: Heard) {
    // Receiver has gone only when the subscription has
    let _ = sender.unbounded_send(heard);
}

fn receive(socket: &UdpSocket, sender: &mpsc::UnboundedSender<Heard>) {
    let mut buffer = vec![0; DATAGRAM];
    let notes = Notes::default();
    while !sender.is_closed() {
        match socket.recv_from(&mut buffer) {
            Ok((length, from)) => commands(&buffer[..length], from.ip())
                .into_iter()
                .filter(|heard| notes.allow(heard, Instant::now()))
                .for_each(|heard| tell(sender, heard)),
            // Error that returns at once would spin the loop
            Err(error) if pauses(&error) => thread::sleep(LOOK),
            // Read timeout, which ends each wait of the socket so that the
            // loop sees a closed stream, means no datagram
            Err(_) => {}
        }
    }
}

/// Returns what a datagram from `sender` gives, in order, a command for each
/// message that is one and a note for each part that is not, and warns of
/// the parts that are not
fn commands(datagram: &[u8], sender: IpAddr) -> Vec<Heard> {
    let ignored = |what: String| {
        let line = format!("OSC from {sender}: {what}");
        warn::once(&line);
        Heard::Ignored(line)
    };
    let Some(messages) = incoming::packet(datagram) else {
        return vec![ignored("not an OSC packet".to_owned())];
    };
    let dropped = (messages.len() > MESSAGES)
        .then(|| ignored(format!("more than {MESSAGES} messages, the rest dropped")));
    messages
        .iter()
        .take(MESSAGES)
        .filter_map(|message| match message.reading() {
            Reading::Command(remote) => Some(Heard::Command(sender, remote)),
            Reading::Released => None,
            Reading::Unknown => Some(ignored(format!("unknown {}", address(message)))),
        })
        .chain(dropped)
        .collect()
}

/// Returns the address of a message as the terminal may show it, quoted
/// with control characters escaped, and cut short, since anyone on the
/// network can send it
fn address(message: &Message) -> String {
    let shown: String = message.address.chars().take(ADDRESS).collect();
    format!("address {shown:?}")
}

/// Returns the reason of a failed bind in words, the kind of the error when
/// it has a plain one and the text of the system otherwise, as a kind such
/// as "uncategorized error" tells nothing
fn reason(error: &io::Error) -> String {
    match error.kind() {
        kind @ (io::ErrorKind::AddrInUse
        | io::ErrorKind::AddrNotAvailable
        | io::ErrorKind::PermissionDenied) => kind.to_string(),
        _ => warn::words(error),
    }
}

/// Returns whether a failed receive may repeat at once, as any error may but
/// a timeout, which has waited already, and a refusal, which the system
/// reports only for a datagram that this socket sent, so that a listener
/// that sends nothing never gets it, and which would delay the next command
/// if waited out
fn pauses(error: &io::Error) -> bool {
    !matches!(
        error.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock | io::ErrorKind::ConnectionRefused
    )
}
