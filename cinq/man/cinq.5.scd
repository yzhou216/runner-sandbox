cinq(5) "cinq" "File Formats Manual"

; SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
; SPDX-License-Identifier: GPL-3.0-or-later

; The lines after "; if" belong only to the pages of the platforms that it
; names, xdg, darwin, or windows, until the next "; if" or "; end".

# NAME

cinq - configuration file of cinq

# SYNOPSIS

; if xdg
_$XDG_CONFIG_HOME/cinq/config.toml_
; if darwin
_~/Library/Application Support/cinq/config.toml_
; if windows
_%APPDATA%\\cinq\\config.toml_
; end

# DESCRIPTION

*cinq* is a cue-based show controller in the spirit of QLab, and the keyboard
runs most of it.
It ships one keymap, *vi*, as *KEYMAPS* describes it.
A configuration file in TOML keeps it or declares keymaps of its own under
*keymaps*,
binds actions to other keys, sets the audio device that outputs play on by
default, how far ahead *cinq* makes sounds ready, and the address on which it
takes OSC commands.
*cinq* reads the file when it starts and again whenever it changes, so that the
keys, the help page that lists them, and the other settings follow each change
as soon as it is saved.
After each change, *cinq* first formats the file in place, as taplo formats
TOML by default: it evens out spaces and blank lines, lines up the comments at
the ends of lines, and folds each array onto one line or unfolds it over
several to fit 80 columns, but never reorders keys or the items of arrays and
inline tables.
A file that is not TOML, or that *cinq* cannot write, stays as it is, and an
editor that has the file open may offer to load it again.

The help page, which the key of *help* opens, names the keymap in use, opens
with a short paragraph of first steps that names the keys of GO and Panic, of
moving and standby, of adding, naming, and undoing, of reaching the File field,
of Enter as the key that chooses a file, of hint mode, of closing the page, of
opening the tutorial, and of saving the show,
links to the tutorial, which the key of *standby* opens too,
lists each action that the keymap binds to a key with its keys and, in a window
wide enough, its name in the file, links to the file that the keys come from,
or would come from, and ends with the notice of the license of cinq and a
Close button.
Following the link to the file opens it in
; if xdg windows
the program that the desktop opens it with,
; if darwin
the default text editor,
; end
and first creates the file when it is not there yet, holding only a line
that points to this page.
Following the link to the tutorial opens the PDF that *FILES* names in the
program that the desktop opens it with.
Why a link failed to open shows below the page until the page closes.
A tutorial that is not installed is told in one line there, and the
directories searched go to standard error, once however often the link is
followed.

# FILES

; if xdg
_$XDG_CONFIG_HOME/cinq/config.toml_
	The configuration file, in the directory of the user's configuration,
	which is _~/.config_ when *XDG_CONFIG_HOME* is unset or is not an
	absolute path, as the XDG Base Directory Specification sets out.
; if darwin
_~/Library/Application Support/cinq/config.toml_
	The configuration file, in the directory of the user's application
	support files.
; if windows
_%APPDATA%\\cinq\\config.toml_
	The configuration file, in the directory of the user's roaming
	application data, such as _C:\\Users\\Alice\\AppData\\Roaming_.
; end

Making or deleting the file applies while *cinq* runs as well, so that
deleting it brings back the keys of vi and the other defaults.

Started with *--config-dir*=_directory_, *cinq* reads _directory/config.toml_
in its place, and follows it as it changes in the same way.

; if xdg
_$XDG_DATA_HOME/doc/cinq/tutorial.pdf_
	The tutorial, which the help page links to.
	*cinq* opens the first _doc/cinq/tutorial.pdf_ that it finds in the
	directory of the user's data, which is _~/.local/share_ when
	*XDG_DATA_HOME* is unset or is not an absolute path, and then in each
	directory that *XDG_DATA_DIRS* lists, separated by colons, which are
	_/usr/local/share_ and _/usr/share_ when it lists no absolute path, as
	the XDG Base Directory Specification sets out.
	The packages for GNU/Linux put it in _/usr/share_.
; if darwin
_/usr/local/share/doc/cinq/tutorial.pdf_
	The tutorial, which the help page links to, where the installer package
	puts it.
; if windows
_tutorial.pdf_
	The tutorial, which the help page links to, in the directory that holds
	_cinq.exe_.
; end

# SETTINGS

*audio_device*
	ID of the audio device that the outputs of a show play on where they name
	no device, as *cinq-show*(5) describes, in place of the default device of
	the system.
	*cinq --audio-devices* lists the ID and the name of each device, separated
	by a tab, one device to a line.
	The *Default device* list in the sidebar of the main window chooses a
	device as well, and *cinq* writes the choice here.
	The *Device* list of an output names that choice in its default option, as
	*Default device (System default)*, and while the highlight of its open menu
	rests on that option, the *Default device* list in the sidebar takes the
	color of the highlight, so that the output and the setting that decides its
	device show as one.
	It changes only the line of this setting, adds the line before the first
	line of the file that is neither blank nor a comment, so that no table
	takes it, or takes the line out for the system default, and leaves the
	rest as it is.
	A change moves those outputs as soon as the file is saved, and the sounds
	that play go on from where they are on the new device.
	Outputs that name a device of their own go on playing unbroken.
	A device can take a moment to start, during which nothing is heard and
	the cues that play wait for it, while the rest of the window goes on.
	While the device is not there, or cannot be opened, those outputs are
	silent, the warnings in the sidebar tell why, and *cinq* tries the device
	again every two seconds, as *cinq-show*(5) describes.
	The warning about the default device names its ID, as in *Default device
	(alsa:front:CARD=PCH,DEV=0)*, and the *Default device* list shows that ID in
	yellow after the words *Not found:* while the device is not found.
; if xdg
	On ALSA, *cinq* plays samples in 32-bit floating point, or in integers of
	32, 24, or 16 bits, as the _alsa:hw:_, _alsa:front:_, and _alsa:hdmi:_
	devices of most sound cards take them while no sound server holds the
	card, and the devices whose IDs begin with _alsa:plughw:_ or
	_alsa:sysdefault:_, _alsa:default_, and _alsa:pipewire_ convert them for
	the card.
	The _alsa:hw:_ device of a card that takes packed 24-bit samples alone,
	as some USB interfaces do, cannot play, and its warning says to choose the
	_alsa:plughw:_ device of the card instead, which converts them.

	PulseAudio spreads a stream that has fewer channels than its device over
	all of the channels of the device.
	So through PulseAudio, as through _alsa:default_, a device of many
	channels sounds on channels that no output names, unless an output names
	the last channel of the device.
	The ALSA plugin of PipeWire, as _alsa:pipewire_ and, where PipeWire
	serves it, _alsa:default_, takes the channels of a stream past the
	second for those of surround sound: on a device whose channels are not
	surround channels, as a desk's are not, only the first two sound, and on
	a surround device the third and fourth of four channels sound on its
	fifth and sixth.
	While a JACK server runs, as PipeWire's can, *cinq* also lists
	_jack:output_, whose channels each connect to the playback port of the
	server with their number, which keeps every channel where its output
	puts it, and so suits a device of more than two channels under
	PipeWire.
	Channels past the last playback port of the server connect to none, so
	that a patchbay of JACK can connect them elsewhere, and *cinq* shows no
	warning for them.
; if windows
	The Windows build that Nix makes also lists the ASIO drivers, by IDs that
	begin with _asio:_, beside the devices of WASAPI.
	The Dante Virtual Soundcard offers all of its channels as one ASIO
	driver, while in WDM mode it offers each pair of its first 16 channels as
	a device of its own.
	An ASIO driver that another program holds, as one in exclusive mode can,
	fails to open with a warning that its hardware is malfunctioning, which
	clears once that program lets it go.
; end
	It is the default device unless the file names another.

*keymap*
	Name of the keymap that the keys start from: *vi*, which *cinq* ships, as
	*KEYMAPS* describes it, or one of the file's own under *keymaps*.
	It is *vi* unless the file names another.
	Earlier versions of *cinq* also shipped a keymap named *cua*.
	Where the file names *cua*, here or in the *from* of a keymap of its own,
	and declares no keymap of that name, *cinq* reports the problem
	_cinq no longer ships the cua keymap, so vi stands in for it_
	at that line, and the keymap starts from the keys of *vi*.
	Any other name that no keymap bears is a problem too: *keymap* then keeps
	*vi*, and a *from* starts from no keys.

*keymaps*
	Table of keymaps of the file's own, each of them a table named after its
	keymap.
	The *from* of such a keymap names the keymap that it starts from, *vi* or
	another of the file's own, and without *from* it starts
	with no keys at all.
	Its other entries bind actions to keys as those of *keys* do.
	A keymap of the file's own cannot take the name of a shipped one.

*keys*
	Table that binds actions to keys in the keymap in use.
	Each entry is named after an action, as listed under *ACTIONS*, and holds
	a key sequence in a string, or several in an array of strings, any of
	which carries out the action.
	An action that the table names loses the keys that the keymap gives it,
	so that an empty array leaves it with no keys at all, and the actions
	that the table leaves out keep the keys of the keymap.
	An entry of which no key can be read is a problem, and its action keeps
	its keys.

*osc_listen*
	IP address and port, as _0.0.0.0:53000_, _127.0.0.1:53000_, or
	_[::]:53000_, on which *cinq* takes commands as Open Sound Control (OSC)
	over UDP, one packet to a datagram, so that a tablet, a console, or another
	program can fire cues from a distance, and a main and a backup machine can
	run together when the same commands go to both.
	A host name is not taken.
	The problem then says so.
	*cinq* listens to nothing unless the file sets it, and follows a change
	of it as soon as the file is saved.
	A packet is a message or a bundle of them.
	A message is carried out at once, and the time tag of a bundle is
	ignored.
	Of the messages of one packet the first 32 are carried out, and a warning
	in the sidebar and on standard error says that the rest were dropped.
	The addresses are those that QLab names, so that remote controls made for
	QLab work as they are:

	_/go_ is GO on the cue list in view, as the key does, and _/panic_ is
	Panic.
	_/pause_ and _/resume_ pause and resume every running cue.
	_/cue/{number}/start_ fires the cue with its chain, as a start cue does, and
	cues the stage window as GO does for a picture or video cue, or resumes it
	where it is paused and leaves the window as it is, and
	_/cue/{number}/stop_ stops it.
	_/cue/{number}/pause_ holds the cue, as a pause cue does, and
	_/cue/{number}/resume_ resumes it.
	_/playhead/{number}_ moves standby to the cue, in the list that holds it, as
	a GoTo cue does, and selects the cue, which shows its list.

	The number is that of the cue as the cue list shows it, matched exactly
	after the spaces around it are taken off.
	Where several cues have the same number, the first of them is meant, in
	the order of the cue lists and then of their rows.
	An address may begin with _/workspace/{id}_ once, whatever the ID.
	There are no wildcards and no replies.
	A message whose only argument is the number 0, whether an integer or a
	float of either width, or false does nothing, as the button of a remote
	sends 1 when it is pressed and 0 when it is released, and the release must
	not fire GO again.
	Any other arguments are ignored.
	The commands act in show mode, while a field has the keyboard, and over the
	help page, as GO and Panic do, and each leaves hint mode as a click does,
	apart from one that does nothing, as below, which leaves it only when the
	sidebar gains the line for it, and changes nothing else.
	A command that moves the selection or the cue list in view also takes the
	keyboard out of the field, so that what is typed next cannot edit another
	cue.
	None of them changes the show, and none can be undone.
	A packet that is not OSC, an address that is not listed, a number that no
	cue has, and a start of a cue that cannot play, as one with no file or no
	destination, do nothing and go to standard error, once for each sender and
	address, until a thousand different lines have gone out, after which one
	last line says that further warnings go unreported and nothing more is
	written.
	The sidebar lists the latest of them, with the address that it came from, as
	one entry among the warnings, as in _OSC from 192.168.1.5: no cue is numbered
	"99"_, until a command acts or the setting changes.
	A start of a cue that cannot play reads as _OSC from 192.168.1.5: cue 4
	cannot play: no destination as host:port_.
	A cue that only waits for its file to load is no problem and is not reported.
	When the address cannot be listened on, as when another program holds the
	port, the warnings in the sidebar say so, naming *osc_listen*, and *cinq*
	tries again every two seconds, so that the port comes into use, and the
	warning goes, once the other program lets it go.
	The first failure is tried again after half a second and is not told, as
	the setting before may still hold the port when the file moves *osc_listen*
	between two addresses of one port.
	Anyone who can reach the address can fire and stop cues, so give the
	address of the interface of the network that the show runs on, or
	_127.0.0.1_ for programs on the same computer, and keep the port from
	other networks.

*ready_ahead*
	Number of seconds, 0 or more and whole or fractional, before a sound starts
	that *cinq* makes it ready, so that GO opens no sound file itself.
	*cinq* makes ready the sounds that start within that time, of the runs
	that GO has fired and have yet to sound, of what the next GO of each list
	would fire, and of what the start cues among them would fire, counting
	each sound from its delay in a chain and its pre-wait, along with the
	pre-wait of the start cue that fires it, and from the sooner of the two
	where two start cues reach one sound.
	The sounds that start together are made ready together, and those that
	start at the instant of GO always are, even at 0.
	As time passes the sounds of a long chain follow, so that a chain of any
	length keeps ahead of its sounds.
	A sound that is not ready when it starts opens as it starts, and a slow
	disk can make it late.
	Each sound made ready holds its file open and a decoder thread that wakes
	about every millisecond, so a small system may do better with fewer
	seconds.
	It is 10 unless the file sets another number.

# KEYMAPS

*vi*
	Keys in the manner of vi: letters move through cues and lists and act on
	them, some commands take two keys, such as *d d*, and the arrows and
	Control with *n* and *p* move as well.

It keeps GO on the space bar and Panic on Escape, as in QLab.
A keymap of the file's own can start from it, as *EXAMPLES* shows, or from no
keys at all.
The table lists the keys that it binds to every action, in the order of the
help page.

|[ *Action*
:< *vi*
|  go
:  Space
|  panic
:  Escape
|  standby
:  Enter
|  cancel
:  Control+c, Control+[
|  pick
:  Control+y
|  close_menu
:  Control+e
|  next
:  j, g j, ArrowDown, Control+n, +
|  previous
:  k, g k, ArrowUp, Control+p, -
|  first
:  g g, Control+Home
|  last
:  G, Control+End
|  half_page_down
:  Control+d
|  half_page_up
:  Control+u
|  page_down
:  Control+f, PageDown
|  page_up
:  Control+b, PageUp
|  scroll_down
:  Control+e
|  scroll_up
:  Control+y
|  top_row
:  H
|  middle_row
:  M
|  bottom_row
:  L
|  row_to_middle
:  z z, z .
|  row_to_top
:  z t, z Enter
|  row_to_bottom
:  z b, z -
|  collapse
:  z c
|  expand
:  z o
|  toggle_group
:  z a
|  collapse_all
:  z M
|  expand_all
:  z R
|  next_list
:  l, ArrowRight
|  previous_list
:  h, ArrowLeft
|  next_tab
:  g t
|  previous_tab
:  g T
|  back
:  Control+o
|  forward
:  Control+i
|  search
:  /
|  search_backward
:  none
|  next_match
:  n
|  previous_match
:  N
|  edit
:  i, a, A
|  edit_start
:  I
|  change_name
:  c c, S
|  next_field
:  Tab
|  previous_field
:  Shift+Tab
|  hint
:  f
|  add_below
:  o
|  add_above
:  O
|  cut
:  d d, x, Delete
|  copy
:  y y, Y
|  paste_below
:  p
|  paste_above
:  P
|  delete
:  none
|  move_down
:  Alt+j, Alt+ArrowDown
|  move_up
:  Alt+k, Alt+ArrowUp
|  indent
:  > >
|  outdent
:  < <
|  undo
:  u
|  redo
:  Control+r
|  open
:  Control+l
|  save
:  Control+s
|  save_as
:  Control+S
|  fullscreen
:  F11
|  help
:  ?, F1
|  close_help
:  q

# KEYS

A key goes by its value in the W3C's UI Events KeyboardEvent key Values.

- A key that types a character is that character, as in *j*, *G*, or *?*.
  Letters tell the case apart, so that *G* is the key that Shift and *g* type
  together.
- Any other key goes by its name, as in *Enter*, *Escape*, *Tab*, *ArrowDown*,
  *PageUp*, or *F5*.
- The space bar is *Space*, since its value, the space it types, would read as
  the gap between two keys.

Control, Alt, and Shift can be held with a key, each joined to it by a *+*, as
in *Control+d*, *Alt+ArrowUp*, or *Control+Shift+Tab*.
A *+* with nothing after it is the plus key itself, as in *Control++*.
Shift with one of the letters *a* to *z* is the capital that it types, so that
*Shift+g* is *G* and *Control+Shift+n* is *Control+N*.
Caps Lock changes the case of no letter held with Control or Alt, so that
*Control+s* is the same with it on.
On Darwin, *Alt* with a letter is that letter, although Option makes another
character of it.
Any other key that types a character is written as the character it types
with Shift held, as in *?* rather than *Shift+/*, and Shift cannot be added to
it.
Shift can be held with the space bar and with any key that goes by its name,
as in *Shift+Space* or *Shift+Tab*.

A modifier key, such as Shift or Control, cannot be bound by itself.
Keys held with the logo key belong to the system, and *cinq* leaves them
alone, so a key cannot hold Meta or Super.

A key sequence is one or more keys separated by spaces, which are typed one
after another, as in *g g*.
The spaces belong to the notation of the file and of this page: the keys
are typed with no Space between them, since Space is GO, and the status bar,
the box of the keys that can come next, and the help page write a sequence of
keys that type characters joined, as Vim does, such as *gg*.

# TYPING

The keys of a sequence wait until the sequence is complete, and the status bar
shows the keys typed so far, while a box above it lists the keys that can come
next and what each of them does, naming outputs instead of cues while the
outputs show.  A count typed before or after an operator shows in what a
motion does, as *the selected cue and the next 6* for *2 d 3 j*.
The help page, which lists each action once, names cues and outputs together.
A key that completes or continues no sequence is dropped together with the
keys typed so far, as Vim beeps and drops them, so that a key mistyped halfway,
such as the *x* of *d x*, never starts a sequence of its own.
Anything done other than typing, such as a click in the main window, drops a
sequence typed halfway, and so does a window taking the keyboard, as when
another window or program gives it back, and a change to the file.
A click or a turn of the wheel in the stage window does not.
The key of *cancel* drops it as well, without any other effect.

Digits typed before a sequence count it, as in *3 j* with the vi keymap.
The count repeats *next*, *previous*, *next_list*, *previous_list*,
*move_down*, *move_up*, *scroll_down*, *scroll_up*, *back*, *forward*,
*next_match*, and *previous_match*.
It copies, cuts, deletes, indents, outdents, or adds as many cues for *copy*,
*cut*, *delete*, *indent*, *outdent*, *add_below*, and *add_above*, pastes as
many times for *paste_below* and *paste_above*, undoes or redoes as many
changes for *undo* and *redo*, and goes back as many tabs for *previous_tab*.
It gives the row, counted from one, that *first* and *last* select, and the
tab, counted from one, that *next_tab* shows.
It gives the row in view, counted from the top for *top_row* and from the bottom
for *bottom_row*, that they select.
It gives the row, counted from one, that *row_to_top*, *row_to_middle*, and
*row_to_bottom* select before they put it in place.
For *half_page_down* and *half_page_up*, it sets how many rows they move, as
in Vim, and later presses of either move that many rows, or lines on the help
page, until another count sets it again, the main window changes size, or
another show opens.
For *page_down* and *page_up*, it counts the pages.
Other actions leave the count unused.
A row is a row that the cue list shows, so that a collapsed group is one row,
as a closed fold is one line in Vim, for every key that moves through rows or
counts them, and for the mouse.
A count larger than what there is, such as *9 Control+o* with fewer places
behind, *9 g t* with fewer tabs, or *3 d d* on the last cue, makes *cinq* go as
far as it can, where Vim would refuse it.
Since digits count, no sequence can start with one.
A zero that starts a sequence counts nothing, as it is a motion in Vim: it is
dropped like any key that begins nothing, so that the status bar shows no
pending key.
A zero after another digit counts, as in *1 0 j*.
The same holds for a zero that would start the count after an operator, so that
*d 0* drops the operator.

The actions *copy*, *cut*, *delete*, *indent*, and *outdent* also act as
operators, as *y*, *d*, *>*, and *<* do in Vim.
An operator key is the key that one of their bindings doubles: *d* for *d d*,
*y* for *y y*, *>* for *> >*, and *<* for *< <* in the vi keymap, and *D* for a
binding *D D* that replaces *d d*.
A binding that does not double a key, such as *Control+x*, *x*, or *c v*, makes
no operator.
The operator key followed by any key bound to *next*, *previous*, *first*, or
*last* applies the action to the cues from the selection to where that motion
would move it, inclusive, as Vim does for whole lines: *d j* cuts the selected
cue and the next, *y k* copies it and the previous, *d G* cuts down to the last
cue, *d g g* up to the first, and *d 3 G* to row 3, in whichever direction the
row lies.
A count before the operator, before the motion, or both counts the motion, and
the two multiply, so that *2 d 3 j* cuts the selected cue and the next six.
The count and the motion count rows, so that a collapsed group is one row of
them and goes with every cue that it holds.
A motion that cannot move, such as *d k* on the first cue, does nothing, and
leaves the status bar as it was, so that the hint to paste stays, and a count
past the end stops at the last cue.
A binding of your own to the keys of an operator and a motion, as *d j* for
another action, acts in place of the operator, unless a count comes between the
keys, and the box of the keys that can come next leaves that key out as a
motion.
The keys of *top_row*, *middle_row*, and *bottom_row* are no motions, since
the rows in view are known only once the window has drawn the list.
The cues of a cut leave the selection on the cue that takes their place, a
copy leaves it on the first cue copied, and *> >* and *< <* with a motion leave
it on the first cue moved.
Cutting is one change for undo, and the clipboard holds the cues in order.
A cue that plays stops when it is cut or deleted, or when it is given another
file, and the status bar says so, as in *Cut cue 2, which stopped*.
Undo brings the cue back and does not start it again.
After a copy or a cut the status bar hints the key to paste until a key does
more than move the selection or switch lists, so that a paste into the other
list keeps its hint.
A paste into another show drops the sends to outputs that have no namesake
there and the targets outside the cues pasted, and the status bar says how
many, as in *Pasted 3 cues, dropped 2 sends*, with the key of undo.
It also counts the light cues that name instruments which the patch of that
show lacks, as in *Pasted 3 cues, 2 unpatched*.
Show mode locks *d*, *>*, and *<* with a motion as it locks *cut*, *delete*,
*indent*, and *outdent*, and leaves *y* with a motion alone, as it leaves
*copy*.
A key that is neither a count nor a motion drops the operator and itself, and
the keys of *go* and *panic* act at once as usual.

A key bound by itself to *go* or *panic* acts at once, even halfway through
another sequence, and when it is bound without modifiers, whatever modifiers
are held with it, unless the other of the two is bound to the key with those
very modifiers.
Holding either key down acts only once, so that holding the key of Panic does
not cut a panic short.
So does holding a key of *fullscreen*, *hint*, or *help*, which would otherwise
toggle the stage window, the labels, or the help page back and forth, while
every other key repeats as it is held.

# SEARCHING

The key of *search* opens a search line at the left of the status bar, where
the status stood, which reads a slash and then the text typed, and the key of
*search_backward*, which the vi keymap leaves unbound, opens it with a
question mark for a search up the list.
As the text changes, the selection moves to the first cue of the list on
screen, from the cue after the selected one down, one of whose columns holds the
text, in any case, and the list scrolls to it.
A search looks at the number, type, name, target, action, and *Then* columns,
with the text that the list shows in each, so that a search for *rumble* finds
the cue of *rumble.ogg*, as a search of Vim finds the text of a whole line.
The waits and the state are left out, since they are timings and what the
show has made of a cue, and so are the marks of the groups.
A search finds the cues that a collapsed group hides as well, and expands the
groups that hide the match, as Vim opens the folds of a match.
While the search line is open, only the groups that hide the current match
stay open, and those that the line opened close again when the match moves
elsewhere or the search is cancelled, as the incremental search of Vim closes
the folds that it opened.
Enter, and the keys of the next and previous match, keep the groups of the
match open.
A search up the list begins at the cue before the selected one.
The search wraps around the list and ends on the selected cue, as a search of
Vim does from the start of a line, so that the selected cue is the match only
when no other cue holds the text.
While the outputs show in place of the cue list, the search goes through the
name, the device, the channels, and the new cues of the outputs, as the table
shows them, in the same way, by the same rules, and never looks at the cues out
of sight.
The number of a row and the state are left out.
Enter accepts the search and closes the line, and remembers the text and the
direction for *next_match* and *previous_match*.
*next_match* finds the next cue that holds the text in the direction of the
search accepted last, and *previous_match* in the other direction, as *n* and
*N* do in Vim, wrapping around at either end.
When a search or a step wraps around, the status bar says that it continued at
the top or at the bottom, as Vim says that the search hit the bottom, and says
that no cue matches when a step finds none, and that there is no previous
search when no search was accepted yet, until the next action.
Enter on an empty line repeats the search accepted last in the direction of
the line.
Backspace on an empty line closes it, and so does Escape, which is also
*panic*, and so does the key of *cancel*, which stops no playback.
Each puts the selection back where it was when the line opened.
Where the keymap binds no key to *cancel*, as a keymap of the file's own may,
the status bar names Backspace while the line is empty, and Enter once text is
typed.
Any press of a mouse button closes the line and keeps the selection where it
is, and a press on a cue selects that cue.
Going *back* or *forward*, with the keys or the buttons of the mouse, moving
to another field with *next_field* or *previous_field*, and opening a show
close the line as well, and keep the selection where the search put it.
When no cue holds the text, the selection stays where it was when the line
opened and the line says so for a few seconds, and Enter then leaves the line
open.
Searches are jumps for *back*, as they are in Vim: accepting a search with
Enter, when it moved the selection, remembers the place where the search
began, and *next_match* and *previous_match* remember the place that they
leave.
The moves made while typing, and cancelling, remember nothing.
A count for *next_match* or *previous_match* goes round the matches at most
once, so that a count equal to the number of matches comes back to the same
cue.
The line takes the keys of Vim's command line that recall a search as well:
ArrowUp and ArrowDown recall the older and the newer of the last fifty
searches accepted, which start with the text typed, as the arrows of the
command line do, and leave the cursor at the end of the text, as Vim does
after a recall.
Control+w and Control+u delete before the caret, as in any text field.
The history is kept until *cinq* quits.
The line keeps the keys typed into it, like a text field, and the hints at
the right of the status bar give way to its text.
The stage window cannot open it by keys while it has the keyboard.

# SCROLLING

The cue list scrolls to keep the selected cue in view, with three rows around
it where there is room, and by the least that does it, as when *next* or *go*
moves the selection a row.
A move or jump that leaves the selected row far outside the view, such as
*last*, *next_match*, or *back*, puts it in the middle of the view instead,
much as Vim does after a jump that far.
Above the view, the row goes to the middle once it, with its three rows of
margin, lies half the view, rounded down to whole rows, less one row above the
top of the view, which is seven rows above it in a view of 23 rows.
Below the view, it goes to the middle once its bottom lies more than half the
view and a row past the bottom of the view, which is 13 rows past the last
row in view in a view of 23 rows.
Vim counts lines of equal height, while a long name that wraps makes a row of
the list taller, so the thresholds of *cinq* are in pixels of the view.

# WHERE KEYS ACT

Keys act on the cue list, whether the main window or the stage window has
the keyboard, apart from the keys typed while one of these has the keyboard:

A text field of the inspector, a drop-down list, or a check box
	A click gives a text field the keyboard, and the keys of *next_field* and
	*previous_field* move it from one text field, drop-down list, or check box
	to the next, which shows a border in the color of focus, twice as thick as at
	rest.
	A text field that these keys reach has its text selected, so that typing
	replaces it, whereas a click puts the caret where it fell.
	A field that cannot be edited, as every field in show mode, or a field
	that the cue ignores, as the post-wait of an auto-follow cue, is a plain
	box that takes no keyboard, and these keys pass it by.  The box keeps the
	height of the field, and shows in dim text what the field shows while it
	holds nothing, as *0* for a wait.
	From the cue list, or the outputs, *next_field* goes first to the first field
	of the inspector, so that the settings of what is selected are one key away,
	and then on through the rest of the inspector, the toolbar, and the sidebar,
	which *previous_field* does the other way.
	Past the last field, and before the first, no field has the keyboard, which
	gives the keys back to the cue list, or the outputs, so that a check box or a
	closed drop-down list never keeps them, even in a keymap that binds no
	*cancel*.
	One more press goes round again.
	It keeps the keys typed, apart from the keys bound by themselves to *go*,
	*panic*, *cancel*, *next_field*, *previous_field*, *back*, *forward*, *open*,
	*save*, and *save_as*, which act unless a text field takes them for typing.
	A text field also takes Control+w, which deletes the word before the caret,
	and Control+u, which deletes everything before it, as Vim does
	in Insert mode, unless a key is bound to an action that acts in fields.
	The edit is part of the typing around it, so that *undo* takes it back with
	the rest.
	A drop-down list that is closed lets the keys of *undo* and *redo* act as
	well, since it has no text to undo, so that a change made from the list
	can be taken back with the keyboard still there.
	It lets the keys of *hint*, *add_below*, and *add_above* act too, and each
	of them gives the keyboard back to the cue list, so that the labels of hint
	mode take letters and the cue that was added has the keys.
	A check box and a button do the same.
	The check boxes of the inspector, *Loop until stopped or devamped* and
	*Stop target when done*, are stops of these keys as well, and show the same
	border around their label.
	Enter flips the one that has the keyboard, and then neither leaves it nor
	puts a cue on standby, while Space and Escape keep GO and Panic and never flip
	it.
	Show mode locks those check boxes, which take no keyboard then.
	The *Show mode* check box of the toolbar, above the cue list and above the
	outputs, is a stop as well, which comes after the fields of the inspector,
	before the list of the kind of cue to add, and which Enter flips in the same
	way, except that the keyboard then goes back to the cue list, since the
	operator locks the show to run it.
	Show mode never locks it, since it is the way out, so that the keyboard
	alone locks the show before a performance and unlocks it after.
	It has no key of its own.
	From the cue list, *previous_field* reaches it in three presses, after the
	list of the sidebar and the list of the kind of cue to add, and reaches
	that list in two.
	The buttons *Add instrument* and *Remove* of the *Light Patch* are stops
	as well, in the order of the window, which Enter presses, with the same
	border, and show mode locks them, so that these keys pass them by then.
	Enter on *Remove* gives the keyboard back to the cue list, since the row
	below slides into the place of the one removed and would take the keyboard
	with it, so that removing instruments one after another takes a Tab each
	time.
	One of the sends of a cue, which scroll on the *Levels* tab, and a list of the
	sidebar, which scrolls in a low window, scroll into view as these keys reach
	it, with the title of the list above it.
	So does a field or a *Remove* button in a row of the *Light Patch* tab, or of
	the *Levels* tab of a light cue, which scroll as well.
	Enter in the empty File field of a cue opens the dialog that chooses the
	file, as the *Choose* button does, and the status bar hints it.
	Anywhere else Enter leaves a text field, opens the menu of a drop-down list,
	which takes the keyboard from there, presses a button, and flips a check box.
	A level typed into a field, the level of a cue, a send, the level of new
	cues of an output, or that of an instrument of a light cue, applies once
	the field is left, by Enter, Tab, a click, GO, or anything else the
	operator does, so that a cue that plays does not follow each key, as the
	*2* of *-25* would make it.
	A level field takes *-inf* as -60, silence, and shows -60 as *-inf*, so
	that silence reads as it is typed in every field of a level.
	A send field or the field of new cues takes *off* as it takes an empty
	field.
	GO then fires the cue at the level typed.
	A path typed into the File field of a cue applies the same way, as one
	change, so that a cue that plays does not stop at the first key and no
	half-typed path reads as a file that does not exist.
	A file chosen with the dialog applies at once.
	The slider next to the level of a cue applies as it moves.
	The address of an instrument applies the same way, so that the
	instrument does not move through the prefixes of the address on its way
	to the rig, and text that is no address leaves the old address as it was.
	Names apply as they are typed.

The help page
	The page fills the main window, while the stage window keeps showing the
	stage.  Closing it gives the main window back as it was, with the cue
	list scrolled where it was and the keyboard in the same field.  The keys
	of *next*, *previous*, *scroll_down*, and *scroll_up* scroll the page, a
	line for each one and as many lines as a count, those of *first* and *last*
	go to its top and its bottom, and those of *half_page_down*,
	*half_page_up*, *page_down*, and *page_up* scroll it by half or all of
	what the window shows, with a count of lines for the half pages, which
	the cue list shares, and of pages for the whole pages.  Of the rest, only
	the keys of *go*, *panic*, *help*, and *close_help* act, whichever
	window has the keyboard, so that no field takes a key meanwhile, and that of
	*standby* opens the tutorial, as its link does, and that of *hint* labels
	its links and its button while the main window has the keyboard, and does
	nothing while the stage window has it.  The keys of *go*
	and *panic* close the page as well, so that the show is in view.  The key
	of *help* opens the page from the main window only, since the stage window
	may cover that window when it is fullscreen, and does nothing while the
	stage window has the keyboard, where the key of *close_help* closes the
	page.

The menu of a drop-down list
	The keys bound by themselves to *next* and *previous* move the highlight
	one option at a time, whatever the count, and the highlight stays put past
	either end.
	Those bound to *page_down* and *page_up* move it by the options in view,
	and those bound to *first* and *last* to the first and last option, which
	includes *g g* and *G* in the vi keymap.
	The sequences *g j* and *g k* move by one option as well.
	PageDown, PageUp, Home, and End move it too, as in the popup menu of Vim,
	whether or not the keymap binds them.
	Enter and the key of *pick* pick the highlighted option, and the key of
	*close_menu* and *cancel* close the menu without picking one, and either
	leaves the keyboard on the list, so that *next_field* goes on to the field
	after it, except that a pick in the *New cue* list of the toolbar gives the
	keyboard back to the cue list, so that the key of *add_below* acts at once.
	The keys of *go* and *panic* act as they do everywhere and close the menu
	as well.
	The key of *help* opens the help page and leaves the menu open, hidden
	beneath the page, so that it is open on the same highlighted option once
	the page closes.
	The menu keeps every other key until it closes.
	While it is open, the status bar hints the keys of *pick* and *close_menu*,
	with Enter for *pick* when no key is bound to it.

While the stage window has the keyboard, as the window manager may leave it
once the stage window opens, and as each GO that makes it fullscreen gives
it, *edit*, *edit_start*, *change_name*, *next_field*, *previous_field*, and
*hint* do nothing, since the keys typed there would never reach a field of
the main window, and neither do *search*, *search_backward*, and *help*, which
open the search line and the page in that window.
The keys that change the show or fill the clipboard do nothing there either:
*add_below*, *add_above*, *cut*, *copy*, *paste_below*, *paste_above*,
*delete*, *move_down*, *move_up*, *indent*, *outdent*, *undo*, and *redo*, and
the operators that begin with a key of *cut*, *copy*, *delete*, *indent*, or
*outdent*, since the cue list is out of sight from that window and a held key
could change cues unseen.
The keys of *go*, *panic*, and *standby*, and those that move the selection,
act as before.
A click on the main window gives it the keyboard.

Show mode locks editing: *edit*, *edit_start*, *change_name*, *add_below*,
*add_above*, *move_down*, *move_up*, *indent*, *outdent*, *delete*, *cut*,
*paste_below*, and *paste_above* do nothing in it, and *undo* and *redo* only
move standby back and forth.
Copying changes no show, so that *copy* acts in show mode, as Vim yanks in a
read-only buffer.
A key that show mode locks, other than *undo* and *redo*, has the status bar
say *Leave show mode to edit*, until the next key that acts.
The stage window says so in the same way for the keys that it leaves to the
main window, and for those of *search*, *search_backward*, *help*, and *hint*,
and adds how to get the keyboard there: *Click the main window to edit* or
*Click the main window for search, help, or hint mode*.

The outputs of the show, which *Outputs* below the cue lists in the sidebar
shows in place of the cue list, take the actions that move through rows as
the cue list takes them, by the same code: *next*, *previous*, *first*, and
*last* select an output, which the inspector then edits, and so do the
actions of half and whole pages, of scrolling by a row, of the rows in view,
and of placing the selected row in view, which only matter once there are more
outputs than rows in the view.
*search*, *search_backward*, *next_match*, and *previous_match* search the
columns of the outputs, in any case, with the rules of *SEARCHING*, and the
line tells that no output matches.
*first* and *last*, and the searches, are jumps there, as they are for cues.
The inspector of an output has a line of help beside each field, which says
what the field takes and what it does, and which turns red and says why where
the value typed is not applied, as for a name that another output has,
channels that are neither one nor two, or a level out of range.
Where the keyboard leaves a field that holds such a value, the status bar
tells the reason, since the field shows the value that holds again.
The actions that edit cues edit the outputs in the same way while they show.
*edit*, *edit_start*, and *change_name* enter the Name field of the output
with the caret at its end, at its start, or with all of it selected.
*next_field* and *previous_field* move among the fields and lists of the
output and select the text of a field that they reach, as they do for a cue.
*add_below* and *add_above* add an output on the default device and its first
two channels, as many as the count says, at most 64, and select the last of
them.
*move_down* and *move_up* move the output.
*cut*, *copy*, *paste_below*, and *paste_above*, and the keys of an operator
with a motion, such as *d j* and *y G*, take the selected output and those
below it, as many as the count says or the motion reaches, into a register of
outputs, which is apart from the cues that were copied, or put the register
back, as many times as the count says, at most 64 outputs, and select the
first output pasted.
*delete* deletes them without copying.
A show keeps at least one output, so that a cut or a delete that reaches all
of them does nothing and the status bar tells so.
A cut output pasted back into the show that it came from takes back its
identity, with the level of its new cues, so that the cues that sent to it send
to it again, as a line that was cut and pasted back in Vim is the line that
was.
A copy, and anything pasted into another show, has an identity of its own, the
name of the original, numbered from 2 on where another output has that name,
no cue that sends to it, and new cues that do not send to it.
A number that ends the name after a space gives way to the new one, so that a
copy of *House R 2* is *House R 3*, not *House R 2 2*.
Cues that name no sends play on the first output, so that cutting, adding,
moving, or pasting at the first place changes where they play, which the
status bar tells first, as in *Cues without sends play on Sub*, with the
names of outputs cut to 10 columns so that it fits in the bar of a small
window, where the key of *paste* gives way to it.
A cue that sends only to outputs that a cut or a delete took away plays on no
output, so that the status bar counts such cues with the change, its *State*
reads *NO OUTPUT* in yellow, and the sidebar lists it among the warnings, until
*undo* brings the outputs back or the cue sends elsewhere.
A cue that sends only to outputs whose devices failed plays on in silence, so
that its *State* reads *OUTPUT FAILED* in yellow, on a line of its own under the
state of a cue that runs, as under *paused* or the time left, and alone for a
cue that does not, until the device answers.
The entry of such a cue that runs, in the Active list of the sidebar, reads
*silent: output failed* in yellow in place of its state.
An output wholly past the channels of its device counts as one that failed.
A pair that overlaps the last channel half sounds, so that it does not count,
and neither does a video cue whose file holds no sound, whose picture still
shows.
A cue whose sends were all cleared plays nowhere by choice, and gets no
warning, but the status bar says *Changed cue 6 to play nowhere* where the last
send was cleared, so that the edit tells what it did.
*undo* and *redo* select the output that the change touched, or the neighbor
of one that it took away, except that undoing an addition or a paste selects
the output that was selected when it was made, as the cursor of Vim returns to
where a change was made.
A change to a cue or to the cue lists leaves the outputs for the cue list in
the same way as a jump, so that *back* returns to the outputs, and the change
is undone or made again there, as are the changes that follow in the count.
Undoing or redoing a change to the outputs while the cue list shows shows the
outputs, and one to the light patch shows the *Light Patch* tab, so that the
place that changed is in sight.
Show mode locks all of these but *copy*, and *undo* and *redo* do nothing
there.
Every other action that acts on cues, such as *standby*, does nothing, since
the cue list is out of sight.
While the outputs show, the status bar names the keys of *add_below*, *edit*,
and *back*, and the key of *change_name* in place of that of *edit* where the
keymap binds none to it.  The bar gives way,
from the first to the last, to the key of *hint*, *change_name*, *forward*,
*add_below*, *edit*, GO, Panic, *fullscreen*, *save*, *back*, and the help page,
so that the key that leaves the outputs outlasts all but the help page.
There the status bar tells a change to the outputs and none to the cues, so
that the notice of a cue never takes the room of these keys.
In every view, the name of the show in the status bar gives way before any of
these keys do, cut short at its end with an ellipsis, down to 16 columns.
A notice of a change that does not fit gives up its middle words first, so that
it keeps the words that tell which cue it names, as in *Undid… cue 6.5*.

# ACTIONS

The actions go by their names in the *keys* table, in the order of the help
page.

*go*
	GO: fire the standby cue, together with every cue that auto-continue or
	auto-follow chains to it.
	While no cue of the chain can play, the *GO* button looks disabled and the
	line under the name of the standby cue says why, as in *GO fires nothing: cue
	1 (no file set)*.  The key still moves standby past the cues that cannot
	play.
	A cue that sends only to outputs whose devices failed still fires, in
	silence, so that the line tells *silent: output failed* for it, as it tells
	*silent: no output* for a cue that sends to no output the show has, and
	agrees with the *State* column of the cue list in which cues it tells.
	For a start, stop, pause, devamp, or GoTo cue the line tells what it does, as
	in *13 (GoTo 2)* or *25 (start E3 in Effects)*, with the list named only
	where the target lies in another.
	While a GoTo cue that has fired waits to put another cue on standby of the
	list, the line leads with *Standby moves to 5 in 2.9s*, until the GoTo cue
	acts or stops.
	While any cue is paused, the line leads with how many, as *2 paused*,
	before *Standby moves to* when both apply.

*panic*
	Panic: fade every playing cue out over one second, and stop the rest.
	A second panic while the first is still fading stops everything at once.
	A cue that Panic or a fade cue is fading reads *fading* in green in the *State*
	column, in place of its time left or *looping*, so that the list tells it from
	a cue at full level.

*standby*
	Put the selected cue on standby, so that the next GO fires it.
	While the selected cue is not the standby cue and no field or list has the
	keyboard, the status bar hints this key, and gives way to the hint right
	after those of *next_tab* and *hint*.

*cancel*
	Drop the keys typed so far, close the search line, with the selection going
	back to where the search began, as Escape does there, close an open menu
	without picking, and take the keyboard out of a text field or drop-down list,
	as Control+c and Control+[ do in Vim.
	It never touches playback, which is how it differs from Escape, so that it is
	the key to reach for when no show should stop.
	It acts in text fields, where a text field that has a selection still copies
	it first.
	While keys wait, the search line is open, or hint mode is on, the status bar
	hints its key.
	While a text field or a drop-down list has the keyboard, it hints the key
	of *next_field* and this key as the one that leaves, so that the way out is
	told where Escape would panic.
	While a check box has it, the bar hints Enter as the key that flips it,
	while a button has it, as the key that presses it, and
	while a closed drop-down list has it, as the key that opens it, unless Enter
	is bound to *go*, *panic*, or another action that acts in fields, so that a
	newcomer does not press Space, which is GO.
	There the hints give way, from the first to the last, to *next_field*,
	*hint*, the keys of other places, what Enter does, GO, Panic, *save*, and
	this key, so that the key that leaves outlasts them all, and GO and Panic
	outlast what only teaches.
	The key that leaves also outlasts the words of the latest change, which
	are cut at a word instead, and so does what Enter does on a check box, a
	button, or a closed drop-down list.
	Where no key is bound to it, a text field has Enter hinted as the key that
	leaves, unless Enter is bound to *go*, *panic*, or another action that acts
	in fields, which it then does instead.
	The status bar hints only keys that act where the keyboard is, so that
	while a text field or a drop-down list has it, it hints no key of
	*paste_below*, *edit*, or *change_name*, and while a text field has it,
	none of *undo*, *redo*, *hint*, or *add_below* either, which act in a closed
	drop-down list.
	While the stage window has the keyboard, it hints none of the keys that it
	leaves to the main window, as listed under the stage window above, and
	neither does the notice of a change name *undo*.

*pick*
	Pick the highlighted option of an open menu, as Enter does and as Control+y
	does in Vim's popup menu.
	It does nothing while no menu is open.
	While a menu is open, the status bar hints the keys of *pick* and
	*close_menu* and tells no notice, so that no notice takes their room.

*close_menu*
	Close an open menu without picking an option, as Control+e does in Vim's
	popup menu.
	Unlike Escape, which closes the menu and also acts as Panic, it does not
	panic.
	It does nothing while no menu is open.

*next*
	Select the next cue, move the highlight of an open menu down, or scroll
	the help page down.
	The vi keymap binds *g j* as well as *j*, since a user of Vim types *gj* to
	move by a displayed line.
	It binds *+* as well, which is the key of Vim for the next line, and *-*
	for the previous line.

*previous*
	Select the previous cue, move the highlight of an open menu up, or
	scroll the help page up.
	The vi keymap binds *g k* as well as *k*, as it binds *g j*.

*first*
	Select the first cue, or the cue on the row counted, or go to the top of
	the help page, or move the highlight of an open menu to its first option.

*last*
	Select the last cue, or the cue on the row counted, or go to the bottom
	of the help page, or move the highlight of an open menu to its last option.

*half_page_down*
	Scroll the cue list half a page down, that is, half as many rows as fit
	in it, or the rows that a count sets, and select the cue that many rows
	further down, so that the selection keeps its row on the screen, as the
	cursor does in Vim.
	At the end of the list the view stops and the selection goes on to the
	last cue.
	On the help page, scroll down by half a page, or by the lines that a
	count sets.

*half_page_up*
	Scroll the cue list half a page up, and select the cue as many rows
	further up.
	At the top of the list the view stops and the selection goes on to the
	first cue.
	On the help page, scroll up by half a page.

*page_down*
	Scroll the cue list a whole page down, less two rows that stay in view,
	as Vim does, or as many pages as the count, and select the first cue in
	view, or keep the selection when it lies further down.
	At the end of the list the view stops and the selection goes on to the
	last cue.
	On the help page, scroll down by a page less two lines.
	In an open menu, move the highlight down by the options in view, or by ten
	before the menu has been drawn.

*page_up*
	Scroll the cue list a whole page up, less two rows, and select the last
	cue in view, or keep the selection when it lies further up.
	At the top of the list the view stops and the selection goes on to the
	first cue.
	On the help page, scroll up by a page less two lines.
	In an open menu, move the highlight up by the options in view.

*scroll_down*
	Scroll the cue list a row down, or as many rows as the count, as
	*Control+e* does in Vim.
	The selection stays on its cue while that is in view, and moves to the
	nearest row in view when the scroll would take it out.
	The view stops at the end of the list.
	On the help page, scroll down a line, or as many lines as the count.
	While a menu is open, *Control+e* closes the menu instead, as in Vim's
	popup menu.

*scroll_up*
	Scroll the cue list a row up, or as many rows as the count, as
	*Control+y* does in Vim, and keep the selection in view as *scroll_down*
	does.
	On the help page, scroll up a line.
	While a menu is open, *Control+y* picks the highlighted option instead.

*top_row*
	Select the first cue row in view, or for a count the row of that number,
	counted from one at the first row in view, stopping at the last row in view,
	as *H* does in Vim.
	The view stays where it is.
	It is a jump for *back*.

*middle_row*
	Select the cue row in the middle of the view, or of the rows in view when
	the list is shorter, taking the upper of two middle rows, as *M* does in
	Vim.
	It is a jump for *back*.

*bottom_row*
	Select the last cue row in view, or for a count the row of that number,
	counted from one at the last row in view going up, stopping at the first
	row in view, as *L* does in Vim.
	It is a jump for *back*.

*row_to_middle*
	Scroll the cue list to put the selected row in the middle of the view, as
	*z z* does in Vim, and so does *z .* in the vi keymap.
	The selection stays on its cue, and the view stops at either end of the
	list, so that a row near an end stays short of the middle.
	A count selects the row, counted from one, first, as in Vim, and the last
	cue for a row past the end.
	It is no jump.

*row_to_top*
	Scroll the cue list to put the selected row at the top of the view, as
	*z t* does in Vim, and so does *z Enter* in the vi keymap, stopping at the
	end of the list.
	A count selects its row first, as for *row_to_middle*.

*row_to_bottom*
	Scroll the cue list to put the selected row at the bottom of the view, as
	*z b* does in Vim, and so does *z -* in the vi keymap, stopping at the top
	of the list.
	A count selects its row first, as for *row_to_middle*.

*collapse*
	Collapse the selected group, so that the cue list shows it as one row and
	leaves out the cues that it holds, as *z c* closes a fold in Vim.
	On a collapsed group or a cue that is no group, it collapses the group that
	holds the cue, and the selection moves to that group.
	A group has a mark before its name, *-* while it is expanded and *+* while it
	is collapsed, and the *Target* column of a collapsed group tells how many cues
	it hides.
	Where the list is too narrow for that column, the name of the group ends in
	the count, as in *(10 cues)*.
	A cue list that holds no group leaves no room for the mark.
	A click on the mark collapses the group, or expands it where it is
	collapsed.
	A collapsed group fires, stops, pauses, and runs its cues as an expanded one
	does, so the keys that act on cues still do.
	The *State* column of a collapsed group reads *NEXT* while standby lies among
	the cues that it hides, and the problem of the first of them that has one,
	such as *NO FILE*, as that cue does, unless the group runs, so that no broken
	cue lies out of sight.
	Unless the group runs, its *State* column also tells how many of the cues
	that it hides run, as *3 running*, and how many of them are paused, as
	*1 paused* in yellow above it, so that no cue that plays lies out of sight.
	Its *State* column also tells the yellow warnings of the cues that it hides,
	as *OUTPUT FAILED* and *NO LEVELS*, each once.
	Collapsing is a way to look at the list, as a fold is in Vim: undo leaves it
	alone, show mode allows it, and the show file keeps it, as *collapsed* in
	*cinq-show*(5) tells.

*expand*
	Expand the selected group, if it is collapsed, as *z o* opens a fold in Vim.
	Whatever selects a cue that a collapsed group hides expands the groups that
	hide it, as Vim opens folds for a search or a jump, so that the selection is
	never out of sight.
	That includes a search match, *back* and *forward*, undo and redo, and GO
	moving the selection with standby.

*toggle_group*
	Collapse the selected group, or expand it where it is collapsed, as *z a*
	does in Vim.

*collapse_all*
	Collapse every group of the cue list, as *z M* does in Vim.
	The selection moves to the outermost group that holds it.

*expand_all*
	Expand every group of the cue list, as *z R* does in Vim.

*next_list*
	Show the next cue list, or the outputs of the show after the last one.
	While no field has the keyboard, the status bar hints its key, which gives
	way before any other.
	While standby lies past the end of the cue list on show and a later list
	exists, where GO fires nothing, the hint gives way instead only after the
	hints of GO, Panic, *fullscreen*, *save*, the help page, and *change_name*,
	so that the key that goes on to the next list shows on a short bar.

*previous_list*
	Show the previous cue list, or the last one from the outputs of the show.
	Where that is the cue list that the outputs took the place of, it comes back
	with the cue that was selected, as does choosing it in the sidebar, and any
	other cue list shows with its standby cue selected, or with the collapsed
	group that hides it, which stays collapsed.

*next_tab*
	Show the next tab of the inspector, or the tab whose number the count gives,
	counted from one, or the last tab when there are fewer.
	While no field has the keyboard, the status bar hints its key, which gives
	way right after the key of *next_list*, before any other.
	Where standby lies past the end of the cue list on show and a later list
	exists, only the key of *hint* gives way before it.

*previous_tab*
	Show the previous tab of the inspector, or go back as many tabs as the count,
	wrapping around at the first.

*back*
	Go back to the place before the last jump, as the jump list of Vim does
	with its *Control+o*.
	A place is a cue list with its selected cue, or the outputs with their
	selected output.
	Switching cue lists, *first* and *last* with or without a count, and
	selecting a cue in another cue list are jumps, and so are the searches
	described under *SEARCHING*.
	So is showing the outputs, with the sidebar, the *Edit outputs* button, a
	warning that names an output, or *next_list* past the last cue list, and
	so is leaving them for a cue list, so that *back* from the outputs returns
	to the cue list that showed, with the cue and the inspector tab that it had.
	Where no jump recorded a place, as after opening a show with the outputs
	shown, *back* from the outputs shows the cue list that was showing, with
	its selected cue, and *forward* returns to the outputs.
	Moving the selection one row at a time, clicking a cue of the cue list
	shown, GO moving the standby cue, adding, deleting, and moving cues,
	changing the tab, and the help page are not, nor are undo and redo, unless
	one shows the outputs.
	A count goes that many places back, or to the oldest.
	Going back remembers the place left, so that *forward* returns to it.
	A jump after going back moves the places ahead behind the place left, in
	the order that the jump list of Vim keeps them, so that going back again
	still visits them.
	When a text field has the keyboard, going back acts there as well, and
	moves the keyboard to the cue list, so that what is typed next cannot
	rename the cue that it went to.
	A cue that was deleted leaves no place to go to.
	The history holds at most 100 places, each once, and ends when *cinq*
	opens another show or quits.
	The status bar names the key and where it goes, as in *Control+o: back to
	Main*, *back to cue 3*, or *back to the outputs*, with the name of a cue
	list or cue cut short where it is long, while there is a place to go back to,
	and always while the outputs show.
	The back button of the mouse does the same from the main window, unless the
	help page is open.

*forward*
	Go forward again to the place that *back* left, as many places as the
	count.
	In Vim, Tab is the same key as Control+i, while in *cinq* Tab moves between
	fields and Control+i goes forward.
	In a text field it acts as well, and moves the keyboard to the cue list.
	The status bar names the key while there is a place ahead.
	The forward button of the mouse does the same from the main window, unless
	the help page is open.

*search*
	Open the search line at the left of the status bar, as described under
	*SEARCHING*.

*search_backward*
	Open the search line for a search up the list, which begins at the cue
	before the selected one, as described under *SEARCHING*.  The vi keymap
	leaves it unbound, since *?* shows the help page, so a keymap in *config.toml*
	binds it to a key.

*next_match*
	Select the next cue that matches the search accepted last, in the
	direction of that search, wrapping around at the end of the list, or as
	many matches further as the count, going around the matches at most once.
	It is a jump for *back*.
	It does nothing before a search.

*previous_match*
	Select the next cue that matches the search accepted last in the opposite
	direction, wrapping around at the end of the list in that direction, or as
	many matches back as the count, going around the matches at most once.
	It is a jump for *back*.

*edit*
	Edit the name of the selected cue: show the *Basics* tab and put the caret
	at the end of the name, as Vim's *a* and *A* do at the end of a line.

*edit_start*
	Edit the name of the selected cue with the caret at the start of the name,
	as Vim's *I* does.

*change_name*
	Edit the name of the selected cue with all of it selected, so that typing
	replaces it, as Vim's *c c* and *S* change a whole line.
	Enter leaves the field with the name unchanged unless a key was typed.
	Unlike in Vim, *c* is no operator, so only *c c* begins a change, and a *c*
	before any other key is dropped with that key, as *TYPING* describes.

*next_field*
	Move the keyboard to the next field: a text field, drop-down list,
	check box, or button of the inspector, or the list of the kind of cue that
	*add_below* and *add_above* add.
	After the last field it gives the keyboard back to the cue list.

*previous_field*
	Move the keyboard to the previous field, or, before the first, back to the
	cue list.

*hint*
	Label every place in the main window that a click acts on, to act on one by
	typing its label, as Vimium and Tridactyl do in a browser.
	The places are the buttons that can be pressed, the check boxes, the tabs of
	the inspector and the lists of the sidebar, the drop-down lists, the text
	fields, the links of the help page, and the mark of a group in the cue list,
	as far as they are in view, but not the buttons of GO and Panic, whose keys
	act from everywhere, in hint mode too.
	The rows of the cue list and of the outputs have keys of their own, and the
	stage window has no place to label.
	Each label is a few letters of *sadfjklewcmpgh*, as many for every place as
	the number of places needs, given out in the order that the places lie in the
	window.
	A label lies left of its place, with its right edge at the left edge of the
	place and its top at the top of the place, so that it hides neither a value
	nor the circle or box that shows a choice.
	Where that would leave the window, it lies right of the place instead, with its
	left edge at the right edge of the place, so that what lies at the left edge of
	the window stays in view, unless another label lies there, when it lies at the
	left edge of the window, so that no label hides another.
	A label typed whole acts as the click would: a button is pressed, a check box
	is flipped, a tab or a list is shown, a drop-down list or a text field takes
	the keyboard, so that Enter opens the menu of the list, and a link opens.
	The letters typed so far hide the labels that they do not begin, and
	Backspace takes the last one back, or leaves hint mode where none is typed.
	A key that types a character but continues no label is ignored, and the mode
	stays on, so that a slip never runs the keys after it as commands.
	A key that types no character leaves hint mode and does nothing else, the key
	of *cancel* among them.
	The keys of *go* and *panic* act as everywhere and leave it too, and so does
	any press of a mouse button or turn of the wheel in the main window, any
	change of the size of the window, any start or end of a cue in the Active list of the sidebar and
	any warning there that comes or goes, which move the places below them, any
	OSC command that acts, the opening of another show, and the stage window
	taking the keyboard.
	The key of this action is a letter in the vi keymap, so that it types a letter
	of a label while the mode is on.
	The status bar hints this key where it acts, and the hint gives way right
	after those of *next_list* and *next_tab*, or first where standby lies past
	the end of the cue list on show and a later list exists, or while the
	outputs show.
	While the mode is on, it shows only what acts there: a request for a label,
	or the letters typed so far, the key of *cancel* as the key that leaves, or
	Backspace where no key is bound to *cancel*, and the keys of *go* and
	*panic*.
	It does nothing while a text field has the keyboard, which takes the key for
	typing, or while the stage window has it.
	A closed drop-down list, a check box, or a button that has the keyboard gives
	it up when the mode begins.

*add_below*
	Add a cue of the kind chosen in the toolbar below the selection, or at the
	end of the list without one.
	A count adds that many cues in a row, at most a thousand, as one change
	that undo takes back, and selects the last of them, as the cursor of Vim
	goes to the last line that *3o* inserts.
	The inspector shows its *Basics* tab, so that the new cue has its name and
	its file in reach.
	Below an expanded group, or below a cue that a group holds, the cue joins the
	group, the notice says *in a group*, and the key of *outdent* takes the cue
	out.
	Below a collapsed group, the cue lands below the cues that the group holds,
	outside it.
	The key acts while a closed drop-down list, a check box, or a button has
	the keyboard, and gives the keyboard back to the cue list.
	The status bar then hints the key of *change_name*, which renames the cue,
	after the key of *undo*, and gives way to it only after the keys of GO,
	Panic, *save*, and the help page, when the bar is short.

*add_above*
	Add a cue of the kind chosen in the toolbar above the selection, or at the
	end of the list without one, as many as the count, as *add_below* does.

*cut*
	Copy the selected cue, or as many cues from it down as the count, to the
	clipboard and delete them, so that *cut* and then *paste_below* moves a
	cue.
	A group goes with the cues that it holds.
	The vi keymap binds *x* and *Delete* to it as well as *d d*, as Vim cuts
	with all three, and *d* followed by a motion cuts the cues the motion
	reaches, as *TYPING* describes.
	The status bar tells how many cues went to the clipboard and names the keys
	of *paste_below* and *undo*, and goes on telling the change as a cut after
	later keys and in the notices of *undo* and *redo*.

*copy*
	Copy the selected cue, or as many cues from it down as the count, to the
	clipboard of *cinq*, in place of what it held.
	A group goes with the cues that it holds.
	The clipboard is *cinq*'s own and not that of the desktop.
	It is no part of the show, and undo does not touch it.
	It acts in show mode, since it changes no show.
	The status bar tells how many cues went to the clipboard and names the key
	of *paste_below*.
	The vi keymap binds *Y* to it as well as *y y*, as Vim yanks a line with
	both, and *y* followed by a motion copies the cues the motion reaches, as
	*TYPING* describes.

*paste_below*
	Paste the cues of the clipboard below the selection, or at the end of the
	list without one, as many times in a row as the count, and select the
	first cue pasted.
	Pasting adds new cues with the settings of the copied ones, as one change
	that undo takes back, and changes nothing while the clipboard is empty,
	which the status bar tells.
	A pasted cue keeps its number unless another cue holds it, when it takes
	the next free number.
	A fade, start, stop, pause, devamp, or GoTo cue pasted together with its
	target, whether copied or cut with it, targets the new copy of that cue,
	as in QLab, in each round of a count.
	Otherwise it keeps its target when it is pasted into the show it was
	copied from and the target is still there, and has none in another show.
	The file of a cue goes by its path from the directory of the show it lands
	in, when it lies there.
	The clipboard stays when another show opens, and one paste adds at most a
	thousand cues, even of a clipboard that holds more.

*paste_above*
	Paste the cues of the clipboard above the selection, as *paste_below*
	does below it.

*delete*
	Delete the selected cue, or as many cues from it down as the count,
	without copying them.
	A group goes with the cues that it holds.
	The vi keymap binds no key to it, since *cut* is what its *d d*, *x*, and
	*Delete* do, as in Vim.

*move_down*
	Move the selected cue one place down its list, or as many places as the
	count.
	A group moves with the cues that it holds, as one block.

*move_up*
	Move the selected cue one place up its list, or as many places as the
	count.
	A group moves with the cues that it holds, as one block.

*indent*
	Move the selected cue into the group above it, together with the cues that it
	holds, or the selected cue and as many cues below it as the count.
	A cue goes one level deeper where the cue above it is a group, or lies deeper
	than the cue, and stays where it is elsewhere, as at the top of a list.
	The vi keymap binds *> >*, and *> j* indents the cues that a motion reaches,
	as *TYPING* describes.
	It is one change that undo takes back, and the selection stays on the first
	cue moved.

*outdent*
	Move the selected cue out of its group, together with the cues that it holds,
	or the selected cue and as many cues below it as the count.
	The cue goes to just below the group, so that the cues of the group that were
	below it stay in the group.
	The vi keymap binds *< <*, and *< j* outdents the cues that a motion reaches.
	It is one change that undo takes back, and the selection stays on the first
	cue moved.

*undo*
	Undo the latest change to the show, or as many of the latest changes as the
	count, stopping at the oldest.
	The status bar names the last change that it took back, and says how many
	changes there were where it took back more than one, as in
	*Undid 5 changes, the last: added cue 43*.
	Undoing an addition or a paste, of all the cues that a count made, selects
	the cue that it went below or above, as the cursor of Vim goes back to
	where the change was made.
	The keys typed into a field during one visit to it are one change, and a
	visit that the keyboard ends by leaving the field, or that a click or a move
	to a field begins again, is a change of its own, as each stay in insert mode
	is in Vim.

*redo*
	Redo the change undone last, or as many changes as the count, stopping at
	the newest.
	The status bar tells it as *undo* does.

*open*
	Open a show that the file chooser of the desktop picks.
	The show on screen keeps its unsaved changes, as *cinq-show*(5)
	describes.
	While the show has no file and no cues, and no field has the keyboard, the
	status bar hints this key, and the hint gives way right after those of
	*next_list*, *next_tab*, and *hint*, before any key that acts.

*save*
	Save the show in its file, or in a file that the file chooser of the
	desktop picks for a show that has none yet.

*save_as*
	Save the show in a file that the file chooser of the desktop picks,
	which the show goes by from then on.

*fullscreen*
	Toggle the stage window, which shows the pictures of picture and video
	cues, between fullscreen and a window, until the next GO that fires such
	a cue makes it fullscreen again, as *cinq-show*(5) describes.
	While the stage window is closed and the show holds a picture or video cue,
	it opens the window fullscreen, as a cue that shows on it does, so that the
	film that runs shows again.
	While the stage window is closed and a picture or video cue shows on it,
	the sidebar warns *Stage window closed*, the status bar counts it, and
	hints this key, which gives way right after those of *go* and *panic*.
	Both go once the window is back or no such cue runs.
	While the stage window has the keyboard, the status bar hints this key, as
	in *F11: fullscreen*, which is the way back to the main window where the
	fullscreen stage window covers it on a single display, and the hint gives
	way at the same place.
; if darwin
	Darwin keeps F11 for Show Desktop, a shortcut of Mission Control, so that
	the key reaches *cinq* only once that shortcut is turned off in the
	keyboard shortcuts of System Settings.
	The green button at the top left of the stage window toggles it all the
	same, as does any other key bound to *fullscreen*.
; end

*help*
	Show the help page, or hide it.  It does nothing while the stage window
	has the keyboard.  The vi keymap binds *?* and F1 to it, and
	departs from Vim here, where *?* searches up the list, so that the most
	common help key works everywhere.  *?* types in a text field and in the
	search line, as any character does, so that the status bar hints this key
	only where it acts.

*close_help*
	Hide the help page.

# DIAGNOSTICS

Problems with the file show among the warnings in the sidebar, each after
*config.toml:*, or after *config.toml:3:* for a problem that lies on line 3 of
the file, and the status bar counts them.
They come in the order of the file, by line, in the sidebar and on standard
error, and those that lie on no line come last.
The sidebar lists at most three warnings, of every kind, and a line below them
says how many more there are, as in *and 5 more*, while the status bar keeps
the whole count.
Where exactly one warning would be left out, the sidebar lists it instead of
the line *and 1 more*.
Each time *cinq* reads the file, as it starts and after each change, it writes
the problems to standard error as well, quoting the lines of the file around
each problem that lies in a part of it, with that part marked.
On a terminal, the problems take a screen of their own while any are left, as
a pager's text does, and those of each reading take the place of those before.
Once none are left, the terminal shows again what it showed before, and
problems still left when *cinq* ends stay on it.
What *cinq* and the libraries that it uses print meanwhile to standard error,
or to standard output when that is a terminal, waits, and comes out on that
screen once the problems clear, or before the problems still left when *cinq*
ends.
Only the newest megabyte is kept, and a line says how many bytes before it
were dropped.
On Windows, only what *cinq* itself prints waits.
A file or a pipe takes the problems of each reading in turn, as does a
terminal that cannot switch screens, such as one whose *TERM* is _dumb_.

- A file that cannot be read, or that is not TOML, leaves the keys of the vi
  keymap, the default audio device, no listening for OSC, and the default
  *ready_ahead*.
- An *audio_device* that is not a string, or is an empty one, leaves the
  default device.
- A *keymap* that names no keymap leaves the keys of vi, and a *keys* that is
  not a table leaves those of the keymap.
  A keymap of the file's own whose *from* names no keymap starts from no keys,
  as does the one whose *from* closes a loop of *from*s followed from the
  keymap in use, and one named after a shipped keymap goes unused.
  A keymap that no setting chooses is checked as well, for entries that name no
  action and for keys that cannot be read, and a value under *keymaps* that is
  not a table is a problem.
- A *ready_ahead* that is not a number, or is below 0, such as a negative
  number or a string, leaves the default of 10 seconds.
- An *osc_listen* that is not a string, or is not an IP address and a port
  above 0, leaves *cinq* listening to nothing.
- A setting other than *audio_device*, *keymap*, *keymaps*, *keys*,
  *osc_listen*, and *ready_ahead*, and an entry of a table that names no
  action, are ignored, and the rest of the file applies.
  A setting written below a table, such as *osc_listen* below *keys*, is
  an entry of that table, and the problem says that it belongs above the
  first table.
- An empty string, a key that names no key or that *KEYS* rules out, a
  sequence that starts with a digit, and a value that is not a string are left
  out, and the action takes the other keys listed for it.
  An entry of which none can be read leaves the action the keys that the
  keymap gave it, and a problem says so, as *go keeps Space* does; an empty
  list, written on purpose, leaves it none.
- A sequence bound to two actions goes to the one listed first under
  *ACTIONS*, and the other loses it, as the problem tells, unless the two act
  in different places: *pick* and *close_menu* act only in an open menu, so a
  sequence may be bound to one of them and to an action that acts on the cue
  list and not in a menu, as *Control+y* is to *pick* and to *scroll_up* in
  the vi keymap.  An action that acts in both, such as *next*, clashes with
  them.
  A sequence that begins a longer one, as *g* begins *g g*, leaves the longer
  one impossible to type.
  A key bound by itself to *go* or *panic* makes every other sequence that
  holds it impossible to type, and when it is bound without modifiers,
  whatever modifiers are held with it there, apart from a key that the other
  of the two has by itself, as *TYPING* describes.

The help page shows the keys as they ended up, and leaves out the actions that
the keymap in use binds to no key.

# EXAMPLES

Declare a keymap of one's own, *desk*, that starts from *vi* and adds desktop
keys that *vi* leaves free: GO on F5 as well as on the space bar, F2 to change
a name, and Control+z and Control+v to undo and to paste, each beside the keys
of *vi*:

```
keymap = "desk"

[keymaps.desk]
from = "vi"
go = ["Space", "F5"]
change_name = ["c c", "S", "F2"]
undo = ["u", "Control+z"]
paste_below = ["p", "Control+v"]
```

Keep the vi keymap, but move through cues and menus with the arrows alone,
and let no key cut a cue:

```
[keys]
next = "ArrowDown"
previous = "ArrowUp"
cut = []
```

Build a keymap of one's own from no keys at all, in which only GO, Panic, and
the arrows act:

```
keymap = "booth"

[keymaps.booth]
go = "Space"
panic = "Escape"
next = "ArrowDown"
previous = "ArrowUp"
```

Make ready the sounds that start within 2.5 seconds, on a small system:

```
ready_ahead = 2.5
```

Take OSC commands, such as _/go_ and _/cue/5/start_, from programs on the same
computer alone, on the port that QLab uses:

```
osc_listen = "127.0.0.1:53000"
```

; if xdg
Play the sound on the second sound card, such as a USB audio interface beside
the card built in, through the plug that converts samples for it:

```
audio_device = "alsa:plughw:CARD=1,DEV=0"
```
; end

# SEE ALSO

*cinq*(1), *cinq-show*(5)

UI Events KeyboardEvent key Values
	https://www.w3.org/TR/uievents-key/

TOML
	https://toml.io/
; if xdg

XDG Base Directory Specification
	https://specifications.freedesktop.org/basedir-spec/latest/
; end
