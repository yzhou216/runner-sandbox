// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! GO button, standby display, and show-wide playback controls

use iced::{
    Element, Fill,
    widget::{Row, column, container, responsive, row, text, text::Wrapping},
};

use super::{
    SPACING, TEXT_SIZE, button, char_width, faults, format, height_of, theme,
    widget::{Button, shortened},
};
use crate::{
    back::{Cue, one_line},
    front::{App, Message},
};

/// Text size of the title, which names the standby cue
const TITLE: f32 = 24.0;

/// Columns of the font that a window needs to keep the buttons beside the
/// title and leave it 16 columns of its own size, as the GO button, the
/// buttons, and the spacing take the rest
const BESIDE: f32 = 68.0;

fn beside(window: f32) -> bool {
    window >= BESIDE * char_width()
}

pub(super) fn view(app: &App, window: f32) -> Element<'_, Message> {
    let list = app.current();
    let standby = list.standby.and_then(|id| app.workspace.cue(id));
    let [title, notes, chain] = lines(app, standby);
    let held = app
        .workspace
        .active()
        .iter()
        .filter(|active| active.paused)
        .count();
    let chain = [
        format::paused(held),
        app.workspace
            .moving(app.list)
            .map(|(target, seconds)| format::moving(target, seconds)),
    ]
    .into_iter()
    .flatten()
    .chain([chain])
    .collect::<Vec<_>>()
    .join("; ");
    let buttons = buttons(app);
    // The notes and the chain take the width under the title and the buttons,
    // which a narrow window puts under the title, so that it reads
    let head: Vec<Element<'_, Message>> = if beside(window) {
        vec![row![line(title, TITLE), buttons].spacing(SPACING).into()]
    } else {
        vec![line(title, TITLE), buttons.into()]
    };
    let summary = column(head)
        .push(line(notes, TEXT_SIZE.0))
        .push(line(chain, TEXT_SIZE.0));
    // GO still presses with nothing to fire, since it steps standby past cues that
    // cannot play, but it looks as a disabled button does
    let fires = app.workspace.next_go(list).next().is_some();
    row![
        go(standby.is_some(), fires),
        container(summary).width(Fill).clip(true),
    ]
    .spacing(SPACING)
    .padding(SPACING)
    .into()
}

/// Returns the title, the notes, and the chain of `standby`, or of the end
/// of the list, always three lines, so that the parts below stay where they
/// are as standby moves
fn lines(app: &App, standby: Option<&Cue>) -> [String; 3] {
    match standby {
        Some(cue) => {
            let notes = Some(one_line(&cue.notes))
                .filter(|notes| !notes.is_empty())
                .unwrap_or_else(|| "none".to_owned());
            [
                format!("Standby: {cue}"),
                format!("Notes: {notes}"),
                format::chain(&app.workspace, app.current(), cue, &faults::failing(app)),
            ]
        }
        None => [
            "Standby: end of list".to_owned(),
            "Notes: none".to_owned(),
            "GO fires nothing".to_owned(),
        ],
    }
}

fn buttons(app: &App) -> Row<'_, Message> {
    let paused = app.workspace.active().iter().any(|active| active.paused);
    row![
        button("Pause all").on_press_maybe(app.running().then_some(Message::PauseAll)),
        button("Resume all").on_press_maybe(paused.then_some(Message::ResumeAll)),
        panic(),
    ]
    .spacing(SPACING)
}

/// Returns the GO button, which hint mode leaves out since Space fires GO from
/// everywhere, in hint mode too
fn go<'a>(standby: bool, fires: bool) -> Button<'a> {
    button(text("GO").size(40))
        .style(if fires {
            theme::go
        } else {
            theme::go_without_cues
        })
        .on_press_maybe(standby.then_some(Message::Go))
        .unhinted()
}

/// Returns the Panic button, which hint mode leaves out since Escape panics
/// from everywhere, in hint mode too
fn panic<'a>() -> Button<'a> {
    button("Panic")
        .style(theme::panic)
        .on_press(Message::Panic)
        .unhinted()
}

/// Returns `told` of text `size`, which ends in an ellipsis where the room
/// runs out, so that the operator sees that a note goes on
fn line<'a>(told: String, size: f32) -> Element<'a, Message> {
    responsive(move |room| {
        let columns = (room.width / (char_width() * size / TEXT_SIZE.0)) as usize;
        text(shortened(&told, columns))
            .size(size)
            .wrapping(Wrapping::None)
            .into()
    })
    .height(height_of(size.into()))
    .into()
}

#[cfg(test)]
mod tests {
    use super::{BESIDE, beside, go, panic};
    use crate::front::view::{button, char_width};

    #[test]
    fn go_and_panic_are_no_places_of_hint_mode_but_the_other_buttons_are() {
        assert!(go(true, true).place().is_none());
        assert!(panic().place().is_none());
        assert!(
            button("Pause all")
                .on_press(crate::front::Message::PauseAll)
                .place()
                .is_some()
        );
    }

    #[test]
    fn a_window_too_narrow_for_the_title_puts_the_buttons_under_it() {
        let least = BESIDE * char_width();
        assert!(!beside(least - 1.0));
        assert!(beside(least));
    }
}
