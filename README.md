<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: native tests on Windows and FreeBSD after the fixes

cinq c129559ff (branch `verify-plat-02-c129559f` of yzhou216/cinq) makes the
six tests that failed on Windows and the one that failed on FreeBSD in
`cinq/2026-11-27-native-tests` hold there.  Run 46115122571, green first time.

| Runner | Result |
| --- | --- |
| windows-latest | 1481 passed, 1 ignored, 0 failed |
| windows-11-arm | 1481 passed, 1 ignored, 0 failed |
| x86_64 FreeBSD 15.1 | 1494 passed, 0 failed; the bounded read test passed 20 more times |

The ignored test is the write timeout, since Windows takes in 64 MiB for a
peer that never reads.  FreeBSD's loopback buffers are
`net.inet.tcp.sendspace` 32768 and `net.inet.tcp.recvspace` 65536, more than
the 64 KiB that one check reads once the peer refills them after a pause.
