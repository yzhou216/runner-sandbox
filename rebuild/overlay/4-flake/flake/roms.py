# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

"""Rebuild the ROM images of the test programs and of the control words
in a copy of the repository, and fail unless they match the images in
progs/hex and the ROMs that the circuits hold.

Usage: roms.py <repository> <as> <objdump>
"""
import os
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET


def roms(path):
    """Words of every ROM of a circuit, by its label."""
    out = {}
    for el in ET.parse(path).getroot().find('visualElements'):
        if el.findtext('elementName') != 'ROM':
            continue
        attrs = {e[0].text: e[1].text or '' for e in el.find('elementAttributes')}
        words = [int(w, 16) for w in attrs.get('Data', '').replace('\n', '').split(',') if w]
        out[attrs.get('Label', '')] = trim(words)
    return out


def image(path):
    """Words of an image in the hex format of Digital."""
    with open(path) as f:
        lines = f.read().split()
    assert lines[0:2] == ['v2.0', 'raw'], path
    return trim([int(w, 16) for w in lines[2:]])


def trim(words):
    """Words without the zeros at the end, which a ROM holds anyway."""
    while words and words[-1] == 0:
        words.pop()
    return words


def main():
    repo, assembler, objdump = sys.argv[1:]
    work = tempfile.mkdtemp()
    shutil.copytree(repo, work, dirs_exist_ok=True)
    for root, _, _ in os.walk(work):
        os.chmod(root, 0o755)
    subprocess.run(['make', '-s', '-C', f'{work}/progs', f'AS={assembler}', f'OBJDUMP={objdump}'],
                   check=True)
    subprocess.run(['make', '-s', '-C', f'{work}/decode'], check=True)

    errors = []
    for name in sorted(os.listdir(f'{work}/progs/hex')):
        if image(f'{work}/progs/{name}') != image(f'{work}/progs/hex/{name}'):
            errors.append(f'progs/hex/{name} differs from what make writes')
    for label, words in sorted(roms(f'{work}/src/inst-mem.dig').items()):
        if words != image(f'{work}/progs/hex/{label}.hex'):
            errors.append(f'the ROM {label} differs from progs/hex/{label}.hex')
    (decoder,) = roms(f'{work}/src/inst-decoder.dig').values()
    if decoder != image(f'{work}/decode/decode.hex'):
        errors.append('the ROM of the decoder differs from what make writes')
    print('\n'.join(errors) or 'The ROMs match their sources')
    sys.exit(1 if errors else 0)


main()
