# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""List the cases whose output differs between the original and the
folded build of each commit, and whether the cache geometry of the case
is one that cache_valid() rejects from the command-line validation on.
"""
import glob
import os
import re
import sys


def cases(path):
    out, cur = {}, None
    for line in open(path, errors='replace'):
        if line.startswith('$ '):
            cur = line[2:].rstrip('\n')
            out[cur] = []
        else:
            out[cur].append(line)
    return out


def valid(args):
    """Whether the cache options of args, if any, are a geometry that
    cache_valid() accepts."""
    w = args.split()
    for i, a in enumerate(w):
        if a == '-dm' and i + 2 < len(w):
            size, block, ways = int(w[i + 1]), int(w[i + 2]), 1
        elif a == '-sa' and i + 3 < len(w):
            size, block, ways = int(w[i + 1]), int(w[i + 2]), int(w[i + 3])
        else:
            continue
        if block < 1 or block > 4 or block & (block - 1) or ways < 1 or size < 1 or size % block:
            return False
        slots = size // block
        if slots > 4096 or slots % ways:
            return False
        sets = slots // ways
        return sets & (sets - 1) == 0
    return True


root = sys.argv[1]
for d in sorted((p for p in glob.glob(f'{root}/*/*.diff') if re.search(r'/\d+\.diff$', p)), key=lambda p: int(re.search(r'/(\d+)\.diff$', p).group(1))):
    n = int(re.search(r'(\d+)\.diff$', d).group(1))
    if not os.path.getsize(d):
        continue
    base = d[:-len('.diff')]
    a, b = cases(f'{base}-orig.cmp'), cases(f'{base}-fold.cmp')
    assert a.keys() == b.keys()
    bad = [k for k in a if a[k] != b[k]]
    kinds = {}
    for k in bad:
        kind = 'invalid geometry' if not valid(k) else ('-sa stub' if '-sa' in k.split() else 'OTHER')
        kinds.setdefault(kind, []).append(k)
    print(n, {k: len(v) for k, v in kinds.items()})
    for k in kinds.get('OTHER', []):
        print('   ', k[:100])
