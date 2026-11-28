// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the notices, undo and redo, and the keys that enter the name, for outputs

mod naming;
mod telling;
mod undoing;

use super::{ALL, locked, names, on, on_clipboard, typed, without};
use crate::{
    back::{PatchEdit, Place},
    front::{
        App, Caret, Focus, Message, Setting, Tab,
        keymap::Action,
        update::tests::{
            after,
            history::{cue, place},
            noticed,
            output_rows::at,
            paging::letter,
            press, show, told, typed_setting,
        },
    },
};
