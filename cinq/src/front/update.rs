// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Update cycle, which applies each message to the interface state
//!
//! Only [`cinq`] reaches the parts with side effects.

/// Prints a line to standard error with the time in seconds since the epoch,
/// for the sandbox tests of the keys
macro_rules! sandbox {
    ($($arg:tt)*) => {{
        let at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0.0, |time| time.as_secs_f64());
        eprintln!("SANDBOX {at:.3} {}", format_args!($($arg)*));
    }};
}

mod cinq;
mod clipboard;
mod clock;
mod device;
mod dispatch;
mod editing;
mod focus_order;
mod history;
mod keyboard;
mod keys;
mod link;
mod loading;
mod navigation;
mod operators;
mod outputs;
mod paging;
mod reveal_focused;
mod rows;
mod scroll;
mod search;
mod select_focused;
mod show;
mod stage;
mod subscription;
#[cfg(test)]
mod tests;
mod viewing;

use iced::{Task, mouse};

use super::{App, Message, Tab};
use crate::back::{Command, Edit, OutputEdit, parse_level};
pub(super) use clipboard::{Clipboard, Clipped};
use dispatch::on_own_thread;
pub(super) use history::{History, Spot};
use link::{edit_config, open_link};
pub(super) use paging::kept::{Measured, Measures};
pub(super) use rows::Rows;
pub(super) use search::{Search, Searched};
pub(super) use show::{Keeper, Read, Reader, Saved, Show};
pub(super) use stage::Stage;

impl App {
    fn handled(self, message: Message) -> (Self, Task<Message>) {
        if !matches!(
            message,
            Message::Tick(_)
                | Message::Typed { .. }
                | Message::Measured(_)
                | Message::Remeasure
                | Message::Drawn(_)
                | Message::Resized(..)
                | Message::WindowFocused(_)
                | Message::Clicked
                | Message::Nothing
        ) {
            sandbox!("message {}", format!("{message:?}").chars().take(200).collect::<String>());
        }
        let app = match message {
            Message::Typed { press, repeat } => return self.typed(press, repeat),
            Message::Focus(focus) => return self.focused(focus),
            // While the outputs show, the keys that move through rows do the
            // same to outputs, by the code that they share with cues, while
            // the keys that change rows act on outputs by their own code, or
            // on nothing, so that none of them reaches the cue list out of
            // sight
            message
                if self.outputs_shown
                    && message.acts_on_rows()
                    && !message.moves_through_rows() =>
            {
                return self.in_outputs(message);
            }
            // Selection scrolls into view as it or the rows around it move,
            // and as the main window changes size
            Message::Select(id) => {
                return self
                    .without_search()
                    .crossing(|app| app.showing_cues().with_selection(id))
                    .revealed();
            }
            Message::Step(offset) => return self.stepped(offset).revealed(),
            Message::Row(row) => return self.jumping(|app| app.at_row(row)).revealed(),
            Message::Page(page, count) => return self.paged(page, count),
            Message::Landed { from, row } if self.spot() == from => {
                self.at_row(row.saturating_add(1))
            }
            Message::Landed { .. } => self,
            Message::Measured(Measured::Rows {
                serial,
                rows,
                count,
                shape,
            }) => self.with_measure(serial, rows, count, shape),
            Message::Measured(Measured::Help(view)) => self.with_help_measure(view),
            Message::Drawn(window) if self.main == Some(window) && self.unmeasured() => {
                let measure = self.measure_rows();
                return (self, measure);
            }
            Message::Drawn(_) => self,
            Message::Remeasure => {
                let app = self.remeasured();
                let measure = app.measure_rows();
                return (app, measure);
            }
            Message::Scroll(rows) => return self.scrolled(rows),
            Message::InView(height, count) => return self.in_view(height, count),
            Message::Reached { from, row } if self.spot() == from => {
                self.jumping(|app| app.at_row(row.saturating_add(1)))
            }
            Message::Reached { .. } => self,
            Message::Place(height, count) => return self.placed(height, count),
            // A menu reports where its highlight rests right after it opens, so
            // that a flag left by another menu, or by a list that left the
            // view while open, never lights the sidebar
            Message::MenuOpen(open) => Self {
                menu: open,
                default_highlighted: false,
                ..self
            },
            Message::Held(held) => Self { held, ..self },
            Message::FileHeld(file_held) => Self { file_held, ..self },
            Message::DefaultHighlighted(highlighted) => Self {
                default_highlighted: highlighted,
                ..self
            },
            Message::SwitchList(list) => {
                return self
                    .without_search()
                    .jumping(|app| app.showing_list(list))
                    .revealed();
            }
            Message::StepList(offset) => {
                return self.jumping(|app| app.stepped_list(offset)).revealed();
            }
            Message::Back(count) => return self.walking(|app| app.went_back(count)),
            Message::Forward(count) => return self.walking(|app| app.went_forward(count)),
            Message::Add(place, count) => return self.with_new_cue(place, count).revealed(),
            Message::Move(by) => return self.with_selected_moved(by).revealed(),
            Message::Copy(count) => self.with_copied(count),
            Message::Cut(count) => return self.cut(count).revealed(),
            Message::Paste(place, count) => return self.with_pasted(place, count).revealed(),
            Message::Delete(count) => return self.without_selected(count).revealed(),
            Message::Across(operator, motion, count) => {
                return self.across(operator, motion, count).revealed();
            }
            Message::Search { backward } => return self.searching(backward),
            Message::Searching(text) => return self.search_typed(text),
            Message::SearchKey(key) => return self.search_edited(key),
            Message::SearchAccepted => return self.search_accepted(),
            Message::SearchStep(offset) => return self.search_stepped(offset),
            Message::Undo(count) => return self.undone(count).revealed(),
            Message::Redo(count) => return self.redone(count).revealed(),
            Message::Resized(window, size) => return self.resized(window, size),
            Message::Recheck => return self.recheck(),
            Message::Open => return self.with_task(Self::choose_show),
            Message::Opened(read) => return self.opened(*read).revealed(),
            Message::Save => return self.with_task(Self::save),
            Message::SaveAs => return self.with_task(Self::choose_file),
            Message::SaveTo(file) => return self.with_task(|app| app.save_to(file)),
            Message::ChooseFile(id) => return self.with_task(|app| app.choose_cue_file(id)),
            Message::OpenLink(target) => return self.opening(move || open_link(&target)),
            Message::EditConfig(file) => return self.opening(move || edit_config(&file)),
            Message::ToggleFullscreen => return self.with_task(Self::toggle_fullscreen),
            Message::Closed(window) => return self.closed(window),
            Message::ListAudioDevices => return self.with_task(|_| device::listing()),
            Message::ChooseAudioDevice(device) => return self.chose_audio_device(device),
            // GO and Panic close the help page, so that the show is in view
            // as it runs or stops, and a selection that GO moves scrolls
            // into view, while the list stays put otherwise
            Message::Go => {
                let selected = self.selected;
                let gone = self.gone();
                return if gone.selected == selected {
                    (gone, Task::none())
                } else {
                    gone.revealed()
                };
            }
            // Escape leaves a text field, so it gives up a search as well,
            // and the selection goes back to where the search began
            Message::Panic => {
                let searching = self.search.is_some();
                let app = Self {
                    help: false,
                    ..self.send(Command::Panic)
                }
                .search_cancelled();
                return if searching {
                    app.revealed()
                } else {
                    (app, Task::none())
                };
            }
            // Keys typed so far went already, as for any message
            Message::Cancel => return self.cancelled(),
            Message::PauseAll => self.send(Command::Pause(true)),
            Message::ResumeAll => self.send(Command::Pause(false)),
            Message::Stop(id) => self.send(Command::Stop(id)),
            Message::Tick(now) => self.ticked(now),
            Message::SearchSettled => self.search_settled(),
            // Nothing without a selected cue, as the toolbar's button does
            Message::StandbySelected => {
                let standby = self.selected_here().map(|cue| Command::Standby {
                    list: self.list,
                    cue: Some(cue),
                });
                self.send_all(standby)
            }
            Message::ShowMode(show_mode) => self.send(Command::ShowMode(show_mode)),
            Message::Loaded(show, found) => self.loaded(show, found),
            Message::Rechecked(show, found) => self.rechecked(show, found),
            Message::Saved(saved) => self.saved(saved),
            // Keys typed halfway went by the bindings before, so the update
            // drops them, as it does for most messages
            Message::Reconfigured(config) => Self { config, ..self },
            Message::LinkOpened(result) => Self {
                link_problem: result.err(),
                ..self
            },
            Message::AudioDevicesListed(listed) => self.with_audio_devices(listed),
            Message::AudioDeviceKept(kept) => Self {
                device_problem: kept.err(),
                ..self
            },
            Message::ShowOutputs => return self.jumping(Self::showing_outputs).revealed(),
            Message::InspectOutput(output) => {
                return self
                    .jumping(|app| {
                        Self {
                            output: Some(output),
                            ..app
                        }
                        .showing_outputs()
                    })
                    .revealed();
            }
            Message::SelectOutput(output) => Self {
                output: Some(output),
                ..self
            },
            Message::SettingTyped(setting, text) => {
                self.settled_but(setting).typed_setting(setting, text)
            }
            Message::OutputDevice(output, device) => {
                self.send(Command::EditOutput(output, OutputEdit::Device(device)))
            }
            // A click on blank space would leave the search line open with no
            // way to close it but Panic
            Message::Clicked => self.without_search(),
            Message::Nothing => self,
            // Buttons of the mouse go back and forward from the main window, as
            // the keys do, apart from the help page, which the keys leave be
            Message::SidePressed(window, mouse::Button::Back)
                if self.main == Some(window) && !self.help =>
            {
                return self.walking(|app| app.went_back(1));
            }
            Message::SidePressed(window, mouse::Button::Forward)
                if self.main == Some(window) && !self.help =>
            {
                return self.walking(|app| app.went_forward(1));
            }
            Message::SidePressed(..) => self,
            // Stage window holds no cue list to keep in view
            Message::WindowFocused(window) => Self {
                stage: self.stage.with_keyboard_in(window),
                ..self
            },
            Message::Edit(id, edit) => self.send(Command::Edit(id, edit)),
            Message::LevelTyped(cue, text) => {
                let edit = parse_level(&text).map(|level| Command::Edit(cue, Edit::Level(level)));
                Self {
                    typed_level: Some((cue, text)),
                    ..self
                }
                .send_all(edit)
            }
            Message::FileChosen(id, path) => {
                let file = self.workspace.file_name_of(&path);
                let playing = self.workspace.cue(id).map(|cue| cue.playing(file));
                self.send_all(playing.map(|edit| Command::Edit(id, edit)))
            }
            Message::ToggleHelp => Self {
                help: !self.help,
                ..self
            },
            Message::CloseHelp => Self {
                help: false,
                ..self
            },
            Message::Tab(tab) => Self { tab, ..self },
            Message::ShowTab(number) => Self {
                tab: Tab::numbered(number),
                ..self
            },
            Message::StepTab(offset) => Self {
                tab: self.tab.stepped(offset),
                ..self
            },
            Message::NewKind(new_kind) => Self { new_kind, ..self },
        };
        (app, Task::none())
    }
}
