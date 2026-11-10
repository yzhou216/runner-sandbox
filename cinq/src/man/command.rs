// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! cinq(1), which clap_mangen renders from the command line, together with
//! the sections that it cannot derive from it

use std::{
    io::{self, Write},
    iter,
};

use clap::{Arg, Command, CommandFactory};
use clap_mangen::{
    Man,
    roff::{Inline, Roff, bold, italic, line_break, roman},
};

use super::{super::cli::Cli, Platform};

type Section = fn(&Man, &mut dyn Write) -> io::Result<()>;

/// Sections of cinq(1) that clap_mangen derives from the command line
/// before the synopsis and after it, in the order of man-pages(7)
const BEFORE_SYNOPSIS: [Section; 2] = [Man::render_title, Man::render_name_section];
const AFTER_SYNOPSIS: [Section; 2] = [Man::render_description_section, Man::render_options_section];

/// Each section comes with the same preamble, which defines the apostrophe,
/// and the page keeps it only once.
pub fn command_page(platform: Platform, date: &str) -> String {
    let mut command = Cli::command();
    // Help and version options, which building adds
    command.build();
    let synopsis = synopsis(&command);
    // man-pages(7) starts the summary in the NAME section in lower case
    let summary = command
        .get_about()
        .map_or_else(String::new, |about| uncapitalized(&about.to_string()));
    // Notice that --version prints after the version, with its link on a
    // line of its own, so that roff neither hyphenates the link nor spreads
    // out the words before it
    let notice = command
        .get_long_version()
        .and_then(|version| version.split_once('\n'))
        .map_or_else(String::new, |(_, notice)| notice.replace(" <", "\n<"));
    let man = Man::new(command.about(summary))
        .date(date)
        .manual("General Commands Manual");
    let render = |sections: &[Section], page: &mut Vec<u8>| {
        sections.iter().try_for_each(|section| section(&man, page))
    };
    let mut page = Vec::new();
    render(&BEFORE_SYNOPSIS, &mut page)
        .and_then(|()| synopsis.to_writer(&mut page))
        .and_then(|()| render(&AFTER_SYNOPSIS, &mut page))
        .and_then(|()| epilogue(platform, &notice).to_writer(&mut page))
        .expect("writing to memory does not fail");
    let preamble = Roff::new().render();
    let body = String::from_utf8(page).expect("the page is UTF-8");
    format!("{preamble}{}", body.replace(&preamble, ""))
}

/// Returns the SYNOPSIS section of cinq(1), which, unlike the one that
/// clap_mangen renders, gives the value that each option takes
fn synopsis(command: &Command) -> Roff {
    let (positionals, options): (Vec<&Arg>, Vec<&Arg>) = command
        .get_arguments()
        .filter(|argument| !argument.is_hide_set())
        .partition(|argument| argument.is_positional());
    let line: Vec<Inline> = iter::once(bold(command.get_name()))
        .chain(options.into_iter().chain(positionals).flat_map(usage))
        .collect();
    let mut roff = Roff::new();
    roff.control("SH", ["SYNOPSIS"]).text(line);
    roff
}

/// Returns `argument` as the synopsis gives it, after a space: its flags
/// apart by bars, then its value, in brackets unless it is required
fn usage(argument: &Arg) -> Vec<Inline> {
    let flags: Vec<Inline> = [
        argument.get_short().map(|short| format!("-{short}")),
        argument.get_long().map(|long| format!("--{long}")),
    ]
    .into_iter()
    .flatten()
    .flat_map(|flag| [roman("|"), bold(flag)])
    .skip(1)
    .collect();
    let value = argument
        .get_value_names()
        .map(|names| italic(names.join(" ")));
    let space = (!flags.is_empty() && value.is_some()).then(|| roman(" "));
    let (open, close) = if argument.is_required_set() {
        ("", "")
    } else {
        ("[", "]")
    };
    iter::once(roman(format!(" {open}")))
        .chain(flags)
        .chain(space)
        .chain(value)
        .chain([roman(close)])
        .collect()
}

fn epilogue(platform: Platform, notice: &str) -> Roff {
    let notice: Vec<Inline> = notice
        .lines()
        .flat_map(|line| [line_break(), roman(line)])
        .skip(1)
        .collect();
    let mut roff = Roff::new();
    if platform == Platform::Xdg {
        roff.control("SH", ["ENVIRONMENT"])
            .control("TP", [])
            .text([bold("WAYLAND_DISPLAY"), roman(", "), bold("WAYLAND_SOCKET")])
            .text([
                roman("Wayland compositor that the window opens on, which "),
                bold("cinq"),
                roman(" needs, since it does not run on X11."),
            ]);
    }
    roff.control("SH", ["FILES"])
        .control("TP", [])
        .text([italic(platform.config_file())])
        .text([
            roman("The configuration file, which "),
            bold("cinq"),
            roman("(5) describes."),
        ])
        .control("TP", [])
        .text([italic(platform.unsaved_directory())])
        .text([
            roman("Directory that keeps the unsaved changes of shows, which "),
            bold("cinq-show"),
            roman("(5) describes."),
        ])
        .control("SH", ["COPYRIGHT"])
        .text(notice)
        .control("SH", ["SEE ALSO"])
        .text([
            bold("cinq"),
            roman("(5), "),
            bold("cinq-show"),
            roman("(5)"),
        ]);
    roff
}

fn uncapitalized(text: &str) -> String {
    text.chars()
        .take(1)
        .flat_map(char::to_lowercase)
        .chain(text.chars().skip(1))
        .collect()
}
