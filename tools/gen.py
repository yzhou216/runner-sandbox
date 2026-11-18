# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Building Digital circuits: parts at grid positions, joined by named
tunnels on short wires, with pin positions measured by Digital."""
import os, sys
from xml.sax.saxutils import escape
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import netlist

LONG = {'Value'}
INT = {'Bits', 'Selector Bits', 'Inputs', 'AddrBits', 'Width', 'fontSize'}
BOOL = {'flipSelPos', 'wideShape', 'Signed', 'mirror'}
SHAPE = {'shapeType'}


def entry(key, value):
    if key == 'rotation':
        v = f'<rotation rotation="{value}"/>'
    elif key == 'inverterConfig':
        v = '<inverterConfig>' + ''.join(f'<string>{x}</string>' for x in value) + '</inverterConfig>'
    elif key in SHAPE:
        v = f'<shapeType>{value}</shapeType>'
    elif key == 'intFormat':
        v = f'<intFormat>{value}</intFormat>'
    elif key in LONG:
        v = f'<long>{value}</long>'
    elif key in INT:
        v = f'<int>{value}</int>'
    elif key in BOOL:
        v = f'<boolean>{"true" if value else "false"}</boolean>'
    else:
        v = f'<string>{escape(str(value))}</string>'
    return f'<entry><string>{key}</string>{v}</entry>'


def element_xml(name, pos, attrs):
    body = ''.join(entry(k, v) for k, v in attrs.items())
    return (f'<visualElement><elementName>{name}</elementName><elementAttributes>{body}'
            f'</elementAttributes><pos x="{pos[0]}" y="{pos[1]}"/></visualElement>')


class Part:
    def __init__(self, circuit, name, pos, attrs):
        self.name, self.pos, self.attrs = name, pos, attrs
        xml = element_xml(name, (0, 0), attrs)
        offs = netlist.measure(xml, circuit.srcdir)
        ins, outs = netlist.names(name, attrs, circuit.srcdir)
        oi = [o for o in offs if o[2] == 'in']
        oo = [o for o in offs if o[2] == 'out']
        assert len(oi) == len(ins) and len(oo) == len(outs), (name, attrs, oi, ins, oo, outs)
        self.offsets = {}
        for (x, y, _), n in list(zip(oi, ins)) + list(zip(oo, outs)):
            self.offsets[n] = (x, y)
        self.outputs = set(outs)
        xs = [x for x, _ in self.offsets.values()]
        ys = [y for _, y in self.offsets.values()]
        self.box = (min(xs), min(ys), max(xs), max(ys))

    def pin(self, n):
        x, y = self.offsets[n]
        return (self.pos[0] + x, self.pos[1] + y)

    def side(self, n):
        x, y = self.offsets[n]
        x0, y0, x1, y1 = self.box
        if x0 == x1:
            return 'right' if n in self.outputs else 'left'
        if x == x0:
            return 'left'
        if x == x1:
            return 'right'
        return 'bottom' if y == y1 else 'top'


class Circuit:
    def __init__(self, srcdir, attrs=None):
        self.srcdir = srcdir
        self.attrs = attrs or {}
        self.elements = []
        self.wires = []

    def add(self, name, pos, **attrs):
        part = Part(self, name, pos, attrs)
        self.elements.append(element_xml(name, pos, attrs))
        return part

    def text(self, pos, description, **attrs):
        self.elements.append(element_xml('Text', pos, {'Description': description, **attrs}))

    def wire(self, a, b):
        if a == b:
            return
        if a[0] != b[0] and a[1] != b[1]:
            corner = (b[0], a[1])
            self.wires.append((a, corner))
            self.wires.append((corner, b))
        else:
            self.wires.append((a, b))

    def net(self, part, pin, net, length=20):
        """A tunnel named net, on a wire from the pin away from the part"""
        x, y = part.pin(pin)
        side = part.side(pin)
        dx, dy, rot = {'left': (-length, 0, 2), 'right': (length, 0, 0),
                       'top': (0, -length, 1), 'bottom': (0, length, 3)}[side]
        t = (x + dx, y + dy)
        self.wire((x, y), t)
        self.elements.append(element_xml('Tunnel', t, {'rotation': rot, 'NetName': net} if rot else {'NetName': net}))

    def nets(self, part, mapping, length=20):
        for pin, net in mapping.items():
            self.net(part, pin, net, length)

    def split_wires(self):
        """Digital joins wires only at their end points, so split every wire
        at the end points of others that lie on it"""
        ws = list(dict.fromkeys(tuple(sorted(w)) for w in self.wires if w[0] != w[1]))
        changed = True
        while changed:
            changed = False
            ends = {p for w in ws for p in w}
            out = []
            for a, b in ws:
                inner = [p for p in ends if p != a and p != b and
                         ((a[0] == b[0] == p[0] and min(a[1], b[1]) < p[1] < max(a[1], b[1])) or
                          (a[1] == b[1] == p[1] and min(a[0], b[0]) < p[0] < max(a[0], b[0])))]
                if inner:
                    pts = sorted([a, b] + inner)
                    out += list(zip(pts, pts[1:]))
                    changed = True
                else:
                    out.append((a, b))
            ws = out
        self.wires = ws

    def write(self, path, header=''):
        self.split_wires()
        attrs = ''.join(entry(k, v) for k, v in self.attrs.items())
        wires = ''.join(f'<wire><p1 x="{a[0]}" y="{a[1]}"/><p2 x="{b[0]}" y="{b[1]}"/></wire>' for a, b in self.wires)
        xml = ('<?xml version="1.0" encoding="utf-8"?>\n<circuit>\n  <version>2</version>\n'
               f'  <attributes>{attrs}</attributes>\n  <visualElements>\n    '
               + '\n    '.join(self.elements) + '\n  </visualElements>\n  <wires>\n    '
               + wires.replace('</wire><wire>', '</wire>\n    <wire>') + '\n  </wires>\n'
               '  <measurementOrdering/>\n</circuit>\n')
        open(path, 'w').write(xml)


def path(c, *pts):
    """Orthogonal wire through the given points"""
    for a, b in zip(pts, pts[1:]):
        assert a[0] == b[0] or a[1] == b[1], (a, b)
        if a != b:
            c.wires.append((a, b))

def probe(c, at, end, label):
    """Probe on a short stub from a wire"""
    path(c, at, end)
    c.add('Probe', end, Label=label, intFormat='hex')


def tunnel(c, part_or_pt, pin_or_net, net=None, d='left', length=20):
    """Tunnel on a short stub from a pin (or a point), pointing in direction d"""
    if net is None:
        (x, y), name = part_or_pt, pin_or_net
    else:
        (x, y), name = part_or_pt.pin(pin_or_net), net
    dx, dy, rot = {'left': (-length, 0, 2), 'right': (length, 0, 0),
                   'up': (0, -length, 1), 'down': (0, length, 3)}[d]
    t = (x + dx, y + dy)
    if length:
        c.wires.append(((x, y), t))
    c.elements.append(element_xml('Tunnel', t, {'rotation': rot, 'NetName': name} if rot else {'NetName': name}))
    return t
