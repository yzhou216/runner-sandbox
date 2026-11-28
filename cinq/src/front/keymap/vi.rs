// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keymap that cinq ships, which the configuration file chooses or builds
//! on, with keys in the manner of vi: letters move and act, some commands
//! take two keys, and the arrows and Control with `n` and `p` move as well
//!
//! Every action lists its keys, which the compiler checks, so that no action
//! added later goes without a list.

use super::Action;

pub(in crate::front) const NAME: &str = "vi";

pub(super) fn keys(action: Action) -> &'static [&'static str] {
    match action {
        Action::Go => &["Space"],
        Action::Panic => &["Escape"],
        Action::Standby => &["Enter"],
        Action::Cancel => &["Control+c", "Control+["],
        Action::Next => &["j", "g j", "ArrowDown", "Control+n", "+"],
        Action::Previous => &["k", "g k", "ArrowUp", "Control+p", "-"],
        Action::First => &["g g", "Control+Home"],
        Action::Last => &["G", "Control+End"],
        Action::HalfPageDown => &["Control+d"],
        Action::HalfPageUp => &["Control+u"],
        Action::PageDown => &["Control+f", "PageDown"],
        Action::PageUp => &["Control+b", "PageUp"],
        Action::ScrollDown => &["Control+e"],
        Action::ScrollUp => &["Control+y"],
        Action::TopRow => &["H"],
        Action::MiddleRow => &["M"],
        Action::BottomRow => &["L"],
        Action::RowToMiddle => &["z z", "z ."],
        Action::RowToTop => &["z t", "z Enter"],
        Action::RowToBottom => &["z b", "z -"],
        Action::Collapse => &["z c"],
        Action::Expand => &["z o"],
        Action::ToggleGroup => &["z a"],
        Action::CollapseAll => &["z M"],
        Action::ExpandAll => &["z R"],
        Action::NextList => &["l", "ArrowRight"],
        Action::PreviousList => &["h", "ArrowLeft"],
        Action::NextTab => &["g t"],
        Action::PreviousTab => &["g T"],
        Action::Back => &["Control+o"],
        Action::Forward => &["Control+i"],
        Action::Search => &["/"],
        Action::SearchBackward => &[],
        Action::NextMatch => &["n"],
        Action::PreviousMatch => &["N"],
        Action::Edit => &["i", "a", "A"],
        Action::EditStart => &["I"],
        Action::ChangeName => &["c c", "S"],
        Action::NextField => &["Tab"],
        Action::PreviousField => &["Shift+Tab"],
        // Vim's f finds a character in a line, which no list has, and it is the key of Vimium and Tridactyl
        Action::Hint => &["f"],
        Action::Pick => &["Control+y"],
        Action::CloseMenu => &["Control+e"],
        Action::AddBelow => &["o"],
        Action::AddAbove => &["O"],
        Action::Cut => &["d d", "x", "Delete"],
        Action::Copy => &["y y", "Y"],
        Action::PasteBelow => &["p"],
        Action::PasteAbove => &["P"],
        Action::Delete => &[],
        Action::MoveDown => &["Alt+j", "Alt+ArrowDown"],
        Action::MoveUp => &["Alt+k", "Alt+ArrowUp"],
        Action::Indent => &["> >"],
        Action::Outdent => &["< <"],
        Action::Undo => &["u"],
        Action::Redo => &["Control+r"],
        Action::Open => &["Control+l"],
        Action::Save => &["Control+s"],
        Action::SaveAs => &["Control+S"],
        Action::Fullscreen => &["F11"],
        // Not Vim's meaning of ?, so that the most common help key works everywhere
        Action::Help => &["?", "F1"],
        Action::CloseHelp => &["q"],
    }
}
