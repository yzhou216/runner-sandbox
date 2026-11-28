// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Levels tab of the inspector: the level of a cue with its slider, followed
//! by the sends of a cue that plays a file

use iced::{
    Center, Element,
    widget::{column, progress_bar, row, slider, text},
};

use super::{
    LEVEL_ROOM, light,
    output::{refused, styled, typing},
    sends,
};
use crate::{
    back::{Cue, Edit, Kind, LEVELS, Media},
    front::{
        App, Message, Setting,
        message::reasons::rejection,
        view::{SPACING, check, format},
    },
};

pub(super) fn view<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    let label = match cue.kind {
        Kind::Video if matches!(cue.media, Media::Loaded { sound: None, .. }) => {
            return text("The file of this cue holds no sound").into();
        }
        Kind::Audio | Kind::Video => "Level",
        Kind::Fade => "Fade target to",
        Kind::Light => return light::levels(app, cue),
        Kind::Picture
        | Kind::Start
        | Kind::Stop
        | Kind::Pause
        | Kind::Devamp
        | Kind::GoTo
        | Kind::Network
        | Kind::Group
        | Kind::Wait
        | Kind::Memo => {
            return text(format!("{} cues have no level", cue.kind)).into();
        }
    };
    let id = cue.id;
    let (field, typed) = typing(
        app,
        Setting::Level(id),
        "",
        format::decibels(cue.level),
        LEVEL_ROOM,
    );
    let reason = rejection(app, Setting::Level(id), &typed);
    let field = styled(app, field, &typed, "", LEVEL_ROOM, reason.is_none());
    column![
        row![text(label), slider_of(app, cue), field, text("dB")]
            .extend(reason.map(|reason| refused(&reason).into()))
            .spacing(SPACING)
            .align_y(Center),
        stop_target(app, cue),
        cue.target
            .and_then(|target| app.workspace.cue(target))
            .filter(|target| fades_nothing(cue, target))
            .map(|_| text("A picture has no level, so this fade changes nothing")),
        cue.kind.plays_file().then(|| sends::view(app, cue)),
    ]
    .spacing(SPACING)
    .into()
}

/// Returns whether the fade cue `cue` leaves `target` as it is: a picture has
/// no level to fade, and only a fade that stops it fades its picture out
fn fades_nothing(cue: &Cue, target: &Cue) -> bool {
    cue.kind == Kind::Fade && target.kind == Kind::Picture && !cue.stop_target
}

/// Returns the slider of the level of `cue`, or a bar where show mode locks
/// editing
fn slider_of<'a>(app: &App, cue: &Cue) -> Element<'a, Message> {
    let id = cue.id;
    if app.editable() {
        slider(LEVELS, cue.level, move |db| {
            Message::Edit(id, Edit::Level(db))
        })
        .step(0.5_f32)
        .into()
    } else {
        progress_bar(LEVELS, cue.level).into()
    }
}

fn stop_target<'a>(app: &'a App, cue: &Cue) -> Option<Element<'a, Message>> {
    let id = cue.id;
    (cue.kind == Kind::Fade).then(|| {
        check(
            "Stop target when done",
            cue.stop_target,
            app.editable()
                .then_some(move |stop| Message::Edit(id, Edit::StopTarget(stop))),
            &app.config.keymap,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::fades_nothing;
    use crate::back::{Cue, CueId, Kind};

    fn cue(kind: Kind, stop_target: bool) -> Cue {
        Cue {
            stop_target,
            ..Cue::new(CueId(1), "1", kind, "Cue").build()
        }
    }

    #[test]
    fn a_fade_that_keeps_a_picture_has_nothing_to_fade() {
        let picture = cue(Kind::Picture, true);
        assert!(fades_nothing(&cue(Kind::Fade, false), &picture));
        assert!(!fades_nothing(&cue(Kind::Fade, true), &picture));
        assert!(!fades_nothing(
            &cue(Kind::Fade, false),
            &cue(Kind::Video, true)
        ));
    }
}
