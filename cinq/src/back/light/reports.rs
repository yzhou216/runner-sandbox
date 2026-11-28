// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The reports of the thread, as a stream that wakes the window
//!
//! The thread tells each change in whether light gets out, and the window
//! has nothing to look at in between, so that a subscription hands each
//! report over as a message instead of the window asking on a timer.

use std::{
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use iced::{
    Subscription,
    futures::{
        Stream, StreamExt,
        channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
        stream,
    },
};

use crate::lock::lock;

/// Sender of the thread, which tells why no light gets out, or that some
/// does
pub(super) type Reporter = UnboundedSender<Option<String>>;

/// Receiving end of the reports, which the subscription takes once
#[derive(Clone)]
pub struct Reports(Arc<Mutex<Option<UnboundedReceiver<Option<String>>>>>);

impl Reports {
    pub(super) fn open() -> (Reporter, Self) {
        let (reporter, receiver) = mpsc::unbounded();
        (reporter, Self(Arc::new(Mutex::new(Some(receiver)))))
    }

    /// Returns the reports as a subscription, which stays the same for as
    /// long as the output lasts
    pub fn subscription(&self) -> Subscription<Option<String>> {
        Subscription::run_with(self.clone(), Reports::stream)
    }

    /// Returns the reports, which only the first call gets, since the
    /// channel has one receiver
    fn stream(&self) -> impl Stream<Item = Option<String>> + use<> {
        let receiver = lock(&self.0).take();
        stream::iter(receiver).flatten()
    }
}

/// Identifies the stream by the channel behind it
impl Hash for Reports {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use iced::futures::{StreamExt, executor::block_on};

    use super::Reports;

    #[test]
    fn the_reports_arrive_in_order_through_the_stream_that_takes_the_receiver_once() {
        let (reporter, reports) = Reports::open();
        let told = [Some("no network interface".to_owned()), None];
        told.iter().for_each(|report| {
            reporter
                .unbounded_send(report.clone())
                .expect("receiver is open")
        });
        drop(reporter);
        assert_eq!(block_on(reports.stream().collect::<Vec<_>>()), told);
        assert!(block_on(reports.stream().collect::<Vec<_>>()).is_empty());
    }
}
