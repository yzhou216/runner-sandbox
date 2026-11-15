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
It ships two keymaps, each of which binds keys to every action, and a
configuration file in TOML chooses one of them, or builds keymaps of its own,
and binds actions to other keys, and it sets the audio device that sound plays
on and how far ahead *cinq* makes sounds ready.
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

The help page, which the key of *help* opens, names the keymap in use, lists
every action with its keys and its name in the file, links to the file that
the keys come from, or would come from, and ends with the notice of the
license of cinq.
Following the link opens the file in
; if xdg windows
the program that the desktop opens it with,
; if darwin
the default text editor,
; end
and first makes the file when it is not there yet, holding only a line
that points to this page.

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

# SETTINGS

*audio_device*
	ID of the audio device that the sound of audio and video cues plays on, in
	place of the default device of the system.
	*cinq --audio-devices* lists the ID and the name of each device, apart by a
	tab, one device to a line.
	The *Audio device* list in the sidebar of the main window chooses a device
	as well, and *cinq* writes the choice here.
	It changes only the line of this setting, or adds the line before the
	first setting of the file, and leaves the rest as it is.
	A change moves the sound as soon as the file is saved, and the sounds that
	play go on from where they are on the new device.
	A device can take a moment to start, during which nothing is heard and
	the cues that play wait for it.
	While the device is not there, or cannot be opened, nothing is heard and
	the warnings in the sidebar tell why, and saving the file tries again.
; if xdg
	On ALSA, *cinq* plays on most sound cards only through a plug that
	converts its samples, as the devices whose IDs begin with _alsa:plughw:_
	or _alsa:sysdefault:_ do, and _alsa:default_ and _alsa:pipewire_.
	The _alsa:hw:_, _alsa:front:_, and _alsa:hdmi:_ devices of most cards take
	samples only in a format of their own, and cannot be opened.
; end
	It is the default device unless the file names another.

*keymap*
	Name of the keymap that the keys start from: one that *cinq* ships, *vi*
	or *cua*, as *KEYMAPS* describes them, or one of the file's own under
	*keymaps*.
	It is *vi* unless the file names another.

*keymaps*
	Table of keymaps of the file's own, each of them a table named after its
	keymap.
	The *from* of such a keymap names the keymap that it starts from, a
	shipped one or another of the file's own, and without *from* it starts
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

*ready_ahead*
	Number of seconds, 0 or more and whole or fractional, before a sound starts
	that *cinq* makes it ready, so that GO opens no sound file itself.
	*cinq* makes ready the sounds that start within that time, of the runs
	that GO has fired and have yet to sound, and of what the next GO of each
	list would fire, counting each sound from its delay in a chain and its
	pre-wait.
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

*cua*
	Keys of IBM's Common User Access, which most desktop programs follow: the
	arrows, Home, and End move, Control with a letter edits, and F1 shows the
	help page, which Control+w closes.

Both keep GO on the space bar and Panic on Escape, as in QLab.
The table lists the keys that each keymap binds to every action, in the order
of the help page.

|[ *Action*
:< *vi*
:< *cua*
|  go
:  Space
:  Space
|  panic
:  Escape
:  Escape
|  standby
:  Enter
:  Enter
|  next
:  j, g j, ArrowDown, Control+n
:  ArrowDown
|  previous
:  k, g k, ArrowUp, Control+p
:  ArrowUp
|  first
:  g g, Control+Home
:  Home
|  last
:  G, Control+End
:  End
|  half_page_down
:  Control+d
:  none
|  half_page_up
:  Control+u
:  none
|  page_down
:  Control+f, PageDown
:  PageDown
|  page_up
:  Control+b, PageUp
:  PageUp
|  scroll_down
:  Control+e
:  none
|  scroll_up
:  Control+y
:  none
|  top_row
:  H
:  none
|  middle_row
:  M
:  none
|  bottom_row
:  L
:  none
|  row_to_middle
:  z z
:  none
|  row_to_top
:  z t
:  none
|  row_to_bottom
:  z b
:  none
|  next_list
:  l, ArrowRight
:  Control+PageDown
|  previous_list
:  h, ArrowLeft
:  Control+PageUp
|  next_tab
:  g t
:  Control+Tab
|  previous_tab
:  g T
:  Control+Shift+Tab
|  back
:  Control+o
:  Alt+ArrowLeft
|  forward
:  Control+i
:  Alt+ArrowRight
|  search
:  /
:  Control+f
|  search_backward
:  ?
:  none
|  next_match
:  n
:  F3
|  previous_match
:  N
:  Shift+F3
|  edit
:  i, a, A
:  F2
|  edit_start
:  I
:  none
|  change_name
:  c c, S
:  none
|  next_field
:  Tab
:  Tab
|  previous_field
:  Shift+Tab
:  Shift+Tab
|  pick
:  Control+y
:  none
|  close_menu
:  Control+e
:  F4
|  add_below
:  o
:  Control+n
|  add_above
:  O
:  Control+N
|  cut
:  d d, x, Delete
:  Control+x
|  copy
:  y y, Y
:  Control+c
|  paste_below
:  p
:  Control+v
|  paste_above
:  P
:  Control+V
|  delete
:  none
:  Delete
|  move_down
:  Alt+j, Alt+ArrowDown
:  Alt+ArrowDown
|  move_up
:  Alt+k, Alt+ArrowUp
:  Alt+ArrowUp
|  undo
:  u
:  Control+z
|  redo
:  Control+r
:  Control+y, Control+Z
|  open
:  Control+l
:  Control+o
|  save
:  Control+s
:  Control+s
|  save_as
:  Control+S
:  Control+S
|  fullscreen
:  F11
:  F11
|  help
:  F1
:  F1
|  close_help
:  q
:  Control+w

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

# TYPING

The keys of a sequence wait until the sequence is complete, and the status bar
shows the keys typed so far, while a box above it lists the keys that can come
next and what each of them does.
A key that completes or continues no sequence is dropped together with the
keys typed so far, as Vim beeps and drops them, so that a key mistyped halfway,
such as the *x* of *d x*, never starts a sequence of its own.
Anything done other than typing, such as a click, drops a sequence typed
halfway, and so does a change to the file.

Digits typed before a sequence count it, as in *3 j* with the vi keymap.
The count repeats *next*, *previous*, *next_list*, *previous_list*,
*move_down*, *move_up*, *scroll_down*, *scroll_up*, *back*, *forward*,
*next_match*, and *previous_match*.
It copies, cuts, deletes, or adds as many cues for *copy*, *cut*, *delete*,
*add_below*, and *add_above*, pastes as many times for *paste_below* and
*paste_above*, undoes or redoes as many changes for *undo* and *redo*, and
goes back as many tabs for *previous_tab*.
It gives the row, counted from one, that *first* and *last* select, and the
tab, counted from one, that *next_tab* shows.
It gives the row in view, counted from the top for *top_row* and from the bottom
for *bottom_row*, that they select.
For *half_page_down* and *half_page_up*, it sets how many rows they move, as
in Vim, and later presses of either move that many rows, or lines on the help
page, until another count sets it again, the main window changes size, or
another show opens.
For *page_down* and *page_up*, it counts the pages.
Other actions leave the count unused.
A count larger than what there is, such as *9 Control+o* with fewer places
behind, *9 g t* with fewer tabs, or *3 d d* on the last cue, makes cinq go as
far as it can, where Vim would refuse it.
Since digits count, no sequence can start with one.

The actions *copy*, *cut*, and *delete* also act as operators, as *y* and *d*
do in Vim.
An operator key is the key that one of their bindings doubles: *d* for *d d*
and *y* for *y y* in the vi keymap, and *D* for a binding *D D* that replaces
*d d*.
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
A motion that cannot move, such as *d k* on the first cue, does nothing, and a
count past the end stops at the last cue.
The keys of *top_row*, *middle_row*, and *bottom_row* are no motions, since
the rows in view are known only once the window has drawn the list.
The cues of a cut leave the selection on the cue that takes their place, and a
copy leaves it on the first cue copied.
Cutting is one change for undo, and the clipboard holds the cues in order.
Show mode locks *d* with a motion as it locks *cut* and *delete*, and leaves *y*
with a motion alone, as it leaves *copy*.
A key that is neither a count nor a motion drops the operator and itself, and
the keys of *go* and *panic* act at once as usual.

A key bound by itself to *go* or *panic* acts at once, even halfway through
another sequence, and when it is bound without modifiers, whatever modifiers
are held with it, unless the other of the two is bound to the key with those
very modifiers.
Holding either key down acts only once, so that holding the key of Panic does
not cut a panic short.
So does holding a key of *fullscreen*, which would otherwise toggle the stage
window back and forth, while every other key repeats as it is held.

# SEARCHING

The key of *search* opens a search line at the left of the status bar, where
the status stood, which reads a slash and then the text typed, and the key of
*search_backward* opens it with a question mark for a search up the list.
As the text changes, the selection moves to the first cue of the list on
screen, from the cue after the selected one down, whose name or number holds
the text, in any case, and the list scrolls to it.
A search up the list begins at the cue before the selected one.
The search wraps around the list and ends on the selected cue, as a search of
Vim does from the start of a line, so that the selected cue is the match only
when no other cue holds the text.
Enter accepts the search and closes the line, and remembers the text and the
direction for *next_match* and *previous_match*.
*next_match* finds the next cue that holds the text in the direction of the
search accepted last, and *previous_match* in the other direction, as *n* and
*N* do in Vim, wrapping around at either end.
Enter on an empty line repeats the search accepted last in the direction of
the line.
Backspace on an empty line closes it, and so does Escape, which is also
*panic*, and either puts the selection back where it was when the line
opened.
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
The line keeps the keys typed into it, like a text field, and the hints at
the right of the status bar give way to its text.
The stage window cannot open it by keys while it has the keyboard.

# WHERE KEYS ACT

Keys act on the cue list, whether the main window or the stage window has
the keyboard, apart from the keys typed while one of these has the keyboard:

A text field of the inspector, or a drop-down list
	A click gives a text field the keyboard, and the keys of *next_field* and
	*previous_field* move it from one text field or drop-down list to the
	next, which shows a border in the color of focus.
	It keeps the keys typed, apart from the keys bound by themselves to *go*,
	*panic*, *next_field*, *previous_field*, *back*, *forward*, *open*, *save*,
	and *save_as*, which act unless a text field takes them for typing.
	Enter leaves a text field, and opens the menu of a drop-down list, which
	takes the keyboard from there.

The help page
	The page fills the main window, while the stage window keeps showing the
	stage.  Closing it gives the main window back as it was, with the cue
	list scrolled where it was and the keyboard in the same field.  The keys
	of *next*, *previous*, *scroll_down*, and *scroll_up* scroll the page, a
	line for each one and as many lines as a count, those of *first* and *last*
	go to its top and its bottom, and those of *half_page_down*,
	*half_page_up*, *page_down*, and *page_up* scroll it by half or all of
	what the window shows, with a count of lines for the half pages, which
	the cue list shares, and of pages for the whole pages.  Of the rest, only the keys of *go*, *panic*, *help*, and
	*close_help* act, whichever window has the keyboard, so that no field
	takes a key meanwhile.  The keys of *go* and *panic* close the page as
	well, so that the show is in view.  The key of *help* opens the page
	from the main window only, since the stage window may cover that window
	when it is fullscreen, and does nothing while the stage window has the
	keyboard, where the key of *close_help* closes the page.

The menu of a drop-down list
	The keys bound by themselves to *next* and *previous* move the highlight
	one option at a time, whatever the count, and the highlight stays put past
	either end.
	Those bound to *page_down* and *page_up* move it by the options in view,
	and those bound to *first* and *last* to the first and last option.
	PageDown, PageUp, Home, and End do so as well, as they do in the popup menu
	of Vim, whether or not the keymap binds them.
	Enter and the key of *pick* pick the highlighted option, and the key of
	*close_menu* closes the menu without picking one.
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
it, *edit*, *edit_start*, *change_name*, *next_field*, and *previous_field* do
nothing, since the keys typed there would never reach a field of the main
window.
A click on the main window gives it the keyboard.

Show mode locks editing: *edit*, *edit_start*, *change_name*, *add_below*,
*add_above*, *move_down*, *move_up*, *delete*, *cut*, *paste_below*, and
*paste_above* do nothing in it, and *undo* and *redo* only move standby back
and forth.
Copying changes no show, so that *copy* acts in show mode, as Vim yanks in a
read-only buffer.

# ACTIONS

The actions go by their names in the *keys* table, in the order of the help
page.

*go*
	GO: fire the standby cue, together with every cue that auto-continue or
	auto-follow chains to it.

*panic*
	Panic: fade every playing cue out over one second, and stop the rest.
	A second panic while the first is still fading stops everything at once.

*standby*
	Put the selected cue on standby, so that the next GO fires it.

*next*
	Select the next cue, move the highlight of an open menu down, or scroll
	the help page down.
	The vi keymap binds *g j* as well as *j*, since a user of Vim types *gj* to
	move by a displayed line.

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
	The keys of *pick* and *close_menu* act only in a menu, so a key may be
	bound to one of them and to a scroll without a clash.

*scroll_up*
	Scroll the cue list a row up, or as many rows as the count, as
	*Control+y* does in Vim, and keep the selection in view as *scroll_down*
	does.
	On the help page, scroll up a line.
	While a menu is open, *Control+y* picks the highlighted option instead.

*top_row*
	Select the first cue row in view, or the row that many rows below it
	for a count, stopping at the last row in view, as *H* does in Vim.
	The view stays where it is.
	It is a jump for *back*.

*middle_row*
	Select the cue row in the middle of the view, or of the rows in view when
	the list is shorter, taking the upper of two middle rows, as *M* does in
	Vim.
	It is a jump for *back*.

*bottom_row*
	Select the last cue row in view, or the row that many rows above it for a
	count, stopping at the first row in view, as *L* does in Vim.
	It is a jump for *back*.

*row_to_middle*
	Scroll the cue list to put the selected row in the middle of the view, as
	*z z* does in Vim.
	The selection stays on its cue, and the view stops at either end of the
	list, so that a row near an end stays short of the middle.
	It is no jump, and leaves the count unused.

*row_to_top*
	Scroll the cue list to put the selected row at the top of the view, as
	*z t* does in Vim, stopping at the end of the list.

*row_to_bottom*
	Scroll the cue list to put the selected row at the bottom of the view, as
	*z b* does in Vim, stopping at the top of the list.

*next_list*
	Show the next cue list.

*previous_list*
	Show the previous cue list.

*next_tab*
	Show the next tab of the inspector, or the tab whose number the count gives,
	counted from one, or the last tab when there are fewer.

*previous_tab*
	Show the previous tab of the inspector, or go back as many tabs as the count,
	wrapping around at the first.

*back*
	Go back to the place before the last jump, as the jump list of Vim does
	with its *Control+o*.
	A place is a cue list with its selected cue.
	Switching cue lists, *first* and *last* with or without a count, and
	selecting a cue in another cue list are jumps, and so are the searches
	described under *SEARCHING*.
	Moving the selection one row at a time, clicking a cue of the cue list
	shown, GO moving the standby cue, adding, deleting, and moving cues,
	undo, redo, changing the tab, and the help page are not.
	A count goes that many places back, or to the oldest.
	Going back remembers the place left, so that *forward* returns to it.
	A jump after going back moves the places ahead behind the place left, in
	the order that the jump list of Vim keeps them, so that going back again
	still visits them.
	When a text field has the keyboard, going back acts there as well, and
	moves the keyboard to the cue list, so that what is typed next cannot
	rename the cue that it went to.
	A cue that was deleted leaves no place to go to.
	The history holds at most 100 places, each once, and ends when cinq opens
	another show or quits.
	The status bar names the key while there is a place to go back to.
	The back button of the mouse does the same from the main window, unless the
	help page is open.

*forward*
	Go forward again to the place that *back* left, as many places as the
	count.
	In Vim, Tab is the same key as Control+i, while in cinq Tab moves between
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
	before the selected one, as described under *SEARCHING*.

*next_match*
	Select the next cue that matches the search accepted last, in the
	direction of that search, wrapping around at the end of the list, or as
	many matches further as the count, going around the matches at most once.
	It is a jump for *back*.
	It does nothing before a search.

*previous_match*
	Select the next cue that matches the search accepted last in the opposite
	direction, wrapping around at the top of the list, or as many matches back
	as the count, going around the matches at most once.
	It is a jump for *back*.

*edit*
	Edit the name of the selected cue: show the Basics tab and put the caret
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
	Move the keyboard to the next field: a text field or drop-down list of the
	inspector, or the list of the kind of cue that *add_below* and *add_above*
	add.

*previous_field*
	Move the keyboard to the previous field.

*pick*
	Pick the highlighted option of an open menu, as Enter does and as Control+y
	does in Vim's popup menu.
	It does nothing while no menu is open.

*close_menu*
	Close an open menu without picking an option, as Control+e does in Vim's
	popup menu.
	Unlike Escape, which closes the menu and also acts as Panic, it does not
	panic.
	It does nothing while no menu is open.

*add_below*
	Add a cue of the kind chosen in the toolbar below the selection, or at the
	end of the list without one.
	A count adds that many cues in a row, at most a thousand, as one change
	that undo takes back, and selects the last of them, as the cursor of Vim
	goes to the last line that *3o* inserts.

*add_above*
	Add a cue of the kind chosen in the toolbar above the selection, or at the
	end of the list without one, as many as the count, as *add_below* does.

*cut*
	Copy the selected cue, or as many cues from it down as the count, to the
	clipboard and delete them, so that *cut* and then *paste_below* moves a
	cue.
	The vi keymap binds *x* and *Delete* to it as well as *d d*, as Vim cuts
	with all three, and *d* followed by a motion cuts the cues the motion
	reaches, as *TYPING* describes.
	The status bar tells how many cues went to the clipboard and names the keys
	of *paste_below* and *undo*.

*copy*
	Copy the selected cue, or as many cues from it down as the count, to the
	clipboard of *cinq*, in place of what it held.
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
	that undo takes back, and does nothing while the clipboard is empty.
	A pasted cue keeps its number unless another cue holds it, when it takes
	the next free number.
	A fade or stop cue pasted together with its target, whether copied or cut
	with it, targets the new copy of that cue, as in QLab, in each round of a
	count.
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
	The vi keymap binds no key to it, since *cut* is what its *d d*, *x*, and
	*Delete* do, as in Vim.
	The cua keymap binds the Delete key to it, as desktop programs do.

*move_down*
	Move the selected cue one place down its list, or as many places as the
	count.

*move_up*
	Move the selected cue one place up its list, or as many places as the
	count.

*undo*
	Undo the latest change to the show, or as many of the latest changes as the
	count, stopping at the oldest.
	Undoing an addition or a paste, of all the cues that a count made, selects
	the cue that it went below or above, as the cursor of Vim goes back to
	where the change was made.

*redo*
	Redo the change undone last, or as many changes as the count, stopping at
	the newest.

*open*
	Open a show that the file chooser of the desktop picks.
	The show on screen keeps its unsaved changes, as *cinq-show*(5)
	describes.

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
	It does nothing while the stage window is closed.
; if darwin
	Darwin keeps F11 for Show Desktop, a shortcut of Mission Control, so that
	the key reaches *cinq* only once that shortcut is turned off in the
	keyboard shortcuts of System Settings.
	The green button at the top left of the stage window toggles it all the
	same, as does any other key bound to *fullscreen*.
; end

*help*
	Show the help page, or hide it.  It does nothing while the stage window
	has the keyboard.

*close_help*
	Hide the help page.

# DIAGNOSTICS

Problems with the file show among the warnings in the sidebar, each after
*config.toml:*, or after *config.toml:3:* for a problem that lies on line 3 of
the file, and the status bar counts them.
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
  keymap, the default audio device, and the default *ready_ahead*.
- An *audio_device* that is not a string, or is an empty one, leaves the
  default device.
- A *keymap* that names no keymap leaves the keys of vi, and a *keys* that is
  not a table leaves those of the keymap.
  A keymap of the file's own whose *from* names no keymap starts from no keys,
  as does the one whose *from* closes a loop of *from*s followed from the
  keymap in use, and one named after a shipped keymap goes unused.
- A *ready_ahead* that is not a number, or is below 0, such as a negative
  number or a string, leaves the default of 10 seconds.
- A setting other than *audio_device*, *keymap*, *keymaps*, *keys*, and
  *ready_ahead*, and an entry of a table that names no action, are ignored, and
  the rest of the file applies.
- An empty string, a key that names no key or that *KEYS* rules out, a
  sequence that starts with a digit, and a value that is not a string are left
  out, and the action keeps the other keys listed for it, which may be none.
- A sequence bound to two actions goes to the one listed first under
  *ACTIONS*, and the other goes without it, unless the two act in different
  places: *pick* and *close_menu* act only in an open menu, so a sequence may
  be bound to one of them and to an action that acts on the cue list, as
  *Control+y* is to *pick* and to *scroll_up* in the vi keymap.
  A sequence that begins a longer one, as *g* begins *g g*, leaves the longer
  one impossible to type.
  A key bound by itself to *go* or *panic* makes every other sequence that
  holds it impossible to type, and when it is bound without modifiers,
  whatever modifiers are held with it there, apart from a key that the other
  of the two has by itself, as *TYPING* describes.

The help page shows the keys as they ended up, and leaves out the actions that
the keymap in use binds to no key.

# EXAMPLES

Use the cua keymap, with GO on F5 as well as on the space bar:

```
keymap = "cua"

[keys]
go = ["Space", "F5"]
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
