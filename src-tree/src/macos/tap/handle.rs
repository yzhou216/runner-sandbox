// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Handling of a single scroll event.

use super::Tap;
use crate::{
    macos::{event::ScrollEvent, hid::HidEvent},
    reverser::{Axis, Device},
};

impl Tap {
    /// Reverses `event` along the selected axes if it comes from a selected
    /// device.
    pub(super) fn handle(&self, event: &ScrollEvent) {
        let hid = event.hid();
        let sender = hid.as_ref().and_then(HidEvent::sender);
        let device = sender
            .and_then(|sender| self.registry.device(sender))
            .unwrap_or_else(|| Device::from_phases(event.phases()));
        if self.reverser.is_verbose() {
            self.trace(event, device, sender);
        }
        self.reverser
            .reversed_axes(device)
            .for_each(|axis| event.reverse(axis, hid.as_ref()));
    }

    fn trace(&self, event: &ScrollEvent, device: Device, sender: Option<u64>) {
        let (scroll, momentum) = event.phases();
        let axes = self
            .reverser
            .reversed_axes(device)
            .map(|axis| axis.to_string())
            .collect::<Vec<_>>();
        log!(
            "{device} scroll from {}: {} px vertical, {} px horizontal, phases {scroll}/{momentum}, reversing {}",
            sender.map_or_else(
                || "an unknown sender".to_owned(),
                |sender| format!("sender {sender:#x}")
            ),
            event.points(Axis::Vertical),
            event.points(Axis::Horizontal),
            if axes.is_empty() {
                "nothing".to_owned()
            } else {
                axes.join(" and ")
            },
        );
    }
}
