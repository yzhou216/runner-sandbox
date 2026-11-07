// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Lays out what cinq installs besides its program in the directory that
//! `CINQ_SHARE_DIR` names, as the share directory of a prefix holds it, so
//! that a package copies it whole into place.  A relative `CINQ_SHARE_DIR`
//! goes from the root of the package.  The manual pages are for the
//! platform of the build, dated by `SOURCE_DATE_EPOCH`, which reproducible
//! builds set, or else by the day of the build.  scdoc renders cinq(5) and
//! cinq-show(5), so it has to be installed.  Builds for Darwin also lay out
//! the app bundle beside the share directory.

// Modules of the program that the build compiles as well, which keep their
// own modules where the program finds them
#[path = "src"]
mod src {
    pub mod bundle;
    // Command line, of which the build reads only the definition
    #[expect(dead_code)]
    pub mod cli;
    pub mod logo;
    pub mod man;
}

use std::{
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::Command,
};

use clap::CommandFactory;
use clap_complete::{Generator, Shell, generate};
use clap_complete_nushell::Nushell;
use embed_manifest::embed_manifest_file;
use time::OffsetDateTime;

use src::{
    bundle,
    cli::Cli,
    logo,
    man::{self, Platform},
};

fn main() {
    // Files that the build reads as it runs, unlike the modules and pages
    // that it compiles in, a change to which rebuilds and so reruns it
    SHARED
        .iter()
        .for_each(|file| println!("cargo::rerun-if-changed=share/{file}"));
    ["CINQ_SHARE_DIR", "SOURCE_DATE_EPOCH"]
        .iter()
        .for_each(|variable| println!("cargo::rerun-if-env-changed={variable}"));
    let os = env::var("CARGO_CFG_TARGET_OS").expect("cargo names the target");
    if os == "windows"
        && let Err(error) = embed()
    {
        println!("cargo::error=cannot embed the manifest: {error}");
    }
    let Some(share) = env::var_os("CINQ_SHARE_DIR").filter(|share| !share.is_empty()) else {
        return;
    };
    let share = Path::new(&share);
    let platform = Platform::of(&os);
    let icon = share.join("icons/hicolor/scalable/apps/cinq.svg");
    [
        ("render the manual pages", render(share, platform)),
        ("write the shell completions", complete(share)),
        ("draw the icon", write(&icon, logo::svg())),
        (
            "copy the desktop entry and the type of show files",
            copy(share),
        ),
    ]
    .into_iter()
    .chain((os == "macos").then(|| ("lay out the app bundle", assemble(share))))
    .filter_map(|(what, done)| done.err().map(|error| (what, error)))
    .for_each(|(what, error)| println!("cargo::error=cannot {what}: {error}"));
}

/// Files that go in the share directory as they are, by their path under
/// `share`
const SHARED: [&str; 2] = ["applications/cinq.desktop", "mime/packages/cinq.xml"];

/// Writes `contents` to `file`, along with the directories that it goes in
fn write(file: &Path, contents: impl AsRef<[u8]>) -> io::Result<()> {
    fs::create_dir_all(file.parent().expect("a file goes in a directory"))?;
    fs::write(file, contents)
}

fn copy(share: &Path) -> io::Result<()> {
    SHARED
        .iter()
        .try_for_each(|file| write(&share.join(file), fs::read(Path::new("share").join(file))?))
}

fn complete(share: &Path) -> io::Result<()> {
    [
        (script(Shell::Bash), "bash-completion/completions/cinq"),
        (script(Shell::Zsh), "zsh/site-functions/_cinq"),
        (script(Shell::Fish), "fish/vendor_completions.d/cinq.fish"),
        (script(Nushell), "nushell/vendor/autoload/cinq.nu"),
    ]
    .into_iter()
    .try_for_each(|(script, file)| write(&share.join(file), script))
}

fn script(generator: impl Generator) -> Vec<u8> {
    let mut script = Vec::new();
    generate(generator, &mut Cli::command(), "cinq", &mut script);
    script
}

/// Lays out the app bundle beside `share`, with its directory for the
/// program left for a package to fill, since the program is linked after
/// the build runs
fn assemble(share: &Path) -> io::Result<()> {
    let contents = share
        .with_file_name("Applications")
        .join("cinq.app/Contents");
    fs::create_dir_all(contents.join("MacOS"))?;
    write(&contents.join("Info.plist"), bundle::info_plist())?;
    write(&contents.join("Resources/cinq.icns"), bundle::icns()?)
}

/// Application manifest of the program on Windows, which starts it without
/// a console window from Explorer, while a shell runs it in its console as
/// it runs any console program
const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <application>
    <windowsSettings>
      <consoleAllocationPolicy xmlns="http://schemas.microsoft.com/SMI/2024/WindowsSettings">detached</consoleAllocationPolicy>
    </windowsSettings>
  </application>
</assembly>
"#;

fn embed() -> io::Result<()> {
    let manifest = out_dir().join("cinq.exe.manifest");
    fs::write(&manifest, MANIFEST)?;
    embed_manifest_file(&manifest)
}

fn out_dir() -> PathBuf {
    env::var_os("OUT_DIR")
        .expect("cargo names the output directory")
        .into()
}

fn render(share: &Path, platform: Platform) -> io::Result<()> {
    let page = |name: &str| {
        let (_, section) = name.rsplit_once('.').expect("a page names its section");
        share.join(format!("man/man{section}/{name}"))
    };
    write(&page("cinq.1"), man::command_page(platform, &date()))?;
    man::SCDOC
        .iter()
        .try_for_each(|(name, source)| scdoc(&man::for_platform(source, platform), &page(name)))
}

/// Returns the day of `SOURCE_DATE_EPOCH`, or else today, written YYYY-MM-DD
fn date() -> String {
    let time = env::var_os("SOURCE_DATE_EPOCH").map_or_else(OffsetDateTime::now_utc, |seconds| {
        seconds
            .to_str()
            .and_then(|seconds| seconds.parse().ok())
            .and_then(|seconds| OffsetDateTime::from_unix_timestamp(seconds).ok())
            .expect("SOURCE_DATE_EPOCH holds a time in seconds since 1970")
    });
    time.date().to_string()
}

/// Renders the scdoc `source` into the manual page `page` with scdoc, which
/// dates it as [`date`] does.  The source reaches scdoc through a file, so
/// that the build need not feed scdoc while it reads what scdoc writes, and
/// the page is written only once scdoc has rendered all of it, so that a
/// failure leaves no page behind.
fn scdoc(source: &str, page: &Path) -> io::Result<()> {
    let input = out_dir()
        .join(page.file_name().expect("a page is a file"))
        .with_added_extension("scd");
    fs::write(&input, source)?;
    let output = Command::new("scdoc")
        .stdin(File::open(&input)?)
        .output()
        .map_err(|error| io::Error::new(error.kind(), format!("cannot run scdoc: {error}")))?;
    if output.status.success() {
        write(page, output.stdout)
    } else {
        let reason = String::from_utf8_lossy(&output.stderr);
        Err(io::Error::other(format!("scdoc failed: {}", reason.trim())))
    }
}
