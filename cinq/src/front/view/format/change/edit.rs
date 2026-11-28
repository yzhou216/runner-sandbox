// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What an edit of one setting of a cue says of itself

use crate::back::Edit;

/// Describes the edit `redo` of the cue numbered `cue`
pub(super) fn edited(cue: &str, redo: &Edit) -> String {
    // Refused time stays in the field and counts as zero, except an end or a
    // loop time, which reads as empty
    let unread = match redo {
        Edit::Start(time) | Edit::PreWait(time) | Edit::Duration(time) | Edit::PostWait(time)
            if !time.is_valid() =>
        {
            Some("zero")
        }
        Edit::End(time) | Edit::LoopStart(time) | Edit::LoopEnd(time) if !time.is_valid() => {
            Some("empty")
        }
        _ => None,
    };
    let of = |part: &str| format!("changed the {part} of cue {cue}");
    let told = match redo {
        Edit::Number(_) => of("number"),
        Edit::Name(_) => of("name"),
        Edit::File(_) | Edit::FileAndName { .. } => of("file"),
        Edit::Start(_) => of("start"),
        Edit::End(_) => of("end"),
        Edit::Loop(_) => format!("changed whether cue {cue} loops"),
        Edit::LoopStart(_) => of("loop start"),
        Edit::LoopEnd(_) => of("loop end"),
        Edit::Target(_) => of("target"),
        Edit::Notes(_) => of("notes"),
        Edit::PreWait(_) => of("pre-wait"),
        Edit::Duration(_) => of("duration"),
        Edit::PostWait(_) => of("post-wait"),
        Edit::Advance(_) => format!("changed what follows cue {cue}"),
        Edit::Level(_) => of("level"),
        Edit::StopTarget(_) => format!("changed whether cue {cue} stops its target"),
        Edit::Sends(Some(sends)) if sends.is_empty() => {
            format!("changed cue {cue} to play nowhere")
        }
        Edit::Sends(_) => format!("changed where cue {cue} plays"),
        Edit::Light(..) => of("light levels"),
        Edit::Destination(_) => of("destination"),
        Edit::Message(_) => of("message"),
    };
    match unread {
        Some(reading) => format!("{told} to {reading}"),
        None => told,
    }
}
