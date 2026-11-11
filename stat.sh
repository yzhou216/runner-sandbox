#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Owner, group, and mode of the directories that the package's payloads
# share with the system
for directory in / /Applications /usr /usr/local /usr/local/bin \
  /usr/local/share /usr/local/share/man /usr/local/share/man/man1 \
  /usr/local/share/man/man5 /usr/local/share/zsh \
  /usr/local/share/zsh/site-functions /usr/local/share/bash-completion \
  /usr/local/share/fish /usr/local/share/nushell /usr/local/share/doc \
  /usr/local/share/doc/cinq; do
  if [ -e "$directory" ]; then
    stat -f '%Su:%Sg %Sp %N' "$directory"
  else
    echo "absent $directory"
  fi
done
