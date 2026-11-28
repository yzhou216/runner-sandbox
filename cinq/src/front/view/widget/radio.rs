// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Radio button that hint mode labels

use iced::Element;

use super::hinted;
use crate::front::Message;

/// Returns iced's radio button for `value`, which a label of hint mode selects
/// as a click does
pub fn radio<'a, V: Copy + Eq + 'a>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Element<'a, Message> {
    let act = on_click(value);
    hinted(
        iced::widget::radio(label, value, selected, |_| act.clone()),
        act,
    )
}
