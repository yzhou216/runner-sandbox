// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which columns of the cue list show, and how wide, in the room there is

use strum::VariantArray;
use unicode_width::UnicodeWidthStr;

use super::{
    CELL_SPACE, Column, FAILED_ROOM, INDENT, NAME_LEAST, NAME_MOST, STATE_LEAST, TARGET_GROWN,
    TARGET_ROOM, columns::counted, faults, layout::Layout,
};
use crate::{
    back::one_line,
    front::{
        App,
        view::{char_width, giving_way},
    },
};

/// What gives way when the room is short, in turn
#[derive(Clone, Copy, PartialEq)]
enum Step {
    /// The Target column narrows to [`TARGET_ROOM`] columns of the font
    Narrow,
    Drop(Column),
}

/// Column with its width in pixels, or none for the name, which takes the rest
type Fitted = (Column, Option<f32>);

/// Returns `columns` with the State column narrowed by `deficit`, but not
/// below [`STATE_LEAST`] columns of the font, and unchanged where the deficit
/// is none.  It is the only column left to narrow once the yielding columns
/// have gone.
fn squeezed(columns: Vec<Fitted>, deficit: f32) -> Vec<Fitted> {
    let floor = STATE_LEAST as f32 * char_width();
    columns
        .into_iter()
        .map(|(column, width)| match (column, width) {
            (Column::State, Some(width)) => (column, Some((width - deficit.max(0.0)).max(floor))),
            shown => shown,
        })
        .collect()
}

/// Returns the columns that a list `room` wide shows, each with its width.
/// A column that holds no text in any row is not shown at all.  The name
/// column needs the width of its widest name, from [`NAME_LEAST`] to
/// [`NAME_MOST`] columns of the font, past the gutter of the mark.  When the
/// other columns leave it less, the Target column narrows to
/// [`TARGET_ROOM`], and then the yielding columns give way in turn.  A column
/// that has given way returns only once the name keeps all that it needs, so
/// that a wider list never leaves a name less than a narrower one did.  When
/// a name still has less than [`NAME_LEAST`], the State column narrows to
/// give it that, down to [`STATE_LEAST`].  Room beyond what the widest name
/// needs goes to the Target column, up to the widest of its texts or
/// [`TARGET_GROWN`], so that a long file name wraps as late as the name
/// allows.  The inspector shows every column.
pub(super) fn fitting(app: &App, layout: &Layout, room: f32) -> Vec<Fitted> {
    let all = shown(app, layout);
    let steps = giving_steps(&all);
    // Room of a name in columns of the font, counting the gutter of the mark
    let room_of = |columns: usize| (columns + layout.gutter()) as f32 * char_width();
    let wanted = room_of(widest_name(app, layout).clamp(NAME_LEAST, NAME_MOST));
    let name_room = |columns: &[Fitted]| {
        room - columns
            .iter()
            .map(|(_, width)| width.unwrap_or(0.0) + CELL_SPACE)
            .sum::<f32>()
    };
    let columns = giving_way(
        &steps,
        |gone| remaining(&all, gone),
        |columns: &Vec<_>| name_room(columns) >= wanted,
    );
    let deficit = room_of(NAME_LEAST) - name_room(&columns);
    let columns = squeezed(columns, deficit);
    let spare = (name_room(&columns) - wanted).max(0.0);
    grown(app, layout, columns, spare)
}

/// Columns that hold text in some row, each with the width of its widest
/// text, the State column wide enough for a failure when a cue fails
fn shown(app: &App, layout: &Layout) -> Vec<Fitted> {
    let groups = &layout.groups;
    let failing = faults::failing(app);
    let failed = groups.list().cues.iter().any(|cue| {
        app.workspace.problem(cue).is_none() && app.workspace.sends_to_failing(cue, &failing)
    });
    Column::VARIANTS
        .iter()
        .filter(|column| column.shown(&app.workspace, groups))
        .map(|&column| (column, column.width(&app.workspace, groups)))
        .map(|(column, width)| match column {
            Column::State if failed => (
                column,
                width.map(|width| width.max(FAILED_ROOM as f32 * char_width())),
            ),
            _ => (column, width),
        })
        .collect()
}

fn giving_steps(all: &[Fitted]) -> Vec<Step> {
    std::iter::once(Step::Narrow)
        .chain(
            Column::YIELDING
                .into_iter()
                .filter(|yielding| all.iter().any(|(column, _)| column == yielding))
                .map(Step::Drop),
        )
        .collect()
}

/// Width in columns of the font of the widest name, with its count and
/// the indent of its depth
fn widest_name(app: &App, layout: &Layout) -> usize {
    let groups = &layout.groups;
    groups
        .list()
        .cues
        .iter()
        .map(|cue| {
            let count = counted(&app.workspace, groups, cue).map_or(0, |count| count.width());
            one_line(&cue.name).width() + count + usize::from(layout.depth(cue)) * INDENT
        })
        .max()
        .unwrap_or(0)
}

fn remaining(all: &[Fitted], gone: &[Step]) -> Vec<Fitted> {
    let narrow = gone.contains(&Step::Narrow);
    all.iter()
        .filter(|(column, _)| !gone.contains(&Step::Drop(*column)))
        .map(|&(column, width)| match (column, width) {
            (Column::Target, Some(width)) if narrow => {
                (column, Some(width.min(TARGET_ROOM as f32 * char_width())))
            }
            shown => shown,
        })
        .collect()
}

/// Returns `columns` with the Target column grown by `spare`, up to the
/// widest of its texts or [`TARGET_GROWN`] columns of the font
fn grown(app: &App, layout: &Layout, columns: Vec<Fitted>, spare: f32) -> Vec<Fitted> {
    let most = Column::Target
        .widest(&app.workspace, &layout.groups)
        .min(TARGET_GROWN) as f32
        * char_width();
    columns
        .into_iter()
        .map(|(column, width)| match (column, width) {
            (Column::Target, Some(width)) => (column, Some(width.max((width + spare).min(most)))),
            shown => shown,
        })
        .collect()
}

#[cfg(test)]
mod tests;
