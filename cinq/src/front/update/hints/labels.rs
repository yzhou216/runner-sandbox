// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Labels that hint mode gives out

/// Letters that labels are made of, Vimium's default, which lie on or near
/// the home row
const LETTERS: &str = "sadfjklewcmpgh";

/// Returns `count` labels in the order that they are given out.  All have the
/// same length, which is as few letters as the count needs, so that no label
/// begins another and typing one whole never has to wait for more.
pub fn labels(count: usize) -> Vec<String> {
    let letters: Vec<char> = LETTERS.chars().collect();
    let base = letters.len();
    let length = (1_u32..)
        .find(|&length| base.saturating_pow(length) >= count)
        .unwrap_or(1);
    (0..count)
        .map(|number| {
            (0..length)
                .rev()
                .map(|place| letters[number / base.pow(place) % base])
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests;
