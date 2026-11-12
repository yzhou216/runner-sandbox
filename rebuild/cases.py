# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Print the cases of the file in the first argument that the nth commit
runs deterministically, n being the second argument, and the others,
each with the reason, to the file in the third.

Before the commit that validates the command line, 37, the cache
simulator accepts geometries that it cannot model, whose blocks it
overflows or misaligns, so that it returns words of whatever follows
the assembly in memory, and cache_print() counts used slots over all
words of the cache, past the end of the slots for more than 4096
words.  The set-associative cache of commits 17 and 18 is a stub that
caches one word per slot at a fixed offset and searches four ways
whatever the configuration, so it returns wrong instruction words.  The
output of those cases depends on the layout of the C code.  Before the
commit that checks more arguments, 41, int_to_str loops forever for
bases 0 and 1 and overruns its buffer until it crashes.
"""
import sys

MAX_SLOTS = 4096
MAX_BLOCK_SIZE = 4


def cache(words):
    """Size, block size, and ways of the cache options of a case."""
    for i, w in enumerate(words):
        try:
            if w == '-dm' and i + 2 < len(words):
                return int(words[i + 1]), int(words[i + 2]), 1
            if w == '-sa' and i + 3 < len(words):
                return int(words[i + 1]), int(words[i + 2]), int(words[i + 3])
        except ValueError:
            return None
    return None


def valid(size, block, ways):
    """Whether cache_valid() accepts the geometry."""
    if block < 1 or block > MAX_BLOCK_SIZE or block & (block - 1) or ways < 1 or size < 1 or size % block:
        return False
    slots = size // block
    if slots > MAX_SLOTS or slots % ways:
        return False
    sets = slots // ways
    return sets & (sets - 1) == 0


def reason(args, n):
    words = args.split()
    geometry = cache(words)
    if n in (17, 18) and '-sa' in words:
        return 'set-associative stub'
    if n < 41 and 'int_to_str' in words and words[-2:] in (['-o', '0'], ['-o', '1']):
        return 'int_to_str overruns its buffer'
    if n < 37 and geometry:
        if not valid(*geometry):
            return 'geometry that the simulator cannot model'
        if geometry[0] > MAX_SLOTS:
            return 'cache_print() reads past the slots'
    return None


def main():
    path, n, skipped = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    with open(path) as f, open(skipped, 'w') as out:
        for line in f:
            args = line.rstrip('\n')
            why = reason(args, n)
            if why:
                print(f'{why}: {args[:100]}', file=out)
            else:
                print(args)


main()
