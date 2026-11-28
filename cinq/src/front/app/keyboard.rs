// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What has the keyboard, which the status bar hints the keys of

#[derive(Default)]
pub(in crate::front) struct Keyboard {
    /// Whether the menu of a drop-down list is open
    pub(in crate::front) menu: bool,
    pub(in crate::front) held: Held,
    /// Whether the highlight of the open menu of the Device list of an
    /// output rests on its default, which the Default device list of the
    /// sidebar tells in color
    pub(in crate::front) default_highlighted: bool,
}

/// Which kinds of widget have the keyboard, as the widgets told after the
/// latest thing that the operator did
#[derive(Clone, Copy, Debug, Default)]
pub struct Held {
    /// Whether a text field or a drop-down list has the keyboard, which the
    /// status bar hints how to leave
    pub(in crate::front) any: bool,
    /// Whether the File field of a cue has the keyboard, which the status bar
    /// hints Enter for while it is empty
    pub(in crate::front) file: bool,
    /// Whether a checkbox has the keyboard, which the status bar hints Enter
    /// for as it flips it, where it opens a drop-down list
    pub(in crate::front) check: bool,
    /// Whether a button has the keyboard, which the status bar hints Enter
    /// for as it presses the button
    pub(in crate::front) button: bool,
    /// Whether the address of an instrument in the light patch has the
    /// keyboard, which shows the help on addresses under the patch
    pub(in crate::front) address: bool,
    /// Whether a text field has the keyboard, which keeps every key but those
    /// that act in fields, so that the status bar hints no other key.  A
    /// closed drop-down list, which `any` alone tells, lets the keys of undo
    /// and redo act as well.
    pub(in crate::front) text: bool,
}

impl Keyboard {
    /// Returns the keyboard with `open` as whether a menu is open, which
    /// leaves the highlight of the menu where it was not yet known to rest
    pub(in crate::front) fn with_menu(self, open: bool) -> Self {
        Self {
            menu: open,
            default_highlighted: false,
            ..self
        }
    }

    pub(in crate::front) fn with_held(self, held: Held) -> Self {
        Self { held, ..self }
    }

    pub(in crate::front) fn with_default_highlighted(self, highlighted: bool) -> Self {
        Self {
            default_highlighted: highlighted,
            ..self
        }
    }
}
