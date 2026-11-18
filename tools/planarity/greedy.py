# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Greedy maximal planar wiring: start with every connection as a tunnel and
wire connections back one at a time while the circuit stays planar."""
import sys, importlib
import networkx as nx
sys.path.insert(0, '/tmp/wire/plan')
import pipe


def graph(m, cut_nets, cut_pins):
    nets = {}
    for name, pins in m.parts.items():
        for pin, net in pins:
            if net and net not in cut_nets and (name, pin) not in cut_pins:
                nets.setdefault(net, []).append((name, pin))
    live = {n for n, ps in nets.items() if len(ps) > 1}
    g = nx.Graph()
    for name, pins in m.parts.items():
        used = [(p, n) for p, n in pins if n in live and (name, p) not in cut_pins]
        if not used:
            continue
        if name in m.free or len(used) < 3:
            for p, n in used:
                g.add_edge(('part', name), ('net', n))
            continue
        ring = [('pin', name, p) for p, _ in used]
        for i, r in enumerate(ring):
            g.add_edge(('part', name), r)
            g.add_edge(r, ring[(i + 1) % len(ring)])
        for (p, n), r in zip(used, ring):
            g.add_edge(r, ('net', n))
    return g, nets


def greedy(o, prefer=None, fixed_cut=()):
    m = pipe.build(o)
    conns = [(name, pin, net) for name, pins in m.parts.items() for pin, net in pins
             if net and net not in pipe.GLOBAL and net not in fixed_cut]
    if prefer:
        conns.sort(key=prefer)
    cut = {(n, p) for n, p, _ in conns}
    for name, pin, net in conns:
        cut.discard((name, pin))
        g, _ = graph(m, pipe.GLOBAL | set(fixed_cut), cut)
        if not nx.is_planar(g):
            cut.add((name, pin))
    # Tunnels: every cut connection, plus one at the wired part of a partly cut net
    g, nets = graph(m, pipe.GLOBAL | set(fixed_cut), cut)
    bynet = {}
    for name, pin, net in conns:
        bynet.setdefault(net, []).append(((name, pin) in cut, name, pin))
    report, tunnels = [], 0
    for net, cs in sorted(bynet.items()):
        ncut = sum(c for c, _, _ in cs)
        if ncut and len(cs) > 1:
            t = ncut + (1 if ncut < len(cs) else 0)
            if ncut == len(cs) - 1 and False:
                pass
            tunnels += t
            report.append((net, [f'{n}.{p}' for c, n, p in cs if c], len(cs)))
    return tunnels, report


if __name__ == '__main__':
    cfg = importlib.import_module(sys.argv[1])
    t, rep = greedy(cfg.o)
    print('tunnels', t)
    for r in rep:
        print('  ', r)
