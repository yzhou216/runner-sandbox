<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: GPL-3.0-or-later
-->

# Changelog

Major changes and improvements in each release of cinq, newest first.

## Unreleased

- cinq ships one keymap, `vi`, and no longer the `cua` keymap.  A configuration
  file can still declare a keymap of its own under `keymaps`, starting from
  another keymap or from no keys, as cinq(5) shows.  A file that names `cua`
  in `keymap` or in `from`, and declares no keymap of that name, gets the
  problem `cinq no longer ships the cua keymap, so vi stands in for it`, and
  the keymap starts from the keys of `vi`, so that GO and Panic keep working.
- Choosing an audio device in the sidebar keeps a `config.toml` that starts
  with a byte order mark, as Notepad writes it, readable.  Before, the new
  setting went before the mark and the file no longer opened.
- Choosing an audio device leaves `config.toml` as it is when `audio_device` is
  a table there, as `[audio_device]` or `audio_device.a = 1` makes it, and says
  `config.toml cannot keep the audio device: audio_device is a table, not an
  ID`, where it wrote a file that could not be read.
- A `config.toml` nested thousands of levels deep no longer aborts cinq when it
  is formatted after a save.
- A `config.toml` of thousands of keymaps, each starting `from` the next, no
  longer aborts cinq as it is read.
- While the light patch holds an instrument, the window no longer wakes every
  second to look for news of the light output, which kept a large show at
  several percent of a processor while idle; it wakes when the news comes.
- The window no longer wakes every two seconds to look at the files of the
  show: a thread looks at their stamps and wakes it only for a file that
  changed, so that a large show idles near zero.
- A name that cinq cuts short no longer runs past its room when it holds a tab
  or another control character, as a file name can, nor begins with an accent
  whose letter was cut, as an output in the sends of the inspector could.
- A time of more than 13 years, such as `1e10` seconds typed in a time field,
  shows in the cue list as the minutes and seconds that it is, where every
  time beyond that limit showed as `7158278:49.5`.
- A search takes the Greek final and other sigma, the Turkish dotted and
  dotless `i` and the plain `i`, and the German `ß` and `ss` for the same
  letters, in any case.  Before, a search for `ΟΔΥΣ` missed a cue named
  `ΟΔΥΣΣΕΥΣ`, and one for `STRASSE` missed `Straße`.
- A cue, cue list, output, light instrument, or show file whose name holds a
  line break or a tab, as a hand-edited show file can give it, shows on one
  line, with each run of white space in it as one space, in the cue list, the
  outputs table, the sends, the headings of the inspector, the sidebar, the
  hints of keys, the notices of the status bar, and the name of the show.
  Before, the name took a second line or counted a column too many.
- The message of a network cue that a show file holds with a line break or a
  tab, as in `message = "/scene\n\"Act 1\""`, shows on one line in the Target
  column of the cue list, as a name does.  Before, it took a second line.
- Tab onto a field or a button of the Light Patch tab, or of the Levels tab of a
  light cue, scrolls the table to show it, as it does on the sends.  Before,
  the keyboard moved to a row below the part in view and typing edited a field
  that nobody could see.
- Hint mode no longer lets two labels cover each other where a button at the
  left edge of the window is close to the next control, as Standby selected is
  to Show mode in a window 800 pixels wide.  Before, the label of the button
  lay on the label of the check box and neither could be read.  Now the label
  of the button lies at the left edge of the window.
- Hint mode no longer lets labels cover each other where several controls lie
  near the left edge of the window.  Before, two labels that both moved to the
  edge lay on each other, and a label at the edge could cover one that lay
  left of its control near that edge.  Now each takes the room that the labels
  before it left.
- Undo and redo of an edit of the light patch tell which instrument they
  changed, as in `Undid: added instrument 4`, `Undid: renamed instrument
  Front to Key`, and `Undid: changed the address of instrument Back to 2/12`.
  Before, they said only `changed the light patch`.
- A level of -60 dB reads `-inf` in the level field of a cue, a send, the new
  cues of an output, and the list of running cues, as `-inf` is typed.
  Before, it read `-60.0`, so that silence read two ways.
- The Levels tab of a fade cue that targets a picture cue and does not stop it
  says `A picture has no level, so this fade changes nothing`.  Before, the
  fade ran and the tab said nothing.
- While any cue is paused, the chain line of the transport leads with how many,
  as `2 paused; GO fires 1 cue: 6`.  Before, only the cue table and Active said
  so, and the transport read as if all were playing.
- A cue that waits before it starts reads `in 0:16.1` in Active and in the
  State column, in the figures of `0:04.0 left`.  Before, it read `in 16.1s`.
- An open drop-down list, such as the list of the kind of cue to add, no longer
  hints `cc: rename` after a cue was added.  Before, the hint stayed although
  the list took the keys, so that `cc` did nothing.
- The help page lists the keys of `pick` and `close_menu` right after that of
  `cancel`, with the keys that act in a menu, and no longer between those of
  `hint` and `add_below`.  The table of cinq(5) has the same order.
- The `Default device` list in the sidebar reads `Not found:` before the ID of
  a device that is not there.  Before, only the yellow of the ID told it.
- A devamp cue whose target is a group, a picture, or any cue but an audio or
  video cue tells `target is not audio or video` in the warnings and the
  inspector, where it said `cannot act on its target cue`, which did not say
  what to change.
- The Then box of a cue in a group shows only `(in a group)`.  Before, it
  showed the value in front of the reason, and at 1024x768 the reason was cut
  to `wait for GO (in a gro`.
- A running fade or devamp cue whose target is not playing says `target not
  playing` in yellow in Active, where it read the time left and `fades 5` in
  green for its whole length, though it changed nothing.
- A running cue in the Active list of the sidebar drops the parts of its line
  that do not fit whole, from the end.  A fading film at 1024x768 read
  `0:05.6 left, -14.9 dB,…`, cut within its last part, and now reads
  `0:05.6 left, -14.9 dB`.  A pause now comes before the level, so that a
  narrow sidebar keeps it.
- The help line for hint mode says "check box" where it said "checkbox".
- A `config.toml` that is a symbolic link, as one kept in a directory of
  dotfiles is, applies each change to the file that it links to while cinq
  runs.  Before, such a change applied only once cinq started again.
- Keys held with Control or Alt mean the same with Caps Lock on, where
  Control+s saved the show as another file, as Control+Shift+s does, and
  Control+c, Control+d, Control+w, and Alt+j did nothing.  Only Shift makes
  the capital of such a key now.
- Holding the key of help down shows the help page once, where each repeat of
  the key closed and opened it again, so that the page flickered and stayed
  open or shut by chance.
- On Darwin, keys bound to Alt with a letter, such as Alt+j and Alt+k, which
  move the selected cue down and up, now act.  Before, Option made another
  character of the letter, as `∆`, which no binding names.
- A key that acts at once, such as one bound to `go`, tells once in the
  problems of `config.toml` that it cuts a sequence short, when two actions
  hold that sequence by default.  Before, the problem showed twice.
- The box of the keys that can follow an operator leaves out a motion key that
  a binding of your own takes, such as `j` after `d` when `d j` is bound to
  another action, since that binding acts there.  Before, the box listed the
  key twice, once for the binding and once for the cut of the next cue.
- An operator over a motion that finds no rows, as `d k` on the first cue,
  leaves the status bar as it was, so that the hint to paste what a copy or
  cut took stays.  Before, the key dropped the hint although nothing happened.
- A key sequence typed halfway, as the `d` that waits for its motion, is
  dropped when a window takes the keyboard, as the stage window does and as
  the main window does when another program gives it back, and as a click
  drops it.  Before, a `j` typed after the keyboard came back completed the
  `d` and cut two cues.
- A keymap of your own that no setting chooses is checked for problems as the
  one in use is: an action it names that does not exist, a key that cannot be
  read, or a `from` that names no keymap.  A value under `[keymaps]` that is
  not a table is a problem too, as `ready_ahead = 5` written below it, which
  now says that the setting belongs above the first table.  Before, these
  showed only on the day that the keymap was chosen, and a value that is not a
  table never did.
- Opening another show, or the stage window taking the keyboard as it does by
  Alt+Tab, takes the hint mode labels away.  Before, the labels stayed up, and
  a label typed then acted on the new show or pressed a button of the main
  window out of sight.
- A file chosen for a cue after another show has opened is dropped.  Before,
  a cue of the new show could take the file.
- A click on a cue list that an undo took away in the same moment leaves cinq
  as it is, where it stopped with an error.
- Saving a show over a file that has unsaved changes kept for it, as Save As
  does when the chosen file is another show with changes that were never saved,
  removes those changes.  Before, opening the saved file brought them back as
  unsaved changes that replaced what was saved.
- Following the tutorial link of the help page, where the tutorial is not
  installed, reads `The tutorial is not installed` even when standard error is
  a closed pipe or a full disk, and the directories searched no longer land on
  the problems screen of `config.toml`.  Before, the page told nothing.
- Saving a show whose file is a symbolic link to a file that is not there, as
  one that a dotfiles directory has yet to hold, creates the file that the link
  names and keeps the link.  Before, the save put a plain file in place of the
  link.
- A click or a turn of the wheel in the stage window no longer acts as one in
  the main window.  Before, it closed the search line and hint mode, dropped a
  command typed halfway, and made the cue list measure its rows anew.
- A picture file whose header declares more than 134 million pixels, or a video
  file that declares pictures larger than any level of H.264 allows, as a
  damaged file can, is marked `holds a picture too large to show` or `holds
  H.264 video larger than any level of H.264 allows` as it loads, or as it
  plays for a size that only its samples declare.  Before, firing its cue asked
  for gigabytes of memory, which could end cinq.
- A video file whose track is marked `avc3`, as some cameras and streaming
  tools write it, plays as one marked `avc1` does, where it read `holds no
  H.264 video`.
- A video that a phone filmed on its side, whose track header turns its
  picture by a quarter, half, or three quarters of a turn, shows upright on
  the stage, as a picture file turned by its orientation does, where it showed
  lying on its side.
- Video cues show the colors that their file declares, in its colr box or in
  the sequence parameter set of its H.264 stream: the matrix, BT.601 or
  BT.709, and the full range of a byte for each color.  Before, cinq took the
  matrix from the size of the picture and always the limited range, so that a
  stream of full range showed with crushed blacks and clipped whites, and a
  stream of another matrix than its size implies showed with tinted colors.
- A cue added to a reopened show no longer becomes the target of a cue that
  targeted a deleted cue.  Before, the new cue could take the identity of the
  deleted one, so that a cue such as a fade that targeted it acted on the new
  cue instead.
- A cue numbered `inf` or `infinity`, in any case, is text, as any other word
  is, and no longer makes every new cue take the number
  `18446744073709551615`.
- A cue that auto-follows or auto-continues an audio or video cue keeps to
  the clock of that cue as the clock follows its sound, so that a sound that
  starts late or drifts holds the next cue back with it.  Before, the next
  cue fired at the time that GO set, and could start while the sound before
  it still played.
- On Windows, a file chosen for a cue, or copied with its cue, is named with a
  slash between the directories, as `sounds/storm.ogg`, so that the show opens
  on GNU/Linux, the BSDs, and Darwin.  Before, the name held a backslash,
  which those systems read as part of one file name, and every such cue read
  `file not found`.
- When cinq opens a show through a path with `..` in it, as in
  `cinq ../shows/a.cinq.toml`, a file chosen in the directory of the show is
  saved by its name from there, so that the show still moves with its files.
  Before, the file was saved with its whole path.
- Deleting thousands of cues at once, or undoing the paste of a thousand, takes
  a few milliseconds where it took a quarter of a second or more, since cinq no
  longer goes through the whole list again for each cue that it removes.
- A devamp cue lets a sound whose file has a low sample rate, such as 22050
  frames a second, leave its loop at the pass end that it plans.  The notice
  that it needs grows with the time that the decoder reads ahead, which a low
  rate lengthens.  Before, such a sound went round the loop once more than
  planned and its run ended while the end of the sound still played.
- A group that auto-follows no longer waits for a looping or picture cue in it
  that cannot play, such as one whose file is missing: the next cue fires as it
  does after a cue that has nothing to play, and the group shows no `hold`.
  Before, the cue after such a group never fired.
- A loop shorter than a millisecond, such as `1` to `1.00001`, reads BROKEN
  with the warning `loop is shorter than a millisecond`, and GO skips its cue.
  Before, kira rounded its ends to one frame, and the thread that decodes the
  sound then spun a processor core for as long as the sound lived.
- Panic ends a group that still waits out its pre-wait, together with the
  group around it where that holds nothing else.  Before, the group stayed in
  the cue list and the sidebar until its pre-wait was over, and Panic did not
  clear it.
- Pasting cues that were copied from inside a group, as a group and a cue that
  follows it, puts the cue beside the group, as it was.  Before, the cue went
  into the pasted group.
- A pause cue that targets a fade cue holds the fade that the cue is making,
  and a start cue that targets the fade cue lets it go on.  Before, the fade
  went on to its end while the fade cue read `paused`.
- Two start cues that fire the same chain make its video cues ready once, as
  one start cue does.  Before, each update of the show started another decoder
  for a picture cue of the chain and dropped the one that was ready.
- A sound or video that two start cues reach, with different pre-waits, is made
  ready for the sooner of the two.  Before, it was made ready for the one that
  the list named first, so a sound that the sooner start cue fired could begin
  late when the other had the longer wait.
- On Windows, a cue pasted into a show in another directory names its file
  with backslashes throughout.  Before, the slashes of its relative name
  stayed among them, as `C:\shows\storm\sounds/rain.ogg`.
- An audio device whose driver fails while cinq opens it is tried again every
  two seconds, as one that refuses is, and its output says `cannot open` with
  the reason `the audio driver failed`.  One whose driver fails as cinq closes
  a stream keeps answering.  Before, either stayed `not answering` until cinq
  restarted.
- A sound file that is cut short, as a copy that stopped halfway is, no longer
  keeps a processor core busy while its sound waits to start: cinq lets the
  sound go once it fails to decode, and opens the file again at GO.  Before,
  the decoding tried again without pause from up to ten seconds before the cue
  until GO.
- A file that makes its reader panic, as a device or a pipe named by a video
  cue does, or a sound file whose header gives a sample rate of zero, now
  reads `cannot be read` among the warnings.  Before, the panic ended the
  loading of the files for good, so that no cue of the show loaded until cinq
  restarted.
- `cinq --audio-devices` ends quietly when the program that reads its list
  stops early, as `head -n 1` does, where it printed that it could not list
  the audio devices and failed.
- A WAV file of more than two channels reads `holds more than two channels` as
  its cue loads, and the cue reads BROKEN, where it passed and went silent at
  GO while a thread of the decoder spun a processor core for good.
- A start that fails, and `cinq --audio-devices` when it fails, leave with
  status 1 even when standard error is a closed pipe or a full disk.  Before,
  the write of the message panicked and the status was 101.
- Moving an output to another place in the list while a cue plays on outputs of
  two audio devices leaves the sound playing.  Before, the sound stopped and
  started again on both devices, which cut it.
- A sound that starts again while its cue plays, as when an edit of the sends
  moves it to another audio device, plays on from where its cue is, even when
  standby has gone back to that cue.  Before, it started from the beginning
  of the file and drew the clock of the cue back with it.
- Choosing another audio device for the outputs on the default device in
  `config.toml` leaves the sound playing on outputs that name their own
  devices.  Before, every device closed and every sound stopped, and the
  outputs on named devices opened again.
- A cue that sends to outputs on two devices goes on playing on one of them
  unbroken while the other changes: its output moves to another device, the
  device stops or opens again, an output needs more of its channels, or the
  cue stops sending to it or begins to again.  Only the sound on that device
  stops or starts, where the sound on both devices was cut and began again.
- A button of a remote that sends its release as a 64-bit integer or a double
  zero, as `oscsend` does with the types `h` and `d`, no longer acts a second
  time as it is released, as one that sends an integer or a float zero does
  not.  Before, the release of such a button fired GO again.
- A network cue whose destination is an IPv4 address written with a zero before
  an octet, as `010.101.100.020:3037`, in hexadecimal, as `0x7f.0.0.1:9000`, or
  with fewer than four octets, as `127.1:9000`, reads `NO DESTINATION`.
  Before, the system took a zero for octal and `0x` for hexadecimal, so that
  the message of `010.0.0.1` went to 8.0.0.1.
- A burst of network cues to a UDP destination given by a host name, as
  `console.local:8000`, looks the name up once and sends every message, even
  when the name is slow to resolve.  Before, each message looked it up again,
  one after another, so that the later messages of a GO went out more than a
  second late and were dropped.
- `/cue/{number}/start` over OSC on a video cue that is paused resumes it and
  leaves the stage window as it is, as a start cue that resumes it does.
  Before, it made the stage window fullscreen and brought it before the other
  windows, or opened it again where the operator had closed it.
- A network cue fired just before the window closes is sent, where cinq
  exited before its message went out when the connection or the lookup of the
  host name took a moment.  Closing now waits for such messages, for a second
  at most.
- An `osc_listen` with a space around an address that is fine otherwise, as
  `" 127.0.0.1:53000"`, now says that the space is the problem.  Before, it
  said that the setting was a host name.
- A destination whose host holds a null byte, as a hand-edited show file can
  give it, reads as no destination.  Before, the interface showed it as a
  valid one, and the message failed with `data provided contains a nul byte`.
- A message to a TCP destination that cannot be written within its second
  shows the problem `message not sent: timed out` on GNU/Linux, the BSDs, and
  Darwin, as it does on Windows.  Before, those systems said `operation would
  block`.
- A TCP destination that sends data back without pause no longer holds its
  messages.  Before, cinq read what the destination had sent for as long as
  more arrived, so that a message to it waited behind that reading for good.
- The notice of an end, loop start, or loop end that reads as no time, as
  `1:0:0:5` does, says `to empty`, as the header of the inspector does.
  Before, it said `to zero`, though such a time plays to the end of the file or
  the bound of the clip.
- The line on standard error for a missing tutorial starts with a lower case
  letter, as every other line of cinq does: `cinq: cannot find tutorial.pdf in
  ...`.  Before, it began with `Cannot find`.
- The problems of an `osc_listen` that is no address say `should be IP:port`,
  as the problems of every other setting say `should be`.  Before, they said
  `must be`.
- A video whose sound track no decoder reads, as one in AC-3 or Opus, reads
  `its sound is in a format that cinq cannot play`.  Before, it read `its
  sound cannot play: not a sound file that cinq can play`, which denied that
  the file was a sound file, though cinq had opened the video.
- The notice of output that cinq dropped while the problems of `config.toml`
  showed reads `cinq: 1 byte of output written while ...` for one byte, and
  `N bytes of output written while ...` for more.  Before, it read `1 bytes`
  and `output from while`.
- The help of an output whose device is not there says `Connect the device, or
  choose another`, as the sidebar says that the user chooses a device.  Before,
  it said `pick another`.
- A network cue whose message waited more than a second says `message not
  sent: more than a second late`.  Before, it said `message not sent: dropped
  rather than sent more than a second late`.
- An audio device that is not there, or that has stopped, is tried again every
  two seconds while the show idles, as cinq-show(5) says.  Before, the window
  looked at the devices only when something else woke it, so that a Dante
  Virtual Soundcard that started later still showed `Not found, retrying`, and
  the first GO after it came played late on it.  The window wakes only while a
  device is being opened or has to be tried again.
- Opening an audio device list while the sound server is slow no longer lets an
  older listing that ends last replace a newer one, which dropped a device just
  plugged in from the list.
- The thread that sends messages to the network and the thread that sends light
  no longer wake every second while the show idles, and the light thread reads
  no list of network interfaces then, so that a large show idles nearer to zero.
- A host that sends OSC datagrams that are no command at a high rate, as a
  program that streams feedback to the wrong port does, no longer makes the
  window run an update for each one, which could delay GO and Panic.  The
  sidebar now takes the line of such a datagram at most once a second.
- A light output that cannot open its socket when cinq starts, as when no
  descriptor is free for a moment, now tries again every two seconds and sends
  once it succeeds.  Before, it gave up for the rest of the session and showed
  `Light output: not sent` in the sidebar.
- Closing the window no longer keeps cinq running for good when the disk that
  holds the unsaved changes hangs.  It waits for them to be written for five
  seconds at most.
- Moving between the cue lists and the outputs no longer leaves the notice of a
  copy in the status bar, whose paste key acts on the other register.
  Before, it kept saying `Copied the output House R | p: paste` over a cue
  list, where `p` pasted an older cue or found the clipboard empty.
- An OSC command that moves the selection, such as `/go` from the standby cue,
  closes the search line as it takes the keyboard out of it.  Before, the line
  stayed drawn with no field to type in, until a click or Control+c closed it.
- The box of keys that can follow those typed halfway no longer lists keys that
  show mode or the stage window refuse, as `d` with its motions in show mode,
  which the status bar already left unhinted.  Before, the box taught them and
  the next key answered `Leave show mode to edit`.
- A key binding in `config.toml` with thousands of `Control+` prefixes no
  longer aborts cinq as it is read.
- Holding the key of open, save, or save as opens one file dialog, as holding
  the key of help opens one page.  Before, each repeat of the held key started
  another request to the file chooser.
- A chain of thousands of cues that continue or follow one another no longer
  aborts cinq when the first of them is stopped, restarted, paused, or
  resumed.
- A chain of thousands of start cues, each firing the next, no longer aborts
  cinq or slows every update to a crawl.
- A video file whose last pictures show before the pictures that decode ahead
  of them no longer makes cinq keep every decoded picture in memory.  The cue
  says that its picture cannot be decoded once more than 32 pictures wait.
- A warning on standard error writes the control characters of a file name or
  a network destination from a show file, such as an escape, as `\u{1b}`.
  Before, it sent them to the terminal, which carried them out.

## v0.1.7

- A new action, `hint`, bound to `f` in both keymaps, labels with a few letters
  every place that a click acts on, as Vimium and Tridactyl do: the buttons,
  checkboxes, tabs of the inspector, cue lists of the sidebar, drop-down lists,
  text fields, and links of the help page, as far as they are in view.  Typing a
  label acts as a click would.  A letter that no label continues is ignored, so
  that a slip never runs the next letters as commands.  Backspace takes a letter
  back, or leaves the mode where none is typed, and `Control+c` in the vi
  keymap, GO, Panic, any click, and any start or end of a cue in the Active list
  leave it too.  A label lies left of its place, or right of it where it would
  leave the window, and the buttons of GO and Panic have none, since their keys
  act everywhere.  The status bar hints the key and the letters typed.
- A collapsed group tells how many cues it hides where the list is too narrow
  for the Target column, as at 1024x768: its name ends in the count, as in `(10
  cues)`.  Hint mode also labels the `+` or `-` mark of a group, which flips it
  as a click does.
- A light cue that sets no instrument, or names one that the patch lacks, as
  after `Remove` takes an instrument out of the Light Patch, reads `NO LEVELS`
  or `UNPATCHED` in yellow in the State column, the sidebar lists it among the
  warnings, as `3 Ghost: names instrument 99, which the patch lacks`, and its
  inspector says so in yellow, as `This cue sets no instrument`.  GO still fires
  it.
- An entry of `[keys]` of which no key can be read, such as `go = "Spacebar"`,
  no longer leaves its action with no key, which could leave GO or Panic dead in
  a show: the action keeps the keys of its keymap, and the problem says so, as
  `go keeps Space`.  An empty list still unbinds.
- A show file that cannot be opened is no longer mistaken for a new empty show:
  the sidebar names the file and tells the cause in one line, as `Cannot open
  big.cinq.toml: line 8, column 14: invalid basic string`, and the empty cue
  table says that the show could not be opened, in place of `No cues yet`.
- While the stage window has the keyboard, the status bar says that the main
  window takes the keys of search, help, and hint mode, as it does for the keys
  that edit.
- The first steps on the help page say how a new cue gets its file, with the key
  of the next field, Enter in the File field, and the key of hint mode, and give
  the key that saves the show.  A tutorial that is not installed is told in one
  line, `The tutorial is not installed`, in place of every data directory
  searched, which go to standard error.
- A new setting, `osc_listen`, gives an IP address and a port, such as
  `"127.0.0.1:53000"`, on which cinq takes commands as OSC over UDP, so that a
  tablet, a lighting console, or a backup machine can fire cues.  The addresses
  are those of QLab: `/go`, `/panic`, `/pause`, `/resume`,
  `/cue/{number}/start`, `stop`, `pause`, and `resume`, and
  `/playhead/{number}`.  Where several cues have a number, the first is meant.
  A message whose only argument is 0 or false does nothing, so that the release
  of a button fires no second GO.  Anyone who can reach the address can fire
  cues.  The sidebar warns, naming `osc_listen`, when the address cannot be
  bound, and cinq tries again every two seconds.  It also shows the latest
  command that did nothing, such as one for a cue number that no cue has, with
  the address that it came from, until a command acts.
- Where exactly one warning would be left out of the sidebar, it lists that
  warning in place of `and 1 more`.
- The header of the inspector keeps to one line, so that a long cue name, or a
  sentence of trouble beside the title such as `GO skips this cue: file not
  found`, no longer pushes the Notes and Then rows out of view at 1024 by 768
  pixels: the title ends in an ellipsis where the room runs out, and where even
  a title of 16 columns leaves too little room, the sentence does.
- A message of the status bar is no longer cut to a stub such as `Added cue…` in
  a narrow window: it keeps at least three words or gives its room to the keys.
  A notice of a change that does not fit gives up its middle words and keeps the
  cue that it names, as `Undid: changed… cue 6.5`.
- A setting written below a table of `config.toml`, as under `[keys]`, is
  reported as one that belongs above the first table, in place of `no action is
  named`.
- In the cua keymap, where no key cancels, `Tab: next field` stays on the status
  bar beside the latest notice while a check box, a closed list, or the empty
  File field has the keyboard, and the empty File field no longer says `Enter:
  leave` beside `Enter: choose a file`.  The bar also names the keys of the
  search line, `Backspace: leave` while it is empty and `Enter: find` once text
  is typed, and `F2: rename` while the outputs show.
- While the show has no file and no cues, the status bar names the key that
  opens a show, `Control+l: open` in the vi keymap and `Control+o: open` in the
  cua keymap.
- Picking a kind in the New cue list with the keys gives the keyboard back to
  the cue list, so that the key that adds a cue acts at once in both keymaps,
  where it did nothing and the cua keymap had no key to leave the list.
- A save that fails is answered on the status bar, in place of the stale notice,
  as `Cannot save: permission denied`, and in the sidebar, as `Cannot save
  storm.cinq.toml: permission denied`, without the path of the temporary file or
  the system error number, which go to standard error.
- A show whose output lacks a field fails to open with a warning that names the
  output by its place, as `` line 5, column 1: output 2: missing field
  `channels` ``, and cinq-show(5) says that every output needs `channels`.
- A network message that failed is told for a cue, as the other cue warnings
  are, such as `3 Send: message not sent: connection refused`, and selects the
  cue when pressed, in place of a red line that named only the destination.  The
  cue is the first that names the destination, and its inspector shows it.
  Standard error keeps the destination, with its scheme, and the system error.
- In the vi keymap, `Control+[`, which Vim treats as Escape, cancels beside
  `Control+c`: it leaves a text field, closes the search line and a menu, drops
  the keys typed so far, and leaves hint mode.  A `0` that starts a sequence is
  dropped, as in Vim, where it is a motion, and no longer waits as a count of
  zero; a `0` after another digit still counts, as in `10j`.
- A text field, the search line included, takes `Control+w` and `Control+u` in
  both keymaps, deleting the word before the caret and everything before it, as
  in the Insert mode of Vim, where the search line took them at the end of its
  text only.
- Every two seconds cinq looks at the size and the modification time of each
  file that a cue plays, not only of files that failed, and reads again one that
  has changed or gone, so that a file that disappears while the show is open
  warns before its cue fires, with `file not found`, and a replaced file plays
  for its new length.  A failure that only playing finds, such as a PNG cut
  short, no longer clears within two seconds: it stays until the file changes.
  A cue that runs is left alone.
- A sound that stops while its cue still runs, as one does whose file is cut
  short, goes away, or holds more than two channels, no longer leaves the cue
  green and silent: the cue turns `BROKEN` with `its sound stopped early`, the
  sidebar warns, and standard error says so once.  Only a sound that stops with
  more than a second left, or a loop that stops at all, while its device plays
  on counts, so a stalled device, a fade, a stop, Panic, and a devamp never do.
- The reason that a file cannot play is told in plain words: `not a sound file
  that cinq can play` in place of `unsupported feature: core (probe): no
  suitable format reader found`, `is cut short, or is not an MP4 file` in place
  of `holds no H.264 video` for an MP4 file whose index is missing, `holds H.264
  video of a profile other than Baseline, Main, or High` in place of `holds
  H.264 video in a profile that cannot be decoded`, and `permission denied` for
  a file that cannot be read, without the system error number.
- A cue that the keys for moving up and down take into or out of a group is told
  so, as `Moved cue 8 into a group`, and so is the undo of that move.  A cut or
  a delete of a cue that plays, or a new file for it, says that the cue stopped,
  as `Cut cue 2, which stopped`, so that a silent house has a named cause;
  nothing asks first, and undo still brings back the cue and not its sound.
- The hint of the key to paste stays in the status bar after a cut or a copy
  while only the selection moves or the list switches, so that a cut in one list
  and a paste in the other keeps it, and goes at the first key that does
  anything else.
- An undo or a redo with a count that turns more than one change tells how many,
  as `Undid 5 changes, the last: added cue 43`, where it looked like a single
  undo.
- The key of `fullscreen`, `F11` by default, opens the stage window, fullscreen,
  when it is closed and the show holds a picture or video cue, so that a film
  that runs shows again, where it did nothing.  While the window is closed and
  such a cue shows on it, the sidebar warns `Stage window closed`, and the
  status bar counts the warning and hints `F11: open stage`; both go once the
  window is back or no such cue runs.
- While the stage window has the keyboard, which each GO that fires a picture or
  video cue gives it on a desktop that grants it, the status bar hints only keys
  that act there, and `F11: fullscreen`, which leaves a fullscreen stage window
  that covers the main window on a single display, and no longer names `u:
  undo`.  A key that belongs to the main window is answered with `Click the main
  window to edit` or `Click the main window for search, help, or hint mode`.
- A path typed into the File field of a cue applies once the field is left, as a
  typed level does, where the first key stopped a cue that plays and every
  half-typed path read as a file that does not exist.  It is one change to undo,
  and the notice `Changed the file of cue 2, which stopped` follows it.  The
  address of an instrument applies once its field is left too, so that a text
  that is no address keeps the old one.
- The Message field of a network cue sits in the wide column of the Basics tab,
  below the destination, and Notes takes the narrow column, so that a message of
  about 36 columns shows whole at 1024 by 768 pixels; Tab follows.  The empty
  Destination field reads `host:port or udp://host:port`, and cinq-show(5) and
  the tutorial say that a UDP message reports no failure, since nothing answers
  it.
- Every network cue whose destination has a send problem reads `SEND FAILED` in
  yellow in its State column, as for `OUTPUT FAILED`, also while it is only
  next, until a message gets there.  The sidebar keeps one line for each
  destination.
- A destination whose host holds a colon, a bracket, or a slash, such as an IPv6
  address, a URL, or a second port, is refused as it is typed, as `eos.local`
  without a port is: the field turns red and the cue reads `NO DESTINATION`,
  where it was found only at GO.  A message with a double quote left open, as in
  `/scene "Act 1`, is refused likewise, and the cue reads `NO MESSAGE`.
- A cue that sends only to an output whose channels all lie past those of its
  device, as channel 7 of a card with two, reads `OUTPUT FAILED`, and the header
  says `(silent: output failed)`, as for a missing device; a pair with one
  channel left still half sounds and stays out.  The Active entry of a running
  cue whose every send goes to a failing output reads `silent: output failed` in
  yellow in place of its state.
- A fault on one output no longer widens every send on the Levels tab: only the
  entry whose name carries the fault words, such as `(not found)`, is wider, so
  that a Dante rig with one failed output still shows as many sends as before.
  An output past the single channel of a mono device reads `Past its 1 channel`,
  not `Past its 1 channels`.
- Clearing the last send of a cue says `Changed cue 6 to play nowhere`, where it
  said `Changed where cue 6 plays`, so that the operator learns what the edit
  did.
- A warning in the sidebar whose words hold a long device ID, such as
  `alsa:surround71:CARD=PCH,DEV=0`, wraps within the ID where it does not fit,
  so that `DEV=0` and `DEV=1` can be told apart at 1024 pixels.  The warning
  about a default device that is not found names its ID, as `Default device
  (alsa:nonexistent-main): not found`, and the Default device list shows it in
  yellow, not in the gray of a disabled list.
- A remote `/cue/{number}/start` for a cue that cannot play, such as one with no
  file or no destination, is reported as an ignored command, `OSC from
  192.168.1.5: cue 4 cannot play: no destination as host:port`, in the sidebar
  and on standard error, as one for a number that no cue has is.  The problem of
  an unreadable `osc_listen` is `osc_listen must be IP:port, as
  127.0.0.1:53000`, short enough to show its example at 1024x768.
- The `State` column keeps the `NEXT` mark of the standby cue while that cue
  runs, with `looping` or the time left below it.
- The `State` column of a collapsed group counts the cues that it hides and that
  run, as `3 running`, and those that are paused, as `1 paused`, and tells their
  yellow warnings, as `OUTPUT FAILED`, `SEND FAILED`, `NO OUTPUT`, `NO LEVELS`,
  and `UNPATCHED`, each once, so that folding a group hides no cue that plays,
  plays in silence, or sets nothing.
- The `GO fires` line of the transport tells what each start, stop, pause,
  devamp, or GoTo cue of the chain does to its target, as `13 (GoTo 2)` or `25
  (start E3 in Effects)`, naming a list only for a target in another, and leads
  with `Standby moves to 5 in 2.9s` while a GoTo cue that has fired waits out
  its pre-wait.
- The target menu of a fade, start, stop, pause, devamp, or GoTo cue says "no
  cue to choose" in place of "choose a cue" while the list holds no cue that it
  can name.
- Adding a cue, by key or by button, shows the Basics tab of the inspector,
  where the tab of an earlier cue stayed, so that the Tab keys that the help
  page teaches reach the File field of the new cue from any tab, where a path
  typed after an edit on Time & Loops or Levels ran as keys.
- The notice of a cue added below a group, or below a cue that a group holds,
  says `Added cue 6 in a group`, as for a move into a group, and cinq(5) tells
  that the key of outdent takes it out.
- The hint of what Enter does on a checkbox, a button, or a closed drop-down
  list (`Enter: open`) outlasts the words of a change, as the key that leaves
  does, so that the drop-downs reached right after an edit still hint it.  The
  hint of going back or forward cuts a long cue number, or the name that stands
  for a blank one, as it cuts a long cue list name, so that it does not push the
  other hints out of the bar.
- While a closed drop-down list, a checkbox, or a button has the keyboard, the
  keys of hint mode and of adding a cue act, as those of undo and redo do, and
  each gives the keyboard back to the cue list, so that labels take letters and
  the new cue has the keys; the cua keymap, which has no key to leave a list,
  needs no run of Tabs.  The status bar hints them there too.
- Light output that cannot go out is told: while the patch has an instrument and
  the computer has no IPv4 network, the socket cannot be opened, or every send
  of a round fails, the sidebar lists `Light output: no network interface` or
  `Light output: not sent: <reason>` and counts it, and standard error says it
  once; it goes once a round gets a packet out.
- The Levels tab of a light cue has a `Now` column with the level that each
  instrument has on stage, as the Light Patch tab has, which goes first where
  the width is short.  In both tables the text of a name and of a level sits on
  the middle of its row, as the fields do, and a level field stays beside its
  name.
- The Time & Loops tab of a light cue says that the Duration on the Basics tab
  is the fade time.  The notice of a refused light level or address leads with
  its range, as `Not applied: 0 to 100, or empty for unchanged`, so that the
  status bar keeps it where the window is narrow.  Adding an instrument to the
  light patch after the last address of the last universe tells in the status
  bar that the patch has no address left.
- A time of more than three parts, such as `1:0:0:5`, is refused instead of
  counting as 60 hours and 5 seconds, and the notice says `Changed the pre-wait
  of cue 4 to zero`, where it only said that the field changed.  A wait ahead of
  a cue, or of a GoTo that moves standby, reads as a clock from a minute on, as
  `in 1:15.0`, like the time left of a running cue.  An audio or video cue that
  starts at or past the end of its file reads `starts past the end of its file`,
  where it read "starts at or after its end".
- A level field takes `-inf` as -60 dB, silence, and a send field, or the field
  of the new cues of an output, takes `off` as it takes an empty field, which
  shows `off`.  An empty level field of an audio, video, or fade cue says `Not
  applied: type a level, -60 to +12 dB`.  The notice of a refused channel map
  names the form `1,3` along with `1-2`, and that of a refused send level says
  that an empty field means no send.
- Undo and redo of a move of standby select the cue that standby is on, in its
  cue list.  Undo and redo of a change to the outputs or the light patch show
  the place that changed: the outputs, with the touched output selected, or the
  Light Patch tab, where the cue list showed.
- The box that lists the keys that can follow, such as after `g` in the vi
  keymap, puts the line of the key typed again first, so that `gg` heads `gj`
  and `gk`.  After a count and an operator such as `2d`, it says how far the
  motion reaches (`Cut the selected cue and the next 6`, or `to row 3` for `gg`
  and `G`).
- In the vi keymap, a zero typed where the count after an operator would start,
  as in `d 0 j`, drops the operator, as a zero at the start of a sequence does,
  where it kept the operator waiting and then cut two rows.
- The help page of a keymap that binds no sequence of keys, such as CUA, does
  not tell how to type one, and the row of the previous inspector tab says that
  the key goes back as many tabs as counted.
- A key bound to two actions in `config.toml` is a problem with a line, as every
  other problem is: the sidebar reads `config.toml:4:` and the terminal marks
  the entry of the action that goes without.  The problem says which action
  keeps the key: `F9` is bound to both go and next, so go keeps it and next
  loses it.  Problems come out in the order of the file, by line, in the sidebar
  and on standard error, those that lie in no line last.
- The key of the next or previous match, pressed before any search, says that
  there is no previous search, where it said that no cue matches.
- A paste into another show tells what that show could not take, as `Pasted 3
  cues, dropped 2 sends and 1 target`, with the key of undo, only when it
  dropped something: a send whose output has no namesake there, or a target
  outside the cues pasted.
- The Target column of a start, stop, pause, devamp, GoTo, or fade cue names the
  cue list of a target that lies in another list, as `9 in Effects`, where it
  gave the bare number, which could be taken for a cue of the same list.  The
  entry of a running fade or devamp cue in the Active list names its target
  after the time, as `0:04.0 left, fades 1`, so that two running fades tell
  apart, and leaves the words out where the name would not fit whole.
- The warning of a network cue whose message leaves a double quote open, and
  the line above its inspector, say so: `OSC message leaves a double quote
  open`.
- The Levels tab of a light cue lists an instrument that the cue names and the
  patch lacks, as `99 (unpatched)`, after those of the patch, so that its level
  can be read and cleared, which puts an `UNPATCHED` cue right.
- A show file that cannot be read, as one without permission or a directory, is
  told in the sidebar as `Cannot open x.cinq.toml: permission denied`, without
  the error number, and standard error holds the whole error with the path, as
  for a failed save.
- The box of the keys that can come next counts on the doubled line too: after
  `3d` it reads `Cut the selected cue and the next 2` for `d`, as `3dd` cuts
  three cues, where it ignored the count.
- An `osc_listen` that names a host, as `localhost:53000`, reads `osc_listen
  must be IP:port, not a host name`.  A packet of more than 32 messages shows
  the dropped rest in the sidebar as well as on standard error, and a rare
  failure to listen reads as the text of the system.
- A paste into another show also counts the light cues that name instruments
  which its patch lacks, as `Pasted 3 cues, 2 unpatched`.
- A key that show mode locks has the status bar say `Leave show mode to edit`,
  which names the way out, where it said `Show mode does not allow edits`.  The
  notice of an edit to an output leaves out `the output`, as `Changed the
  channels of Right`, so that the name survives where the bar cuts the notice at
  1024 pixels.
- The notice that a cue plays nowhere, as `Changed cue 2 to play nowhere`, stays
  whole on a tight bar, ahead of the keys that teach, such as what Enter does in
  a checkbox and the key that takes the change back.
- The Levels tab of a light cue, while the light patch holds no instrument, says
  to add one on the Light Patch tab, where it only said that the patch is empty.
- The Target column of a light cue counts every instrument that the cue sets, as
  the Levels tab lists them, so that a cue that names only an instrument that
  the patch lacks reads `1 instrument` beside `UNPATCHED`.
- The State column of a control cue that names itself, or a group for a fade or
  devamp cue, reads BAD TARGET, and a start cue inside the group that it starts
  reads IN TARGET, where both read NO TARGET beside a Target column that held a
  number.  NO TARGET stays for a cue that names none.
- A network cue whose destination has a port that is not a number from 1 to
  65535, as `127.0.0.1:99999`, reads `BAD PORT` and says so in the sidebar,
  where it read `NO DESTINATION`.
- The header of the Time & Loops tab says where the loop that plays differs from
  the one typed, in the warning color, as `Loop plays from 0:01.0 to 0:08.0`
  where a loop time lies outside the clip, which was silent.  The tab of a wait
  cue says that the Duration on the Basics tab is how long it waits, and that of
  a fade cue that it is the fade time.
- A cue name or notes with a line break or a tab in a show file read on one line
  in the title of the transport, its notes line, and the inspector, where the
  second line of a name was lost without an ellipsis and the notes showed gaps
  and a missing-glyph box.
- Hint mode also leaves when a warning comes or goes, since the sidebar then
  moves the places that its labels stand beside.
- The delays of the `GO fires` line read as a clock from a minute on, as
  `+1:10.0`, where they read `+70.0s`, as a wait ahead of a cue does.
- A system error that has no kind of its own, as `no such device or address`,
  reads in the words of the system, without its error number, in the sidebar
  and in the problems of cues, where it read `uncategorized error`.

## v0.1.6

- A level typed into a field, that of a cue, a send, the new cues of an
  output, or an instrument of a light cue, applies once the field is left, by
  Enter, Tab, a click, GO, or any other key or click.  A cue that plays no
  longer follows each key, as the `2` of `-25` would have made it 28 dB
  louder, and an emptied send no longer silences a cue before a level is typed
  back.  GO fires a cue at the level typed.  The slider still applies as it
  moves.
- Changing the level of an audio or video cue while it plays moves it by the
  change instead of setting the level, so that a cue that a fade cue has
  lowered stays lower, where a nudge of its slider from 0 to -0.5 used to make
  it jump to -0.5 dB.  Moving the level to the bottom and back restores the
  level of the fade, and the list of running cues never shows less than
  -60 dB.  A fade under way that leaves the cue playing goes on, from where the
  change moved the level, instead of ending.
- Changing the level of a fade cue while it fades now sends the fade to the
  new level in the time it has left, and a fade cue that has yet to act fades
  to the new level, where the change used to wait for the next run of the
  fade cue.  Its duration, target, and stop setting still wait for the next
  run.
- Start, pause, and GoTo cues act on other cues, as those of QLab do, and
  take no time.  A start cue fires its target together with the cues that
  chain to it, as GO does, or resumes it where it is paused.  A pause cue
  pauses its target while it runs, wherever it is, together with the cues
  that wait for it to start them.  A GoTo cue moves the standby cue of the
  list of its target to the target, so that the next GO of that list fires
  it.  Each targets any cue but itself, in any list, chosen on the Basics tab
  of the inspector, and a new one targets the cue above it.  The sounds and
  pictures that a start cue fires are made ready ahead, as those of GO are,
  and a picture or video cue that it fires brings up the stage window.  The
  chapter of the tutorial on sequences covers pause and start cues.  cinq
  0.1.5 and older refuse a show with start, pause, or GoTo cues.
- An audio or video cue that loops can loop a section of its file, so that one
  file holds an introduction, a vamp, and an ending: the cue plays from its
  start to its loop end, then goes back to its loop start each time it reaches
  its loop end.  The Loop start and Loop end fields of the Time & Loops tab
  set them, blank for the start and the end of the cue, which loops the whole
  clip as before, and they are editable while the cue loops.  A loop that
  reaches outside the cue counts as the part of it that lies within, and a loop
  that starts at or after its end cannot play: the cue reads BROKEN and GO
  skips it.  The show file keeps `loop_start` and `loop_end` of a cue that
  sets them, looping or not.  cinq 0.1.5 and older ignore `loop_start` and
  `loop_end`, and loop the whole clip.
- Devamp cues, named as in QLab, let an audio or video cue that loops leave its
  loop: the run of the target plays on to the end of a pass through its loop,
  then from the loop end to the end of its cue, and ends there.  It leaves at
  the first end of a pass that lies more than half a second ahead, which the
  sound needs to leave its loop cleanly, so that a loop shorter than about half
  a second can go round once more than planned.  A devamp cue takes no time,
  and its target is any audio or video cue, in any list, chosen on the Basics
  tab.  A target that does not loop reads NO LOOP, and GO skips the cue.  A run
  that a devamp cue has let go on shows the time it has left, and fades, stop
  cues, pause cues, and panics act on it as on any run.  The chapter of the
  tutorial on sequences covers loop sections and devamp cues.  cinq 0.1.5 and
  older refuse a show with devamp cues.
- Group cues, which act as the timeline group of QLab does, hold the cues
  below them that lie one level deeper, and one GO on a group fires all of
  them at once, each after its own pre-wait and those of the groups that hold
  it.  Standby moves past the group, and the group goes on by its own Then
  setting, to the cue below it: auto-follow once the last of its cues ends.
  The Then setting of a cue inside a group has no effect, and the inspector
  shows it disabled.  A group shows as running while any cue in it runs, with
  the time left while the group has an end.  Stop, pause, start, and GoTo
  cues, and the Stop button, act on a group and on every cue in it, while fade
  and devamp cues cannot name one, and a start cue inside the group that it
  starts cannot play.  A new cue below a group goes into it, and takes no
  target from the group.  Pasted cues keep their shape, a group moves, is cut,
  copied, and deleted with the cues it holds, and undo takes each back in one
  step.  `>>` and `<<` in the vi keymap, and `Control+]` and `Control+[` in the
  cua keymap, move a cue into the group above it and out of its group, and take
  counts and motions as `d` and `y` do.  The show file keeps each cue's
  `depth`.  cinq 0.1.5 and older refuse a show with group cues and ignore
  `depth`.
- A group collapses in the cue list, as a fold closes in Vim: the list shows
  it as one row, with a `+` before its name where an open group has a `-`, and
  leaves out the cues that it holds.  `z c`, `z o`, and `z a` collapse, expand,
  and toggle the selected group in the vi keymap, `z M` and `z R` act on every
  group, `ArrowLeft` and `ArrowRight` collapse and expand in the cua keymap,
  and a click on the mark flips a group.  A collapsed group still fires, stops,
  and pauses its cues, and its row reads NEXT while standby lies among them,
  and the problem of the first of them that has one, such as NO FILE.  Every
  key and count that moves through rows counts a collapsed group as one row,
  and a search, a jump, undo, or GO that selects a hidden cue expands the
  groups that hide it, while those that the search line opens to preview a
  match close again when the match moves on or the search is cancelled.  A
  list switched to selects the collapsed group that hides its standby cue.
  Collapsing is no change for undo, show mode allows it, and the show file
  keeps it as `collapsed = true` on the group.
- The transport ends the name and the notes of the standby cue in an ellipsis
  where they run out of room, where they were cut without a sign, and the Show
  mode checkbox moves to the toolbar, so that the notes take the whole width
  under the title: a long note shows 89 characters instead of 56 at 1024x768.
- A standby cue that has a problem reads NEXT and the problem on lines of their
  own in the State column, where the problem used to hide NEXT, and the column
  holds NO DESTINATION on one line.  The column reads `fading` for a cue that a
  fade or Panic is fading, and OUTPUT FAILED in yellow, under its state, for a
  cue that sends only to outputs whose devices failed, which the line under the
  standby cue tells as silent, as it tells a cue that sends to no output.  The
  cue list leaves out a Pre, Post, Then, or Target column that no cue fills,
  lets other columns give way while a name lacks the room that its names need,
  and lets the Target column grow into the room that the names leave, so that
  a long file name wraps as late as it can.
- The sidebar keeps each cue list to one line, cutting a long name short in an
  ellipsis and keeping the number of its cues, and scrolls to the list that
  shows whenever it changes, and back to it as Tab moves on.  A running cue
  keeps its name and its status to a line each, around its bar, with the time
  left and the level first, so that a status cut short loses the words of its
  phase before them.  The sidebar grows to 560 pixels on a wide window, and an
  empty Active or Warnings list leaves out the gap of its empty body.
- The status bar hints a key only where it acts: while a text field has the
  keyboard it no longer offers undo, redo, paste, add, rename, or help, which
  would only type there, and while a closed drop-down list has it, only undo
  and redo of these.  Where the keymap binds no key to cancel, as the cua
  keymap does, it hints Enter to leave a field.  It hints the key of standby
  while the selected cue is not the standby cue, tells no notice while a menu
  is open, so that the keys that pick and close it show, and in the outputs
  view tells changes to the outputs only, so that the keys of that view show.
  The rename of a cue just added outlasts the hints of GO, Panic, and saving
  at 1024x768, the key of the next inspector tab is hinted last of all, and a
  gap keeps the status apart from the hints.  The status counts warnings only
  while there are any, which leaves the hints the room of `0 warnings`.  A key
  that edits in show mode, or while the stage window has the keyboard, now
  says once why it did nothing.
- The inspector tells in words what the color of a field told alone: its header
  says why GO skips the selected cue, such as a missing file or no target, and
  which time reads as no time, with the forms that a time takes, and a refused
  level, send, light level, or address gives its reason beside the field or
  above or under its table.  A field that the cue ignores says why, as
  "(in a group)", "(with auto-follow)", or "(not looping)", and a value that
  cannot change draws on the black of the window, where it reads at 7:1, in a
  box of the height of a field that shows the dim placeholder where the value
  is empty, so that the inspector keeps its height as GO moves the selection.
  Tab no longer stops on a field that cannot change, and it reaches the Loop
  and Stop target checkboxes, which Enter flips and show mode locks.  The sends
  of the Levels tab lie side by side in rows that wrap, so that most of them
  show at once, each with as much of its output name as names need, up to 32
  columns, keeping the end of a longer one.  The inspector grows to a quarter
  of a window taller than 960 pixels, the Basics tab gives the larger share to
  the name and the file, the destination of a network cue has a row of its
  own, a kind with no target has no Target row, and the Light Patch tells that
  it has no instruments, and what an address takes only while an address has
  the keyboard or is refused.
- Tab reaches the Show mode checkbox, which show mode never locks, and Enter
  flips it and gives the keys back to the cue list, so that the keyboard alone
  locks the show before a performance and unlocks it after.  On the Light
  Patch, Tab reaches Add instrument and each Remove, which Enter presses and
  show mode locks.  Tab past the last field, and Shift+Tab before the first,
  leave every field and give the keys back to the cue list, or the outputs, so
  that a checkbox or a closed list no longer keeps them in the cua keymap,
  which has no key to leave a field.  F11 out of fullscreen gives the main
  window the keyboard.
- The status bar hints `Enter: flip` while a checkbox has the keyboard,
  `Enter: open` while a closed drop-down list does, and `Enter: press` while a
  button does, and the key that shows the next cue list, or the outputs after
  the last one, as `l: next list` in vi.  At the end of a list that another
  follows, where GO fires nothing, that hint stays on a short bar until after
  GO and Panic.  While a field has the keyboard, GO, Panic, and the key that
  leaves it outlast the keys that only teach, and while a text field has it,
  the bar hints a key of GO that types nothing there, or none, since Space
  types a space.  The rename of a cue just added outlasts the help hint too.
- The first paragraph of the help page names the key that closes it, the key of
  standby opens the tutorial from the page, and the table of keys takes the
  width of the window up to one and a half times that of the paragraphs, so
  that most descriptions fit one line, while the paragraphs keep their width.
- The outputs view hands the room that the device does not need to the name
  and the state, so that they wrap later in a narrow window, and its notices
  say that a cue sends to no output, as the State column does.
- A path typed into the File field of an audio, video, or picture cue names the
  cue after the file, as the file chooser does, while the cue goes by the name
  it was added with or by the name of its previous file.  The name follows the
  path as it is typed, and a name that was typed stays.  While a field has the
  keyboard, the key that leaves it outlasts a long notice in the status bar.
- A narrow main window, as one that a tiling compositor sets beside the stage
  window on one display, keeps every control and value readable: the toolbar
  wraps onto a second line, the buttons of the transport go under the title,
  the two forms of the inspector stack and its header wraps, the inspector of
  an output scrolls, the sidebar keeps 160 pixels, and each cell of the cue
  list keeps within its column, breaking a word that does not fit, while the
  State column keeps its words whole and the Action column gives way last.
  The outputs view lets State and then Channels give way, the Light Patch its
  Now column, and the help page the names in the configuration file, and an
  open menu stays within the window, shortening its rows.  A column that gave
  way returns only while every name keeps the room it needs, up to 48 columns,
  so that a wider window never wraps a name that a narrower one did not.  A
  notice too long for the status bar ends in an ellipsis, and the search line
  keeps a gap before its note.

## v0.1.5

- `back` and `forward` go back and forward between the places that jumps left,
  as the jump list of Vim does: switching cue lists, `first`, `last`, selecting
  a cue in another cue list, searching, and showing the outputs and leaving
  them are jumps.  The keys are `Control+o` and `Control+i` in the vi keymap
  and `Alt+ArrowLeft` and `Alt+ArrowRight` in the cua keymap, and the status
  bar names them while there is a place to go to.  A jump after going back keeps
  the places ahead, as in Vim, going back or forward in a text field moves the
  keyboard to the cue list, and the back and forward buttons of the mouse do the
  same from the main window, unless the help page is open.
- The status bar says where the keys of back and forward go, as `Control+o:
  back to Main`, `back to cue 3`, or `back to the outputs`, so that a pass
  through an empty cue list on the way to the outputs is no surprise, and it
  cuts the name of a long cue list short.  While the outputs show, it names the
  keys that go back, add an output, and rename one, and the key that goes back
  is the last of them to give way to a narrow bar.
- In the vi keymap, the key of *open* is now `Control+l`, since `Control+o` is
  Vim's way back through the places jumped to, and Vim's `Control+l` only
  redraws the screen, which cinq never needs.  The cua keymap keeps
  `Control+o`.
- `Control+Home` and `Control+End` select the first and last cue as `gg` and
  `G` do, each with a count.  `+` and `-` select the next and the previous cue,
  as in Vim.
- A count typed before the keys of undo and redo turns that many changes, as
  `3u` does in Vim, and stops at the oldest or newest.  A count before
  `next_tab` shows the inspector tab of that number, as `2gt` does, and before
  `previous_tab` goes back that many tabs.
- A key that completes or continues no sequence of keys cancels what was typed
  so far and does nothing itself, as Vim beeps and drops the keys, so that a
  mistyped `dx` no longer cuts a cue, and `gj` and `gk` move the selection down
  and up in the vi keymap, as they do in Vim.  The keys of GO and Panic still
  act at once.
- `Control+c` cancels in the vi keymap, as in Vim, through the new `cancel`
  action, which has no key in the cua keymap, where `Control+c` copies.  It
  drops the keys typed so far, closes the search line with the selection going
  back to where the search began, takes the keyboard out of a text field, and
  closes an open drop-down list without picking, and never stops playback, so
  that no one reaches for `Escape`, which is Panic.  The status bar hints the
  key while keys wait or the search line is open, and a text field with a
  selection still copies it first.
- While the stage window has the keyboard, the keys that change the show or fill
  the clipboard, such as `x`, `Delete`, `o`, `u`, `p`, and the operators, do
  nothing, as the keys of editing a name, of searching, and of the help page
  already did, so that a key held down there cannot change cues out of sight.
  GO, Panic, standby, and the moves still act.
- The help page, the box of the keys that can follow, and the status bar write
  a sequence of keys that type characters joined, as Vim does: `dd`, `gg`,
  `gt`, and `zz`, no longer `d d`, since a newcomer pressed the Space between
  them, which is GO.  A sequence with a named key, such as `z Enter`, keeps its
  spaces, as does `config.toml`, and the help page says that the keys of a
  sequence are typed with no Space between them.
- The keys of undo and redo act while a closed drop-down list has the keyboard,
  as `Control+z` does in the cua keymap, where they did nothing and said
  nothing.  A text field still keeps them, since the text typed there is typed
  into it.
- The keys of Vim that move by a page work in the vi keymap: `Control+d` and
  `Control+u` scroll the cue list half a page and move the selection the same
  number of rows, so that it keeps its row on the screen, and `Control+f`,
  `Control+b`, `PageDown`, and `PageUp` scroll a whole page less two rows and
  select the first row in view going down, or the last going up.  A count before
  `Control+d` or `Control+u` sets how many rows they move, until the window
  changes size or another show opens, and a count before the whole pages counts
  them.  The same keys scroll the help page, and the cua keymap binds `PageDown`
  and `PageUp` to the whole pages.
- `Control+e` and `Control+y` scroll the cue list a row down and up in the vi
  keymap, as in Vim, or as many rows as a count, through the new `scroll_down`
  and `scroll_up` actions.  The selection stays on its cue unless it would leave
  the view, when it moves to the nearest row in view.  The keys of `pick` and
  `close_menu` act only in a menu, so a key may be bound to one of them and to a
  scroll.
- `H`, `M`, and `L` select the first, middle, and last cue row in view in the vi
  keymap, as in Vim, through the new `top_row`, `middle_row`, and `bottom_row`
  actions.  A count before `H` counts rows from the top and before `L` from the
  bottom.  They are jumps for `back` and, unlike the motions `j`, `k`, `gg`, and
  `G`, take no operator.
- `zz`, `zt`, and `zb`, and `z.`, `z Enter`, and `z-`, put the selected row in
  the middle, at the top, or at the bottom of the view in the vi keymap, as in
  Vim, through the new `row_to_middle`, `row_to_top`, and `row_to_bottom`
  actions.  A count selects that row first, as `{count}zt` does.  The selection
  stays on its cue, and the view stops at the ends of the list.
- A move or jump that leaves the selected cue far outside the view, such as `G`,
  `n`, or going back, scrolls the list to put it in the middle, with Vim's
  thresholds: seven rows above the view and 13 rows below it in a view of 23
  rows.  A move to a row near the view still scrolls by the least, as does GO
  moving the standby a row.
- Cues can be copied, cut, and pasted with a clipboard of cinq's own that
  outlasts the show it was filled from: `yy`, `Y`, `p`, and `P` in the vi keymap
  copy and paste, `dd`, `x`, and `Delete` cut, and `Control+c`, `Control+x`,
  `Control+v`, and `Control+V` do the same in the cua keymap.  The new `copy`,
  `cut`, `paste_below`, and `paste_above` actions carry these keys.  The cua
  `Delete` key still deletes without copying, and `delete` has no key in the vi
  keymap, where `cut = []` in `config.toml` now lets no key remove a cue, as
  `delete = []` did.
- A count copies or cuts that many cues and pastes that many times, a paste is
  one change that undo takes back, and a fade pasted together with its target
  targets the new copy, as in QLab.  The status bar tells what was cut or copied
  and names the key that pastes, and undo and redo still call a cut a cut.  A
  paste with nothing to paste says that the clipboard is empty, and the `copy`
  action works in show mode, as Vim yanks in a read-only buffer.
- In the vi keymap, `d` and `y` take a motion, as in Vim: `dj` cuts the selected
  cue and the next, `yk` copies it and the previous, and `dG`, `dgg`, and `d3G`
  reach the last cue, the first, and row 3.  Counts before the operator and the
  motion multiply, so that `2d3j` cuts seven cues, and the box above the status
  bar lists the motions after the operator.  The operator key is the doubled key
  of the binding, so that `cut = ["D D"]` makes `DD` and `Dj` cut, and a cut of
  several cues is one change for undo.
- A count before the keys of `add_below` and `add_above`, such as `3o`, adds
  that many cues in a row, at most a thousand, as one change for undo, and
  selects the last of them, as the cursor of Vim goes to the last line that `3o`
  inserts.
- Undoing the addition of a cue or a paste below a cue selects the cue that was
  selected when it was made, as the cursor of Vim goes back to where a change
  was made, in place of the cue that follows.
- A search line opens with `/` in the vi keymap and `Control+f` in the cua
  keymap, and `search_backward` opens one for a search up the list on a key
  that a keymap in `config.toml` gives it.  `?` and `F1` both show the help page
  in both keymaps, as an exception to Vim's `?`.  As the text is typed, the
  selection moves to the first cue after it that holds the text, in any case,
  in any column that the list shows except the waits and the state, so that
  `/rumble` finds the cue of `rumble.ogg`, as Vim's search matches a whole
  line.  While the outputs show, it searches their name, device, channels, and
  new cues, so that `/Dante` finds the outputs on that device.
- `Enter` accepts a search, and on an empty line repeats the last one.
  `Escape`, `Backspace` on an empty line, a click, going back or forward,
  moving to another field, and opening a show close the line.  `n` and `N`, or
  `F3` and `Shift+F3`, find the next match in the direction of the last search
  and in the other, a count goes round the matches at most once, and the status
  bar says when a step wrapped around or found none.
- Searches are jumps, so that `back` returns to where a search began or to the
  cue before a step to the next match.  The search line takes the keys of Vim's
  command line: `Control+w` deletes the word before the cursor, `Control+u`
  everything before it, and `ArrowUp` and `ArrowDown` recall the older and newer
  of the last fifty searches accepted that start with the text typed.  The keys
  act on the end of the text.
- The vi keymap enters the name of the selected cue, or of the output, as Vim
  enters text: `i`, `a`, and `A` put the caret at its end, `I` at its start, and
  `cc` and `S` select all of it, so that typing replaces it.  The cua keymap
  selects the whole name with `F2`, as desktop file managers do, and in both
  keymaps `Tab` and `Shift+Tab` select the text of the field that they reach,
  while a click puts the caret where it fell.  The new `edit_start` action has
  no key in the cua keymap, `change_name` has `F2`, and `c` is no operator, so
  only `cc` begins a change.
- `Tab` from the cue list or the outputs goes first to the first field of the
  inspector, so that the settings of the selected cue are one key away.  It then
  goes on through the rest of the inspector, the toolbar, and the sidebar, and
  round again, and `Shift+Tab` goes the other way.
- `Tab` onto a send on the Levels tab of a cue scrolls it into view, with the
  name of its output above it, where it stayed out of sight below or above the
  part of the sends that showed.  `Tab` onto a list of the sidebar does the
  same, with its title, where a warning in a low window pushed the Default
  device list out of sight.
- Enter in the empty File field of an audio, video, or picture cue opens the
  dialog that chooses the file, which Tab never reaches since it skips the
  Choose button, and the status bar hints `Enter: choose a file` while the
  empty field has the keyboard.
- Each visit to a field, such as the name of a cue or of an output, is one step
  of undo, where two visits in a row, each ended by leaving the field, undid as
  one.  A click, or a move to a field, ends a visit as well.
- In an open drop-down list in the vi keymap, `Control+y` picks the highlighted
  option and `Control+e` closes the list without picking, as in Vim's popup
  menu, and `F4` closes it in the cua keymap, through the new `pick` and
  `close_menu` actions.  Closing with these keys does not also panic, as
  `Escape` does, and after a choice or a close by key the keyboard stays on the
  list, so that `Tab` goes on to the next field.  `gg` and `G`, `Home` and
  `End`, or the keys of `first` and `last`, go to the first and last option, and
  `PageDown` and `PageUp` move by the options in view.
- While a drop-down list is open, the status bar hints the keys that pick and
  close it, as `Control+y: pick | Control+e: close` in the vi keymap and
  `Enter: pick | F4: close` in the cua keymap, and gives way when the bar is
  short.  While a text field or a drop-down list has the keyboard, it hints the
  keys of the next field and, in the vi keymap, of leaving it.
- The help page and the table of cinq(5) list the actions in groups, GO, Panic,
  and standby first, then the moves, then the edits, then the files and the
  window, in place of the order in which actions were added.
- The help page leaves out the actions that the keymap in use binds to no key,
  such as `delete` in the vi keymap, so that its table is shorter.
- The help page opens with a short paragraph of first steps, above the table of
  keys, that names the keys of GO and Panic, of moving the selection and of
  standby, and of adding, naming, and undoing, in the keymap in use, since the
  table begins with the keys of scrolling and a newcomer found `o` and `cc` only
  on its second page.
- The help page links to the tutorial below its first steps, and following the
  link opens the PDF in the program that the desktop opens it with.  cinq looks
  for it in the XDG data directories, such as `/usr/share`, where the packages
  for GNU/Linux put it, in `/usr/local/share` on Darwin, and beside `cinq.exe`
  on Windows, and says where it looked when it finds none.  Why a link failed to
  open shows below the page, in view however far the page has scrolled, where it
  showed near the end of the page.
- The GO button looks disabled, as a disabled button does, while no cue that GO
  would fire can play, and the line under the name of the standby cue says why
  in words, as `GO fires nothing: cue 1 (no file set)`.  The notes and that line
  give way at a word where they meet the controls, no longer in the middle of
  one.
- After a cue is added, the status bar hints the key that renames it, as `Added
  cue 1 | cc: rename | u: undo` in the vi keymap and `F2: rename` in the cua
  keymap, where only the key of undo followed, and the new cue kept the name
  that a newcomer then found no key for.
- The list of cue types beside Add cue carries the label `New cue:`, since it
  only sets the type of the next cue that Add cue makes, and a newcomer took it
  for the type of the selected cue.
- Sounds play through a mixer of cinq's own, which opens each audio device on a
  thread of its own, so that a sound can reach any channel of a device and a
  slow device, as a Dante Virtual Soundcard that restarts, holds up no window,
  clock, GO, or other device.  An output shows no warning until its device has
  answered, and reads not answering, in yellow, once five seconds have passed
  without an answer.  A device whose stream stops says so among the warnings,
  where the sound once moved to the default device.
- A show has outputs, the named places that its cues send their sound to, each
  on one channel or two of an audio device, such as the house speakers on the
  first two channels of a desk and the subwoofer on its third.  Each audio and
  video cue sends to the outputs it names, each at a level of its own, and
  moving an output to other channels or another device moves every cue that
  sends to it.  Show files hold them in `output` and in the `sends` of each
  cue, as cinq-show(5) describes.  cinq 0.1.4 and older open such a show
  without them, and drop them when they save it.
- A show without outputs plays on the first two channels of the default device.
  An output that cannot play says why among the warnings, and the cues play on
  the others.
- The Outputs entry below the cue lists in the sidebar shows the outputs of the
  show in place of the cue list, each with its device, its channels, and in a
  few words why it cannot play where it cannot: in yellow where cinq tries its
  device again by itself, and in red where the output must change.  The Device
  column keeps at least 20 columns, the others giving way to it.  The outputs on
  a device that fails share one warning in the sidebar, which shows them when
  pressed.
- The inspector edits the selected output, whose Device list reads not found,
  in yellow, while its device is not there.  A name or a level applies as it is
  typed, and channels once Enter, which leaves the field as in every other, is
  pressed or the field is left, since a device opens again for them.
- The Audio device list of the sidebar is now Default device, since it chooses
  the device of the outputs that name none, and its first option reads System
  default.
- The Levels tab of an audio or video cue sets the level at which the cue sends
  to each output of the show, and a field left empty sends nothing there.  The
  fields scroll, so that every output stays in reach.  The name of an output
  that cannot play says what is wrong in words, as `House L (not found)` or
  `Lobby (past its channels)`, in yellow, or in red where the output must
  change, beside how many of those that the cue sends to cannot.
- The tutorial has a chapter on outputs, in which the reader sends the sound of
  a cue to each speaker of the computer in turn, and its first chapter names the
  outputs and the default device among what the sidebar shows.
- A device that is not there, or that stops while it plays, as one unplugged or
  a Dante Virtual Soundcard that restarts, opens again on its own within two
  seconds of its return, and so does a device whose sample rate changes, as
  Dante Controller can change that of the soundcard.  The cues that play on it
  pick up there from where they are, at the new rate where it changed.
- While a device is away, the cues on it wait for it until it tells that it
  stopped, or until it has asked for no sound for three seconds, then go on by
  the clock of the show, so that the cues that follow them still fire on time.
  A device that has just opened may take up to ten seconds for its first call
  for sound, and the clocks of the cues on it wait at the start until then.
- A device that has fewer channels than an output names opens again once it
  offers them, and is looked at ever more seldom, up to every thirty seconds,
  until what it offers changes.  Changing an output opens again only the
  devices that the change touches, so that the sound on the others plays on.
- On GNU/Linux and the BSDs, cinq also plays on the device of a running JACK
  server, such as PipeWire's, as `jack:output`, whose channels each reach the
  playback port of their number.  PulseAudio spreads a stream over every channel
  of a device that has more, and the ALSA plugin of PipeWire takes the channels
  past the second for those of surround sound, both of which JACK leaves alone.
  Building on GNU/Linux needs JACK, unless the build leaves out the default
  features, and the device lists leave out the null device of ALSA, which takes
  sound and plays none.
- Audio devices that take samples only as integers of 32, 24, or 16 bits, such
  as the `hw` devices of most sound cards on ALSA, now play, where cinq played
  floating-point samples alone.  A mix past full scale clips there instead of
  wrapping into a crack.
- On Windows, the build that Nix makes also plays on ASIO drivers, such as that
  of the Dante Virtual Soundcard, which offers all of its channels as one device
  where its WDM mode offers them in pairs.  The ASIO SDK of Steinberg goes into
  that build under the GPL version 3.
- On Windows, the thread that plays sound on WASAPI devices, as the WDM mode of
  the Dante Virtual Soundcard offers, asks Windows to schedule it for audio, as
  audio programs do there, so that a busy machine is less likely to break up the
  sound.
- Going back from the outputs, or leaving them with the key of the previous cue
  list or its entry in the sidebar, shows the cue list that showed before them
  with the cue that was selected there and its inspector tab.  Where no jump
  recorded a place, as after opening a show, it shows the cue list that was
  showing, and any other cue list shows with its standby cue.
- The keys that move through the rows of the cue list, such as `j`, `k`, `gg`,
  and `G`, the keys of half and whole pages, the keys that scroll by a row or
  select a row in view, and the keys that place the selected row, do the same to
  the outputs while they show, and the key of the next cue list reaches them
  past the last one.  The keys that act on cues alone, such as standby, do
  nothing there, so that none of them changes a cue list out of sight.  The
  outputs show their titles above the rows, as the cue list does.
- The keys that edit cues edit outputs while they show: `o` and `O` add one, and
  a count adds that many, up to 64, `dd`, `x`, and the motions such as `dj` cut
  them, `yy` and `Y` copy them, `p` and `P` paste them, `Alt+j` and `Alt+k` move
  one, and the keys of the name enter it.  Outputs have a register of their own
  beside that of cues, and a show keeps at least one output.
- A cut output pasted back into the same show takes back its identity, so that
  the cues that sent to it send to it again, while a copy is a new output that
  no cue sends to, and a copy of `House R 2` is `House R 3`.  The status bar
  tells first where the cues that name no sends play when the first output
  changes, and names the output that moved or was pasted.  Undo and redo select
  the output that they changed, except that undoing an addition or a paste
  selects the output that was selected when it was made, and undoing a change to
  a cue shows the cue list, as a jump that going back returns from.
- A cue pasted into another show sends to the outputs of the same names there,
  and plays on the first output where that show has none of them.
- A cue that sends only to outputs that a cut or a delete took away plays on
  no output, which the status bar counts with the change, as in `1 cue now
  sends nowhere`.  The State of such a cue reads `NO OUTPUT` in yellow, and the
  sidebar lists it among the warnings, until undo or a paste brings the outputs
  back.  A cue that names no sends still plays on the first output.
- The box that lists the keys that can follow those typed, such as the ones
  after `d`, names outputs where the outputs show, as in "Cut the selected
  output and the next", and cues where a cue list shows.  The help page, which
  lists each action once, names both.
- The fields of an output in the inspector are as wide as what they hold, and a
  line of help beside each says what it takes and what it does.  The line turns
  red and tells why where a value is not applied, as for a name that another
  output has, channels that are neither one nor two, or a level out of range,
  and an output whose device is not there also tells what to do.  Where the
  keyboard leaves a field that holds a value that was not applied, the status
  bar tells why, as `Not applied: another output has this name`, since the field
  shows the old value again.
- The default option of the Device list of an output names the choice of the
  Default device list in the sidebar, as in `Default device (System default)`,
  and while the highlight of the open menu rests on that option, by the mouse or
  the keys, the Default device list in the sidebar turns the blue of the
  highlight, which shows where the default is set.  An output on the Default
  device whose device is not there keeps its name in its list, as in `Default
  device (Studio), not found`, and its help says to connect it or to set the
  default device in the sidebar.
- The sidebar lists the warnings above the Default device list, which a show
  sets once, and at most three of them, then a line such as `and 5 more`, so
  that at 1024x768 the whole button of a warning shows and the Default device
  list stays in reach without a scrollbar.  The warnings count in the title and
  in the status bar is still the whole count.
- The outputs table drops its New cues column once the Device column cannot
  show its widest name on one line, as at 1024x768 beside a Dante Virtual
  Soundcard, and keeps the column in a wider window, where it fits.  The
  inspector keeps the value as New cues send.
- A long name of the show gives way first in the status bar: it is cut short at
  its end with an ellipsis, down to 16 columns, before any key hint goes, so
  that the mode and the warning count stay, and the keys of GO and Panic as well
  while no notice shows.
- The border of a field or a list that has the keyboard is two pixels thick,
  in the same blue, where it was one, so that it stands out from the fields at
  rest at a glance.  A field that holds a rejected value keeps its red border,
  thick while it has the keyboard.
- Red text on the selected row of the outputs table and the cue table is a
  lighter red, with a contrast of 4.9 to 1 against the row where the red of the
  rest of the window had 3.9 to 1, below the 4.5 to 1 that text needs.
- The text of a drop-down list that lies across the lower edge of the sidebar,
  as the Default device list does at 1024 by 768 pixels with a notice in the
  status bar, no longer draws below the sidebar, over the inspector, where the
  software renderer let it spill.
- Light cues fade the instruments of a light patch to levels of their
  own.  An instrument keeps its level until a later light cue names it, as
  in QLab.  The show file holds the patch, which the inspector edits on its
  Light Patch tab.  cinq sends the levels as Art-Net and as sACN on every
  network that the computer is on.  Universes count from 1, as in sACN.
  Each instrument has one level, its intensity.  cinq 0.1.4 and older
  refuse a show with light cues, and drop the patch when they save one.
- On Darwin, `cinq.app` says why it reaches the local network, and it is
  signed under the identifier of its bundle, so that Allow takes effect.
- The tutorial has a chapter on lights, which runs the light cues of a new
  demo show, `lights.cinq.toml`.
- Network cues send an OSC message as they start, over TCP or UDP, as
  those of QLab do.  A GO can then fire a cue of a lighting console, or of
  any other device or program that takes OSC, alongside sound and video.
  The inspector edits the destination and the message of a network cue on
  its Basics tab.  The chapter of the tutorial on lights covers them too,
  with a cue that sends its message to a program that prints it.  cinq 0.1.4
  and older refuse a show with network cues.

## v0.1.4

- While the problems of `config.toml` take a screen of their own on a
  terminal, what cinq and its libraries print meanwhile, such as the
  messages of the sound system, waits and comes out on the main screen once
  the problems clear, or before the problems left when cinq ends, in place
  of being lost when the terminal switches back.  On Windows, only cinq's
  own messages wait.
- A drop-down list too narrow for its options cuts the chosen one short
  with an ellipsis, and its menu grows as wide as the longest option.
- `nix build .#demo` builds the demo shows with the sounds, picture, and
  film that they play, which `doc/demo/build.sh` generates with ffmpeg, in
  place of the `demo` branch and its files in Git LFS.
- A tutorial in eight chapters runs the storm show of the demo.
  `nix build .#doc` builds it into `share/doc/cinq/tutorial.pdf`, with
  screenshots that it takes of cinq in a headless sway session, and puts
  the demo beside it in `share/doc/cinq/demo`.  Systems other than
  GNU/Linux build it on a GNU/Linux builder, such as `nix.linux-builder` in
  nix-darwin.  The deb, rpm, Arch, and Darwin packages carry both.
- On GNU/Linux and on Apple silicon, `nix build .#darwin-pkg` builds an
  installer package for Darwin on Apple silicon, which puts `cinq.app` in
  `/Applications` and the `cinq` command, its manual pages, its
  completions, and the tutorial with its demo in `/usr/local`.  The
  package is unsigned.
- The clock of an audio cue follows how far its sound has played, as that
  of a video cue does, so that its fades, its end, and the cues that follow
  it stay with what is heard.  A sound that drifts from the clock of the
  show, or a device that waits before it plays, no longer leaves the cue to
  end before its sound and cut off its last moments.
- The sound of audio and video cues can play on another audio device than
  the default device of the system.  The Audio device list in the sidebar
  chooses one, and `config.toml` keeps the choice in `audio_device`, by an
  ID that `cinq --audio-devices` lists along with the name of each device.
  A choice applies at once, and the sounds that play go on from where they
  are on the new device once it starts.  The list asks for the devices each
  time it opens, so that a device plugged in since shows.

## v0.1.3

- On Windows, the unsaved changes of shows with the same path on different
  drives, or on different network shares, no longer share one kept file and
  take each other's place.  The drive or share now leads the kept file's
  path, so changes that an earlier version kept for a show on Windows do
  not come back.  Other systems keep their kept files as they were.
- cinq plays up to 1024 sounds at once, where it took the 129th for a
  broken file and kept checking that file again.
- Stopping a cue also cancels the cues of its chain that it would have
  started, even when a cue between them had been fired again or could not
  play.
- GO on a cue that is already running restarts it and cancels the cues that
  its earlier run had yet to start, as QLab does, in place of leaving them
  to play on.
- The help page fills the main window in place of a box over it, and closing
  it gives the window back as it was, with the cue list scrolled where it
  was and the keyboard in the same field.  A field of the main window no
  longer takes the keys typed while it shows.  Besides the keys of the page,
  only those of GO and Panic act.  The text stays in a column at most 960
  pixels wide.  The key of Help opens the page from the main window only,
  since a fullscreen stage window may cover the main window, and does
  nothing while the stage window has the keyboard.  The key of Help also
  opens the page while the menu of a drop-down list is open, and the menu is
  open on the same option when the page closes.
- Opening a show right after switching away from it, while cinq is still
  keeping its unsaved changes, brings back the newest of them and no longer
  older ones.
- A failure to keep unsaved changes now shows on a line of its own among the
  warnings, and no longer replaces the problem of opening or saving a show.
  It also takes back the status note that changes were kept.
- On the stage window, the pictures of earlier cues show around a picture
  of another shape than the window, as in QLab.  The space around it was
  black until the picture began to fade, when they appeared there at once.
- cinq makes ready the sounds that start within ten seconds, instead of those
  of every cue that one GO fires, so that a long chain of auto-continue cues no
  longer opens all its sound files at once.  `ready_ahead` in `config.toml`
  sets that number of seconds, 0 or more and 10 unless the file says
  otherwise, so that owners of small systems can open fewer files at once.
  A change applies as soon as the file is saved.  Sounds that start together
  are always made ready together.
- The clock of a video cue follows how far its sound has played, in place of
  the clock of the show, so that its picture and sound no longer drift apart
  over a long video, as they did by the difference between the clock of the
  audio device and that of the show.  A video without sound, or one whose
  sound has ended, still follows the clock of the show.
- A damaged video file now fails to load with a reason, where it could
  freeze or end cinq, and the files loaded after it load as they would.
- Damaged tags, chapters, or cover art in a video file no longer stop it
  from playing.  cinq reads no tags or cover art, and skips a damaged track
  other than the picture's, which it reports once on standard error.
- QuickTime `.mov` files with AAC sound, which cinq used to refuse, now
  load.

## v0.1.2

- On Windows, a terminal that starts cinq shows what cinq prints, such as
  the problems of `config.toml` and the output of `--help`, and waits until
  cinq closes.  Explorer still starts cinq without a console window.
- Video cues decode with rust_h264, a decoder written in Rust, in place of
  Cisco's openh264, so that building cinq compiles no C++, its program no
  longer needs the C++ library, and it builds on riscv64.  A sample that
  fails to decode now costs no picture but its own.
- On a terminal, the problems of `config.toml` take a screen of their own
  while any are left, as a pager's text does, so that fixing the file brings
  back the prompt and what came before it instead of a cleared screen.
  Problems still left when cinq ends stay on the terminal.
- The Nix packages for other systems go by the systems that they run on, as
  `.#aarch64-darwin`, `.#x86_64-windows`, and `.#aarch64-windows`, in place of
  `.#aarch64-darwin-cross`, `.#x86_64-windows-cross`, and
  `.#aarch64-windows-cross`.
- A picture fading out no longer jumps back to whole when a panic, or
  another fade that stops its cue, takes over; it fades on from where it
  was.
- Video 1280 pixels wide that is cropped below 720 lines, such as
  letterboxed high definition, shows in the colors of high definition.
- On Darwin, cinq follows the changes to `config.toml` in a directory
  reached through a symbolic link, such as one under `/tmp`.
- On FreeBSD and the other BSDs, cinq follows the changes that an editor
  writes into `config.toml` in place, and not only those that rename another
  file over it.
- A cue that GO fires right after a sound or video that could not play
  ended the last running cue no longer starts ahead by the time spent idle.
- Choosing again the file that a running audio or video cue plays, or
  typing its name with a space around it, leaves the cue playing.
- cinq formats `config.toml` each time it is saved, as taplo formats TOML by
  default, but never reorders keys or the items of arrays and inline tables.
  A file that is not TOML stays as it is.
- On GNU/Linux, Nix cross-builds cinq for FreeBSD, as `.#x86_64-freebsd` and
  `.#aarch64-freebsd`, and for GNU/Linux on riscv64, as `.#riscv64-linux`.

## v0.1.1

- Picture cues show a PNG or JPEG on the stage window until a fade, a stop
  cue, or Panic ends them.
- The stage window opens as a window, which the operator can move to the
  display of the audience, and each GO that fires a picture or video cue
  makes it fullscreen on that display.  The fullscreen key turns it back
  into a window until the next such GO.
- Problems in `config.toml` show on standard error each time that cinq
  reads the file, at startup and on each change, with the lines around each
  problem and its place marked, and the sidebar gives the line of each.  On
  a terminal, only the problems of the latest reading stay on screen, so
  that fixing the file clears it, and a file without problems prints
  nothing.
