# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

import sys, copy
sys.path.insert(0, '/tmp/wire/plan')
import full, pipe2, greedy
import networkx as nx


def model():
    m = pipe2.build(full.O)
    # EX controls in the gap come from their own splitter on a CTL tunnel
    m.parts['split2'] = [(p, n) for p, n in m.parts['split2'] if p in ('ALUOp', 'ALUSrcA', 'ALUSrcB', 'WDsel', 'in')]
    m.part('gsplit', [('BrOp', 'BrOp_2'), ('PCsel', 'PCsel_2'), ('LD', 'LD_2'), ('RFW', 'RFW_2')], free=True)
    m.parts['rdec2'] = [(p, n) for p, n in m.parts['rdec2'] if p != 'IW']
    return m


def check(extra=()):
    m = model()
    g, _ = greedy.graph(m, pipe2.GLOBAL | full.NETS, full.PINS | set(extra))
    ok, cert = nx.check_planarity(g, counterexample=True)
    if ok:
        return True
    deg = dict(cert.degree()); seen = set(); out = []
    for b in [n for n, d in deg.items() if d > 2]:
        for nb in cert.neighbors(b):
            if (b, nb) in seen: continue
            p = [b, nb]; prev, cur = b, nb
            while deg[cur] == 2:
                nxt = [x for x in cert.neighbors(cur) if x != prev][0]; prev, cur = cur, nxt; p.append(cur)
            seen.add((cur, prev)); seen.add((b, nb))
            if len(p) > 2: out.append(' - '.join(str(q[1]) + ('.' + q[2] if len(q) > 2 else '') for q in p))
    return out


if __name__ == '__main__':
    r = check([tuple(a.split('.')) for a in sys.argv[1:]])
    print(r if r is True else '\n'.join(r))
