<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq native tests, OLD and NEW, 2026-11-27

`cargo test --locked --no-fail-fast` of cinq from a git clone at the full SHA,
with `CINQ_SHARE_DIR=` and `FONTCONFIG_FILE=/dev/null`, in run 46111320658
(`.github/workflows/tests.yml`).  OLD is e304bd4e581af53fa32c5ae9a598c2d655130e61,
NEW is efa362c0f5193bc3bd73bbc4494284fa6c6429ce.  Every step ends in `|| true`,
so the verdicts below come from the uploaded `test-<runner>-<build>` logs.

| Runner | OLD | NEW |
| --- | --- | --- |
| macos-latest | 1494 passed, 0 failed | 1494 passed, 0 failed |
| windows-latest | 1476 passed, 6 failed | 1476 passed, 6 failed |
| windows-11-arm | 1476 passed, 6 failed | 1476 passed, 6 failed |
| x86_64 FreeBSD 15.1 | 1493 passed, 1 failed | 1493 passed, 1 failed |

Failing on both Windows runners, with OLD and NEW alike:

- `back::failure::tests::unreadable_file_is_told_without_its_error_number`
  (the data is invalid, not permission denied)
- `back::network::tests::failures::failure_to_reach_a_destination_shows_until_a_message_gets_there`
  (connection timed out, not refused)
- `back::network::tests::tcp::write_that_times_out_is_told_as_timed_out_on_every_system`
  (the write succeeded)
- `back::workspace::tests::pasting::copy_holds_its_file_by_absolute_path_and_paste_makes_it_relative_where_it_lands`
  (a backslash in the path)
- `back::workspace::tests::show::relative_sound_files_go_from_wherever_the_file_is`
- `front::update::show::tests::opening::directory_in_place_of_a_show_is_told_without_an_error_number`
  (no message)

Failing on FreeBSD, with OLD and NEW alike:
`back::network::tests::tcp::check_of_a_connection_reads_a_bounded_amount_of_what_the_peer_sent`
(the check read everything).

No test has a different verdict between OLD and NEW.
