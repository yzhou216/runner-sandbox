// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the State column for a network cue whose destination failed

use std::collections::BTreeSet;

use super::{
    super::columns::states::{Look, lines},
    yellows,
};
use crate::back::{Cue, CueId, Kind, fresh};

fn network(id: u32, destination: &str) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Network, "Fire")
        .destination(destination)
        .message("/eos/cue/1/fire")
        .build()
}

#[test]
fn every_cue_that_names_a_destination_that_failed_reads_in_yellow_that_the_send_failed() {
    let workspace = fresh(vec![
        network(1, "10.0.0.1:3037"),
        network(2, "10.0.0.1:3037"),
        network(3, "10.0.0.2:3037"),
        network(4, "udp://10.0.0.1:3037"),
    ]);
    let failed = BTreeSet::from(["tcp://10.0.0.1:3037".to_owned()]);
    let read = |index: usize, next: bool| {
        let cue = &workspace.lists()[0].cues[index];
        let look = Look::default().with_next(next).with_unreached(&failed);
        yellows(lines(&workspace, cue, look))
    };
    let told = || ("SEND FAILED".to_owned(), true);
    assert_eq!(read(0, false), [told()]);
    // Also while it only waits as NEXT
    assert_eq!(read(1, true), [("NEXT".to_owned(), false), told()]);
    assert_eq!(read(2, false), []);
    assert_eq!(read(3, false), []);
}

#[test]
fn a_cue_reads_no_send_failure_once_the_problem_clears() {
    let workspace = fresh(vec![network(1, "10.0.0.1:3037")]);
    let cue = &workspace.lists()[0].cues[0];
    let read = |failed: &BTreeSet<String>| {
        lines(&workspace, cue, Look::default().with_unreached(failed)).len()
    };
    assert_eq!(read(&BTreeSet::from(["tcp://10.0.0.1:3037".to_owned()])), 1);
    assert_eq!(read(&BTreeSet::new()), 0);
}
