// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Device lookup for the senders of HID events.

mod service;

use std::{cell::RefCell, collections::HashMap};

use self::service::Service;
use crate::reverser::Device;

/// Devices behind the HID senders seen so far, keyed by registry entry ID
#[derive(Default)]
pub struct Registry(RefCell<HashMap<u64, Option<Device>>>);

impl Registry {
    /// Returns the device behind `sender`, looked up and logged once, or
    /// `None` if the I/O Registry lacks it.
    pub fn device(&self, sender: u64) -> Option<Device> {
        *self
            .0
            .borrow_mut()
            .entry(sender)
            .or_insert_with(|| identify(sender))
    }
}

fn identify(sender: u64) -> Option<Device> {
    let found = Service::with_id(sender).map(|service| (service.device(), service.product()));
    match &found {
        Some((device, Some(product))) => log!("{product} (sender {sender:#x}) is a {device}"),
        Some((device, None)) => log!("sender {sender:#x} is a {device}"),
        None => log!("sender {sender:#x} is not in the I/O Registry; using gesture phases"),
    }
    found.map(|(device, _)| device)
}
