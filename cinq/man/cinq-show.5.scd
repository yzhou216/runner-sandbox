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
The file lists the outputs of the show, the cue lists of the show in order,
each with its cues, and each cue with the settings that its kind uses.
What belongs to a session rather than to the show stays out of the file: the
cue on standby, which is the first of each list once the show opens, the cues
that run, and how long each sound or video file plays.

Saving leaves out the settings that the kind of a cue does not use, and those
that keep their defaults, apart from *level*, *duration*, and *stop_target*,
which it writes for every cue whose kind uses them.
A file can leave out any setting but the *name* of a list and the *id* and the
*kind* of a cue.
*cinq* passes over settings that it does not know.

# OUTPUTS

An output is a named place that cues send their sound to, such as the house
speakers, a subwoofer, or the monitors on stage.
It plays on one channel of an audio device, or on two, left first.
Cues name outputs and never devices or channels, so that moving an output to
other channels, or to another device at another venue, moves every cue that
sends to it.
Each output is a table in the array *output*, in order:

*name*
	Name of the output, which no other output of the show has.

*device*
	ID of the audio device that the output plays on, as
	*cinq --audio-devices* lists it.
	An output without one plays on the default device, which *cinq*(5) can
	choose for each machine.

*device_name*
	Name of the device, which finds the device on a machine that gives it
	another ID, as long as no other device there has the name.

*channels*
	Channels of the device that the output plays on, counted from 1 to 256: _3_
	for one channel, and _1-2_ or _1,3_ for two, the left first.
	A stereo file plays its left and right on the two channels of a stereo
	output, and its sum on the channel of a mono one.
	A mono file plays on every channel of the output at its own level.

*new_cues*
	Level in dB at which new audio and video cues send to the output, which
	they do not without one.

A file without outputs has one, Main, on the first two channels of the default
device, at which new cues send at 0 dB, and a show that has Main alone, as it
was made, writes no outputs.
An output that cannot play, as one whose device is not there, shows among the
warnings, and the cues play on the others.
So does an output on channels past those of its device, while the other outputs
of the device play.
*cinq* tries such a device again every two seconds, so that a device plugged in,
or a Dante Virtual Soundcard started, plays with nothing for the operator to do.
A device that stops while cues play on it, as one unplugged or a Dante Virtual
Soundcard that restarts, holds their clocks until it tells that it stopped, or
for three seconds at most, after which they go on by the clock of the show, and
once the device opens again they play on there from where they are.
A file in which two outputs share a name, or whose channels are neither one
nor two, cannot be opened.

## Dante

On a Dante network, outputs play on the channels of the Dante Virtual
Soundcard, which Dante Controller routes on to the channels of a board or of
any other Dante device: channel 3 of an output is transmit channel 3 of the
soundcard.
On Darwin, the soundcard is one Core Audio device with as many channels as its
control panel sets.
On Windows, *cinq* built with ASIO, as Nix builds it, reaches every channel of
the soundcard as one ASIO driver, while the WDM mode of the soundcard offers
each pair of its first 16 channels as a device of its own, so that an output
on channel 3 there plays on channel 1 of _DVS Transmit 3-4_.
*cinq* plays at the sample rate of the soundcard, which Dante Controller sets,
whatever the rate of the files, and opens the soundcard again at once at a new
rate set while cues play, as it opens a device that restarts.

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
	What the cue does: *audio*, *video*, *picture*, *fade*, *stop*, *wait*, or
	*memo*.
	An audio cue plays a sound file, a video cue plays a video file, and a
	picture cue shows a picture file, as *STAGE* describes.
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
	Seconds that the action of a fade or wait cue takes, which are five by
	default.

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

*sends*
	Table of the levels in dB at which an audio or video cue sends to
	outputs, each entry named after an output, between -60 and 12.
	The cue plays on those outputs alone, after its *level*, which fade cues
	move.
	A cue without one sends to the first output at 0 dB.

*notes*
	Notes of the cue, for the operator.

A file in which two cues share an identity, a level lies outside its range, or
a cue sends to an output that the show does not have, cannot be opened.

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

A sound whose sample rate differs from that of its device, as one of 44.1 kHz
on a Dante network of 48 kHz, plays through cubic interpolation, which dulls
its highest frequencies and adds faint tones to them: a tone of 10 kHz loses
0.4 dB and gains others 26 dB below it.
A sound at the rate of its device plays unchanged.

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

A show whose house speakers play on the first two channels of a desk, its
subwoofer on the third, and its monitors on the fifth and sixth, and a storm
that plays in the house and the subwoofer, but not in the monitors:

```
[[output]]
name = "House"
device = "alsa:plughw:CARD=X32,DEV=0"
device_name = "X32, USB Audio"
channels = "1-2"
new_cues = 0.0

[[output]]
name = "Sub"
device = "alsa:plughw:CARD=X32,DEV=0"
device_name = "X32, USB Audio"
channels = "3"
new_cues = -6.0

[[output]]
name = "Monitors"
device = "alsa:plughw:CARD=X32,DEV=0"
device_name = "X32, USB Audio"
channels = "5-6"

[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Storm"
file = "sounds/storm.ogg"
level = -4.0

[list.cue.sends]
House = 0.0
Sub = -6.0
```

# SEE ALSO

*cinq*(1), *cinq*(5)

TOML
	https://toml.io/
; if xdg

XDG Base Directory Specification
	https://specifications.freedesktop.org/basedir-spec/latest/
; end
