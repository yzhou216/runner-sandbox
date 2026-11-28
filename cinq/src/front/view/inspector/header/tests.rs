// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how the header of the inspector shares its line between the
//! title and the sentences of trouble

use iced::widget::text;
use unicode_width::UnicodeWidthStr;

use super::{Sentence, TITLE_LEAST, fit, gap};

const TITLE: &str = "21 A rather long name of a cue for this test, Audio cue";

fn sentence(words: &str) -> Sentence {
    Sentence {
        words: words.to_owned(),
        style: text::danger,
    }
}

fn width(title: &str, sentences: &[Sentence]) -> usize {
    title.width()
        + sentences
            .iter()
            .map(|sentence| gap() + sentence.words.width())
            .sum::<usize>()
}

#[test]
fn a_title_with_no_sentence_beside_it_takes_the_line() {
    let (title, sentences) = fit(TITLE, Vec::new(), 40);
    assert_eq!(title.width(), 40);
    assert!(sentences.is_empty());
}

#[test]
fn the_title_gives_up_the_room_that_a_sentence_takes() {
    let words = "GO skips this cue: file not found";
    let (title, sentences) = fit(TITLE, vec![sentence(words)], 80);
    assert_eq!(sentences[0].words, words);
    assert_eq!(width(&title, &sentences), 80);
}

#[test]
fn a_sentence_that_does_not_fit_beside_the_least_title_is_cut_short() {
    let words = "GO skips this cue: no OSC message as /address arguments";
    let (title, sentences) = fit(TITLE, vec![sentence(words)], 64);
    assert_eq!(title.width(), TITLE_LEAST);
    assert!(sentences[0].words.ends_with('\u{2026}'));
    assert_eq!(width(&title, &sentences), 64);
}

#[test]
fn the_first_sentence_keeps_its_words_before_the_second_is_cut() {
    let first = "GO skips this cue: file not found";
    let second = "Post-wait counts as zero: use seconds or minutes:seconds";
    let kept = "Post-wait counts as zero";
    // Room for the kept words and the ellipsis, as the gaps take more columns
    // where the font is narrower
    let columns = TITLE_LEAST + 2 * gap() + first.width() + kept.width() + 1;
    let (title, sentences) = fit(TITLE, vec![sentence(first), sentence(second)], columns);
    assert_eq!(sentences[0].words, first);
    assert!(sentences[1].words.starts_with(kept));
    assert!(sentences[1].words.ends_with('\u{2026}'));
    assert!(width(&title, &sentences) <= columns);
}

#[test]
fn a_sentence_with_no_room_left_is_left_out() {
    let first = "GO skips this cue: no OSC message as /address arguments";
    let second = "Pre-wait counts as zero: use seconds or minutes:seconds";
    let (title, sentences) = fit(TITLE, vec![sentence(first), sentence(second)], 64);
    assert_eq!(sentences.len(), 1);
    assert_eq!(width(&title, &sentences), 64);
}
