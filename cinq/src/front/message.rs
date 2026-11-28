// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that the update applies: the operator's intent, which the view
//! and the keys send, and the reports of the clock, of the configuration
//! file's watch, and of the work done on threads of its own

mod field;
mod line;
pub(super) mod reasons;
mod rows;
mod scrolling;
mod spot;
mod tab;

use std::{ffi::OsString, net::IpAddr, path::PathBuf, time::Instant};

use iced::{Size, mouse, window};

use super::{
    app::Held,
    config::Config,
    keymap::{Action, Press},
    update::{Measured, Read, Saved},
    view::Site,
};
use crate::back::{Command, CueId, Device, Edit, Kind, OutputId, PatchEdit, Place, Remote};
pub use field::{Caret, Focus, Setting};
pub use line::LineKey;
pub use scrolling::{Fold, Height, Page};
pub use spot::Spot;
pub use tab::Tab;

/// Operator intent or a report of work done, which becomes a back-end
/// command or an interface change
#[derive(Debug, Clone)]
pub enum Message {
    Go,
    Panic,
    /// Drops the keys typed so far, gives up the search line, and leaves a
    /// field, without touching playback
    Cancel,
    PauseAll,
    ResumeAll,
    Stop(CueId),
    /// Time of a tick of the playback clock, or of the moment that the
    /// next picture of a video cue comes due
    Tick(Instant),
    Select(CueId),
    /// Moves the selection by a number of cues, downwards when positive
    Step(isize),
    /// Selects the row counted from one, or the last cue past the end
    Row(usize),
    /// Scrolls the list by a page and moves the selection with it.  The
    /// count typed before the keys sets the rows of a half page and counts
    /// the whole pages.
    Page(Page, Option<usize>),
    /// Selects the row, counted from zero, where a page scrolled the list
    /// to, unless the place that the key found has changed since, as a key
    /// pressed in between would have changed it
    Landed {
        from: Spot,
        row: usize,
    },
    Measured(Measured),
    /// The mouse may have scrolled the list, so that it is measured again
    Remeasure,
    /// A window has drawn, which measures the rows when no measure of them
    /// is kept, since the list is laid out only once the window has drawn
    Drawn(window::Id),
    /// Scrolls the list a number of rows, down when positive, and keeps the
    /// selection in view
    Scroll(isize),
    /// Selects the row at a height in view as a jump, counted from the top
    /// or the bottom with the count typed before the keys
    InView(Height, Option<usize>),
    /// Selects the row, counted from zero, that a measure of the list
    /// found, as a jump, unless the place that the key found has changed
    /// since
    Reached {
        from: Spot,
        row: usize,
    },
    /// Scrolls the list to put the selected row at a height in view, after
    /// selecting the row that the count typed before the keys names
    Place(Height, Option<usize>),
    Fold(Fold),
    /// Collapses the group, or expands it where it is collapsed, as a click
    /// on its mark does
    FlipGroup(CueId),
    MenuOpen(bool),
    /// The highlight of the open menu of the Device list of an output moved
    /// onto its default option, or off it, or the menu closed, which tells
    /// that no option has it
    DefaultHighlighted(bool),
    StandbySelected,
    SwitchList(usize),
    StepList(isize),
    /// Goes a number of places back, among those that jumps have left
    Back(usize),
    Forward(usize),
    /// Opens the search line, which selects the rows that its text matches,
    /// searching up the list when `backward`
    Search {
        backward: bool,
    },
    Searching(String),
    /// Recalls a search into the search line with a key of Vim's command
    /// line
    SearchKey(LineKey),
    SearchAccepted,
    /// Selects the row a number of matches of the last search further on in
    /// the direction that it went, or the other way when negative
    SearchStep(isize),
    /// Time that the search line has told that no row matches has passed
    SearchSettled,
    ShowMode(bool),
    Tab(Tab),
    StepTab(isize),
    ShowTab(usize),
    NewKind(Kind),
    /// Adds a number of cues in a row next to the selection, on the side that
    /// the function picks, as one change
    Add(fn(CueId) -> Place, usize),
    /// Moves the selected cue by a number of places, downwards when
    /// positive
    Move(isize),
    /// Moves a number of cues from the selection down into the group above
    /// them, with the cues that they hold
    Indent(usize),
    /// Moves a number of cues from the selection down out of their groups,
    /// with the cues that they hold
    Outdent(usize),
    /// Copies a number of cues from the selection down to the clipboard
    Copy(usize),
    /// Copies a number of cues from the selection down to the clipboard and
    /// deletes them
    Cut(usize),
    /// Pastes the clipboard a number of times next to the selection, on the
    /// side that the function picks
    Paste(fn(CueId) -> Place, usize),
    Delete(usize),
    /// Applies the operator `Cut`, `Copy`, `Delete`, `Indent`, or `Outdent`
    /// to the cues from the selection to where the motion `Next`,
    /// `Previous`, `First`, or `Last` would move it, counted as the motion
    /// is
    Across(Action, Action, Option<usize>),
    Edit(CueId, Edit),
    Patch(PatchEdit),
    /// Asks which file an audio, video, or picture cue plays or shows
    ChooseFile(CueId),
    /// Answers [`Self::ChooseFile`] with the file for the cue of the show
    /// with that serial number, which a show opened since does not take
    FileChosen(u64, CueId, PathBuf),
    /// Undoes a number of changes, stopping at the oldest
    Undo(usize),
    /// Redoes a number of changes, stopping at the newest
    Redo(usize),
    Focus(Focus),
    /// Labels every place that a click acts on, or takes the labels away if
    /// they are there
    Hint,
    /// Places that a click acts on and the operator can see, in the order
    /// that they lie in the window, which the labels of hint mode go to
    Hinted(Vec<Site>),
    /// Nothing, which the keys of a menu amount to outside one
    Nothing,
    ToggleHelp,
    CloseHelp,
    /// Key that no widget kept, whether typed outside any text field or one
    /// that acts in fields, or a key of GO or Panic that a field took
    /// without typing it, and whether it is a repeat of a key held down
    Typed {
        press: Press,
        repeat: bool,
    },
    /// What loading the files of cues found, for the show of an identity
    Loaded(u64, Vec<Command>),
    /// Time to read again the files that have changed, gone, or come back
    Recheck,
    /// Time to look for news of the network output
    NetworkChecked,
    /// Change in whether light gets out, which the light thread told: why
    /// none does, or none for light that gets out
    LightTold(Option<String>),
    /// Command that arrived over OSC from a host, which acts as a click
    /// that leaves hint mode
    Remote(IpAddr, Remote),
    /// Packet or message that arrived over OSC and is no command
    OscIgnored(String),
    /// Why the address of `osc_listen` could not be listened on
    OscFailed(String),
    /// The address of `osc_listen` that could not be listened on is now
    OscListening,
    /// What loading again the files that could not be played found, for
    /// the show of an identity
    Rechecked(u64, Vec<Command>),
    /// Configuration as its file holds it, read once the watch on the file
    /// is in place and again after each change to it
    Reconfigured(Box<Config>),
    /// Opens a web page or a file in the program that the desktop opens it
    /// with
    OpenLink(OsString),
    /// Opens the configuration file in the program that the desktop opens
    /// it with, making it first when it is not there yet
    EditConfig(PathBuf),
    /// Opens the tutorial in the program that the desktop opens PDF files
    /// with
    OpenTutorial,
    LinkOpened(Result<(), String>),
    /// Asks for the audio devices afresh, as their list opens
    ListAudioDevices,
    /// ID and name of each audio device that sound can play on, or why they
    /// could not be listed, for the asking of that number
    AudioDevicesListed(u64, Result<Vec<(String, String)>, String>),
    /// Plays the sound on the audio device of an ID, or on the default
    /// device without one, and keeps the choice in the configuration file
    ChooseAudioDevice(Option<String>),
    /// Whether the configuration file kept the audio device chosen last
    AudioDeviceKept(Result<(), String>),
    /// Shows the outputs of the show in place of a cue list
    ShowOutputs,
    /// Shows the outputs and selects one of them, as a warning about it does
    InspectOutput(OutputId),
    /// Selects an output, which the inspector then shows
    SelectOutput(OutputId),
    /// Text typed into the field of a setting, which applies once it reads
    /// as a value, or once the field is left for a level or channels
    SettingTyped(Setting, String),
    OutputDevice(OutputId, Device),
    Open,
    /// Show read from its file, with the unsaved changes kept for it
    Opened(Box<Read>),
    /// Saves the show in its file, or asks where to save a show that has
    /// none
    Save,
    SaveAs,
    SaveTo(PathBuf),
    Saved(Saved),
    /// Mouse button pressed anywhere in a window
    Clicked,
    /// Back or forward button of the mouse pressed in a window
    SidePressed(window::Id, mouse::Button),
    /// Window changed size, to no size at all while it is minimized
    Resized(window::Id, Size),
    WindowFocused(window::Id),
    /// Which widgets have the keyboard, as they told after the latest thing
    /// that the operator did
    Held(Held),
    ToggleFullscreen,
    Closed(window::Id),
}
