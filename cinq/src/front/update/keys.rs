// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keys typed outside the fields, and the keys that act in them
//!
//! Keys typed outside any text field go to the key bindings in
//! [`crate::front::keymap`], and the keys of a command typed halfway wait
//! in the interface state until the command is complete.  The fields keep
//! the keys typed into them, apart from the keys that act in fields.  iced
//! moves the keyboard focus only on clicks, so the keys that edit fields
//! move it themselves, through [`super::focus`].

mod across;
mod help;

use iced::{Task, keyboard, widget::operation};

use super::focus::{move_focus, reveal_focused, select_focused, unfocus};
use crate::{
    back::Kind,
    front::{
        App, Caret, Focus, Message, Tab,
        keymap::{Action, KeyPress, Press},
        view,
    },
};

/// What the status bar tells when the stage window has the keyboard and a key
/// of search, help, or hint mode reaches it, which belong to the main window,
/// and how to give it the keyboard
pub(super) const MAIN_WINDOW: &str = "Click the main window for search, help, or hint mode";

impl App {
    /// Returns why a key of `action` does nothing now, when it is one that
    /// changes the show or names a cue, or one that the stage window leaves
    /// to the main window.  Copying and undoing carry on in show mode, so
    /// that they stay unremarked there, and the help page covers the cues,
    /// so that no key is refused for them.  The hints of the status bar ask
    /// [`Action::stays_in_main_window`] too, so that they never teach a key
    /// that this refuses.
    pub(in crate::front) fn refusal(&self, action: Action) -> Option<&'static str> {
        let locked = action.changes_the_show()
            && !matches!(action, Action::Copy | Action::Undo | Action::Redo);
        match (self.help, self.stage.has_keyboard()) {
            (_, true) if action.stays_in_main_window() => Some(if action.changes_the_show() {
                "Click the main window to edit"
            } else {
                MAIN_WINDOW
            }),
            (true, _) => None,
            (false, _) => (!self.editable() && locked).then_some("Leave show mode to edit"),
        }
    }

    /// Applies a key press that no field captured, in stages: hint mode takes
    /// it first, then the key bindings parse it, and the operator or action
    /// that it completes, if any, acts
    pub(super) fn typed(self, press: Press, repeat: bool) -> (Self, Task<Message>) {
        if self.hints.is_some() {
            return self.hint_typed(press, repeat);
        }
        let outcome = self.config.keymap.parse(&self.keys, press);
        let across = outcome.across();
        let action = outcome.action;
        // An operator over a motion drops the notices once it finds rows to act on
        let app = self.parsed(outcome.pending, action.filter(|_| across.is_none()));
        match across {
            Some((operator, motion, count)) => app.operated(operator, motion, count),
            None => app.acted(action, repeat),
        }
    }

    /// Keeps the keys of a command typed halfway.  A key that completes an
    /// action drops the remark and the unapplied text, and the clipboard
    /// mark too, unless the action only moves.
    fn parsed(self, pending: Vec<Press>, action: Option<(Action, Option<usize>)>) -> Self {
        Self {
            keys: pending,
            notices: self.notices.after_key(action.map(|(action, _)| action)),
            ..self
        }
    }

    /// Applies the action that a key completed, if any, unless it is dropped
    /// or refused first
    fn acted(self, action: Option<(Action, Option<usize>)>, repeat: bool) -> (Self, Task<Message>) {
        let refusal = action.and_then(|(action, _)| self.refusal(action));
        match action {
            Some((action, _)) if repeat && action.acts_once_a_press() => (self, Task::none()),
            // Cue list is out of sight from the stage window, so that the keys
            // that change the show or the clipboard are left to the main window,
            // as are the page, the search line, and hint mode, which would open
            // out of sight under a fullscreen stage window, and show mode locks
            // the show, so that the status bar says why
            Some(_) if refusal.is_some() => (
                Self {
                    notices: self.notices.with_remark(refusal),
                    ..self
                },
                Task::none(),
            ),
            Some((action, count)) if self.help => self.help_keyed(action, count, repeat),
            Some((action, count)) => self.keyed(action, count),
            None => (self, Task::none()),
        }
    }

    /// Applies the message of the key of `action`, after leaving the list
    /// that holds the keyboard if the action needs the keyboard elsewhere
    fn keyed(self, action: Action, count: Option<usize>) -> (Self, Task<Message>) {
        // The list that held the keyboard would swallow the letters
        // of a label, and the key of a new cue would find it there again
        let leaves = self.keyboard.held.any && action.leaves_lists();
        let (app, task) = self.handled(action.message(count));
        (app, if leaves { unfocus().chain(task) } else { task })
    }

    /// Gives up the search line, with the selection going back to where the
    /// search began, and takes the keyboard out of the field that has it,
    /// as Control+c does in Vim.  Playback goes on, since it is no Panic.
    pub(super) fn cancelled(self) -> (Self, Task<Message>) {
        let (app, reveal) = self.search_abandoned();
        (app, Task::batch([reveal, unfocus()]))
    }

    /// Takes the kind that a pick chose for the cues that the key of adding
    /// makes.  A pick by key leaves the keyboard on the list, from which that
    /// key would do nothing, so that the keyboard goes back to the cue list,
    /// as it does when Enter leaves a text field.  A pick by the pointer gives
    /// the list no keyboard, so that the same step changes nothing.  The state
    /// of the interface does not hold the keyboard yet while the menu hands it
    /// back, so that this does not ask for it.
    pub(super) fn with_new_kind(self, new_kind: Kind) -> (Self, Task<Message>) {
        (Self { new_kind, ..self }, unfocus())
    }

    /// Editing the name shows the Basics tab, which holds the Name field, or
    /// the Name field of the output while the outputs show.  No field takes
    /// the keyboard while the stage window has it, since the keys typed there
    /// would never reach the field.
    pub(super) fn focused(self, focus: Focus) -> (Self, Task<Message>) {
        match focus {
            Focus::Name(_) | Focus::Next | Focus::Previous if self.stage.has_keyboard() => {
                (self, Task::none())
            }
            Focus::Name(caret)
                if self.outputs_shown && self.editable() && self.output.is_some() =>
            {
                (self, in_field(view::OUTPUT_NAME, caret))
            }
            Focus::Name(_) if self.outputs_shown => (self, Task::none()),
            Focus::Name(caret) if self.editable() && self.selected.is_some() => (
                Self {
                    tab: Tab::Basics,
                    ..self
                },
                in_field(view::NAME, caret),
            ),
            Focus::Name(_) => (self, Task::none()),
            Focus::Next => (self.without_search(), reached(move_focus(false))),
            Focus::Previous => (self.without_search(), reached(move_focus(true))),
            Focus::Cues => (self, unfocus()),
        }
    }
}

/// Reads a key that no widget captured, with Shift applied, so that Shift
/// and `g` make `G`
pub(super) fn key_press(event: keyboard::Event) -> Option<Message> {
    let KeyPress { press, repeat, .. } = KeyPress::of(&event)?;
    press.map(|press| Message::Typed { press, repeat })
}

/// Selects the text of the field that a move by key reached, so that typing
/// replaces it as in desktop programs, while a click keeps the caret where
/// it fell, and scrolls the sends or the sidebar to show the field if it is
/// in one of them
pub(super) fn reached(moved: Task<Message>) -> Task<Message> {
    moved.chain(select_focused()).chain(reveal_focused())
}

/// Focuses the text field `id`, with the caret where the operator asked for
/// it
fn in_field(id: &'static str, caret: Caret) -> Task<Message> {
    match caret {
        // Focusing puts the caret at the end, as Vim's `A` does
        Caret::End => operation::focus(id),
        Caret::Start => operation::focus(id).chain(operation::move_cursor_to_front(id)),
        Caret::All => operation::focus(id).chain(operation::select_all(id)),
    }
}
