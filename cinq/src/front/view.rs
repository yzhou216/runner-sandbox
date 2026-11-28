// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! View functions, which keep iced's default look, apart from the
//! exceptions that [`theme`] lists, and only space the controls apart

mod cue_table;
mod devices;
mod faults;
pub(super) mod format;
mod help;
mod ids;
pub use ids::{
    ADDRESS, BUTTON, CHECK, CUES, CURRENT_LIST, FILE, FORMS, HELP, INSPECTOR, NAME, OUTPUT_NAME,
    OUTPUTS, SEARCH, SELECTED, SENDS, SIDEBAR_SCROLL,
};
mod inspector;
pub(super) use cue_table::searchable as cue_texts;
mod next_keys;
mod outputs;
pub(super) use outputs::searchable as output_texts;
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
use unicode_width::UnicodeWidthStr;

use super::{
    App, Message,
    keymap::{Action, Keymap},
};
pub(super) use widget::{Act, Site, hint};
use widget::{
    button, check, check_and_leave, dropdown, dropdown_width, hide, hinted, keep_keys, line_keys,
    marked_row, radio, reached,
};

pub const NO_MATCH: &str = "No cue matches";

const NO_OUTPUT_MATCH: &str = "No output matches";

/// Returns what the search line and the status bar tell when no row matches,
/// for the rows that show
pub fn no_match(outputs: bool) -> &'static str {
    if outputs { NO_OUTPUT_MATCH } else { NO_MATCH }
}

/// Monospace, so that columns of text line up
pub const FONT: Font = Font::MONOSPACE;

pub const TEXT_SIZE: Pixels = Pixels(16.0);

fn height_of(size: Pixels) -> f32 {
    f32::from(LineHeight::default().to_absolute(size))
}

pub fn line_height() -> f32 {
    height_of(TEXT_SIZE)
}

pub const SPACING: f32 = 10.0;

const SIDEBAR: f32 = 560.0;

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

/// Layers that come and go lie over those that stay, since iced matches the
/// state of each widget to the widget in the same place, and leaves a missing
/// layer out of its stack.  That keeps the cue list scrolled and the fields
/// filled as the layers come and go.  The window beneath the help page is
/// hidden rather than left out, so that it comes back as it was, with the
/// keyboard focus where it was.
pub fn view(app: &App) -> Element<'_, Message> {
    let parts = responsive(move |window| {
        column![
            transport::view(app, window.width),
            rule::horizontal(1),
            toolbar::view(app, window.width),
            rule::horizontal(1),
            responsive(move |size| {
                let sidebar = sidebar::width(size.width);
                row![
                    container(if app.outputs_shown {
                        outputs::view(app)
                    } else {
                        cue_table::view(app)
                    })
                    .clip(true),
                    rule::vertical(1),
                    sidebar::view(app, sidebar),
                ]
                .into()
            }),
            rule::horizontal(1),
            inspector::view(app, window),
        ]
        .into()
    });
    let window = column![
        stack![parts, next_keys::view(app)],
        rule::horizontal(1),
        status_bar::view(app),
    ];
    stack![
        widget::window(stack![
            hide(window, app.help),
            app.help.then(|| help::view(app))
        ]),
        app.hints.as_ref().map(|hints| hint::layer(hints.labels()))
    ]
    .into()
}

/// Returns the first result of `shown` that `fits` accepts as the items of
/// `yielding` give way in turn, the first to go first, or else the result
/// with all of them gone
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
pub(super) fn char_width() -> f32 {
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

/// Returns the columns of the font that the widest of `texts` takes, two for
/// a wide character such as a Chinese one
fn widest<S: AsRef<str>>(texts: impl IntoIterator<Item = S>) -> usize {
    texts
        .into_iter()
        .map(|text| text.as_ref().width())
        .max()
        .unwrap_or(0)
}

pub(super) fn warnings(app: &App) -> usize {
    app.workspace.problems().count()
        + app.workspace.stranded().count()
        + app.workspace.unlit_cues().count()
        + faults::warnings(app).len()
        + app.problems.listed()
        + usize::from(app.stage.missing(&app.workspace))
        + app.config.problems.len()
}
