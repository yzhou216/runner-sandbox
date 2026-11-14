cinq-show(5) "cinq" "File Formats Manual"

; SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
; SPDX-License-Identifier: GPL-3.0-or-later

; The lines after "; if" belong only to the pages of the platforms that it
; names, xdg, darwin, or windows, until the next "; if" or "; end".

# NAME

cinq-show - show files of cinq

# SYNOPSIS

_show_.cinq.toml

# DESCRIPTION

A show of *cinq* lives in a file in TOML, which *cinq* opens from its command
line or from the file chooser of the desktop, and which saving writes.
The file lists the instruments of the light patch, then the cue lists of the
show in order, each with its cues, and each cue with the settings that its
kind uses.
What belongs to a session rather than to the show stays out of the file: the
cue on standby, which is the first of each list once the show opens, the cues
that run, the levels of the lights, and how long each sound or video file
plays.

Saving leaves out the settings that the kind of a cue does not use, and those
that keep their defaults, apart from *level*, *duration*, and *stop_target*,
which it writes for every cue whose kind uses them.
A file can leave out any setting but the *name* of a list and the *id* and the
*kind* of a cue.
*cinq* passes over settings that it does not know.

# LISTS

Each cue list is a table in the array *list*, in order:

*name*
	Name of the list, as the sidebar shows it.

*cue*
	Array of the cues of the list, in order.

A file without lists holds one empty list.

# CUES

Each cue is a table in the array *cue* of its list:

*id*
	Identity of the cue, a whole number that no other cue of the show has,
	by which fade and stop cues name their targets.

*number*
	Number of the cue, as the cue list shows it, which can be any text.

*kind*
	What the cue does: *audio*, *video*, *picture*, *light*, *network*,
	*fade*, *stop*, *wait*, or *memo*.
	An audio cue plays a sound file, a video cue plays a video file, and a
	picture cue shows a picture file, as *STAGE* describes, a light cue
	fades instruments of the light patch, as *LIGHTS* describes, and a
	network cue sends a message over the network, as *NETWORK* describes.
	The clock of an audio or video cue follows its sound as the audio device
	plays it.
	Its fades, its end, and the cues that follow it therefore stay with what
	is heard, even when the device waits before it plays, as a device just
	chosen can.

*name*
	Name of the cue.

*file*
	Sound file that an audio cue plays, video file that a video cue plays, or
	picture file that a picture cue shows: an absolute path, or one relative
	to the directory of the show file.
	Sound files hold WAV, FLAC, Ogg Vorbis, MP3, or AAC in MP4.

*start*
	Seconds into its file at which an audio or video cue starts, which are
	none by default.

*end*
	Seconds into its file at which an audio or video cue ends, which is the
	end of the file by default.
	A cue that starts at or after its end cannot play.

*loop*
	Whether an audio or video cue plays from its start again each time it
	reaches its end, which it does not by default.
	A cue that loops plays until something stops it, such as a fade cue that
	stops it, a stop cue, or a panic.

*target*
	Identity of the cue that a fade or stop cue acts on.
	A fade cue acts on an audio, video, or picture cue, and a stop cue on any
	cue but itself.

*pre_wait*
	Seconds between the cue firing and its action, which are none by
	default.

*duration*
	Seconds that the action of a light, fade, or wait cue takes, which are
	five by default.

*post_wait*
	Seconds between the action of the cue beginning, once its pre-wait is
	over, and the next cue firing, for a cue that continues to it, which are
	none by default.

*advance*
	What happens to the next cue once this one has fired: *manual*, the
	default, leaves it waiting for GO, *continue* fires it once the
	post-wait is over, and *follow* fires it once the action of this cue
	ends, which that of a picture cue, or of an audio or video cue that loops,
	never does.

*level*
	Level in dB that an audio or video cue plays at, which is 0 by default, or
	that a fade cue fades its target to, which is -60, silence, by default.
	It lies between -60 and 12.

*stop_target*
	Whether a fade cue stops its target once the fade is over, which it does
	by default.

*notes*
	Notes of the cue, for the operator.

*lights*
	Table of the levels in percent, from 0 to 100, that a light cue fades
	instruments to, by the *id* of each instrument.

*destination*
	Where a network cue sends its message.

*message*
	Message that a network cue sends.

A file in which two cues or two instruments share an identity, or a level or
an address lies outside its range, cannot be opened.

# STAGE

A video cue plays H.264 video of the Baseline, Main, or High profile in MP4, as
files ending in _.mp4_, _.m4v_, or _.mov_ hold it, along with the sound of its
AAC track, in mono or stereo.
A file without a sound track plays its picture alone.
The picture of a video cue follows the clock of the cue, so that it stays with
the sound over a long video.
cinq reads no tags or cover art, and skips a track other than the picture's
that cannot be read, such as one of chapters, which it says once on standard
error.
A video cue takes as long as the picture of its file, and its *start*, *end*,
*loop*, *level*, and waits work as those of an audio cue do.

A picture cue shows a picture in PNG or JPEG, as files ending in _.png_,
_.jpg_, or _.jpeg_ hold it, turned upright as the file tells, and laid on
black where it is not wholly opaque.
It shows until something stops it, such as a fade cue that stops it, a stop
cue, or a panic, so that a cue that follows it waits for GO.

The pictures of picture and video cues show in a stage window of their own,
which opens beside the main window once the show holds a picture or video
cue, as a window that the operator can move to the display of the audience.
It shows black while no picture shows, and otherwise the picture of the cue
that fired last, whole and as large as the window allows.
Each GO that fires a picture or video cue makes the stage window fullscreen,
on the display that it is on, and brings it before the other windows, so
that the picture fills the screen as it comes up.
A picture that comes up hides those of the cues that fired before it where
it lies over them, which show again once it has gone, as that of a picture
cue does once a video that fired after it ends, unless something has stopped
the picture cue.
Where its shape differs from that of the window, the pictures of the cues
that fired before it show around it, as in QLab, and black where there are
none.
A fade cue that stops a picture or video cue fades its picture out, along
with the sound of a video, as a panic does: to black, or to the picture of
an earlier cue that shows on.
Pausing holds the picture of a video, and a video cue that loops loops its
picture with its sound.

Closing the stage window closes it until a picture or video cue fires again,
which opens it anew, fullscreen, wherever the platform puts it.
Closing the main window quits *cinq*.
The key of *fullscreen* toggles the stage window between fullscreen and a
window, until the next GO that fires a picture or video cue, and the other
keys act on the cue list from either window, as *cinq*(5) describes.

# LIGHTS

Each instrument of the light patch is a table in the array *instrument*, in
order, and has one level, its intensity, which one DMX address sets:

*id*
	Identity of the instrument, a whole number that no other instrument of
	the show has, by which light cues name it.

*name*
	Name of the instrument, as the inspector shows it.

*universe*
	Universe of the instrument, from 1 to 32768, which is 1 by default.
	Universes count from 1, as in sACN, and Art-Net, which counts from 0,
	carries universe 1 as its Port-Address 0 and each universe after it as
	the next Port-Address, as most consoles do.

*address*
	DMX address of the instrument in its universe, from 1 to 512.

The inspector edits the patch on its tab *Light Patch*, where an address
reads as the DMX address alone in universe 1, and as the universe and the
DMX address with a slash between them in the others, as in _2/12_.
A new instrument takes the address after that of the last.

A light cue fades the instruments that it names from their levels as it
starts to its own over its *duration*, and leaves the others as they are, so
that each instrument keeps the level of the last light cue that named it,
as in QLab.
A light cue takes the instruments that it names over from an older one that
is still fading them.
Pausing a light cue pauses its fade, and stopping it, as a panic does,
leaves its instruments where the fade has brought them.
Every instrument has no level until a light cue gives it one, each time
that the show opens.

*cinq* sends the levels of each universe that the patch uses both as
Art-Net, to the broadcast address of every IPv4 network that the computer
is on, and as sACN, to the multicast group of the universe on each of those
networks, as soon as they change and again every second.
It sends both to the computer itself as well.
Its sACN has the priority 100, the default of sACN, so that a receiver that
merges sources of one priority takes the higher level of each address from
*cinq* and any other source, such as a lighting console.
Instruments that share an address set it to the highest of their levels.
When *cinq* ends, receivers of sACN hold its levels for as long as they
hold those of any source that has gone quiet.

# NETWORK

A network cue sends an Open Sound Control (OSC) message once its pre-wait is
over, as the network cues of QLab do, so that a GO in *cinq* can fire a cue
of a lighting console or of another program.
It takes no time, and does nothing else.

The *destination* of a network cue is _host:port_ for TCP, whose packets
*cinq* frames with SLIP, as OSC 1.1 asks of streams, or _udp://host:port_ for
UDP, with one packet to each datagram.
The host is an IPv4 address, or a name that *cinq* reaches at its first IPv4
address.
*cinq* keeps a TCP connection to a destination open once it has made it, and
makes a new one once the destination closes it, as a console that restarts
does.

The *message* of a network cue holds its address, which starts with a slash,
followed by its arguments after spaces, each an integer, a decimal number, or
else a string.
Double quotes keep a string whole across spaces, and keep it a string where
it reads as a number.

A network cue whose destination or message does not read as one cannot play.
A message that cannot go out, as when the destination refuses the connection
or takes more than a second to take it, is lost.
So is a message that waits more than a second behind others, as behind a
console that does not answer, since a cue that fires late can do more harm
than one that does not fire.
A console that loses its power or its cable shows among the warnings as a
lost connection within about ten seconds, and the next message to it makes
a new connection.
The warnings in the sidebar say why until a message to the same destination
gets there, and *cinq* says so on standard error as well, once for each
reason.

Consoles and programs tell in their manuals which messages they take, and on
which ports.
For example, the Eos consoles of ETC take OSC over TCP with SLIP on port
3037, and fire cue 5 of cue list 1 for _/eos/cue/1/5/fire_.

# UNSAVED CHANGES

Changes to a show that are not saved yet never go away unasked.
As they happen, *cinq* keeps them in a file of their own, in the directory

; if xdg
	_$XDG_STATE_HOME/cinq/unsaved_

where *XDG_STATE_HOME* is _~/.local/state_ when it is unset or is not an
absolute path, as the XDG Base Directory Specification sets out.
; if darwin
	_~/Library/Application Support/cinq/unsaved_
; if windows
	_%LOCALAPPDATA%\\cinq\\unsaved_
; end

The changes to a show file go under _shows_ in that directory, by the
absolute path of the file, and those to a show that has no file yet go in
_untitled.cinq.toml_.
; if windows
The drive of the file leads its path, as _C_ for _C:\\_, and a network share
does as _UNC\\server\\share_, so that files of the same path on another drive
or share keep their changes apart.
; end

Closing the window, opening another show, or a crash leaves the kept changes
in place.
The next time that the show opens, they come back as one change, which undo
takes back to the show as saved.
Saving the show, or taking back each of its changes, removes them.

# BUGS

While a video is paused, its sound stands a hundredth of a second ahead of its
picture, the length of the fade that keeps a pause from clicking, until the
video resumes and the picture catches up.

The sound of a video plays the silence that AAC encoders put before and after
it, which the file marks for players to skip, so that it comes a few
hundredths of a second after its picture.

A picture wider or taller than the largest texture of the GPU, as a video in
4K or a large photograph is on some older GPUs, does not show.

; if windows
Each tick of the show clock waits for both windows to draw, since Windows would
otherwise let a window that draws slowly keep the other from drawing at all.
Without a GPU, as in many virtual machines, drawing is slow enough that a video
of high definition shows only some of its pictures.
; end

# EXAMPLES

A list in which a storm loops from half a second into its file, and continues
to a fade cue, which fades it out over eight seconds and then stops it:

```
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Storm"
file = "sounds/storm.ogg"
start = 0.5
loop = true
level = -4.0
advance = "continue"
post_wait = 2.0

[[list.cue]]
id = 2
number = "2"
kind = "fade"
name = "Storm out"
target = 1
duration = 8.0
level = -60.0
stop_target = true
```

A list in which a video plays from two seconds into its file, a little below
the level of its sound, until a fade cue fades its picture to black and its
sound to silence over three seconds, and then stops it:

```
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "video"
name = "Opening titles"
file = "video/titles.mp4"
start = 2.0
level = -6.0

[[list.cue]]
id = 2
number = "2"
kind = "fade"
name = "Titles out"
target = 1
duration = 3.0
level = -60.0
stop_target = true
```

A list in which a picture shows before the show, until GO starts a film,
which hides it at once while a fade cue that continues from the film fades
it out, so that it does not show again once the film ends:

```
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "picture"
name = "Preshow"
file = "pictures/preshow.png"

[[list.cue]]
id = 2
number = "2"
kind = "video"
name = "Film"
file = "video/film.mp4"
advance = "continue"

[[list.cue]]
id = 3
number = "3"
kind = "fade"
name = "Preshow out"
target = 1
duration = 1.0
level = -60.0
stop_target = true
```

A patch of two instruments, and a list in which a light cue brings the front
light up at once, and the next fades it down while it brings the back light
up over five seconds:

```
[[instrument]]
id = 1
name = "Front"
address = 1

[[instrument]]
id = 2
name = "Back"
address = 2

[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "light"
name = "Preset"
duration = 0.0

[list.cue.lights]
1 = 100.0

[[list.cue]]
id = 2
number = "2"
kind = "light"
name = "Back light"
duration = 5.0

[list.cue.lights]
1 = 40.0
2 = 80.0
```

A list in which GO fires cue 5 of an Ion Xe at its default address, along with
a sound:

```
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "5"
kind = "network"
name = "Lights for the storm"
advance = "continue"
destination = "10.101.100.20:3037"
message = "/eos/cue/1/5/fire"

[[list.cue]]
id = 2
number = "5.1"
kind = "audio"
name = "Storm"
file = "storm.wav"
level = 0.0
```

# SEE ALSO

*cinq*(1), *cinq*(5)

TOML
	https://toml.io/
; if xdg

XDG Base Directory Specification
	https://specifications.freedesktop.org/basedir-spec/latest/
; end
