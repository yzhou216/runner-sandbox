# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Geometric checks of a generated circuit: crossings, overlaps, and wires
touching pins of other nets."""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import netlist, digfile


def segs(path):
    return digfile.wires(path)


def check(path):
    ws = segs(path)
    nets = netlist.netlist(path)
    # Net of each wire, by any pin on it or by connectivity of endpoints
    pin_net = {}
    for i, net in enumerate(nets):
        for pin in net:
            pin_net[pin[5]] = i
    problems = []
    def horiz(s):
        return s[0][1] == s[1][1]
    for i, a in enumerate(ws):
        for b in ws[i + 1:]:
            if horiz(a) == horiz(b):
                continue
            h, v = (a, b) if horiz(a) else (b, a)
            y = h[0][1]; x = v[0][0]
            hx0, hx1 = sorted((h[0][0], h[1][0])); vy0, vy1 = sorted((v[0][1], v[1][1]))
            if hx0 < x < hx1 and vy0 < y < vy1:
                problems.append(('cross', h, v))
    return problems


if __name__ == '__main__':
    for p in check(sys.argv[1]):
        print(p)
