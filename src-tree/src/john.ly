%% SPDX-FileCopyrightText: 2025 Yiyu Zhou <yiyu@yiyuzhou.io>
%%
%% SPDX-License-Identifier: 0BSD OR CC0-1.0

\version "2.25.27"

\header {
  title = "John"
  composer = \markup \right-column {
    "MELODY BY JOHN LENNON"
    "WORDS BY YIYU ZHOU"
  }
}

global = {
  \time 4/4
  \key a \major
  \partial 4
  \tempo 4 = 110
}

melody = \relative c' {
  \global
  \clef treble
  e8 fis | b4 a b cis | e8 cis4. r4 b8 cis | b4. (cis16 b a4.) f8 |
  e cis'4 cis8~ 4 e,8 fis | b4 a b cis | e8 cis4. r4 b8 cis |
  b4. (cis16 b a4.) f8 | e4 cis'8 8~ 4 r4
}

text = \lyricmode {
  There's a | world that he i -- | ma -- gined.  All the | people, just |
  living in peace.  When he | left the world that eve -- ning, |
  part of me had died with him.
}

\score {
  <<
    \new Staff \context Voice = "vocal" \melody
    \new Lyrics \lyricsto "vocal" \text
  >>
  \layout { }
  \midi { }
}
