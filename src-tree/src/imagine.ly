%% SPDX-FileCopyrightText: 2025 Yiyu Zhou <yiyu@yiyuzhou.io>
%%
%% SPDX-License-Identifier: 0BSD OR CC0-1.0

\version "2.25.27"

\header {
  title = "Imagine"
  composer = \markup \right-column {
    "Words and Music by"
    "JOHN WINSTON ONO LENNON"
    "and YOKO ONO LENNON"
  }
}

global = {
  \time 4/4
  \key c \major
  \tempo 4 = 76
}

melody = \relative c'' {
  \global
  \clef treble
  \fff
  \repeat unfold 2 { r1 | r }
  \repeat segno 2 {
    \repeat volta 2 {
      r8 g8 8 8 4 b8 8 | a4 r2. | r8 g8 8 8 8 b4 8 (|) a4 r2. |
      r4 g16 16 8~ 4 b8 8 | a4 r2. | r4 r8. g16 16 8. b8 8 | a4 r2. |
      r8 a8 c a c4 e8 e | d (a)~ a4 r2 | b8 8~ 4 4 c8 d~ | d2 e8 (g~ g e16 d |)
    }

    c4 r16 a c8 b (c16 b a8) 16 b | c8 8~ 4 r2 |
    r4 r16 a c8 b8 16 a~ 16 16 e8~ | 4 r2. | r8. a16 8 c b (c16 b a8) b |
    c8 16 (a g4) r2 | ^\markup { \italic { "rit. la 2" \super "a" "volta" } }
    r4. c16 16 d8 (e16 d c8) d |

    \alternative {
      \volta 1 { e16 c16 8~ 4 r2 }
      \volta 2 \volta #'() {
        \section
        \sectionLabel "Coda"
      }
    }
  }

  e16 c16 8~ 2\fermata r4 \fine
}

upper = {
  \global
  \clef bass
  \mf
  r8 c <e g> c <e g> c <e g b> c
  \repeat unfold 5 {
    <f a c'> c <f a c'> c <f a c'> c <f a>16 ais b8 |
    <e g> c <e g> c <e g> c <e g b> c
  }
  <f a> c <f a> c <f a> c <f a> c | <a c'> f <a c'> f <a c'> e <a c'> e |
  <f a c'> d <f a c'> d <f a c'> c <f a c'> c |
  <b d'> g <b d'> g <b d'> g <c' e'> g | <g d' f'>1 |
  \repeat unfold 3 {
    <a c'>8 f <a c'>8 f  | <g b> d <g b> d |
    <c' g> e <b g> e <b gis> e <d' gis> e
  }
  <a c'>8 f <a c'>8 f  | <b d'> g <b d'> g | <c' e g>1 \bar "||" <c' e g>1 \fine
}

lower = {
  \clef bass
  \mf
  _\markup { \italic "m.s. 8va bassa sempre, con Ped." }
  \repeat unfold 5 { c2. e4 | f2. f4 }
  c2. e4 | f1
  f4. 8 | e4. 8 | d4. 8 | c4. 8 | g2 2 | 1
  \repeat unfold 3 { e2 g4. 8 | c2 e }
  e2 g4. 8 | c c' d c' a, c' b, c' | c1 \fine
}

chordNames = \chordmode {
  \repeat unfold 6 { c2. c4:maj7 | f1 }
  f2 a:min/e | d:min7 f/c | g2. c4/g | g1:maj7
  \repeat unfold 3 { f2 g | c e:maj7 }
  f g | c1 | 1
}

firstVerse = \lyricmode {
  I -- ma -- gine there's no hea -- ven. | It's ea -- sy if you try. |
  No __ _ hell be -- low | us. | A -- bove us on -- ly sky. |
  I -- ma -- gine all the peo -- ple | li -- ving for to -- day | I __
}

secondVerse = \lyricmode {
  I -- ma -- gine there's no coun -- tries. | It is -- n't hard to do. |
  Nothing to kill or die | for, | and no reli -- _ gion too. |
  I -- ma -- gine all the peo -- ple | li -- ving life in peace | You __
}

thirdVerse = \lyricmode {
  I -- ma -- gine no pos -- ses -- sions. | I won -- der if you can. |
  No need for greed or hunger. | A bro -- ther hood of man. |
  I -- ma -- gine all the peo -- ple | shar -- ing all the world | You __
}

commonVerse = \lyricmode {
  You may say I'm a | dream -- er, | but I'm not the on -- ly one. |
  I hope some day you'll | join us, | and the world will | be as one. |
  live as one.
}

\score {
  <<
    \new ChordNames \chordNames
    \new Staff \context Voice = "vocal" \melody
    \new Lyrics \lyricsto "vocal" {
      <<
        \firstVerse
        \new Lyrics \lyricsto "vocal" \secondVerse
        \new Lyrics \lyricsto "vocal" \thirdVerse
      >>
      \commonVerse
    }
    \new PianoStaff <<
      \new Staff = "upper" \upper
      \new Staff = "lower" \lower
    >>
  >>
  \layout { }
  \midi { }
}