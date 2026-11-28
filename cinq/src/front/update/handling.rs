// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that start a task or change what the rows show, each applied by
//! the method of its family
//!
//! This is the first stop of a message after [`App::next`], and the one that
//! may return a task.  [`family`] names every message, with no arm for the
//! rest, so that a new message has no place to go until it is given a
//! family.  A message of a family whose guard does not hold goes to
//! [`App::settled_by`], which only changes state, as does every message of
//! no other family.

mod changing;
mod finding;
mod moving;
mod outside;
mod playing;
mod showing;
mod windows;

use iced::Task;

use super::{App, Message};
use crate::back::Command;

/// Part of the interface that a message acts on, which names the method
/// that applies it
enum Family {
    Keys,
    Moving,
    Changing,
    Finding,
    Outside,
    Windows,
    Playing,
    Showing,
    Settled,
}

impl App {
    pub(super) fn handled(self, message: Message) -> (Self, Task<Message>) {
        match family(&message) {
            // While the outputs show, the keys that move through rows do the
            // same to outputs, by the code that they share with cues, while
            // the keys that change rows act on outputs by their own code, or
            // on nothing, so that none of them reaches the cue list out of
            // sight
            _ if self.outputs_shown && message.acts_on_rows() && !message.moves_through_rows() => {
                self.in_outputs(message)
            }
            Family::Keys => self.keyed_by(message),
            Family::Moving => self.moved_by(message),
            Family::Changing => self.changed_by(message),
            Family::Finding => self.searched_by(message),
            Family::Outside => self.outside_by(message),
            Family::Windows => self.windowed_by(message),
            Family::Playing => self.playing_by(message),
            Family::Showing => self.shown_by(message),
            Family::Settled => (self.settled_by(message), Task::none()),
        }
    }

    fn keyed_by(self, message: Message) -> (Self, Task<Message>) {
        match message {
            Message::Typed { press, repeat } => self.typed(press, repeat),
            // A key that moves the keyboard to another field ends the run of
            // edits to the field that it leaves, as a click there does
            Message::Focus(focus) => self.send(Command::EndRun).focused(focus),
            message => (self.settled_by(message), Task::none()),
        }
    }
}

/// Returns the family of `message`, which a new message must be given
fn family(message: &Message) -> Family {
    match message {
        Message::Typed { .. } | Message::Focus(..) => Family::Keys,
        Message::Select(..)
        | Message::Step(..)
        | Message::Row(..)
        | Message::Page(..)
        | Message::Drawn(..)
        | Message::Remeasure
        | Message::Scroll(..)
        | Message::InView(..)
        | Message::Place(..)
        | Message::Fold(..)
        | Message::FlipGroup(..)
        | Message::SwitchList(..)
        | Message::StepList(..)
        | Message::Back(..)
        | Message::Forward(..)
        | Message::SidePressed(..) => Family::Moving,
        Message::NewKind(..)
        | Message::Add(..)
        | Message::Move(..)
        | Message::Indent(..)
        | Message::Outdent(..)
        | Message::Cut(..)
        | Message::Paste(..)
        | Message::Delete(..)
        | Message::Across(..)
        | Message::Undo(..)
        | Message::Redo(..) => Family::Changing,
        Message::Search { .. }
        | Message::Searching(..)
        | Message::SearchKey(..)
        | Message::SearchAccepted
        | Message::SearchStep(..)
        | Message::Cancel => Family::Finding,
        Message::Open
        | Message::Opened(..)
        | Message::Save
        | Message::SaveAs
        | Message::SaveTo(..)
        | Message::ChooseFile(..)
        | Message::ChooseAudioDevice(..)
        | Message::ListAudioDevices
        | Message::EditConfig(..)
        | Message::OpenLink(..)
        | Message::OpenTutorial
        | Message::Recheck => Family::Outside,
        Message::Resized(..) | Message::Closed(..) | Message::ToggleFullscreen => Family::Windows,
        Message::Go | Message::Panic | Message::Remote(..) => Family::Playing,
        Message::Hint | Message::Hinted(..) | Message::ShowOutputs | Message::InspectOutput(..) => {
            Family::Showing
        }
        Message::PauseAll
        | Message::ResumeAll
        | Message::Stop(..)
        | Message::Tick(..)
        | Message::Landed { .. }
        | Message::Measured(..)
        | Message::Reached { .. }
        | Message::MenuOpen(..)
        | Message::DefaultHighlighted(..)
        | Message::StandbySelected
        | Message::SearchSettled
        | Message::ShowMode(..)
        | Message::Tab(..)
        | Message::StepTab(..)
        | Message::ShowTab(..)
        | Message::Copy(..)
        | Message::Edit(..)
        | Message::Patch(..)
        | Message::FileChosen(..)
        | Message::Nothing
        | Message::ToggleHelp
        | Message::CloseHelp
        | Message::Loaded(..)
        | Message::NetworkChecked
        | Message::LightTold(..)
        | Message::OscIgnored(..)
        | Message::OscFailed(..)
        | Message::OscListening
        | Message::Rechecked(..)
        | Message::Reconfigured(..)
        | Message::LinkOpened(..)
        | Message::AudioDevicesListed(..)
        | Message::AudioDeviceKept(..)
        | Message::SelectOutput(..)
        | Message::SettingTyped(..)
        | Message::OutputDevice(..)
        | Message::Saved(..)
        | Message::Clicked
        | Message::WindowFocused(..)
        | Message::Held(..) => Family::Settled,
    }
}
