// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hints about the stage window

use strum::VariantArray;

use super::{
    super::hints::{acts_here, notices},
    added, has_hint, hinted, opened, opened_with,
};
use crate::{
    back::{Command, Cue, CueId, Kind, fresh, loaded, silent},
    front::{
        App,
        keymap::{Action, Keymap, desk},
        update::Stage,
    },
};

/// Returns the state with a video cue that shows once `go` fires it, and a
/// stage window that the operator closed
fn closed(go: bool) -> App {
    let cue = Cue::new(CueId(1), "1", Kind::Video, "Film")
        .file("film.mp4")
        .build();
    let workspace = fresh(vec![cue]).apply(loaded(1, "film.mp4", silent(10.0)));
    let workspace = if go {
        workspace.apply(Command::Go { list: 0 })
    } else {
        workspace
    };
    App {
        workspace,
        stage: Stage::closed(),
        ..opened(false, false)
    }
}

#[test]
fn a_closed_stage_window_that_a_cue_shows_on_hints_its_key_but_not_over_the_help_page() {
    let told = |app: App| has_hint(&app, "F11: open stage");
    assert!(told(closed(true)));
    assert!(!told(closed(false)), "nothing to show");
    let open = App {
        stage: Stage::default(),
        ..closed(true)
    };
    assert!(!told(open));
    let help = App {
        help: true,
        ..closed(true)
    };
    assert!(!told(help));
}

#[test]
fn a_closed_stage_window_that_a_cue_shows_on_counts_as_a_warning() {
    let counted = |app: &App| crate::front::view::warnings(app);
    let nothing = counted(&closed(false));
    assert_eq!(counted(&closed(true)), nothing + 1);
    let open = App {
        stage: Stage::default(),
        ..closed(true)
    };
    assert_eq!(counted(&open), nothing);
}

fn keyboard(keymap: Keymap) -> App {
    App {
        stage: Stage::holding_keyboard(),
        ..opened_with(keymap, false, false)
    }
}

#[test]
fn the_stage_window_with_the_keyboard_gets_only_the_hints_of_keys_that_act_there() {
    [Keymap::default(), desk("")]
        .into_iter()
        .for_each(|keymap| {
            let hints = hinted(&App {
                workspace: added(),
                ..keyboard(keymap)
            });
            let words = hints
                .iter()
                .map(|(_, hint)| hint.as_str())
                .collect::<Vec<_>>();
            assert!(words.contains(&"F11: fullscreen"), "{words:?}");
            assert!(words.contains(&"Space: GO"), "{words:?}");
            assert!(
                hints
                    .iter()
                    .filter_map(|(action, _)| *action)
                    .all(|action| !action.stays_in_main_window()),
                "{words:?}"
            );
            assert!(words.iter().all(|hint| !hint.contains("undo")), "{words:?}");
        });
}

#[test]
fn the_notice_of_a_change_names_no_undo_while_the_stage_window_has_the_keyboard() {
    let app = |stage| App {
        workspace: added(),
        stage,
        ..opened(false, false)
    };
    let notice = |app: &App| notices::notice(app).expect("a change was made");
    assert!(notice(&app(Stage::default())).contains("u: undo"));
    assert!(!notice(&app(Stage::holding_keyboard())).contains("undo"));
}

#[test]
fn the_hints_and_the_refusals_agree_on_the_keys_that_act_in_the_stage_window() {
    let app = keyboard(Keymap::default());
    Action::VARIANTS.iter().for_each(|action| {
        assert_eq!(
            acts_here(&app, *action),
            app.refusal(*action).is_none(),
            "{action:?}"
        );
    });
}
