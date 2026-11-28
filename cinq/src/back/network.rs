// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Network output, where the back end has side effects
//!
//! [`Network::send`] hands the messages of network cues to a thread of its
//! own, which passes each to a worker thread of its destination.  A worker
//! sends its messages in order over a TCP connection that lasts through a
//! burst, as [`connection`] tells, or as one UDP datagram each to the
//! address that the host name resolved to at the start of the burst.
//! Messages to different destinations therefore do not wait for each other,
//! and the host name of a destination is resolved on its own worker.
//!
//! A message that cannot go out within [`PATIENCE`] of being handed over,
//! as one behind a console that does not answer, is dropped rather than
//! sent late, since a console cue that fires late does more harm than one
//! that does not fire.  Dropping the network waits for the messages on their
//! way, so that a cue fired as the window closes still goes out.  Why a
//! message to a destination last failed stays for the interface to show until
//! a message gets there or no cue names the destination any more, and warns
//! once on standard error.

mod address;
mod connection;
#[cfg(test)]
mod tests;

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, ErrorKind},
    net::{SocketAddr, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant},
};

use super::osc::{self, Destination, Outgoing};
use crate::{lock::lock, warn};
use address::{ipv4_address, udp_socket};

/// Longest that a message may wait to go out, and longest quiet that a TCP
/// connection or the resolved address of a UDP host outlasts
const PATIENCE: Duration = Duration::from_secs(1);

const CHECK: Duration = Duration::from_millis(100);

#[derive(Default)]
struct Shared {
    /// Why the last message to each destination failed, in the words of
    /// [`warn::words`], until one gets there
    problems: Mutex<BTreeMap<String, String>>,
    pending: AtomicUsize,
}

pub struct Network {
    sender: mpsc::Sender<Vec<(Instant, Outgoing)>>,
    shared: Arc<Shared>,
}

impl Network {
    pub fn open() -> Self {
        let (sender, messages) = mpsc::channel();
        let shared = Arc::<Shared>::default();
        let sending = Arc::clone(&shared);
        thread::spawn(move || dispatch(&messages, &sending));
        Self { sender, shared }
    }

    pub fn send(&self, messages: Vec<Outgoing>) {
        self.send_since(messages, Instant::now());
    }

    /// Hands over `messages` as though at `handed`, for tests of lateness
    fn send_since(&self, messages: Vec<Outgoing>, handed: Instant) {
        if !messages.is_empty() {
            self.shared
                .pending
                .fetch_add(messages.len(), Ordering::Relaxed);
            let stamped = messages
                .into_iter()
                .map(|message| (handed, message))
                .collect();
            // Fails only once the thread has gone, which a panic there causes
            let _ = self.sender.send(stamped);
        }
    }

    /// Returns why messages to the `named` destinations last failed, as
    /// `message not sent: connection refused`, each with the value that
    /// `named` holds for its destination, such as the cue that names it, whose
    /// inspector shows the destination.  A destination that no cue names any
    /// more is forgotten, since fixing a typo in a cue would otherwise leave
    /// the warning of the old one.
    pub fn problems<T: Copy>(&self, named: &BTreeMap<String, T>) -> Vec<(T, String)> {
        let mut problems = lock(&self.shared.problems);
        problems.retain(|destination, _| named.contains_key(destination));
        problems
            .iter()
            .filter_map(|(destination, reason)| {
                named
                    .get(destination)
                    .map(|&named| (named, format!("message not sent: {reason}")))
            })
            .collect()
    }

    /// Returns the `named` destinations that messages last failed to reach,
    /// so that each cue that names one can tell so on its own row
    pub fn failed<T>(&self, named: &BTreeMap<String, T>) -> BTreeSet<String> {
        lock(&self.shared.problems)
            .keys()
            .filter(|destination| named.contains_key(*destination))
            .cloned()
            .collect()
    }

    /// Returns how often the interface should look again while messages are
    /// on their way, since they can change the problems
    pub fn checks(&self) -> Option<Duration> {
        (self.shared.pending.load(Ordering::Relaxed) > 0).then_some(CHECK)
    }
}

impl Drop for Network {
    /// Waits for the messages that are on their way, for [`PATIENCE`] at
    /// most, which is as long as any of them may take, so that a network cue
    /// fired just before the window closes still goes out
    fn drop(&mut self) {
        let end = Instant::now() + PATIENCE;
        while self.shared.pending.load(Ordering::Relaxed) > 0 && Instant::now() < end {
            thread::sleep(Duration::from_millis(5));
        }
    }
}

/// Queue into the worker of one destination, with each message as handed
/// over
type Queue = mpsc::Sender<(Instant, Outgoing)>;

/// What a worker keeps of its destination, with when it last took a message
type Kept = (Instant, Link);

/// Open TCP connection of a destination, or the address that the host name
/// of a UDP destination resolved to, which a burst of datagrams then looks
/// up once instead of once each, since a name such as one ending in `.local`
/// can take longer to resolve than the messages may wait
enum Link {
    Tcp(TcpStream),
    Udp(SocketAddr),
}

/// Hands each message to the worker of its destination, which it starts at
/// the first message, so that a destination that does not answer cannot
/// delay the others.  A worker whose destination has been quiet for
/// [`PATIENCE`] is let go: it delivers what is queued, ends, and closes its
/// connection.
fn dispatch(messages: &mpsc::Receiver<Vec<(Instant, Outgoing)>>, shared: &Arc<Shared>) {
    // Each worker's queue by destination, with when it last took a message
    let mut workers = BTreeMap::<String, (Instant, Queue)>::new();
    loop {
        // Without a worker to let go, only a message needs a wake
        let received = if workers.is_empty() {
            messages.recv().map_err(|_| RecvTimeoutError::Disconnected)
        } else {
            messages.recv_timeout(PATIENCE)
        };
        let batch = match received {
            Ok(batch) => batch,
            Err(RecvTimeoutError::Timeout) => Vec::new(),
            Err(RecvTimeoutError::Disconnected) => return,
        };
        for (handed, outgoing) in batch {
            let destination = outgoing.destination.to_string();
            let queue = workers
                .remove(&destination)
                .map_or_else(|| started(&destination, shared), |(_, queue)| queue);
            // Fails only once the worker has gone, which a panic there causes
            let _ = queue.send((handed, outgoing));
            workers.insert(destination, (Instant::now(), queue));
        }
        workers.retain(|_, (used, _)| used.elapsed() <= PATIENCE);
    }
}

fn started(destination: &str, shared: &Arc<Shared>) -> Queue {
    let (queue, messages) = mpsc::channel();
    let destination = destination.to_owned();
    let shared = Arc::clone(shared);
    thread::spawn(move || work(&destination, &messages, &shared));
    queue
}

/// Delivers the messages to `destination` in order until its queue ends,
/// which closes the connection
fn work(destination: &str, messages: &mpsc::Receiver<(Instant, Outgoing)>, shared: &Shared) {
    messages.iter().fold(None, |kept, (handed, outgoing)| {
        // Burst ends after a quiet second, whenever the dispatcher notices
        let fresh = kept.filter(|(used, _): &Kept| used.elapsed() <= PATIENCE);
        let kept = match deliver(fresh, outgoing, handed, ipv4_address) {
            Ok(kept) => {
                drop(lock(&shared.problems).remove(destination));
                kept
            }
            Err(error) => {
                warn::once(&format!("network cue to {destination}: {error}"));
                lock(&shared.problems).insert(destination.to_owned(), warn::words(&error));
                None
            }
        };
        shared.pending.fetch_sub(1, Ordering::Relaxed);
        kept
    });
}

/// Returns what the worker keeps of the destination after the message, if
/// anything, with `resolve` finding the address of a UDP host that it does
/// not keep
fn deliver(
    kept: Option<Kept>,
    Outgoing {
        destination,
        packet,
    }: Outgoing,
    handed: Instant,
    resolve: impl FnOnce(&str) -> io::Result<SocketAddr>,
) -> io::Result<Option<Kept>> {
    let link = kept.map(|(_, link)| link);
    match destination {
        Destination::Udp(address) => {
            time_left(handed)?;
            let to = match link {
                Some(Link::Udp(to)) => to,
                _ => resolve(&address)?,
            };
            // Finding the host may have taken the rest of the time
            time_left(handed)?;
            udp_socket()?.send_to(&packet, to)?;
            Ok(Some((Instant::now(), Link::Udp(to))))
        }
        Destination::Tcp(address) => {
            let stream = link.and_then(|link| match link {
                Link::Tcp(stream) => Some(stream),
                Link::Udp(_) => None,
            });
            connection::written(stream, &address, &osc::framed(&packet), handed)
                .map(|stream| Some((Instant::now(), Link::Tcp(stream))))
        }
    }
}

fn time_left(handed: Instant) -> io::Result<Duration> {
    PATIENCE
        .checked_sub(handed.elapsed())
        .filter(|left| !left.is_zero())
        .ok_or_else(|| io::Error::new(ErrorKind::TimedOut, "more than a second late"))
}
