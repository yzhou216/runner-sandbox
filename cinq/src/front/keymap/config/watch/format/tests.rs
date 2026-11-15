// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of formatting the file after it changes

use std::{fs, io};

use tempfile::TempDir;

use super::{formatted, reformat};

#[test]
fn spaces_and_blank_lines_even_out_and_comments_line_up() {
    let sloppy = "# Keys\nkeymap=\"booth\"\n\n\n\n[keymaps.booth]\n  from  =  \"cua\"   # base\n\
                  go=[ \"Space\",\"F5\" ]\nfly = { action=\"go\" }";
    let tidy = "# Keys\nkeymap = \"booth\"\n\n\n[keymaps.booth]\nfrom = \"cua\"            # base\n\
                go = [\"Space\", \"F5\"]\nfly = { action = \"go\" }\n";
    assert_eq!(formatted(sloppy).as_deref(), Some(tidy));
    assert_eq!(formatted(tidy), None);
}

#[test]
fn keys_and_items_keep_their_order_while_arrays_fit_the_width() {
    assert_eq!(
        formatted("zeta = 1\nalpha = 2\nnext = [\"Z\", \"A\"]\n"),
        None
    );
    let several = "next = [\n  \"PageDown\",\n  \"ArrowDown\",\n]\n";
    assert_eq!(
        formatted(several).as_deref(),
        Some("next = [\"PageDown\", \"ArrowDown\"]\n")
    );
    let long = format!("next = [{}]\n", ["\"PageDown\""; 12].join(", "));
    let unfolded = format!("next = [\n{}]\n", "  \"PageDown\",\n".repeat(12));
    assert_eq!(formatted(&long), Some(unfolded));
}

#[test]
fn line_endings_stay_and_what_is_not_toml_stays() {
    assert_eq!(
        formatted("a=1\r\nb=2\r\n").as_deref(),
        Some("a = 1\r\nb = 2\r\n")
    );
    assert_eq!(formatted("next = [\"F5\"\n"), None);
}

#[test]
fn reformatting_writes_only_toml_that_formatting_changes() -> io::Result<()> {
    let home = TempDir::new()?;
    let file = home.path().join("config.toml");
    fs::write(&file, "keymap=\"cua\"\n")?;
    reformat(&file);
    assert_eq!(fs::read_to_string(&file)?, "keymap = \"cua\"\n");
    fs::write(&file, "keymap=\n")?;
    reformat(&file);
    assert_eq!(fs::read_to_string(&file)?, "keymap=\n");
    Ok(())
}
