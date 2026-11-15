# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Copy standard input to standard output, each line behind its time."""

import sys
import time

for line in sys.stdin:
    print(f"{time.time():.3f} {line}", end="", flush=True)
