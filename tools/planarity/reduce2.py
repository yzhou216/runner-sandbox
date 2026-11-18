# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Wire tunnel connections back while planar and the tunnel count drops"""
import sys, itertools, random
sys.path.insert(0, '/tmp/wire/plan')
import full, full2, pipe2, greedy
import networkx as nx

m = full2.model()
pins_of = {}
for name, pins in m.parts.items():
    for p, n in pins:
        if n and n not in pipe2.GLOBAL:
            pins_of.setdefault(n, []).append((name, p))
base = set(full.PINS)
for name, pins in m.parts.items():
    for p, n in pins:
        if n in full.NETS:
            base.add((name, p))


def cost(cut):
    t = 0
    for n, ps in pins_of.items():
        c = sum(1 for x in ps if x in cut)
        live = len(ps) - c
        if c and len(ps) > 1:
            t += c + (1 if live >= 1 else 0)
    return t


def planar(c):
    g, _ = greedy.graph(m, pipe2.GLOBAL, c)
    return nx.is_planar(g)


best = (cost(base), base)
print('start', best[0])
for seed in range(40):
    cut = set(base)
    order = sorted(cut)
    random.Random(seed).shuffle(order)
    improved = True
    while improved:
        improved = False
        for c in order:
            if c not in cut:
                continue
            trial = cut - {c}
            if cost(trial) < cost(cut) and planar(trial):
                cut = trial
                improved = True
        # Pairs: wire two pins of one net together
        for n, ps in pins_of.items():
            cps = [x for x in ps if x in cut]
            for a, b in itertools.combinations(cps, 2):
                trial = cut - {a, b}
                if cost(trial) < cost(cut) and planar(trial):
                    cut = trial
                    improved = True
                    break
    if cost(cut) < best[0]:
        best = (cost(cut), cut)
        print(seed, best[0], flush=True)
cut = best[1]
for n, ps in sorted(pins_of.items()):
    c = [x for x in ps if x in cut]
    if c and len(ps) > 1:
        live = [x for x in ps if x not in cut]
        print(f'{n:10s} tunnels {[f"{a}.{b}" for a, b in c]}  wired {[f"{a}.{b}" for a, b in live]}')
