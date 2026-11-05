# SPDX-FileCopyrightText: 2025-2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

SRC_DIR := src
DIST_DIR := dist
BUILD_DIR := build
NIX_OUT_DIR := result

SOURCE_FILES := $(wildcard $(SRC_DIR)/*.ly)
PDF_FILES := $(patsubst $(SRC_DIR)/%.ly,$(DIST_DIR)/%.pdf,$(SOURCE_FILES))
MIDI_FILES := $(patsubst $(SRC_DIR)/%.ly,$(DIST_DIR)/%.midi,$(SOURCE_FILES))

# LaTeX page with LilyPond-engraved key signatures, so not a plain .ly build
PAPER := manuscript
PAPER_SRC := $(SRC_DIR)/$(PAPER)
PAPER_BUILD := $(BUILD_DIR)/$(PAPER)
PAPER_PDF := $(DIST_DIR)/$(PAPER).pdf

all: $(PDF_FILES) $(MIDI_FILES) $(PAPER_PDF)

$(DIST_DIR)/%.pdf: $(SRC_DIR)/%.ly | $(DIST_DIR)
	lilypond --loglevel=ERROR --output=$(DIST_DIR)/$* $<

# Optional MIDI files depends on PDFs to avoid running LilyPond twice
$(DIST_DIR)/%.midi: $(DIST_DIR)/%.pdf
	@:

# A single run writes all fourteen signatures, so a stamp stands in for them
$(PAPER_BUILD)/keysigs.stamp: $(PAPER_SRC)/keysigs.ly | $(PAPER_BUILD)
	lilypond -dcrop --loglevel=ERROR --output=$(PAPER_BUILD)/keysigs $<
	touch $@

# Batch mode keeps the transcript off the terminal; show it if a pass fails
LATEX = pdflatex -interaction=batchmode -halt-on-error \
	-output-directory=$(PAPER_BUILD)
SHOW_LOG = || { cat $(PAPER_BUILD)/$(PAPER).log; exit 1; }

# Twice, because TikZ remember picture needs a second pass to settle
$(PAPER_PDF): $(PAPER_SRC)/$(PAPER).tex $(PAPER_BUILD)/keysigs.stamp | $(DIST_DIR)
	$(LATEX) $< >/dev/null $(SHOW_LOG)
	$(LATEX) $< >/dev/null $(SHOW_LOG)
	cp $(PAPER_BUILD)/$(PAPER).pdf $@

$(PAPER_BUILD):
	mkdir --parents $@

$(DIST_DIR):
	mkdir --parents $(DIST_DIR)

clean:
	rm -rf $(DIST_DIR) $(BUILD_DIR) $(NIX_OUT_DIR) flake.lock

.PHONY: all clean
