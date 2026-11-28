// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Names cut short to fit their room, in a list, the status bar, or a notice

#[cfg(test)]
mod tests;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(in crate::front::view) const ELLIPSIS: &str = "\u{2026}";

/// Returns the columns that `character` takes.  A control character takes
/// one, as it does in the width of a string, so that a cut name is never
/// wider than the room that the width of the whole name was measured against.
fn columns_of(character: char) -> usize {
    character.width().unwrap_or(1)
}

/// Returns the characters of `chars` that fit in `columns` columns of the
/// font, which is monospace, with the last column left for the ellipsis
fn kept(chars: impl Iterator<Item = char>, columns: usize) -> impl Iterator<Item = char> {
    chars.scan(0, move |taken, character| {
        *taken += columns_of(character);
        (*taken < columns).then_some(character)
    })
}

/// Returns `name` cut short to `columns` columns of the font, which is
/// monospace, with an ellipsis in place of the rest
pub(in crate::front::view) fn shortened(name: &str, columns: usize) -> String {
    if name.width() <= columns {
        return name.to_owned();
    }
    format!(
        "{}{ELLIPSIS}",
        kept(name.chars(), columns).collect::<String>()
    )
}

/// Returns `name` cut short to `columns` columns with an ellipsis in place of
/// its start, since names that share a beginning, as `House speaker pair 01`
/// and `House speaker pair 02` do, differ at the end
pub(in crate::front::view) fn shortened_start(name: &str, columns: usize) -> String {
    if name.width() <= columns {
        return name.to_owned();
    }
    let tail = kept(name.chars().rev(), columns).collect::<Vec<_>>();
    let tail = tail.into_iter().rev().skip_while(|&c| columns_of(c) == 0);
    format!("{ELLIPSIS}{}", tail.collect::<String>())
}
