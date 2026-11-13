# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Summarize how two versions of a Digital circuit differ.

Components are compared as multisets of their name, attributes, and
position, and wires as multisets of their end points, so that the order
in which Digital writes them out does not matter.  With --moves, a
component that only moved is reported as moved instead of as removed and
added.
"""
import sys
import xml.etree.ElementTree as ET
from collections import Counter


def value(node):
    if len(node) == 0:
        return node.text or ''
    return ET.tostring(node, encoding='unicode').strip()


def load(path):
    if path == '-':
        root = ET.fromstring('<circuit><visualElements/><wires/></circuit>')
    else:
        root = ET.parse(path).getroot()
    comps = []
    for el in root.find('visualElements'):
        name = el.findtext('elementName')
        attrs = []
        ea = el.find('elementAttributes')
        if ea is not None:
            for entry in ea:
                key, val = entry[0].text, entry[1]
                v = value(val)
                if len(v) > 80:
                    v = v[:77] + '...'
                attrs.append(f'{key}={v}')
        pos = el.find('pos')
        comps.append((name, tuple(sorted(attrs)), (pos.get('x'), pos.get('y')) if pos is not None else None))
    wires = []
    for w in root.find('wires'):
        p1, p2 = w.find('p1'), w.find('p2')
        wires.append(tuple(sorted([(int(p1.get('x')), int(p1.get('y'))),
                                   (int(p2.get('x')), int(p2.get('y')))])))
    attrs = {}
    top = root.find('attributes')
    if top is not None:
        for entry in top:
            attrs[entry[0].text] = value(entry[1])[:200]
    return attrs, Counter(comps), Counter(wires)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    old, new = load(args[0]), load(args[1])
    for k in sorted(set(old[0]) | set(new[0])):
        if old[0].get(k) != new[0].get(k):
            print(f'circuit attribute {k}: {old[0].get(k)!r} -> {new[0].get(k)!r}')
    removed = old[1] - new[1]
    added = new[1] - old[1]
    if '--moves' in sys.argv:
        for r in list(removed.elements()):
            match = [a for a in added.elements() if a[:2] == r[:2]]
            if match:
                removed[r] -= 1
                added[match[0]] -= 1
                print(f'  moved {r[0]} {" ".join(r[1])} {r[2]} -> {match[0][2]}')
        removed, added = +removed, +added
    for c in sorted(removed.elements()):
        print(f'- {c[0]} {" ".join(c[1])} at {c[2]}')
    for c in sorted(added.elements()):
        print(f'+ {c[0]} {" ".join(c[1])} at {c[2]}')
    wr, wa = old[2] - new[2], new[2] - old[2]
    print(f'wires: {sum(wr.values())} removed, {sum(wa.values())} added')


main()
