// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Forms of the tabs that hold two, which lie side by side while each has
//! room for its values, share the width in the proportions that the tab
//! sets, and give way to one another only as far as that room, and lie one
//! above the other, in a scrollable that Tab scrolls to the field that it
//! reaches, when the width is too short for both

use iced::{
    Element, Fill, FillPortion, Length, Size,
    widget::{column, container, responsive, row, scrollable, table, text},
};
use unicode_width::UnicodeWidthStr;

use super::field::Field;
use crate::{
    back::Cue,
    front::{
        App, Message,
        view::{FORMS, SPACING, char_width, cue_table::CELL_SPACE, dropdown_width, widest},
    },
};

/// Columns of the font that a value keeps its room for in a form, enough to
/// read a time or the first words of a name
const VALUE_ROOM: usize = 10;

/// Returns the fields of `fields` that the cue has a row for
fn rows<'a>(cue: &'a Cue, fields: &'a [Field]) -> impl Iterator<Item = Field> + 'a {
    fields
        .iter()
        .map(|field| field.of(cue))
        .filter(|field| field.has_row(cue))
}

fn form<'a>(app: &'a App, cue: &'a Cue, fields: &[Field]) -> Element<'a, Message> {
    table::table(
        [
            table::column(text("Setting"), move |field: Field| text(field.label(cue))),
            table::column(text("Value"), move |field: Field| field.input(app, cue)).width(Fill),
        ],
        rows(cue, fields),
    )
    .into()
}

/// Returns the width that the form of `fields` needs: its widest label, its
/// header, and a value of [`VALUE_ROOM`] columns, which a drop-down list takes
/// more room for than a text field
fn least(cue: &Cue, fields: &[Field]) -> f32 {
    let label = widest(rows(cue, fields).map(|field| field.label(cue))).max("Setting".width());
    label as f32 * char_width() + CELL_SPACE + dropdown_width(VALUE_ROOM as f32) + CELL_SPACE
}

/// Returns the forms of `columns` side by side, which share the width in the
/// proportions of `shares`, with what `below` makes under them, or one above
/// the other where the width is too short for the least of both
pub(super) fn forms<'a, B>(
    app: &'a App,
    cue: &'a Cue,
    columns: [&'a [Field]; 2],
    shares: [u16; 2],
    below: B,
) -> Element<'a, Message>
where
    B: Fn() -> Option<Element<'a, Message>> + 'a,
{
    responsive(move |size| {
        let leasts = columns.map(|fields| least(cue, fields));
        let widths = widths(size, leasts, shares);
        let forms = columns.map(|fields| form(app, cue, fields));
        let content: Element<'a, Message> = match widths {
            Some(widths) => row(forms
                .into_iter()
                .zip(widths)
                .map(|(form, width)| container(form).width(width).into()))
            .spacing(SPACING)
            .into(),
            None => scrollable(column(forms).spacing(SPACING).width(Fill))
                .spacing(SPACING)
                .id(FORMS)
                .width(Fill)
                .height(Fill)
                .into(),
        };
        column![content].extend(below()).spacing(SPACING).into()
    })
    .into()
}

/// Returns the widths of two forms side by side in `size`, or none where
/// they cannot both keep their `leasts`.  The forms share the width as
/// `shares` say, but the one that would get less than its least gets that
/// and leaves the other the rest.
fn widths(size: Size, leasts: [f32; 2], shares: [u16; 2]) -> Option<[Length; 2]> {
    let room = size.width - SPACING;
    let [first, second] = shares.map(f32::from);
    let wanted = [
        room * first / (first + second),
        room * second / (first + second),
    ];
    match (wanted[0] >= leasts[0], wanted[1] >= leasts[1]) {
        (true, true) => Some(shares.map(FillPortion)),
        (false, _) if room >= leasts[0] + leasts[1] => {
            Some([Length::Fixed(leasts[0]), Length::Fixed(room - leasts[0])])
        }
        (_, false) if room >= leasts[0] + leasts[1] => {
            Some([Length::Fixed(room - leasts[1]), Length::Fixed(leasts[1])])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use iced::{FillPortion, Length, Size};

    use super::widths;
    use crate::front::view::SPACING;

    const LEASTS: [f32; 2] = [250.0, 270.0];

    fn within(width: f32) -> Option<[Length; 2]> {
        widths(Size::new(width, 240.0), LEASTS, [3, 2])
    }

    #[test]
    fn forms_with_room_share_the_width_as_the_tab_says() {
        assert_eq!(within(900.0), Some([FillPortion(3), FillPortion(2)]));
    }

    #[test]
    fn the_form_that_the_shares_would_leave_less_than_its_least_keeps_its_least() {
        let room = 560.0;
        assert_eq!(
            within(room + SPACING),
            Some([Length::Fixed(room - LEASTS[1]), Length::Fixed(LEASTS[1])])
        );
    }

    #[test]
    fn forms_lie_one_above_the_other_where_they_cannot_both_keep_their_least() {
        assert_eq!(within(LEASTS[0] + LEASTS[1] + SPACING - 1.0), None);
        assert!(within(LEASTS[0] + LEASTS[1] + SPACING).is_some());
    }
}
