// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of a file chosen for a new cue and a failed link

use super::{after, helping, typed_into};
use crate::{
    back::{CueId, Edit, Place, Workspace},
    front::{App, Focus, Message, Setting, app::Held},
};

#[test]
fn sound_file_chosen_for_a_new_cue_names_it_by_its_path_from_the_show() {
    let app = App {
        workspace: Workspace::new(Vec::new()).directory("/show"),
        ..App::default()
    };
    let (added, _) = app.handled(Message::Add(Place::After, 1));
    let id = added.selected.expect("a new cue is selected");
    let chosen = Message::FileChosen(added.show.id(), id, "/show/sounds/storm.ogg".into());
    let (chosen, _) = added.handled(chosen);
    let cue = chosen.workspace.cue(id).unwrap();
    assert_eq!(
        (cue.file.as_str(), cue.name.as_str()),
        ("sounds/storm.ogg", "storm")
    );
}

#[test]
fn file_chosen_while_another_show_opened_changes_no_cue_of_it() {
    let app = App {
        workspace: Workspace::new(Vec::new()).directory("/show"),
        ..App::default()
    };
    let (added, _) = app.handled(Message::Add(Place::After, 1));
    let id = added.selected.expect("a new cue is selected");
    let other = added.show.id().wrapping_add(1);
    let (chosen, _) = added.handled(Message::FileChosen(other, id, "/show/storm.ogg".into()));
    assert_eq!(chosen.workspace.cue(id).unwrap().file, "");
}

#[test]
fn link_that_fails_to_open_shows_until_the_help_page_closes() {
    let failure = || Message::LinkOpened(Err("Cannot open it".to_owned()));
    let open = helping(App::default());
    let (failed, _) = open.next(failure());
    assert_eq!(failed.problems.link.as_deref(), Some("Cannot open it"));
    let (closed, _) = failed.next(Message::CloseHelp);
    assert_eq!(closed.problems.link, None);
    let (unseen, _) = closed.next(failure());
    assert_eq!(unseen.problems.link, None);
}
/// Returns the state after the keys of `path` go one at a time into the File
/// field of cue `id`, the way the field reports them, with the keyboard still
/// in it
fn typed_path(app: App, id: CueId, path: &str) -> App {
    typed_into(app, path, |prefix| {
        Message::SettingTyped(Setting::File(id), prefix)
    })
}

/// Returns the state after the keyboard leaves the File field
fn left(app: App) -> App {
    after(app, [Message::Held(Held::default()), Message::Clicked])
}

fn added() -> (App, CueId) {
    let (added, _) = App::default().handled(Message::Add(Place::After, 1));
    let id = added.selected.expect("a new cue is selected");
    (added, id)
}

fn named(app: &App, id: CueId) -> (String, String) {
    let cue = app.workspace.cue(id).unwrap();
    (cue.file.clone(), cue.name.clone())
}

#[test]
fn a_typed_path_waits_until_the_field_is_left_and_then_names_the_cue() {
    let (app, id) = added();
    let before = named(&app, id);
    let partway = typed_path(app, id, "sounds/ra");
    assert_eq!(named(&partway, id), before);
    assert_eq!(
        partway.typed,
        Some((Setting::File(id), "sounds/ra".to_owned()))
    );
    let (app, id) = added();
    let typed = left(typed_path(app, id, "sounds/rain.flac"));
    assert_eq!(typed.typed, None);
    assert_eq!(
        named(&typed, id),
        ("sounds/rain.flac".to_owned(), "rain".to_owned())
    );
}

#[test]
fn every_way_of_leaving_the_field_applies_the_typed_path() {
    [
        Message::Clicked,
        Message::Focus(Focus::Next),
        Message::Focus(Focus::Cues),
        Message::Go,
    ]
    .into_iter()
    .for_each(|leaving| {
        let (app, id) = added();
        let typed = after(typed_path(app, id, "rain.flac"), [leaving.clone()]);
        assert_eq!(named(&typed, id).0, "rain.flac", "{leaving:?}");
    });
}

#[test]
fn undo_takes_the_whole_typed_path_back_in_one_step() {
    let (app, id) = added();
    let before = named(&app, id);
    let typed = left(typed_path(app, id, "sounds/rain.flac"));
    assert_eq!(named(&after(typed, [Message::Undo(1)]), id), before);
}

#[test]
fn a_name_that_the_operator_typed_stays_when_a_path_is_typed() {
    let (app, id) = added();
    let renamed = after(app, [Message::Edit(id, Edit::Name("Thunder".to_owned()))]);
    let typed = left(typed_path(renamed, id, "rain.flac"));
    assert_eq!(
        named(&typed, id),
        ("rain.flac".to_owned(), "Thunder".to_owned())
    );
}

#[test]
fn a_path_typed_again_after_the_field_was_emptied_names_the_cue_again() {
    let (app, id) = added();
    let cleared = Message::SettingTyped(Setting::File(id), String::new());
    let emptied = left(after(left(typed_path(app, id, "rain.flac")), [cleared]));
    assert_eq!(named(&emptied, id).1, "New Audio cue");
    let again = left(typed_path(emptied, id, "wind.flac"));
    assert_eq!(named(&again, id).1, "wind");
}
