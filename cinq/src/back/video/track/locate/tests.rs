// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where the samples of a track lie, in chunks of the sizes that
//! its tables give, and of the tables that cannot be right

use mp4_atom::{Co64, Stbl, Stco, Stsc, StscEntry, Stsz, StszSamples};

use super::{Place, places};

fn runs(runs: &[(u32, u32)]) -> Stsc {
    Stsc {
        entries: runs
            .iter()
            .map(|&(first_chunk, samples_per_chunk)| StscEntry {
                first_chunk,
                samples_per_chunk,
                sample_description_index: 1,
            })
            .collect(),
    }
}

/// Returns the tables of a track whose chunks start at `offsets`, which
/// group their samples in the runs of `groups`, and whose samples have
/// `sizes`
fn tables(offsets: &[u32], groups: &[(u32, u32)], sizes: &[u32]) -> Stbl {
    Stbl {
        stsc: runs(groups),
        stco: Some(Stco {
            entries: offsets.to_vec(),
        }),
        stsz: Stsz {
            samples: StszSamples::Different {
                sizes: sizes.to_vec(),
            },
        },
        ..Stbl::default()
    }
}

fn place(offset: u64, size: u32) -> Place {
    Place { offset, size }
}

#[test]
fn samples_lie_one_after_another_in_each_chunk() {
    let stbl = tables(&[100, 200, 300], &[(1, 2), (3, 1)], &[10, 20, 30, 40, 50]);
    assert_eq!(
        places(&stbl, 1000),
        Some(vec![
            place(100, 10),
            place(110, 20),
            place(200, 30),
            place(230, 40),
            place(300, 50),
        ])
    );
}

#[test]
fn chunks_of_64_bits_hold_samples_where_those_of_32_bits_are_none() {
    let stbl = Stbl {
        stco: None,
        co64: Some(Co64 {
            entries: vec![1 << 40],
        }),
        ..tables(&[], &[(1, 2)], &[7, 8])
    };
    assert_eq!(
        places(&stbl, 1 << 41),
        Some(vec![place(1 << 40, 7), place((1 << 40) + 7, 8)])
    );
}

#[test]
fn samples_of_one_size_follow_each_other_in_their_chunks() {
    let stbl = Stbl {
        stsz: Stsz {
            samples: StszSamples::Identical { count: 4, size: 5 },
        },
        ..tables(&[0, 100], &[(1, 3), (2, 1)], &[])
    };
    assert_eq!(
        places(&stbl, 100),
        Some(vec![place(0, 5), place(5, 5), place(10, 5), place(100, 5)])
    );
}

#[test]
fn samples_end_where_the_chunks_or_the_sizes_run_out() {
    // Three samples fit the chunks, and only two have sizes
    let sizes = tables(&[0, 50], &[(1, 2)], &[4, 4]);
    assert_eq!(places(&sizes, 100), Some(vec![place(0, 4), place(4, 4)]));
    // Four have sizes, and the third chunk is not in the table of chunks
    let chunks = tables(&[0, 50], &[(1, 1)], &[4, 4, 4, 4]);
    assert_eq!(places(&chunks, 100), Some(vec![place(0, 4), place(50, 4)]));
    assert_eq!(places(&tables(&[], &[(1, 1)], &[4]), 100), Some(vec![]));
    assert_eq!(places(&tables(&[0], &[], &[4]), 100), Some(vec![]));
}

#[test]
fn offsets_that_overflow_stop_at_the_largest_number() {
    let stbl = Stbl {
        stco: None,
        co64: Some(Co64 {
            entries: vec![u64::MAX - 5],
        }),
        ..tables(&[], &[(1, 2)], &[10, 10])
    };
    assert_eq!(
        places(&stbl, 100),
        Some(vec![place(u64::MAX - 5, 10), place(u64::MAX, 10)])
    );
}

#[test]
fn counts_that_the_file_cannot_hold_give_no_samples() {
    let identical = |count, size| Stbl {
        stsz: Stsz {
            samples: StszSamples::Identical { count, size },
        },
        ..tables(&[0], &[(1, u32::MAX)], &[])
    };
    assert_eq!(
        places(&identical(20, 5), 100).map(|places| places.len()),
        Some(20)
    );
    assert_eq!(places(&identical(21, 5), 100), None);
    assert_eq!(places(&identical(u32::MAX, u32::MAX), 1 << 40), None);
}

#[test]
fn samples_of_one_size_that_no_h264_sample_has_give_no_samples() {
    let identical = |size| Stbl {
        stsz: Stsz {
            samples: StszSamples::Identical { count: 100, size },
        },
        ..tables(&[0], &[(1, u32::MAX)], &[])
    };
    // Samples of no bytes fit any number in a file, and those of fewer than
    // five bytes make one for nearly every byte of it
    assert_eq!(places(&identical(0), 1 << 30), None);
    assert_eq!(places(&identical(4), 1 << 30), None);
    assert_eq!(
        places(&identical(5), 1 << 30).map(|places| places.len()),
        Some(100)
    );
}

#[test]
fn runs_out_of_order_give_no_samples() {
    let stbl = tables(&[0, 10, 20], &[(1, 1), (3, 1), (2, 1)], &[1, 1, 1]);
    assert_eq!(places(&stbl, 100), None);
}
