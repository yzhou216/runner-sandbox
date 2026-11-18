# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Reading Digital circuits: elements, attributes, pins, and wires."""
import os
import xml.etree.ElementTree as ET


def attrs(el):
    out = {}
    ea = el.find('elementAttributes')
    if ea is None:
        return out
    for entry in ea.findall('entry'):
        key = entry[0].text
        val = entry[1]
        if val.tag in ('string', 'int', 'long', 'boolean'):
            out[key] = val.text
        elif val.tag == 'rotation':
            out[key] = 'rot' + val.get('rotation')
        elif val.tag == 'data':
            out[key] = '<data>'
        elif val.tag == 'testData':
            out[key] = '<testData>'
        else:
            out[key] = '<' + val.tag + '>'
    return out


def elements(path):
    root = ET.parse(path).getroot()
    out = []
    for ve in root.find('visualElements'):
        pos = ve.find('pos')
        out.append({'name': ve.find('elementName').text, 'attrs': attrs(ve),
                    'pos': (int(pos.get('x')), int(pos.get('y')))})
    return out


def wires(path):
    root = ET.parse(path).getroot()
    out = []
    for w in root.find('wires'):
        p1, p2 = w.find('p1'), w.find('p2')
        out.append(((int(p1.get('x')), int(p1.get('y'))), (int(p2.get('x')), int(p2.get('y')))))
    return out


def circuit_attrs(path):
    root = ET.parse(path).getroot()
    return attrs(root.find('attributes').getparent() if False else root) if False else attrs_of_root(root)


def attrs_of_root(root):
    a = root.find('attributes')
    out = {}
    if a is None:
        return out
    for entry in a.findall('entry'):
        out[entry[0].text] = entry[1].text if entry[1].tag != 'rotation' else 'rot'
    return out


def pins(path):
    """In and Out elements of a subcircuit, in file order"""
    ins, outs = [], []
    for e in elements(path):
        if e['name'] == 'In':
            ins.append((e['attrs'].get('Label'), int(e['attrs'].get('Bits', 1)), e['pos'], e['attrs'].get('rotation')))
        elif e['name'] == 'Out':
            outs.append((e['attrs'].get('Label'), int(e['attrs'].get('Bits', 1)), e['pos'], e['attrs'].get('rotation')))
    return ins, outs
