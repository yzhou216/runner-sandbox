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
The file lists the outputs of the show, the instruments of its light patch,
and its cue lists in order, each with its cues, and each cue with the
settings that its kind uses.
What belongs to a session rather than to the show stays out of the file: the
cue on standby, which is the first of each list once the show opens, the cues
that run, the levels of the lights, and how long each sound or video file
plays.

A file that cannot be opened, because it is not TOML or holds a mistake that
this page names, leaves the show on screen as it was, which is an empty one
when *cinq* has just started, and a warning in the sidebar names the file and
tells in one line where the mistake lies and what it is.
The cue table then says that the show could not be opened, in place of
inviting cues.

Saving leaves out the settings that the kind of a cue does not use, and those
that keep their defaults, apart from *level*, *duration*, and *stop_target*,
which it writes for every cue whose kind uses them.
Where the file is a symbolic link, saving writes the file that the link names,
and creates it where it is not there, and keeps the link.
A file can leave out any setting but the *name* of a list, the *name* and the
*channels* of an output, the *id* and the *address* of an instrument, and the
*id* and the *kind* of a cue.
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
	A dash joins two neighboring channels in rising order, and a comma any
	two different ones.
	Every output has them, and a file whose output lacks them cannot be opened:
	the warning names the output by its place among the *output* tables, as
	*output 2*, and gives the line of its table.
	A stereo file plays its left and right on the two channels of a stereo
	output, and the average of them on the channel of a mono one, so that a
	sound on one side alone is 6 dB quieter.
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
A device that takes no samples that *cinq* can play, as an ALSA card that takes
packed 24-bit samples alone does, is not tried again, and its warning says
why.
A device that has fewer channels than an output names is looked at again as
well, so that a Dante Virtual Soundcard whose channels are raised plays them,
ever more seldom while it offers no more, up to every thirty seconds.
A device that does not answer when it is opened, as an ASIO driver that waits
for a dialog does not, is shown as not answering after five seconds, and it
holds up no other device.
Changing an output opens again only the devices that the change touches, so
that the sound on the others plays on.
A cue that sends to outputs on two devices goes on playing on one of them while
the other changes: its output moves to another device, the device stops or
opens again, an output needs more of its channels, or the cue stops sending to
it or begins to.
Only the sound on the device that changed stops or starts, from where the cue
is.
Moving an output to another place in the list changes no device and leaves the
sound playing.
A device that stops while cues play on it, as one unplugged or a Dante Virtual
Soundcard that restarts, holds their clocks until it tells that it stopped, or
until it has asked for no sound for three seconds, after which they go on by
the clock of the show, and once the device opens again they play on there from
where they are.
A device that has just opened may take up to ten seconds for its first call for
sound, as a Bluetooth sink that sleeps does, and until then the clocks of the
cues on it wait at the start.
A file in which an output has no name, two outputs share a name, or the channels
of an output are neither one nor two, cannot be opened.

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
	by which the cues that act on other cues name their targets.

*number*
	Number of the cue, as the cue list shows it, which can be any text.

*kind*
	What the cue does: *audio*, *video*, *picture*, *light*, *network*,
	*fade*, *start*, *stop*, *pause*, *devamp*, *goto*, *group*, *wait*, or
	*memo*.
	An audio cue plays a sound file, a video cue plays a video file, and a
	picture cue shows a picture file, as *STAGE* describes.
	A light cue fades instruments of the light patch, as *LIGHTS*
	describes, and a network cue sends a message over the network, as
	*NETWORK* describes.
	A start cue starts another cue, a pause cue pauses it, a devamp cue lets
	it leave its loop, and a GoTo cue moves the standby cue of the list that
	holds its target to the target, as *CONTROL CUES* describes.
	A group cue fires the cues below it that it holds all at once, as
	*GROUPS* describes.
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
	A cue whose file cannot be played, as one whose file is not there,
	reads *BROKEN* in its state, the warnings in the sidebar tell why, and
	GO skips it.
	A file that no decoder of sound reads, as one of another format or
	one cut off within its first bytes, reads _not a sound file that cinq
	can play_.
	A WAV file of more than two channels reads _holds more than two channels_,
	since the decoder of *cinq* plays one or two.
	Every two seconds, *cinq* looks at the size and the modification time of
	each file of a cue that does not run, and reads again a file that has
	changed or gone, so that a file that goes warns before its cue fires, and
	replacing or restoring the file clears the warning.
	A failure that only playing finds, such as a picture cut short, stays
	until the file changes.
	A sound that stops while more than a second of its file remains, or a
	sound that loops and stops at all, while its audio device plays on,
	is such a failure, as a file cut short or one of more than two channels in
	a format other than WAV, which *cinq* cannot decode, makes it: its cue
	reads *BROKEN* with the reason _its sound stopped early_, and *cinq* says
	so once on standard error.
	A device that stalls, a fade, a stop, a panic, or a devamp never
	counts.

*start*
	Seconds into its file at which an audio or video cue starts, which are
	none by default.

*end*
	Seconds into its file at which an audio or video cue ends, which is the
	end of the file by default.
	A cue that starts at or after its end, or past the end of its file, cannot
	play.

*loop*
	Whether an audio or video cue plays from the start of its loop again each
	time it reaches the end of its loop, which it does not by default.
	A cue that loops plays until something stops it, such as a fade cue that
	stops it, a stop cue, or a panic, or until a devamp cue lets it go on, as
	*CONTROL CUES* describes.

*loop_start*, *loop_end*
	Seconds into its file at which the loop of an audio or video cue starts
	and ends, which are the start and the end of the cue by default.
	A cue that loops plays from its start to its loop end, and goes back to its
	loop start each time it reaches its loop end, so that one file can hold an
	introduction, a section that repeats, and an ending.
	The loop lies within the cue: a loop start before the *start* counts as the
	*start*, and a loop end after the *end* counts as the *end*, which the
	header of the inspector tells as the loop that plays.
	A cue whose loop then starts at or after its end, or lasts less than a
	millisecond, cannot play, and shows among the warnings.
	Saving writes both for a cue that sets them, whether it loops or not, so
	that turning the loop off and on keeps them.

*target*
	Identity of the cue that a fade, start, stop, pause, devamp, or GoTo cue
	acts on.
	A fade cue acts on an audio, video, or picture cue, a devamp cue on an
	audio or video cue, and each of the others on any cue but itself, in any
	list of the show.
	A stop, pause, start, or GoTo cue may name a group cue, as *GROUPS*
	describes, while a fade or devamp cue may not.

*depth*
	How many groups deep the cue lies, which is 0 by default, for a cue that
	lies in no group.
	A cue lies in the nearest group cue above it that is one level less deep,
	so that a group holds the cues below it, up to the first that lies no
	deeper than the group, and groups nest.
	A depth counts only as far as the cues above allow: no deeper than the cue
	above, or one level deeper where that cue is a group.
	A depth that goes further, as one left by a group that is no longer there,
	counts as the deepest that the cues above allow, so that no depth breaks
	the structure.
	Saving writes it for a cue that is not 0.

*collapsed*
	Whether a group cue is collapsed, which is false by default, so that the
	cue list shows it as one row and hides the cues that it holds, as a closed
	fold hides lines in Vim.
	A collapsed group fires, stops, pauses, and runs its cues exactly as an
	expanded one does, and collapsing is no change that undo takes back.
	Saving writes it for a collapsed group only, and *cinq* ignores it on a
	cue that is no group.

*pre_wait*
	Seconds between the cue firing and its action, which are none by
	default.
	A time typed in the inspector, here and in the other settings of seconds,
	reads as seconds, as _90_ or _1.5_, as minutes and seconds, as _1:30_, or as
	hours, minutes, and seconds.
	Text that reads as no time, as _5s_ does, counts as zero, and the header of
	the inspector says so in red.
	For *end*, *loop_start*, and *loop_end* it is empty instead, which leaves
	the end at the end of the file and the loop at the bounds of the cue, and the
	header says that.
	A setting that the cue ignores, as the loop times of a cue that does not
	loop, or the *post_wait* of a cue that follows automatically or lies in a
	group, never brings the sentence.

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
	Changing the *level* of an audio or video cue while it plays moves the
	level that it plays at by the change, rather than to the new level, so
	that a cue that a fade cue has lowered stays lower.
	The level that it plays at goes no higher than 12.
	Below -60 the cue is silent, but the level goes on counting, so that
	moving the *level* down to -60 and back restores the level of the fade.
	A fade under way that leaves the cue playing goes on from the moved level
	to its own, while a fade that stops the cue, from a fade cue or a panic,
	leaves it fading out as before.
	Changing the *level* of a fade cue while it fades sends the fade to the new
	level in the time it has left, and a fade cue that has yet to act, in its
	pre-wait or its place in a chain, fades to the new level when it does.
	The *duration*, *target*, and *stop_target* of a fade cue take effect the
	next time that it runs.

*stop_target*
	Whether a fade cue stops its target once the fade is over, which it does
	by default.
	A picture has no level, so a fade cue that targets a picture and does
	not stop it changes nothing, as the Levels tab of the inspector says.

*sends*
	Table of the levels in dB at which an audio or video cue sends to
	outputs, each entry named after an output, between -60 and 12.
	The cue plays on those outputs alone, after its *level*, which fade cues
	move.
	A cue without one sends to the first output at 0 dB, and one with an empty
	table sends to no output.

*notes*
	Notes of the cue, for the operator.

*lights*
	Table of the levels that a light cue fades instruments to, in percent
	from 0 to 100, by the *id* of each instrument.

*destination*
	Where a network cue sends its message, as *NETWORK* describes.

*message*
	Message that a network cue sends, as *NETWORK* describes.

A file in which two cues or two instruments share an identity, a level or an
address lies outside its range, or a cue sends to an output that the show
does not have, cannot be opened.

# STAGE

A video cue plays H.264 video of the Baseline, Main, or High profile in MP4, as
files ending in _.mp4_, _.m4v_, or _.mov_ hold it, along with the sound of its
AAC track, in mono or stereo.
A picture that the header of its track turns by a quarter, half, or three
quarters of a turn, as a phone that films on its side writes it, shows upright.
A file without a sound track plays its picture alone.
The colors of a picture follow what its file declares: the colr box of the
track, else the parameter sets of the H.264 stream, give the matrix, as BT.601
or BT.709, and whether the range is the full range of a byte.
A file that declares no matrix gets BT.709 where the picture is at least 1280
pixels wide or 720 lines high and BT.601 where it is smaller, and one that
declares no range has the limited range.
A file that holds no index of its tracks, as a copy that stopped before its
end does, or a file that is not MP4, reads _is cut short, or is not an MP4
file_, and one that holds an index but no H.264 track reads _holds no H.264
video_.
One in another profile reads _holds H.264 video of a profile other than
Baseline, Main, or High_.
A track whose parameter sets may lie in its samples, as one of the kind _avc3_
allows, plays as one whose description holds them.
A file whose pictures are larger than any level of H.264 allows, as a damaged
file can declare, reads _holds H.264 video larger than any level of H.264
allows_, as it loads or, where only its samples declare the size, as it plays.
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
A picture of more than 134 million pixels, as a damaged file can declare, reads
_holds a picture too large to show_.
It shows until something stops it, such as a fade cue that stops it, a stop
cue, or a panic, so that the *follow* advance never fires the cue after it.

The pictures of picture and video cues show in a stage window of their own,
which opens beside the main window once the show holds a picture or video
cue, as a window that the operator can move to the display of the audience.
It shows black while no picture shows, and otherwise the picture of the cue
that fired last, whole and as large as the window allows.
Each GO that fires a picture or video cue, itself or through a start cue,
makes the stage window fullscreen, on the display that it is on, and brings it
before the other windows, so that the picture fills the screen as it comes up.
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
or the key of *fullscreen* is typed, which opens it anew, fullscreen,
wherever the platform puts it.
While the stage window is closed and a picture or video cue shows on it, the
main window says so, with a warning in the sidebar and a hint of the key of
*fullscreen*, since the audience sees nothing of what the cue shows.
Closing the main window quits *cinq*.
The key of *fullscreen* otherwise toggles the stage window between fullscreen
and a window, until the next GO that fires a picture or video cue, and the
other keys act on the cue list from either window, as *cinq*(5) describes.

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
	Universes count from 1, as in sACN.
	Art-Net counts from 0, so it carries universe 1 as its Port-Address 0,
	and each universe after it as the next Port-Address, as most consoles
	do.

*address*
	DMX address of the instrument in its universe, from 1 to 512.

The inspector edits the patch on its *Light Patch* tab.
There an address reads as the DMX address alone in universe 1, and as the
universe, a slash, and the DMX address in the others, as in _2/12_.
A new instrument takes the address after that of the last, and none is added
after address 512 of universe 32768.

A light cue fades the instruments that it names from their levels at its
start to its own levels, over its *duration*.
It leaves the other instruments as they are, so that each instrument keeps
the level of the last light cue that named it, as in QLab.
A light cue takes the instruments that it names over from an older one that
is still fading them.
Pausing a light cue pauses its fade, and stopping it, as a panic does,
leaves its instruments where the fade has brought them.
Each time that the show opens, every instrument starts at level 0 until a
light cue sets it.
Removing an instrument from the patch leaves its levels in the cues that name
it, so that *undo* brings them back.
A light cue that sets no instrument, or names one that the patch lacks, still
fires, since its other instruments and its time count, but its *State* reads
*NO LEVELS* or *UNPATCHED* in yellow, the header of its inspector says why in
yellow, and the sidebar lists it among the warnings, as in _names instrument
99, which the patch lacks_, until the cue or the patch is put right.  The
Levels tab lists such an instrument as _99 (unpatched)_, where emptying its
field clears it.

*cinq* sends the levels of each universe that the patch uses in two ways.
Art-Net goes to the broadcast address of every IPv4 network that the computer
is on, and sACN to the multicast group of the universe on each of those
networks.
Both go to the computer itself as well.
*cinq* sends them as soon as the levels change, and again every second.
Every ten seconds it lists the universes that it sends, as the universe
discovery of sACN asks, so that monitors and receivers find them.
Its sACN has priority 100, the default of sACN.
A receiver that merges sources of one priority therefore takes the higher
level of each address from *cinq* and any other source, such as a lighting
console.
Instruments that share an address set it to the highest of their levels.
A universe that leaves the patch, by an edit or by the opening of another
show, goes to level 0 on every address, and then its sACN stream says that it
has ended, so that no receiver keeps its last levels.
When *cinq* ends, receivers of sACN hold its levels for as long as they
hold those of any source that has gone quiet.
Light output that cannot go out is told while the patch has an instrument:
where the computer has no IPv4 network, the socket cannot be opened, or every
send of a round fails, the sidebar lists a warning, such as _Light output: no
network interface_ or _Light output: not sent: network unreachable_, and
counts it, and standard error says it once.
The warning goes once a round of sending gets a packet out.
A socket that cannot be opened is tried again every two seconds, and the warning
goes once it opens.

# NETWORK

A network cue sends an Open Sound Control (OSC) message once its pre-wait is
over, as the network cues of QLab do, so that a GO in *cinq* can fire a cue
of a lighting console or of another program.
It takes no time, and does nothing else.

The *destination* of a network cue is _host:port_, or _tcp://host:port_, for
TCP, where *cinq* frames each packet with SLIP, as OSC 1.1 asks of streams, or
_udp://host:port_ for UDP, with one packet in each datagram.
The host is an IPv4 address, or a name that *cinq* reaches at its first IPv4
address.
It holds no colon, bracket, or slash, so that an IPv6 address and a URL read as
no destination.
A host whose last part is a number or begins with _0x_ must be an IPv4 address
of four decimal numbers without a leading zero, so that _010.0.0.1_,
_0x7f.0.0.1_, and _127.1_ read as no destination, where the system would take
them for other addresses.
A port that is not a number from 1 to 65535 reads *BAD PORT*.
An address avoids waiting for a name to resolve, which holds up the messages
behind it for as long as the name server takes.
A UDP destination that holds a name is resolved once for each run of messages
that follow each other within a second, as a TCP connection is kept.
*cinq* keeps a TCP connection to a destination open while messages to it
follow each other within a second, and makes a new one for the first message
after that, so that a message never waits in a connection whose cable is
out, to arrive late once it is back in.

The *message* of a network cue holds its address, which starts with a slash,
followed by its arguments after spaces, each an integer, a decimal number, or
else a string.
Double quotes keep a string whole across spaces, and keep it a string where
it reads as a number.
A string cannot hold a double quote itself, and a message with a double quote
left open, as in _/scene "Act 1_, reads as no message, and the warning says that
the quote is open.

A network cue whose destination or message does not read as one cannot play.
A message that cannot go out within a second of its cue starting is lost,
since a cue that fires late can do more harm than one that does not fire.
This happens when the destination refuses the connection, or does not take it
in time, as a console that has lost its power or its cable does.
It happens as well when the message waits behind others.
A UDP message fails only where the host cannot be found in time or the system
cannot send it, since nothing answers it.
Where *cinq* ends with messages on their way, it waits for them, for a second at
most, so that a cue fired just before the window closes still sends.
The sidebar warns of a lost message for the first cue that names the
destination, as _3 Send: message not sent: connection refused_, and pressing
the warning selects the cue, whose inspector shows the destination.
The *State* of every network cue that names the destination reads *SEND
FAILED* in yellow, as that of a cue that sends to a failed output reads
*OUTPUT FAILED*, also while the cue is only next, so that the cue that was
fired shows it whichever cue the sidebar names.
The warning says why until a message to the same destination gets there, or
until no cue names the destination any more, and *cinq* says so on standard
error as well, once for each reason, with the destination and the error of
the system.

Consoles and programs tell in their manuals which messages they take, and on
which ports.
For example, the Eos consoles of ETC take OSC over TCP with SLIP on port
3037, and fire cue 5 of cue list 1 for _/eos/cue/1/5/fire_.

# CONTROL CUES

A start, pause, devamp, or GoTo cue acts on its *target* once its pre-wait is
over, as the transport, devamp, and GoTo cues of QLab do.
It takes no time, as a stop cue does, and its target can be any cue but itself,
in its own list or in another, except that a devamp cue needs an audio or
video cue.
A cue of these kinds that names no target cannot play, and shows among the
warnings.

A start cue starts its target as GO starts the standby cue.
It fires the target together with every cue that auto-continue or auto-follow
chains to it in the list of the target, passing over those that cannot play,
and restarts any of them that already runs.
Where the target is paused, the start cue resumes it instead, together with
the cues that wait for it to start them, and fires nothing else.
It leaves the standby cue of every list where it is, and puts nothing in the
history that undo takes back.
The cues that a start cue fires act on other cues, the lights, and the network
from the next tick of the clock, a fortieth of a second later, while their sound
and pictures come at once, so that start cues that start each other without
waits take turns rather than hold up cinq, until something stops them, such as
a panic.

A pause cue pauses its target, whatever the target is doing: waiting in a
chain, waiting out its pre-wait, playing, or looping.
It pauses as well the cues that wait for the target to start them, so that the
chain holds where it is.
A target that does not run, or no longer does, is left alone, even where cues
that it fired still wait out its post-wait.
Pausing holds the fades of a cue, as *Pause all* does.
A pause cue whose target is a fade cue holds the fade that the cue is making at
the level it has reached, and the fade goes on from there when the fade cue
resumes.
A start cue that targets the paused cue, or *Resume all*, resumes it.

A devamp cue lets its target leave its loop, as the devamp cue of QLab does.
Its target is an audio or video cue that loops, in any list.
A target that does not loop cannot be devamped, so the cue shows among the
warnings, and GO skips it.
The run of the target then plays on to the end of a pass through its loop,
goes from the loop end on to the end of its cue, and ends there, as a cue that
does not loop ends.
Where the loop ends with the cue, the run ends at the end of that pass.
The pass that it leaves at is the first to end more than half a second after
the devamp cue acts, since the sound needs that long to leave its loop at the
end of a pass, neither a pass early nor a pass late.
For a sound of a sample rate below 46.8 kHz that time is longer: the 16384
frames that its decoder reads ahead of it, which are 0.37 seconds at 44.1 kHz,
and 0.15 seconds more.
A loop shorter than that time can still go round once more.
A target that does not run is left alone, and a run that a devamp cue has let
go on already keeps the pass that it leaves at.
A target that is paused leaves at a pass end chosen from where it is paused,
once it resumes and reaches it.
A target that still waits in a chain or out its pre-wait plays through its loop
once and goes on.
Fades, stop cues, pause cues, and panics act on a run that has been let go on
as on any other, and GO or a start cue that fires the target again starts a
new run, which loops again.
A cue that loops fires no cue by auto-follow, whether a devamp cue has let it
go on or not.

A GoTo cue moves the standby cue of the list that holds its target to the
target, so that the next GO of that list fires it, and fires nothing itself.
The move is made as GO makes its own: undo does not take it back, and the
moves of standby that undo could have taken back before it in that list are
dropped.
A GoTo cue in the chain that a GO fires sets the standby cue that the GO
leaves, since it acts after the GO has moved standby past the chain.

# GROUPS

A group cue fires the cues that it holds all at once, as the timeline group of
QLab does, so that one GO starts a whole scene.
It does nothing else itself, and it takes no *target*.
Each cue below it that lies one level deeper, as *depth* describes, belongs to
the group, along with the cues that those hold in turn.
A group that holds no cue does nothing.

GO on a group fires it together with every cue that it holds, at any depth.
Each of those begins its action once its own pre-wait is over, plus the
pre-waits of the groups that hold it, counted from the GO.
Standby moves past the whole group, as it moves past any chain that GO fires.
GO with a cue inside a group on standby fires that cue alone, together with
the cues that it holds where it is a group, and standby moves to the next cue
below it, whether that is in the group or not.

The *advance* and *post_wait* of a cue inside a group have no effect, since the
group fires the cues beside it, and the inspector shows both settings disabled.
The group goes on by its own *advance*, to the cue below everything that it
holds:

- *continue* fires that cue once the pre-wait and the post-wait of the group
  are over.
- *follow* fires it once the action of the group ends, which is when the
  last of the actions of the cues that it holds ends, each counted from the end
  of the pre-wait of the group.
  A group never ends its action where a cue in it never ends, as a cue that
  loops, or a picture cue, never does, so that it fires nothing by *follow*.

The run of a group begins its action once its pre-wait is over, and lasts while
any cue that the group holds runs.
It ends once its action has begun and none of those runs, which happens when
they all end or all stop, and its end stops nothing.
The cue list and the sidebar show it as they show any run: the time that is
left, while the group has an end, and _running_ while a cue in it never ends.

The cues that act on other cues act on a group as follows:

- A stop cue stops the run of the group and the run of every cue that it
  holds, each as a stopped cue, so that the cues that wait for them are stopped
  as well.
  The *Stop* button of the sidebar does the same.
- A pause cue pauses the run of the group and the runs of the cues that it
  holds, along with the runs that each would start.
- A start cue fires the group as GO does, together with every cue that it
  holds, or resumes the group and those cues where it is paused.
  A start cue inside the group that it starts cannot play, since the group
  would fire it again without end, so that GO skips it and the cue list shows
  IN TARGET.
- A fade or devamp cue cannot name a group, as in QLab, and reads BAD TARGET,
  since a fade needs a sound, video, or picture, and a devamp cue an audio or
  video cue.
  The warning of a devamp cue says that its target is not audio or video.

*Panic* and *Pause all* act on every run, those of groups and of the cues in
them alike.

A group may be collapsed, as the *collapsed* key tells, which only changes how
the cue list shows it: as one row, with the cues that it holds hidden at any
depth, as Vim shows a closed fold as one line.
It fires, stops, pauses, and runs its cues as an expanded group does, and
standby may stay on a cue that it hides.
The row of a collapsed group counts as one row wherever the cue list counts
rows, and editing acts on it as on a closed fold, on the whole group:

- A cue added or pasted below a collapsed group goes below the cues that the
  group holds, at the depth of the group, so that it lies outside the group.
  A stop, pause, start, or GoTo cue added there targets the group, the row
  above it.
- Moving a cue or a group by a row moves it past a collapsed group and the
  cues that it holds, to below them at the depth outside the group, and never
  into it.
- Indenting the cue just below a collapsed group moves it into the group as
  its last cue, which expands the group.

Editing keeps a group together with the cues that it holds:

- A new cue below a cue takes the deepest depth that cue allows, which is its
  own, or one more where it is a group.
  A cue added below a group therefore goes into it, as its first cue, and one
  added below a cue in a group joins that group.
  A cue added above a cue takes the depth of that cue.
  A new cue that acts on another cue targets the cue above it where it can
  act on that cue, but never a group that holds it, so that a stop, pause,
  start, or GoTo cue added below a group goes into the group with no target.
- Pasted cues keep their shape, with all their depths moved by the same
  amount, so that the first of them takes the depth that a new cue would take
  there.
- Moving a group up or down moves it as one block, together with the cues that
  it holds, and the depths of the block move so that its first cue takes the
  depth that a new cue would take at its new place.
  Moving a cue by one row onto a group therefore takes it into the group or out
  of it.
  Undo takes the move back in one step.
- Deleting, cutting, and copying a group takes it together with every cue that
  it holds, and undo brings them all back where they were.
- Indenting a cue moves it into the group above it, together with the cues that
  it holds.
  Outdenting a cue moves it out of its group to just below the group, together
  with the cues that it holds, so that the cues of the group that lay below it
  stay in the group.
  *cinq*(5) lists the keys.

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
does as _server\\share_, so that files of the same path on another drive
or share keep their changes apart.
; end

Closing the window, opening another show, or a crash leaves the kept changes
in place.
Closing the window first waits for the changes to be written, for five seconds at
most, so that a disk that hangs cannot keep *cinq* running.
The next time that the show opens, they come back as one change, which undo
takes back to the show as saved.
Saving the show, or taking back each of its changes, removes them.
Saving the show over another file removes the changes kept for that file as
well, so that they do not come back as changes to what the save wrote.

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

A patch of two instruments, and a list of two light cues.
The first brings the front light up at once.
The second fades the front light down to 40 percent, and brings the back
light up to 80 percent, over five seconds:

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

A list in which one GO fires cue 5 of a lighting console, at the address of
the console on its network, and plays a sound along with it:

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

A list in which music loops until the second GO pauses it, the third GO resumes
it where it paused, and the fourth fades it out and stops it:

```
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Vamp"
file = "sounds/vamp.ogg"
loop = true

[[list.cue]]
id = 2
number = "2"
kind = "pause"
name = "Hold vamp"
target = 1

[[list.cue]]
id = 3
number = "3"
kind = "start"
name = "Resume vamp"
target = 1

[[list.cue]]
id = 4
number = "4"
kind = "fade"
name = "Vamp out"
target = 1
duration = 4.0
level = -60.0
stop_target = true
```

A list in which a file of music holds an introduction up to four seconds, a
vamp from four to twelve seconds, and an ending after that.
The first GO plays the introduction and then loops the vamp, and the second GO
lets the music leave the vamp at the end of a pass and play its ending:

```
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Scene music"
file = "sounds/scene.ogg"
loop = true
loop_start = 4.0
loop_end = 12.0

[[list.cue]]
id = 2
number = "2"
kind = "devamp"
name = "Leave vamp"
target = 1
```

A list in which one GO plays a storm and, two seconds later, a thunder crack,
which a group holds at one level deeper than itself, and the group goes on to
the applause once both sounds have ended:

```
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "group"
name = "Storm scene"
advance = "follow"

[[list.cue]]
id = 2
number = "1.1"
kind = "audio"
name = "Storm"
file = "sounds/storm.ogg"
depth = 1

[[list.cue]]
id = 3
number = "1.2"
kind = "audio"
name = "Thunder"
file = "sounds/thunder.wav"
pre_wait = 2.0
depth = 1

[[list.cue]]
id = 4
number = "2"
kind = "audio"
name = "Applause"
file = "sounds/applause.ogg"
```

# SEE ALSO

*cinq*(1), *cinq*(5)

TOML
	https://toml.io/
; if xdg

XDG Base Directory Specification
	https://specifications.freedesktop.org/basedir-spec/latest/
; end
