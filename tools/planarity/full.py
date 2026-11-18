# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Full-circuit planarity with the decided orders and tunnels"""
import sys, copy
sys.path.insert(0, '/tmp/wire/plan')
import fwdb, pipe2, greedy
import networkx as nx

O = {
    'ifdr': ['PC4', 'PC', 'IW'],
    'drex': ['PC4', 'PC', 'CTL', 'RD0', 'RD1', 'IW'],
    'mux0': [('in_0', 'RD0_2'), ('in_1', 'WD_4'), ('in_2', 'V_3')],
    'mux1': [('in_0', 'RD1_2'), ('in_1', 'WD_4'), ('in_2', 'V_3')],
    'exmem': ['CTL', 'D', 'X', 'IW'],
    'memwb': ['CTL', 'DOut', 'X', 'IW'],
}
NETS = fwdb.NETS | {'SZ_3'}
PINS = fwdb.PINS | {('srca', 'in_1'), ('drex', 'PC_out'), ('xmux', 'in_0'), ('drex', 'PC4_out'), ('exmem', 'D_in'),
                    ('hz', 'RD_3'), ('hz', 'RFW_3'), ('hz', 'RD_4'),
                    ('split2', 'ALUSrcB'), ('srcb', 'sel')}


def check(extra=()):
    m = pipe2.build(O)
    g, _ = greedy.graph(m, pipe2.GLOBAL | NETS, PINS | set(extra))
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
