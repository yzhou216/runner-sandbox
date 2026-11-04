// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Date printed in the manual pages.

use std::{env, path::Path, process::Command};

/// Returns the date from `SOURCE_DATE_EPOCH`, else of the last commit.
pub fn release_date(root: &Path) -> String {
    env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|epoch| epoch.parse::<i64>().ok())
        .map(civil_date)
        .or_else(|| last_commit_date(root))
        .unwrap_or_else(|| "unreleased".to_owned())
}

fn last_commit_date(root: &Path) -> Option<String> {
    Command::new("git")
        .args(["log", "-1", "--format=%cs"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|date| date.trim().to_owned())
        .filter(|date| !date.is_empty())
}

/// Formats POSIX epoch seconds as a UTC date, by Howard Hinnant's
/// `civil_from_days`.
fn civil_date(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epochs_become_calendar_dates() {
        [
            (0, "1970-01-01"),
            (951_782_400, "2000-02-29"),
            (1_793_491_200, "2026-11-01"),
            (1_793_577_599, "2026-11-01"),
            (-86_400, "1969-12-31"),
        ]
        .into_iter()
        .for_each(|(seconds, date)| assert_eq!(civil_date(seconds), date, "{seconds}"));
    }
}
