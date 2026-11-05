%% SPDX-FileCopyrightText: 2025 Yiyu Zhou <yiyu@yiyuzhou.io>
%%
%% SPDX-License-Identifier: 0BSD OR CC0-1.0

\version "2.25.27"

\header {
  title = "Happy Birthday To You"
  composer = "PATTY S. and MILDRED J. HILL"
}

global = {
  \time 3/4
  \key g \major
  \partial 4
  \tempo 4 = 122
}

melody = \relative c' {
  \global
  \clef treble
  d8 8 | e4 d g | fis2 d8 8 | e4 d a' | g2 d8 8 | d'4 b g |
  fis e\fermata c'8 8 | b4 g a | g2 \fine
}

harmony = \chordmode {
  \global
  r4 | g2. | d:maj7 | r | g | r | c | g2 d4:maj7 | g
}

text = \lyricmode {
  Hap -- py | birth -- day to | you.  Hap -- py | birth -- day to |
  you.  Hap -- py | birth -- day dear | - -.  Hap -- py | birth -- day to | you.
}

\score {
  <<
    \new ChordNames \transpose g f \harmony
    \new Staff \context Voice = "vocal" \transpose g f \melody
    \new Lyrics \lyricsto "vocal" \text
  >>
  \layout { }
  \midi { }
}
