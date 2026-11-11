# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Nix's errors, each with the lines indented under it, such as the last lines
# of a failed builder's log, but not those of builds that a failed dependency
# stopped, which follow the first failure by the hundred
function flush() {
  if (block != "" && block !~ /Reason: [0-9]+ dependenc/) {
    print block
    fflush()
  }
  block = ""
}
/^error:/ { flush(); block = $0; next }
block != "" && /^ / { block = block "\n" $0; next }
{ flush() }
END { flush() }
