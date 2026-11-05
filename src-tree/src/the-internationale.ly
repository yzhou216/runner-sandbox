%% SPDX-FileCopyrightText: 2025 Yiyu Zhou <yiyu@yiyuzhou.io>
%%
%% SPDX-License-Identifier: 0BSD OR CC0-1.0

\version "2.25.27"

\header {
  title = "L'Internationale"
  composer = \markup \right-column {
    "Paroles d'Eugène Pottier"
    "Musique de Pierre De Geyter"
  }
}

global = {
  \time 4/4
  \key a \major
  \tempo "Alla marcia" 4 = 120
  \partial 8
}

melody = \relative c' {
  \global
  \clef treble
  e8\p | a4. gis8 b a e cis | fis2 d4.( fis8) | b4. a8 gis fis e d | \break
  cis2. r8 e | a4. gis8 b a e cis | fis2 d4. a'16( 16) |
  gis4 8 b d4 gis, | a2. cis8( b8) | \break
  gis4. 8 fis gis a fis | gis2 e4. 16 16 | fis4 8 8 b4 a | gis2. r8 b | \break
  b4. gis8 e8 8 dis e | cis'2 e,8 fis gis a |
  gis4 b a4 fis | e2 cis'4. b8 | \break
  a2 e4 fis | fis2 d4 b'8.( a16) | gis4 4 fis e | 2. 4 | \break
  cis'4. 8 b4 e, | a2 gis4.( 8) | fis4. eis8 fis4 b | b2 cis4. b8 | \break
  a2 e4 fis | fis2 d4 b'8.( a16) | gis4 4 fis4. e8 | cis'2. 4 | \break
  e4. 8 d4 cis | b cis d4. 8 | cis4. 8 b4. a8 | a2. \bar "|."
}

upper = \relative c' {
  \global
  \clef treble
  e8 |
  <<
    {
      \voiceOne
      a4. gis8 b a e cis | fis2 d4. fis8 | b4. a8 gis fis e d | cis2 fis4. e8 |
      a4. gis8 b a e cis | fis2 d4. a'16 16 | gis4 b d gis, | a2. cis8 b |
      gis4. 8 fis gis a fis | gis2 e4. 8 | fis4. 8 b4 a | <e gis>2 b'4. 8 |
      b4. gis8 e e dis e | cis'2 a8 fis gis a | gis4 b a fis | e2 cis'4. b8 |
      a2 e4 fis | fis2 d4 b'8. a16 | gis4 4 fis e | 2. 4 |
      cis'4. 8 b4 e, | a2 gis4. 8 | fis4. eis8 fis4 b | b2 cis4. b8 |
      a2 e4 fis | fis2 d4 b'8. a16 | gis4 4 fis4. e8 | cis'2. 4 |
      e4. 8 d4 cis | b cis d4. 8 | cis4. 8 b4. a8 | a2.
    }

    \new Voice {
      \voiceTwo
      \absolute {
        cis'2 e'4 r | <a d'>4 <ais cis'> a2 | d'2 d'4 r | a2 <a d'>4 <gis d'> |
        cis'2 e'4 r | <a d'>4 <ais cis'> a2 | d'2 2 |
        <cis' e'>4 <b d'> <a cis'>2 | <b e'>2 e'4 dis' | <b e'>4 <a dis'> gis2 |
        e'2 dis' | b2 e'4 dis' |
        <e' gis' b'>2 <dis' fis' ais'>4 <e' gis' b'> |
        <a cis' e' a'>2 <e' gis' b'>4 <e' gis' b'> |
        <e' gis' b'>4 <dis' fis' a'> <d' fis' a'>2 |
        <cis' e' a'>2 <b d' gis'>4 <a cis' e'> |
        <cis' e' a'>2 <b d' gis'>4 <a cis' e'> |
        <d' fis' a'>2 <b d' fis'> |
        <e' gis' b'>2 <e' gis' b'>4 <d' fis' a'> |
        <cis' e' a'>2 <b d' gis'>4 <a cis' e'> |
        <cis' e' a'>2 <b d' gis'>4 <a cis' e'> |
        <cis' e' a'>2 <b d' gis'>4 <cis' e' a'> |
        <d' fis' a'>2 <cis' eis' gis'>4 <d' fis' a'> |
        <e' gis' b'>2 <dis' fis' ais'>4 <e' gis' b'> |
        <cis' e' a'>2 <b d' gis'>4 <a cis' e'> |
        <d' fis' a'>2 <b d' fis'> |
        <e' gis' b'>2 <e' gis' b'>4 <d' fis' a'> |
        <a cis' e' a'>2 <b d' gis'>4 <a cis' e' a'> |
        <a cis' e' a'>2 <b d' gis'>4 <a cis' e' a'> |
        <e' gis' b'>4 <d' fis' a'> <cis' e' a'>4. <b d' gis'>8 |
        <a cis' e' a'>2 <b d' gis'>4 <a cis' fis'> | <a cis' e'>2.
      }
    }
  >>
}

lower = {
  \global
  \clef bass
  r8 |
  <a, e>2 <cis a> | d4 cis b, a, | <e, gis>2 <gis, e> | <a, e>2 b,4 e, |
  <a, e>2 <cis a> | d4 cis b, a, | <gis, e>2 <e, gis> | <a, e>1 |
  << { r2 a2 } \\ { e,2 b,4 b,, } >> | e,4 fis, gis,2 |
  << { a1 } \\ { b,2 b,, } >> | << { r2 a2 } \\ { e4 dis cis b, } >> |
  <e, gis>4 <dis, dis> <d, d>2 | <cis, cis>4 <b,, b,> <a,, a,>2 |
  <e, b,>2 <b, a> | <e, b,>2 <e, gis> |
  a,4 b, cis2 | d4 cis b, <a, fis> | <gis, e>2 <e, d> |
  <a, e>4 gis, <fis, d> e, |
  <a, e>2 <gis, e> | <fis, e>2 <f, cis> |
  <d, a,>2 <dis, b,> | <e, a>2 <e, gis> |
  a,4 b, cis2 | d4 cis b, <a, fis> | <gis, e>2 <e, d> |
  << { e1 } \\ { <a, a>2 <g, g> } >> |
  <fis, fis>2 <b,, b,>4 <cis, cis> | <d, d>4 <cis, cis> <b,, b,> <f, f> |
  e4 e, <e, a> gis | << { <a, e a>2. } \\ { r2 a,,4 } >> \bar "|."
}

chordNames = \chordmode {
  s8 |
  a1 | d | e:7 | a2 e:7 |
  a1 | d | e:7 | a | e2 b:7 | e1 | b1:7 |
  \repeat unfold 2 { e2 b:7 | e2 e:7 | a1 } |
  d | e:7 | a2 e:7 | a2 e:7 | fis:m cis:7 | d b:7 | e e:7 |
  a1 | d | e:7 | a2 a:7 | fis1:7 | b2:m7 e:7 | a e:7 | a1 |
}

text = \lyricmode {
  De -- bout les dam -- nés de la ter -- re,
  De -- bout les for -- çats de la faim!
  La rai -- son tonne en son cra -- tè -- re,
  C'est l'é -- rup -- tion de la fin.

  Du pas -- sé fai -- sons ta -- ble ra -- se,
  Fou -- le es -- cla -- ve, de -- bout, de -- bout!
  Le mon -- de va chan -- ger de ba -- se:
  Nous ne som -- mes rien, so -- yons tout!

  C'est la lut -- te fi -- na -- le,
  Grou -- pons-nous, et de -- main,
  L'In -- ter -- na -- tio -- na -- le
  Se -- ra le gen -- re hu -- main.

  C'est la lut -- te fi -- na -- le,
  Grou -- pons-nous, et de -- main,
  L'In -- ter -- na -- tio -- na -- le
  Se -- ra le gen -- re hu -- main.
}

\score {
  <<
    \new ChordNames \chordNames
    \new Staff \context Voice = "vocal" \melody
    \new Lyrics \lyricsto "vocal" \text
    \new PianoStaff <<
      \new Staff = "left" \upper
      \new Staff = "lower" \lower
    >>
  >>
  \layout { }
  \midi { }
}
