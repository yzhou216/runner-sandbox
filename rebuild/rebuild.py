# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Rebuild the history of project06 as rv64_cpu.

Every commit of the result takes its files from a commit of the original
history, moved into the layout of rv64_cpu: the circuits in src, with
project06.dig as rv64_cpu.dig, the test programs and their ROM images in
progs, and the sources of the decoder ROM in decode.  LICENSE holds the
GPL from the first commit on.  A few files come from other commits, to
fold fixes into the commits that they fix, and the commits that only
cleaned up or fixed others are gone.  The commits from 2026 come from
the directories in overlay, applied on top of the last commit from 2024.
"""
import os
import re
import subprocess
import sys
import tempfile

ORIG = os.environ['ORIG']
OUT = os.environ['OUT']
HERE = os.path.dirname(os.path.abspath(__file__))
NAME, EMAIL = 'Yiyu Zhou', 'yiyu@yiyuzhou.io'
GPL = os.path.join(HERE, "..", "LICENSES", "GPL-3.0-or-later.txt")


def git(repo, *args, stdin=None, env=None):
    res = subprocess.run(['git', '-C', repo, *args], input=stdin, capture_output=True,
                         env={**os.environ, **(env or {})})
    if res.returncode:
        sys.exit(f'git {" ".join(args)}: {res.stderr.decode()}')
    return res.stdout


def remap(path):
    """Path in rv64_cpu of a file of project06."""
    if path == 'LICENSE':
        return path
    if path == 'project06.dig':
        return 'src/rv64_cpu.dig'
    if re.fullmatch(r'[\w-]+\.dig', path):
        return 'src/' + path
    if path == 'mkdecoderom.py':
        return 'decode/mkdecoderom.py'
    for old, new in (('rom/decode/', 'decode/'), ('rom/programs/', 'progs/'), ('rom/', 'progs/')):
        if path.startswith(old):
            return new + path[len(old):]
    raise ValueError(path)


def files(rev):
    """Mode and contents of every file of an original commit, by the path
    that it has in rv64_cpu."""
    out = {}
    for line in git(ORIG, 'ls-tree', '-r', rev).decode().splitlines():
        meta, path = line.split('\t', 1)
        mode, kind, sha = meta.split()
        assert kind == 'blob', line
        new = remap(path)
        assert new not in out, new
        out[new] = (mode, git(ORIG, 'cat-file', 'blob', sha))
    out['LICENSE'] = ('100644', open(GPL, 'rb').read())
    return out


def blob(rev, path):
    return git(ORIG, 'show', f'{rev}:{path}')


def once(data, old, new):
    assert data.count(old) == 1, (old[:200], data.count(old))
    return data.replace(old, new)


def without_pc_probe(dig):
    """The top-level circuit without the probe on the wire from pc to the
    instruction memory, which the commit that loads sort added, and with
    that wire back in one piece."""
    dig = once(dig, b'''    <visualElement>
      <elementName>Probe</elementName>
      <elementAttributes/>
      <pos x="-360" y="940"/>
    </visualElement>
''', b'')
    for p1, p2 in (((-380, 900), (-380, 940)), ((-380, 940), (-360, 940)), ((-380, 900), (-360, 900))):
        dig = once(dig, f'''    <wire>
      <p1 x="{p1[0]}" y="{p1[1]}"/>
      <p2 x="{p2[0]}" y="{p2[1]}"/>
    </wire>
'''.encode(), b'')
    return once(dig, b'''      <p1 x="-400" y="900"/>
      <p2 x="-380" y="900"/>''', b'''      <p1 x="-400" y="900"/>
      <p2 x="-360" y="900"/>''')


def date(rev):
    """Author date of an original commit, in the time zone of the laptop
    that made it, since the classroom made the first in UTC."""
    head = git(ORIG, 'cat-file', 'commit', rev).decode().split('\n\n', 1)[0]
    stamp = re.search(r'^author .* (\d+) ([+-]\d{4})$', head, re.M)
    secs, tz = int(stamp.group(1)), stamp.group(2)
    return f'{secs} {"-0700" if tz == "+0000" else tz}'


STORE = 'bd1803c'  # Fix store

# Original commit, commit whose date the result takes, new message, and
# files that come from elsewhere
HISTORY = [
    ('ed01f49', '11642a2', '''treewide: initial commit

The project consists of the instruction memory of a RISC-V processor in
Digital, with a ROM for each of the test programs in RISC-V assembly,
and scripts that turn assembled programs and tables of control words
into ROM images.  The ROMs of the first sixteen programs hold their
images, and the processor does not exist yet.
''', {}),
    ('374801a', '374801a', '''progs: add the rom images of the test programs

make writes the images with as, objdump, and makerom3.py, in the format
in which the ROMs of Digital load them.
''', {}),
    ('f142781', 'f142781', '''rv64_cpu: add the single-cycle datapath

Every clock cycle fetches the instruction at pc from the ROM of the
program that PROG selects, decodes it, executes it, and writes back its
result, which EN enables.  The unimp at the end of every program raises
DONE, which stops the clock, and CLR clears pc and the registers.  A0,
A1, and A2 show the argument registers.  The decoder recognizes add,
addi, jal, and jalr, enough for the programs of add, addi, jal, and j.
''', {}),
    ('c061149', 'c061149', '''decode: make room for sixteen instructions

The priority encoder takes sixteen inputs instead of four, and the ROM
holds sixteen control words of eleven bits, since PCsel widens to two
bits and ALUSrcB to three.
''', {}),
    ('b109643', 'b109643', 'rv64_cpu: execute sll\n', {}),
    ('ebf31fa', 'ebf31fa', 'rv64_cpu: execute slli\n', {}),
    ('5eeb2ae', '5eeb2ae', 'rv64_cpu: execute srl\n', {}),
    ('7416ade', '7416ade', 'rv64_cpu: execute srli\n', {}),
    ('310f313', '310f313', 'rv64_cpu: execute mul\n', {}),
    ('0287a60', '0287a60', '''rv64_cpu: execute lui

The immediate decoder extracts the U-type immediate as well.
''', {}),
    ('3969298', '3969298', '''decode: invert WDsel

WDsel is now 1 for the instructions that write back the result of the
ALU and 0 for jal and jalr, which write back the return address, and
the multiplexer that it drives has its inputs swapped.
''', {}),
    ('864879c', '864879c', 'decode: draw the addi matcher like the others\n', {}),
    ('cb74e71', 'cb74e71', '''rv64_cpu: add the branch condition unit and beq

bcu compares the two source registers as BrOp selects and tells the
multiplexer of pc whether to take the branch, and the immediate decoder
extracts the B-type offset.  The control words grow to fourteen bits
for the three bits of BrOp.
''', {}),
    ('b821a8a', 'b821a8a', 'rv64_cpu: execute blt\n', {}),
    ('aa81357', 'aa81357', 'rv64_cpu: execute bne\n', {}),
    ('af18cf5', 'af18cf5', 'rv64_cpu: execute bge\n', {}),
    ('35a5822', '35a5822', 'inst-mem: load quadratic\n', {}),
    ('18b009a', '18b009a', '''progs: keep the object files

make deleted them as intermediate files of the images.
''', {}),
    ('f0eb78d', 'f0eb78d', '''inst-mem: load get_bitseq

get_bitseq needs addiw, which the decoder does not recognize yet.
''', {}),
    ('d992c03', 'd992c03', 'rv64_cpu: execute sub\n', {}),
    ('7034d14', '7034d14', '''rv64_cpu: execute or and and

The decoder grows to 32 instructions, since these are its sixteenth and
seventeenth.
''', {}),
    ('75e6374', '75e6374', 'rv64_cpu: execute addiw\n', {}),
    ('91793a1', '91793a1', '''decode: generate the rom image with make

decode.bin holds the control word of every instruction as a binary
literal, from which make and mkdecoderom.py write the image that the ROM
of the decoder loads.
''', {}),
    ('995dbbf', '995dbbf', '''rv64_cpu: add the data memory with load and store units

The RAM holds 128 64-bit words.  The store unit merges a narrower store
into the word that it replaces, and the load unit extracts a narrower
load from the word that it reads.  The control words grow to nineteen
bits for sz, st, ld, and m2r.
''', {'src/store.dig': (STORE, 'store.dig')}),
    ('dff1d46', 'e863f8f', 'rv64_cpu: execute ld and sd\n', {'src/store.dig': (STORE, 'store.dig')}),
    ('b157e71', 'b157e71', 'rv64_cpu: execute lw and sw\n', {'src/store.dig': (STORE, 'store.dig')}),
    ('938b398', '938b398', 'rv64_cpu: execute lb and sb\n',
     {'src/store.dig': (STORE, 'store.dig'), 'src/rv64_cpu.dig': (STORE, 'project06.dig')}),
    ('0c5f59a', '0c5f59a', 'inst-mem: load max3\n',
     {'src/store.dig': (STORE, 'store.dig'), 'src/rv64_cpu.dig': (STORE, 'project06.dig')}),
    ('3a44933', '3a44933', 'inst-mem: load fib_rec\n', {}),
    ('66e12cc', '66e12cc', 'inst-mem: load is_pal\n', {}),
    ('8d7ecc1', '8d7ecc1', 'inst-mem: load to_upper\n', {}),
    ('1fdd8bf', '1fdd8bf', 'inst-mem: load str_to_int\n', {}),
    ('1fdd8bf', 'edbeb9f', '''inst-mem: load sort

sort packs its results with shifts by 32, which slli does not decode
yet.
''', {'src/inst-mem.dig': ('edbeb9f', 'inst-mem.dig')}),
    ('f885f3e', '9ae7b52', '''decode: decode slli and srli by funct6

slli and srli compared all seven bits of funct7 with zero, but on RV64
the lowest of them is the top bit of the six-bit shift amount, so the
decoder did not recognize shifts by 32 or more.  They compare the six
bits of funct6 instead, with a matcher of their own, and the decoder
loses the constants that nothing used.
''', {'src/rv64_cpu.dig': ('f885f3e', 'project06.dig', without_pc_probe)}),
    ('143e8a0', '143e8a0', '''rv64_cpu: take shift amounts from the shamt field

The immediate decoder extracts the shift amount of slli and srli, which
ALUSrcB selects for them instead of the I-type immediate.
''', {'src/rv64_cpu.dig': ('143e8a0', 'project06.dig', without_pc_probe)}),
    ('4a01877', '4a01877', '''rv64_cpu: show pc on a probe

A tunnel carries pc to a probe below the datapath, next to the probes of
the registers.
''', {}),
    ('375bb48', '375bb48', '''decode: invert m2r

m2r is now 0 for the loads, which write back the loaded value, and 1 for
every other instruction, and the multiplexer that it drives sits below
the data memory with its inputs swapped.
''', {}),
    ('4cb9980', '329494f', '''decode: document the control words

rom.csv lists the fields of the control word of every instruction next
to its encoding, and rom.pdf typesets the same table.
''', {}),
]


def write_tree(tree):
    entries = []
    for path in sorted(tree):
        mode, data = tree[path]
        sha = git(OUT, 'hash-object', '-w', '--stdin', stdin=data).decode().strip()
        entries.append(f'{mode} {sha}\t{path}')
    fd, index = tempfile.mkstemp()
    os.close(fd)
    os.unlink(index)
    env = {'GIT_INDEX_FILE': index}
    git(OUT, 'update-index', '--add', '--index-info', stdin=('\n'.join(entries) + '\n').encode(), env=env)
    sha = git(OUT, 'write-tree', env=env).decode().strip()
    os.unlink(index)
    return sha


def commit(tree, parent, msg, when):
    env = {'GIT_AUTHOR_NAME': NAME, 'GIT_AUTHOR_EMAIL': EMAIL, 'GIT_AUTHOR_DATE': when,
           'GIT_COMMITTER_NAME': NAME, 'GIT_COMMITTER_EMAIL': EMAIL, 'GIT_COMMITTER_DATE': when}
    args = ['commit-tree', tree] + (['-p', parent] if parent else [])
    return git(OUT, *args, stdin=msg.encode(), env=env).decode().strip()


def overlay(tree, step):
    """Apply the files of overlay/<step>: a file replaces the one at its
    path, and the file DELETE lists paths to remove."""
    root = os.path.join(HERE, 'overlay', step)
    tree = dict(tree)
    for dirpath, _, names in os.walk(root):
        for name in names:
            full = os.path.join(dirpath, name)
            rel = os.path.relpath(full, root)
            if rel in ('DELETE', 'MESSAGE', 'DATE', 'transform.py'):
                continue
            mode = '100755' if os.access(full, os.X_OK) else '100644'
            tree[rel] = (mode, open(full, 'rb').read())
    if os.path.exists(os.path.join(root, 'DELETE')):
        for path in open(os.path.join(root, 'DELETE')).read().split():
            del tree[path]
    return tree


def main():
    parent, tree = None, None
    for src, meta, msg, extra in HISTORY:
        tree = files(src)
        for path, (rev, orig, *fix) in extra.items():
            data = blob(rev, orig)
            for f in fix:
                data = f(data)
            tree[path] = (tree[path][0], data)
        sha = write_tree(tree)
        parent = commit(sha, parent, msg, date(meta))
        print(parent[:7], msg.split('\n', 1)[0])
    steps = sorted(os.listdir(os.path.join(HERE, 'overlay'))) if os.path.isdir(os.path.join(HERE, 'overlay')) else []
    for step in steps:
        root = os.path.join(HERE, 'overlay', step)
        tree = overlay(tree, step)
        if os.path.exists(os.path.join(root, 'transform.py')):
            scope = {}
            exec(open(os.path.join(root, 'transform.py')).read(), scope)
            tree = scope['transform'](tree)
        msg = open(os.path.join(root, 'MESSAGE')).read()
        when = open(os.path.join(root, 'DATE')).read().strip()
        parent = commit(write_tree(tree), parent, msg, when)
        print(parent[:7], msg.split('\n', 1)[0])
    git(OUT, 'update-ref', 'refs/heads/master', parent)
    print('master', parent)


main()
