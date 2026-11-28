// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sentences of the help page that name the keys of the keymap in use

use crate::front::keymap::{Action, Keymap};

/// Tells in a few lines what to press first, with the keys of the keymap in
/// use, since the table below puts the keys of scrolling before those that
/// add and name a cue
pub(super) fn first_steps(keymap: &Keymap) -> String {
    let key = |action| keymap.key(action).unwrap_or_else(|| "(no key)".to_owned());
    format!(
        "First steps: {go} is GO, which fires the standby cue, and {panic} is Panic, which stops everything.  {next} and {previous} move the selection, and {standby} puts the selected cue on standby.  {add} adds a cue below, {name} names it, and {undo} undoes a change.  {next_field} moves through the fields to the File field, where Enter chooses a file.  {hint} labels each place to click.  {close} closes this page, and {standby} opens the tutorial.  {save} saves the show.  The table lists every key.",
        go = key(Action::Go),
        panic = key(Action::Panic),
        next = key(Action::Next),
        previous = key(Action::Previous),
        standby = key(Action::Standby),
        add = key(Action::AddBelow),
        name = key(Action::ChangeName),
        undo = key(Action::Undo),
        next_field = key(Action::NextField),
        hint = key(Action::Hint),
        close = key(Action::CloseHelp),
        save = key(Action::Save),
    )
}

/// Tells what a count typed before a key does, which is the same for the
/// cues and the outputs that show in their place
pub(super) fn counts(keymap: &Keymap) -> String {
    let example = keymap
        .key(Action::Next)
        .map_or_else(String::new, |keys| format!(", as in 3 {keys}"));
    format!(
        "A count typed before a key repeats a move{example}.  It goes that many places back or forward, undoes or redoes that many changes, copies, cuts, deletes, or adds that many cues or outputs, indents or outdents that many cues, pastes that many times, goes that many matches along, scrolls that many rows, and goes that many pages down or up.  It picks the row for the first and last cue and for putting a row in place, the row in view for the top and bottom rows, and the tab for the next tab, and sets the rows of a half page."
    )
}

/// Returns what the operators do, with the `keys` of the keymap that start
/// one, or nothing where the keymap has none
pub(super) fn operators(keymap: &Keymap) -> Option<String> {
    Some(keymap.operator_keys())
        .filter(|keys| !keys.is_empty())
        .map(|keys| format!("After {}, a key that selects the next, previous, first, or last cue or output applies the action to the cues or outputs from the selection to where that key would move it.  A count before the operator, before the key, or both counts the move, and the two multiply.", keys.join(" or ")))
}
