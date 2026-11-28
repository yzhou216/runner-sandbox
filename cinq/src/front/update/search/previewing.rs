// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The groups that the preview of a search expands and closes again

use super::Search;
use crate::front::App;

impl App {
    /// Closes the groups that the preview of the search expanded, so that
    /// only those that hide the match of the moment stay open
    pub(super) fn search_reclosed(self) -> Self {
        let opened = self
            .search
            .as_ref()
            .map(|search| search.opened.clone())
            .unwrap_or_default();
        let app = self.collapsing(opened, true);
        Self {
            search: app.search.map(|search| Search {
                opened: Vec::new(),
                ..search
            }),
            ..app
        }
    }

    /// Expands the collapsed groups that hide the selected match, and
    /// remembers them for the preview to close
    pub(super) fn search_previewed(self) -> Self {
        let hiding = self.hiding_selection();
        let app = self.with_selection_unhidden();
        Self {
            search: app.search.map(|search| Search {
                opened: hiding,
                ..search
            }),
            ..app
        }
    }
}
