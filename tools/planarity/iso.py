# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Planarity of a sub-circuit: only the named parts"""
import sys, itertools, copy
import networkx as nx
sys.path.insert(0, '/tmp/wire/plan')
import fwdb, pipe2, greedy

REGIONS = {
    'IF': ['pc', 'add', 'npc', 'addr', 'imem', 'run', 'clr1', 'ifdr'],
    'DR': ['ifdr', 'idec', 'merge', 'rdec1', 'rf', 'drex', 'clr2'],
    'EX': ['drex', 'split2', 'rdec2', 'imm', 'mux0', 'mux1', 'srca', 'srcb', 'alu', 'bcu', 'br', 'jump', 'hi', 'tgt',
           'xmux', 'hz', 'exmem'],
    'MEM': ['exmem', 'split3', 'rdec3', 'maddr', 'ram', 'load', 'store', 'vfwd', 'memwb'],
    'WB': ['memwb', 'split4', 'rdec4', 'res', 'wen', 'done', 'gclk'],
}


def sub(o, parts, nets=fwdb.NETS, pins=fwdb.PINS):
    m = pipe2.build(o)
    m.parts = {k: v for k, v in m.parts.items() if k in parts}
    g, _ = greedy.graph(m, pipe2.GLOBAL | set(nets), set(pins))
    return g


def explain(g):
    ok, cert = nx.check_planarity(g, counterexample=True)
    if ok:
        return 'planar'
    return sorted({u[1] for u in cert.nodes if u[0] in ('part', 'pin')}), sorted({u[1] for u in cert.nodes if u[0] == 'net'})


if __name__ == '__main__':
    o = copy.deepcopy(fwdb.O)
    for r, parts in REGIONS.items():
        print(r, explain(sub(o, parts)))
