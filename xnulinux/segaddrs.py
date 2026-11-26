#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0
"""The kernel's segment slots for the free kernel collection, without Apple's kernelcache.

    segaddrs.py DVM LAYOUT.json

xnulinux's vm-xnu-rebuild.sh regenerates macOS-DroidVM's k27/segaddrs.sh with
k27/kc27-layout.py, which reads Apple's macOS 27.2 kernelcache.  Without it the
committed k27/segaddrs.sh stays, which was made for DroidVM's own kernel, and
xnulinux's larger kernel no longer fits.  This applies kc27-layout.py's rules
with the slots of Apple's kernel read back from the committed k27/segaddrs.sh,
checks the result against the collection layout of free-kc/layout.json, and
rewrites k27/segaddrs.sh when the slots move.  Exit status 0: unchanged,
3: rewritten (link the kernel again), 1: it does not fit.
"""
import json
import re
import struct
import subprocess
import sys
from pathlib import Path

PAGE = 0x4000
MARGIN = 0x10000
GROWS = ('__TEXT', '__DATA_CONST', '__TEXT_EXEC', '__DATA', '__BOOTDATA')
EMPTIES = ['__PRELINK_DATA', '__PRELINK_INFO', '__PLK_TEXT_EXEC', '__PLK_DATA_CONST', '__PLK_LLVM_COV', '__PLK_LINKEDIT']
MOVE = '-Wl,-rename_section,__TEXT,__cstring,__TEXT_EXEC,__cstring'


def up(v, a=PAGE):
    return (v + a - 1) // a * a


def segments(path):
    data = Path(path).read_bytes()
    count, = struct.unpack_from('<I', data, 16)
    off, segs, cstring = 32, {}, 0
    for _ in range(count):
        cmd, length = struct.unpack_from('<II', data, off)
        if cmd == 0x19:
            name = data[off + 8:off + 24].split(b'\0')[0].decode()
            vmaddr, vmsize = struct.unpack_from('<QQ', data, off + 24)
            segs[name] = (vmaddr, vmsize)
            for index in range(struct.unpack_from('<I', data, off + 64)[0]):
                at = off + 72 + index * 80
                if name == '__TEXT' and data[at:at + 16].split(b'\0')[0] == b'__cstring':
                    cstring = struct.unpack_from('<Q', data, at + 40)[0]
        off += length
    return segs, cstring


def main():
    dvm, layout = Path(sys.argv[1]), json.loads(Path(sys.argv[2]).read_text())
    script = dvm / 'k27/segaddrs.sh'
    old = script.read_text()
    kb = int(re.search(r'^KB=(0x[0-9a-f]+)', old, re.M).group(1), 16)
    flags = subprocess.run(['bash', str(script)], capture_output=True, text=True, check=True).stdout
    slot = {n: int(a, 16) - kb for n, a in re.findall(r'-segaddr,(\w+),(0x[0-9a-f]+)', flags)}
    top = {n: (int(o, 16), int(o, 16) + int(s, 16)) for n, o, s, _, _ in layout['segments']}

    segs, cstring = segments(dvm / 'out/k27/kernel-k27.macho')
    for n, (va, size) in segs.items():
        print(f'kernel {n:18} {va - kb:#10x}-{va - kb + size:#10x} ({size:#x})')
    ours = {n: size for n, (va, size) in segs.items()}
    if cstring:
        ours['__TEXT'] = up(ours['__TEXT'] - cstring)
        ours['__TEXT_EXEC'] = up(ours['__TEXT_EXEC'] + cstring)
    need = {n: up(ours[n] + (MARGIN if n in GROWS else 0)) for n in ours}

    # kc27-layout.py's placement, Apple's slots as the committed script has them
    place = {'__TEXT': 0x4000, '__DATA_CONST': slot['__DATA_CONST'], '__LASTDATA_CONST': slot['__LASTDATA_CONST'],
             '__KLD': slot['__KLD'], '__LAST': slot['__LAST'], '__PRELINK_TEXT': slot['__PRELINK_TEXT'],
             '__LINKINFO': slot['__LINKINFO']}
    place['__TEXT_EXEC'] = slot['__KLD'] - need['__TEXT_EXEC']
    end = slot['__PRELINK_DATA']    # the end of Apple's __BOOTDATA, where the empty markers start
    place['__KLDDATA'] = end - need['__KLDDATA'] - need['__DATA'] - need['__BOOTDATA']
    place['__DATA'] = place['__KLDDATA'] + need['__KLDDATA']
    place['__BOOTDATA'] = place['__DATA'] + need['__DATA']

    checks = [('__TEXT', top['__PRELINK_TEXT']), ('__DATA_CONST', top['__DATA_CONST']),
              ('__LASTDATA_CONST', top['__DATA_CONST']), ('__TEXT_EXEC', top['__TEXT_EXEC']),
              ('__KLD', top['__TEXT_EXEC']), ('__LAST', top['__TEXT_EXEC']),
              ('__KLDDATA', (top['__PRELINK_INFO'][0], top['__DATA'][1])),
              ('__DATA', (top['__PRELINK_INFO'][0], top['__DATA'][1])), ('__BOOTDATA', top['__DATA'])]
    bad, spans = False, []
    for name, (lo, hi) in checks:
        s, e = place[name], place[name] + need[name]
        spans.append((s, e, name))
        inside = lo <= s and e <= hi
        print(f'slot {name:18} {s:#10x}-{e:#10x} ours {ours.get(name, 0):#9x} '
              + ('ok' if inside else f'outside {lo:#x}-{hi:#x}'))
        bad |= not inside
    spans.sort()
    for (s0, e0, n0), (s1, e1, n1) in zip(spans, spans[1:]):
        if s1 < e0:
            print(f'overlap {n0} / {n1}')
            bad = True
    linkinfo = int(layout['kernel_linkinfo'][1], 16)
    if ours.get('__LINKINFO', 0) > linkinfo:
        print('__LINKINFO outgrows its slot')
        bad = True
    if bad:
        sys.exit('the kernel does not fit the free collection layout')

    lines = ['# -segaddr list for linking the kernel into the kc27 fileset KC (kc27-layout.py\'s rules, '
             'Apple\'s slots from the committed list; offsets from the KC header page)',
             f'KB={kb:#x}', f'echo -n "{MOVE} "', 'a() { printf -- "-Wl,-segaddr,%s,0x%x " "$1" $((KB + $2)); }', '{']
    lines.append(' '.join(f'a {n} {place[n]:#x};' for n in ('__TEXT', '__DATA_CONST', '__LASTDATA_CONST', '__TEXT_EXEC',
                                                              '__KLD', '__LAST', '__KLDDATA', '__DATA', '__BOOTDATA',
                                                              '__PRELINK_TEXT')))
    lines.append(f'o={end:#x}; for n in {" ".join(EMPTIES)}; do a $n $o; o=$((o+{PAGE:#x})); done')
    lines.append(f'a __LINKINFO {place["__LINKINFO"]:#x}')
    lines.append("} | sed 's/ $//'")
    new = '\n'.join(lines) + '\n'
    new_flags = subprocess.run(['bash', '-c', new], capture_output=True, text=True, check=True).stdout
    if new_flags == flags:
        print('slots unchanged')
        return 0
    script.write_text(new)
    print('wrote', script)
    print(new_flags)
    return 3


sys.exit(main())
