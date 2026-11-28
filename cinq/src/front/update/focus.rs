// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keyboard focus, which iced moves only on clicks, and which the update
//! cycle therefore finds, moves, and brings into view itself

mod focus_order;
mod holder;
mod reveal_focused;
mod reveal_list;
#[cfg(test)]
mod scrolled;
mod select_focused;

pub(super) use focus_order::{move_focus, unfocus};
pub(super) use holder::held;
pub(super) use reveal_focused::reveal_focused;
pub(super) use reveal_list::reveal_current_list;
pub(super) use select_focused::select_focused;
