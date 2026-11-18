# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Planarity model of a circuit topology.

Each component becomes a wheel: its connected pins in clockwise order on a
cycle around a hub, so the embedding keeps the pin order of the part.  A
component given without an order is a single node whose pins may go in any
order.  Each net is a node joined to its pins.  Leaves (In, Out, Const,
Probe, tunnels) are left out.
"""
import networkx as nx


class Model:
    def __init__(self):
        self.parts = {}   # name -> list of (pin, net) in clockwise order, or dict for free order
        self.free = set()

    def part(self, name, pins, free=False):
        self.parts[name] = pins
        if free:
            self.free.add(name)

    def graph(self, cut=()):
        cut = set(cut)
        g = nx.Graph()
        nets = {}
        for name, pins in self.parts.items():
            for pin, net in pins:
                if net and net not in cut:
                    nets.setdefault(net, []).append((name, pin))
        live = {n for n, ps in nets.items() if len(ps) > 1}
        for name, pins in self.parts.items():
            used = [(p, n) for p, n in pins if n in live]
            if not used:
                continue
            if name in self.free or len(used) < 3:
                for p, n in used:
                    g.add_edge(('part', name), ('net', n))
                continue
            hub = ('part', name)
            ring = [('pin', name, p) for p, _ in used]
            for i, r in enumerate(ring):
                g.add_edge(hub, r)
                g.add_edge(r, ring[(i + 1) % len(ring)])
            for (p, n), r in zip(used, ring):
                g.add_edge(r, ('net', n))
        return g

    def check(self, cut=()):
        ok, cert = nx.check_planarity(self.graph(cut), counterexample=True)
        return ok, cert

    def explain(self, cert):
        parts, nets = set(), set()
        for u in cert.nodes:
            if u[0] == 'net':
                nets.add(u[1])
            else:
                parts.add(u[1])
        return sorted(parts), sorted(nets)


def planar(model, cut=()):
    return nx.is_planar(model.graph(cut))
