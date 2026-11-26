// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::{self, Write};

use anyhow::Result;
use itertools::Itertools;
use tabwriter::TabWriter;

use super::hazards;

#[derive(clap::Args)]
pub struct Args {
    #[command(flatten)]
    hazards: hazards::Args,
}

pub fn run(args: &Args) -> Result<()> {
    let catalog = args.hazards.load()?;
    let mut out = TabWriter::new(io::stdout().lock());
    writeln!(out, "NAME\tINPUTS\tTYPE\tSOURCE")?;
    catalog.entries().try_for_each(|entry| {
        let hazard = &entry.hazard;
        writeln!(
            out,
            "{}\t{}\t{}\t{}",
            hazard.name(),
            hazard.inputs().iter().format(" "),
            hazard.sample_type(),
            entry.source
        )
    })?;
    out.flush()?;
    Ok(())
}
