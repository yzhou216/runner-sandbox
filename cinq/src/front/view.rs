// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! View functions, which keep iced's default look, apart from the
//! exceptions that [`theme`] lists, and only space the controls apart

mod cue_table;
mod format;
mod help;
mod inspector;
mod next_keys;
mod sidebar;
pub(super) mod stage;
mod status_bar;
pub(super) mod theme;
mod toolbar;
mod transport;
mod widget;

use std::sync::LazyLock;

use iced::{
    Element, Font, Pixels, Size,
    advanced::text::{self as core_text, Paragraph as _},
    alignment,
    widget::{
        column, container, responsive, row, rule, stack,
        text::{LineHeight, Shaping, Wrapping},
    },
};
use itertools::Itertools;

use super::{
    App, Message,
    keymap::{Action, Keymap},
};
use widget::{button, dropdown, hide, keep_keys, marked_row};

/// Identity of the cue list, through which the selection scrolls into view
pub const CUES: &str = "cues";

/// Identity of the selected row, through which the list finds where to
/// scroll
pub const SELECTED: &str = "selected";

/// Identity of the help page, which the keys that move the selection
/// scroll while it is open
pub const HELP: &str = "help";

/// Identity of the Name field, through which the key of the edit action
/// gives it the keyboard focus
pub const NAME: &str = "name";

/// Monospace, so that columns of text line up
pub const FONT: Font = Font::MONOSPACE;

pub const TEXT_SIZE: Pixels = Pixels(16.0);

pub fn line_height() -> f32 {
    f32::from(LineHeight::default().to_absolute(TEXT_SIZE))
}

const SPACING: f32 = 10.0;

/// Widest that the sidebar grows
const SIDEBAR: f32 = 300.0;

/// Line between the parts of the status bar, and between a message and the
/// hint of the key that answers it
const LINE: &str = " | ";

/// Returns `told` followed by the hint of the key of `action`, which
/// answers it, when `keymap` binds the action to a key
pub fn answered(told: String, keymap: &Keymap, action: Action) -> String {
    match keymap.hint(action, action.name()) {
        Some(hint) => format!("{told}{LINE}{hint}"),
        None => told,
    }
}

/// Layers that come and go lie over those that stay, since iced leaves a
/// missing layer out of its stack and matches the state of each widget to
/// the widget in the same place, so that the cue list stays scrolled where
/// it was, and the fields keep what they hold, as the layers come and go.
/// The help page takes the whole window, and the window beneath it is
/// hidden rather than left out, so that it comes back as it was, with the
/// keyboard focus where it was.
pub fn view(app: &App) -> Element<'_, Message> {
    let parts = column![
        transport::view(app),
        rule::horizontal(1),
        toolbar::view(app),
        rule::horizontal(1),
        responsive(move |size| {
            // Sidebar takes at most a quarter of the width, so that a narrow
            // window, such as one tiled beside another, leaves the cue list
            // room
            let sidebar = (size.width / 4.0).min(SIDEBAR);
            row![
                container(cue_table::view(app)).clip(true),
                rule::vertical(1),
                sidebar::view(app, sidebar),
            ]
            .into()
        }),
        rule::horizontal(1),
        inspector::view(app),
    ];
    let window = column![
        stack![parts, next_keys::view(app)],
        rule::horizontal(1),
        status_bar::view(app),
    ];
    stack![hide(window, app.help), app.help.then(|| help::view(app))].into()
}

/// Returns what `shown` makes of the items of `yielding` that have given
/// way, as they give way in turn, the first to go first: the first result
/// that `fits` accepts, or else the one in which all of them have gone
fn giving_way<Y, T>(yielding: &[Y], shown: impl Fn(&[Y]) -> T, fits: impl Fn(&T) -> bool) -> T {
    (0..=yielding.len())
        .map(|gone| shown(&yielding[..gone]))
        .find_or_last(fits)
        .expect("there is always the way in which none give way")
}

type Paragraph = <iced::Renderer as core_text::Renderer>::Paragraph;

/// Returns the width of one column of the font, which is monospace, so
/// that a line of text is as wide as the columns that it takes, two for a
/// wide character.  It is measured once, as the font stays the same while
/// the program runs.
fn char_width() -> f32 {
    static WIDTH: LazyLock<f32> = LazyLock::new(|| {
        Paragraph::with_text(core_text::Text {
            content: "0",
            bounds: Size::INFINITE,
            size: TEXT_SIZE,
            line_height: LineHeight::default(),
            font: FONT,
            align_x: core_text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::Basic,
            wrapping: Wrapping::None,
        })
        .min_width()
    });
    *WIDTH
}

/// Counts the warnings that the sidebar lists
fn warnings(app: &App) -> usize {
    app.workspace.problems().count()
        + usize::from(app.audio_error.is_some())
        + usize::from(app.file_problem.is_some())
        + usize::from(app.keep_problem.is_some())
        + app.config.problems.len()
}
