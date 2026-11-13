# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Print the wires that two versions of a Digital circuit do not share."""
import sys
import xml.etree.ElementTree as ET
from collections import Counter
def wires(p):
    r = ET.parse(p).getroot()
    return Counter(tuple(sorted([(int(w.find('p1').get('x')), int(w.find('p1').get('y'))), (int(w.find('p2').get('x')), int(w.find('p2').get('y')))])) for w in r.find('wires'))
a, b = wires(sys.argv[1]), wires(sys.argv[2])
for w in sorted((a - b).elements()): print('-', w)
for w in sorted((b - a).elements()): print('+', w)
