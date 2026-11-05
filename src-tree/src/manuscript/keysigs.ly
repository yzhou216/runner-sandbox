%% SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
%%
%% SPDX-License-Identifier: 0BSD OR CC0-1.0

\version "2.25.27"

%% One \book per key, so each signature lands in its own cropped PDF for
%% manuscript.tex to place around the ring.

\paper {
  indent = 0
  page-breaking = #ly:one-line-breaking
  print-page-number = ##f
  print-first-page-number = ##f
}

#(set-global-staff-size 14)

keysig =
#(define-music-function (key) (ly:music?)
   #{
     \new Staff \with {
       \remove "Time_signature_engraver"
     } {
       %% The staff symbol reaches only as far as the last grob that has a
       %% width, so without a terminating bar line the later accidentals hang
       %% off the end of it.  Packing the spacing keeps the tail short.
       \override Score.SpacingSpanner.packed-spacing = ##t
       \override Score.SpacingSpanner.spacing-increment = #0
       \clef treble
       $key
       s4
       \bar ""
     }
   #})

\book { \bookOutputSuffix "g-major"       \score { \keysig { \key g   \major } } }
\book { \bookOutputSuffix "d-major"       \score { \keysig { \key d   \major } } }
\book { \bookOutputSuffix "a-major"       \score { \keysig { \key a   \major } } }
\book { \bookOutputSuffix "e-major"       \score { \keysig { \key e   \major } } }
\book { \bookOutputSuffix "b-major"       \score { \keysig { \key b   \major } } }
\book { \bookOutputSuffix "f-sharp-major" \score { \keysig { \key fis \major } } }
\book { \bookOutputSuffix "c-sharp-major" \score { \keysig { \key cis \major } } }

\book { \bookOutputSuffix "f-major"       \score { \keysig { \key f   \major } } }
\book { \bookOutputSuffix "b-flat-major"  \score { \keysig { \key bes \major } } }
\book { \bookOutputSuffix "e-flat-major"  \score { \keysig { \key ees \major } } }
\book { \bookOutputSuffix "a-flat-major"  \score { \keysig { \key aes \major } } }
\book { \bookOutputSuffix "d-flat-major"  \score { \keysig { \key des \major } } }
\book { \bookOutputSuffix "g-flat-major"  \score { \keysig { \key ges \major } } }
\book { \bookOutputSuffix "c-flat-major"  \score { \keysig { \key ces \major } } }
