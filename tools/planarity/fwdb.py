# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

import sys, itertools, copy, json
import networkx as nx
sys.path.insert(0, '/tmp/wire/plan')
import pipe2, greedy

NETS = {'WD_4', 'V_3', 'RD_3', 'RFW_3', 'RD_4', 'WEN_4', 'TARGET_2', 'FLUSH', 'STALL', 'DCUR_3'}
PINS = {('hz', 'RFW_4'), ('hz', 'RS1_1'), ('hz', 'RS2_1')}
O = {
    'ifdr': ['PC4', 'PC', 'IW'],
    'drex': ['PC4', 'PC', 'CTL', 'IW', 'RD0', 'RD1'],
    'mux0': [('in_0', 'RD0_2'), ('in_1', 'WD_4'), ('in_2', 'V_3')],
    'mux1': [('in_1', 'V_3'), ('in_2', 'WD_4'), ('in_3', 'RD1_2')],
    'exmem': ['CTL', 'X', 'D', 'IW'],
    'memwb': ['CTL', 'X', 'DOut', 'IW'],
}


def graph(o, nets=NETS, pins=PINS):
    m = pipe2.build(o)
    g, _ = greedy.graph(m, pipe2.GLOBAL | set(nets), set(pins))
    return g


def planar(o, nets=NETS, pins=PINS):
    return nx.is_planar(graph(o, nets, pins))


def explain(o, nets=NETS, pins=PINS):
    ok, cert = nx.check_planarity(graph(o, nets, pins), counterexample=True)
    if ok:
        return 'planar'
    return sorted({u[1] for u in cert.nodes if u[0] in ('part', 'pin')}), sorted({u[1] for u in cert.nodes if u[0] == 'net'})


if __name__ == '__main__':
    print(explain(copy.deepcopy(O)))
