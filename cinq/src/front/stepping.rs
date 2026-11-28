// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Index and scroll arithmetic that the keys, the rows, and the drop-down
//! menus share

/// Returns the item `offset` places after `from`, or before it when
/// `offset` is negative, among `count` items.  Stepping stays put past
/// either end, and without an item to step from, it starts at the first.
pub(super) fn step(from: Option<usize>, offset: isize, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    Some(from.map_or(0, |index| index.saturating_add_signed(offset).min(last)))
}

/// Returns where a view that shows `height` of its content from `scrolled`
/// on has to scroll to, to show the span from `top` to `bottom` while
/// scrolling as little as it can, or `None` when the span is in view
pub(super) fn reveal(top: f32, bottom: f32, scrolled: f32, height: f32) -> Option<f32> {
    if top < scrolled {
        Some(top)
    } else if bottom > scrolled + height {
        Some(bottom - height)
    } else {
        None
    }
}
