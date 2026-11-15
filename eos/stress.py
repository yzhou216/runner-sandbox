# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Write a show that loads the network and light output: one GO fires 50
network cues to fakeeos.py over TCP at once, the next 50 over UDP, the
next 10 over TCP, after fakeeos.py restarts, 5 to the address of the first
50 and 5 to localhost, a light cue on universes 1, 2, 300, and 999, and
300 network cues a tenth of a second apart, to soak the output.

Usage: stress.py SHOW
"""

import sys


def cue(id, kind, name, **settings):
    lines = [f"[[list.cue]]", f"id = {id}", f'number = "{id}"', f'kind = "{kind}"', f'name = "{name}"']
    lines += [f"{key} = {value}" for key, value in settings.items()]
    return "\n".join(lines) + "\n"


def chain(start, destinations, label):
    """Network cues, one to each of `destinations`, that each continue to
    the next, but the last"""
    return [
        cue(
            start + n,
            "network",
            f"{label} {n}",
            destination=f'"{destination}"',
            message=f"'/{label} {n}'",
            advance='"continue"' if n < len(destinations) - 1 else '"manual"',
        )
        for n, destination in enumerate(destinations)
    ]


instruments = [(1, 1, 1), (2, 2, 1), (3, 300, 512), (4, 999, 1)]
show = "".join(
    f'[[instrument]]\nid = {id}\nname = "U{universe}"\nuniverse = {universe}\naddress = {address}\n\n'
    for id, universe, address in instruments
)
show += '[[list]]\nname = "Main"\n\n'
cues = (
    chain(1, ["127.0.0.1:3037"] * 50, "tcp")
    + chain(51, ["udp://127.0.0.1:8000"] * 50, "udp")
    + chain(101, ["127.0.0.1:3037"] * 5 + ["localhost:3037"] * 5, "again")
)
show += "\n".join(cues)
show += "\n" + cue(111, "light", "Three universes", duration="1.0")
show += "\n[list.cue.lights]\n1 = 100.0\n2 = 50.0\n3 = 20.0\n4 = 70.0\n"
soak = [
    cue(
        201 + n,
        "network",
        f"soak {n}",
        destination='"127.0.0.1:3037"',
        message=f"'/soak {n}'",
        post_wait="0.1",
        advance='"continue"' if n < 299 else '"manual"',
    )
    for n in range(300)
]
show += "\n" + "\n".join(soak)
open(sys.argv[1], "w").write(show)
