// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Header of the inspector: the title of the cue and the sentences that tell
//! what is wrong with it, in one line, so that the rows below never lose
//! their room to a second line

#[cfg(test)]
mod tests;

use iced::{
    Center, Element, Fill, Theme,
    widget::{column, row, text, text::Wrapping},
};
use unicode_width::UnicodeWidthStr;

use super::{
    field::{Field, unread},
    lock,
};
use crate::{
    back::{Cue, Problem},
    front::{
        App, Message,
        view::{SPACING, char_width, format, widget::shortened},
    },
};

/// Columns that the title keeps at least, which is the number and the first
/// words of the name, before a sentence of trouble is cut short instead
const TITLE_LEAST: usize = 16;

/// Sentence beside the title and the style of its text
struct Sentence {
    words: String,
    style: fn(&Theme) -> text::Style,
}

fn gap() -> usize {
    (3.0 * SPACING / char_width()).ceil() as usize
}

/// Returns the header of a cue over `fields`: its title, kept to one line of
/// `columns` columns and ending in an ellipsis where the room runs out, so
/// that a long name never pushes the rows below out of view, then what keeps
/// GO from playing the cue or from lighting all that it names, and which of
/// the `times` that it shows reads as no time
pub(super) fn inspected<'a>(
    app: &'a App,
    cue: &Cue,
    columns: usize,
    times: &[&[Field]],
    fields: Element<'a, Message>,
) -> Element<'a, Message> {
    let sentences = [
        trouble(app, cue),
        unlit(app, cue),
        unread(cue, app.current().group_of(cue.id).is_some(), times).map(|words| Sentence {
            words,
            style: text::danger,
        }),
        loop_played(cue, times),
    ]
    .into_iter()
    .flatten()
    .collect();
    let (title, sentences) = fit(
        &format!("{cue}, {} cue{}", cue.kind, lock(app)),
        sentences,
        columns,
    );
    let header = row![text(title).wrapping(Wrapping::None)]
        .extend(sentences.into_iter().map(|sentence| {
            text(sentence.words)
                .style(sentence.style)
                .wrapping(Wrapping::None)
                .into()
        }))
        .spacing(3.0 * SPACING)
        .align_y(Center);
    column![header, fields].spacing(SPACING).width(Fill).into()
}

/// Returns the `title` cut to the room that `sentences` leave in a line of
/// `columns` columns, with the sentences, each cut short in turn where the
/// room that the title keeps would run out, and left out where none is left
fn fit(title: &str, sentences: Vec<Sentence>, columns: usize) -> (String, Vec<Sentence>) {
    let sentences: Vec<Sentence> = sentences
        .into_iter()
        .scan(columns.saturating_sub(TITLE_LEAST), |room, sentence| {
            let left = room.saturating_sub(gap());
            let words = shortened(&sentence.words, left);
            *room = left.saturating_sub(words.width());
            Some((left > 0).then_some(Sentence { words, ..sentence }))
        })
        .flatten()
        .collect();
    let used: usize = sentences
        .iter()
        .map(|sentence| gap() + sentence.words.width())
        .sum();
    (shortened(title, columns.saturating_sub(used)), sentences)
}

/// Returns what keeps GO from playing the cue, in words, as the State column
/// and the sidebar tell it, so that the place where the fix is made says it
/// too.  A file that has yet to load is no fault.
fn trouble(app: &App, cue: &Cue) -> Option<Sentence> {
    let problem = app
        .workspace
        .problem(cue)
        .filter(|problem| *problem != Problem::NotLoaded)?;
    Some(Sentence {
        words: format!("GO skips this cue: {problem}"),
        style: text::danger,
    })
}

/// Returns what keeps a light cue from setting all that it should, in the
/// warning color, since GO still fires the cue
fn unlit(app: &App, cue: &Cue) -> Option<Sentence> {
    app.workspace.unlit(cue).map(|unlit| Sentence {
        words: format!("This cue {unlit}"),
        style: text::warning,
    })
}

/// Returns what the loop that plays differs by from the loop typed, in the
/// warning color, while `times` holds the loop fields and no typed time
/// reads as no time, which the sentence of that fault tells first
fn loop_played(cue: &Cue, times: &[&[Field]]) -> Option<Sentence> {
    times
        .iter()
        .any(|column| column.contains(&Field::LoopStart))
        .then(|| format::loop_played(cue))
        .flatten()
        .map(|words| Sentence {
            words,
            style: text::warning,
        })
}
