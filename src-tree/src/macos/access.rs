// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Accessibility access, which an event tap that modifies events needs.

use std::{convert, env, iter, ptr, thread, time::Duration};

use super::{cf::Cf, error::Error, sys};

const RETRY_INTERVAL: Duration = Duration::from_secs(2);

/// Runs `attempt` until it succeeds, prompting for access if it first fails.
///
/// Darwin only honours a grant in a new process, so a grant that `attempt`
/// still fails after ends in [`Error::Restart`].
pub fn with_access<T>(attempt: impl Fn() -> Option<T>) -> Result<T, Error> {
    attempt().map_or_else(|| wait_for_access(&attempt), Ok)
}

fn wait_for_access<T>(attempt: &impl Fn() -> Option<T>) -> Result<T, Error> {
    if request() {
        return Err(Error::Refused);
    }
    log!(
        "waiting for Accessibility access; allow {} under System Settings > Privacy & Security > Accessibility",
        executable()
    );
    iter::repeat_with(|| {
        wait(RETRY_INTERVAL);
        retry(attempt)
    })
    .find_map(convert::identity)
    // Retries never run out, so the fallback is unreachable
    .unwrap_or(Err(Error::Restart))
}

fn retry<T>(attempt: &impl Fn() -> Option<T>) -> Option<Result<T, Error>> {
    attempt()
        .inspect(|_| log!("Accessibility access granted"))
        .map(Ok)
        .or_else(|| is_trusted().then_some(Err(Error::Restart)))
}

/// Prompts for access unless the process has it, returning whether it does.
fn request() -> bool {
    Cf::dictionary(
        sys::kAXTrustedCheckOptionPrompt.cast(),
        sys::kCFBooleanTrue.cast(),
    )
    .is_some_and(|options| {
        // SAFETY: `options` is a dictionary
        unsafe { sys::AXIsProcessTrustedWithOptions(options.as_ptr().cast()) != 0 }
    })
}

fn is_trusted() -> bool {
    // SAFETY: Null options are allowed and mean no prompt
    unsafe { sys::AXIsProcessTrustedWithOptions(ptr::null()) != 0 }
}

/// Waits in the run loop, where the frameworks receive the notification that
/// refreshes their cached access, or sleeps if the loop has no sources.
fn wait(interval: Duration) {
    // SAFETY: The default mode is a valid run loop mode
    let result =
        unsafe { sys::CFRunLoopRunInMode(sys::kCFRunLoopDefaultMode, interval.as_secs_f64(), 0) };
    if result == sys::kCFRunLoopRunFinished {
        thread::sleep(interval);
    }
}

fn executable() -> String {
    env::current_exe().map_or_else(
        |_| "scrollreversed".to_owned(),
        |path| path.display().to_string(),
    )
}
