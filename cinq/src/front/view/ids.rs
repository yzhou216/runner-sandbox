// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Identities of the widgets that keys and the status bar reach by name

/// Cue list, which the selection scrolls into view
pub const CUES: &str = "cues";
/// Selected row, through which the list finds where to scroll
pub const SELECTED: &str = "selected";
/// Outputs list, which the selected output scrolls into view
pub const OUTPUTS: &str = "outputs";
/// Sends of a cue, which the field that Tab reached scrolls into view
pub const SENDS: &str = "sends";
/// Forms of a tab that lie one above the other in a narrow window, the
/// settings of an output, and the tables of the light patch and of the levels
/// of a light cue, which the field that Tab reached scrolls into view
pub const FORMS: &str = "forms";
/// Sidebar, which the list that Tab reached scrolls into view
pub const SIDEBAR_SCROLL: &str = "sidebar";
/// Entry of the sidebar that shows, through which the sidebar finds where to
/// scroll
pub const CURRENT_LIST: &str = "current list";
/// Help page, which the keys that move the selection scroll while it is open
pub const HELP: &str = "help";
/// Name field, which the key of the edit action gives the keyboard focus
pub const NAME: &str = "name";
/// Inspector, whose first field the first Tab from the cue list reaches
pub const INSPECTOR: &str = "inspector";
/// File field of an audio, video, or picture cue, which the status bar hints
/// Enter for while it is empty and has the keyboard
pub const FILE: &str = "file";
/// Every checkbox that takes the keyboard, which the status bar hints Enter
/// for as it flips the box
pub const CHECK: &str = "check";
/// Every button that takes the keyboard, which the status bar hints Enter for
/// as it presses the button
pub const BUTTON: &str = "button";
/// Address fields of the light patch, which show the help on addresses while
/// one of them has the keyboard
pub const ADDRESS: &str = "address";
/// Search line in the status bar, which the key of the search action gives
/// the keyboard focus
pub const SEARCH: &str = "search";
/// Name field of an output, which the key of the edit action gives the
/// keyboard focus while the outputs show
pub const OUTPUT_NAME: &str = "output name";
