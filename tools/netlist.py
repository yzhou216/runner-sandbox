# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Netlist of a Digital circuit, with pin positions measured by Digital.

Each distinct element (name, attributes, rotation) is exported once at the
origin with `CLI svg`; the pin circles of the export, in pin order, inputs
blue and outputs red, give the pin offsets.  Wires and tunnels then join
pins into nets.
"""
import hashlib, json, os, re, subprocess, sys, tempfile
import xml.etree.ElementTree as ET
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import digfile

JAVA, JAR = os.environ['JAVA'], os.environ['JAR']
CACHE = '/tmp/pipe/pincache.json'

BUILTIN = {
    'Multiplexer': lambda a: (['sel'] + [f'in_{i}' for i in range(2 ** int(a.get('Selector Bits', 1)))], ['out']),
    'Add': lambda a: (['a', 'b', 'c_i'], ['s', 'c_o']),
    'Sub': lambda a: (['a', 'b', 'c_i'], ['s', 'c_o']),
    'Mul': lambda a: (['a', 'b'], ['mul']),
    'Comparator': lambda a: (['a', 'b'], ['>', '=', '<']),
    'Const': lambda a: ([], ['out']),
    'Clock': lambda a: ([], ['C']),
    'In': lambda a: ([], ['out']),
    'Out': lambda a: (['in'], []),
    'Probe': lambda a: (['in'], []),
    'LED': lambda a: (['in'], []),
    'Tunnel': lambda a: (['tunnel'], []),
    'Not': lambda a: (['in'], ['out']),
    'RAMDualPort': lambda a: (['A', 'D_in', 'str', 'C', 'ld'], ['D']),
    'Register': lambda a: (['D', 'C', 'en'], ['Q']),
    'ROM': lambda a: (['A', 'sel'], ['D']),
    'BarrelShifter': lambda a: (['in', 'shift'], ['out']),
    'BitExtender': lambda a: (['in'], ['out']),
    'PriorityEncoder': lambda a: ([f'in{i}' for i in range(2 ** int(a.get('Selector Bits', 1)))], ['num', 'any']),
    'Splitter': lambda a: (split(a.get('Input Splitting', '4,4')), split(a.get('Output Splitting', '8'))),
}
for g in ('And', 'Or', 'XOr', 'NAnd', 'NOr'):
    BUILTIN[g] = lambda a: ([f'In_{i + 1}' for i in range(int(a.get('Inputs', 2)))], ['out'])


def split(spec):
    out, bit = [], 0
    for part in spec.split(','):
        part = part.strip()
        if '-' in part:
            lo, hi = (int(x) for x in part.split('-'))
            out.append(f'{lo}-{hi}')
            bit = hi + 1
        else:
            n = int(part)
            out.append(f'{bit}-{bit + n - 1}' if n > 1 else f'{bit}')
            bit += n
    return out


def raw_element(ve):
    return ET.tostring(ve, encoding='unicode')


def measure(ve_xml, srcdir):
    """Pin offsets of one element at the origin, measured with Digital"""
    ve0 = ET.fromstring(ve_xml)
    sub = os.path.join(srcdir, ve0.find('elementName').text)
    content = open(sub).read() if sub.endswith('.dig') and os.path.exists(sub) else ''
    key = hashlib.sha1((ve_xml + srcdir + content).encode()).hexdigest()
    cache = json.load(open(CACHE)) if os.path.exists(CACHE) else {}
    if key in cache:
        return cache[key]
    ve = ET.fromstring(ve_xml)
    ve.find('pos').set('x', '0')
    ve.find('pos').set('y', '0')
    with tempfile.TemporaryDirectory(dir=srcdir, prefix='.cal') as d:
        circ = os.path.join(d, 'cal.dig')
        # Subcircuits resolve relative to the circuit's directory
        for f in os.listdir(srcdir):
            if f.endswith('.dig'):
                os.symlink(os.path.join(srcdir, f), os.path.join(d, f))
        open(circ, 'w').write('<?xml version="1.0" encoding="utf-8"?><circuit><version>2</version><attributes/><visualElements>'
                              + ET.tostring(ve, encoding='unicode') + '</visualElements><wires/><measurementOrdering/></circuit>')
        subprocess.run([JAVA, '-Djava.awt.headless=true', '-cp', JAR, 'CLI', 'svg', '-dig', circ],
                       capture_output=True, check=True)
        svg = open(circ + '.svg').read()
    pins = []
    for m in re.finditer(r'<circle cx="([-0-9.]+)" cy="([-0-9.]+)" r="2.0" stroke="#(\w+)"', svg):
        pins.append((round(float(m.group(1))), round(float(m.group(2))), 'in' if m.group(3) == '0000b2' else 'out'))
    cache[key] = pins
    json.dump(cache, open(CACHE, 'w'))
    return pins


def names(name, attrs, srcdir):
    if name in BUILTIN:
        return BUILTIN[name](attrs)
    ins, outs = digfile.pins(os.path.join(srcdir, name))
    return [p[0] for p in ins], [p[0] for p in outs]


def netlist(path):
    srcdir = os.path.dirname(os.path.abspath(path))
    root = ET.parse(path).getroot()
    parent = {}
    def find(x):
        while parent.setdefault(x, x) != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x
    def union(a, b):
        parent[find(a)] = find(b)
    pins = []   # (element index, element name, label, pin name, direction, point, element pos)
    tunnels = {}
    for idx, ve in enumerate(root.find('visualElements')):
        name = ve.find('elementName').text
        if name in ('Text', 'Testcase', 'Rectangle'):
            continue
        attrs = digfile.attrs(ve)
        px, py = int(ve.find('pos').get('x')), int(ve.find('pos').get('y'))
        offs = measure(raw_element(ve), srcdir)
        ins, outs = names(name, attrs, srcdir)
        oi = [o for o in offs if o[2] == 'in']
        oo = [o for o in offs if o[2] == 'out']
        assert len(oi) == len(ins) and len(oo) == len(outs), (path, name, attrs, len(oi), ins, len(oo), outs)
        label = attrs.get('Label') or attrs.get('NetName') or ''
        for (x, y, _), n in list(zip(oi, ins)) + list(zip(oo, outs)):
            pt = (px + x, py + y)
            d = 'in' if n in ins and (x, y, 'in') in oi else 'out'
            pins.append((idx, name, label, n, d, pt, (px, py)))
            find(pt)
        if name == 'Tunnel':
            tunnels.setdefault(attrs['NetName'], []).append((px, py))
    ws = digfile.wires(path)
    points = {p for w in ws for p in w} | {p[5] for p in pins}
    for a, b in ws:
        union(a, b)
        # Points that lie on the wire connect to it
        for p in points:
            if p != a and p != b and on_segment(p, a, b):
                union(p, a)
    for pts in tunnels.values():
        for p in pts[1:]:
            union(p, pts[0])
    nets = {}
    for pin in pins:
        nets.setdefault(find(pin[5]), []).append(pin)
    return list(nets.values())


def on_segment(p, a, b):
    (x, y), (x1, y1), (x2, y2) = p, a, b
    if x1 == x2 == x:
        return min(y1, y2) <= y <= max(y1, y2)
    if y1 == y2 == y:
        return min(x1, x2) <= x <= max(x1, x2)
    return False


def describe(pin):
    idx, name, label, pn, d, pt, epos = pin
    who = f"{name}" + (f"[{label}]" if label else "") + f"@{epos[0]},{epos[1]}" if name not in ("Tunnel",) else f"~{label}"
    return f'{who}.{pn}'


if __name__ == '__main__':
    for net in sorted(netlist(sys.argv[1]), key=lambda n: describe(n[0])):
        drivers = [describe(p) for p in net if p[4] == 'out']
        loads = [describe(p) for p in net if p[4] != 'out']
        if len(net) > 1 or '--all' in sys.argv:
            print(' + '.join(drivers) or '(no driver)', '->', ', '.join(loads))
