// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Development tasks, run with `cargo xtask <task>`.

mod man;

use std::{
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

const USAGE: &str = "\
Usage: cargo xtask <TASK>

Tasks:
  man  Render scrollreversed(1) and scrollreversed(5) into target/man
";

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        Some("man") => {
            let root = workspace_root();
            man::write(&root, &root.join("target/man")).map_or_else(
                |error| {
                    eprintln!("xtask: cannot write the manual pages: {error}");
                    ExitCode::FAILURE
                },
                |paths| {
                    paths.iter().for_each(|path| println!("{}", path.display()));
                    ExitCode::SUCCESS
                },
            )
        }
        _ => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}
