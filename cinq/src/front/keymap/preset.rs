// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keymaps that cinq ships, which the configuration file chooses from or
//! builds on
//!
//! Every action lists its keys in each keymap, which the compiler checks, so
//! that no keymap forgets an action added later.

use strum::{EnumString, IntoStaticStr, VariantArray};

use super::Action;

#[derive(Debug, Clone, Copy, Default, EnumString, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "lowercase")]
pub enum Preset {
    /// Keys in the manner of vi: letters move and act, some commands take
    /// two keys, and the arrows and Control with `n` and `p` move as well
    #[default]
    Vi,
    /// Keys of IBM's Common User Access, which most desktop programs follow:
    /// the arrows, Home, and End move, Control with a letter edits, and F1
    /// helps.  Space and Escape keep GO and Panic, as in QLab.
    Cua,
}

impl Preset {
    pub(super) fn name(self) -> &'static str {
        self.into()
    }

    pub(super) fn keys(self, action: Action) -> &'static [&'static str] {
        // Keys of vi, then of CUA, as the table of cinq(5) lists them
        let [vi, cua]: [&[&str]; 2] = match action {
            Action::Go => [&["Space"], &["Space"]],
            Action::Panic => [&["Escape"], &["Escape"]],
            Action::Next => [&["j", "ArrowDown", "Control+n"], &["ArrowDown"]],
            Action::Previous => [&["k", "ArrowUp", "Control+p"], &["ArrowUp"]],
            Action::NextList => [&["l", "ArrowRight"], &["Control+PageDown"]],
            Action::PreviousList => [&["h", "ArrowLeft"], &["Control+PageUp"]],
            Action::First => [&["g g", "Control+Home"], &["Home"]],
            Action::Last => [&["G", "Control+End"], &["End"]],
            Action::HalfPageDown => [&["Control+d"], &[]],
            Action::HalfPageUp => [&["Control+u"], &[]],
            Action::PageDown => [&["Control+f", "PageDown"], &["PageDown"]],
            Action::PageUp => [&["Control+b", "PageUp"], &["PageUp"]],
            Action::NextTab => [&["g t"], &["Control+Tab"]],
            Action::PreviousTab => [&["g T"], &["Control+Shift+Tab"]],
            Action::Back => [&["Control+o"], &["Alt+ArrowLeft"]],
            Action::Forward => [&["Control+i"], &["Alt+ArrowRight"]],
            Action::Standby => [&["Enter"], &["Enter"]],
            Action::Edit => [&["i"], &["F2"]],
            Action::NextField => [&["Tab"], &["Tab"]],
            Action::PreviousField => [&["Shift+Tab"], &["Shift+Tab"]],
            Action::AddBelow => [&["o"], &["Control+n"]],
            Action::AddAbove => [&["O"], &["Control+N"]],
            Action::MoveDown => [&["Alt+j", "Alt+ArrowDown"], &["Alt+ArrowDown"]],
            Action::MoveUp => [&["Alt+k", "Alt+ArrowUp"], &["Alt+ArrowUp"]],
            Action::Delete => [&[], &["Delete"]],
            Action::Copy => [&["y y"], &["Control+c"]],
            Action::Cut => [&["d d", "x", "Delete"], &["Control+x"]],
            Action::PasteBelow => [&["p"], &["Control+v"]],
            Action::PasteAbove => [&["P"], &["Control+V"]],
            Action::Undo => [&["u"], &["Control+z"]],
            Action::Redo => [&["Control+r"], &["Control+y", "Control+Z"]],
            Action::Open => [&["Control+l"], &["Control+o"]],
            Action::Save => [&["Control+s"], &["Control+s"]],
            Action::SaveAs => [&["Control+S"], &["Control+S"]],
            Action::Fullscreen => [&["F11"], &["F11"]],
            Action::Help => [&["?", "F1"], &["F1"]],
            Action::CloseHelp => [&["q"], &["Control+w"]],
        };
        match self {
            Preset::Vi => vi,
            Preset::Cua => cua,
        }
    }
}
