# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Fold the modular layout of fold.py into every commit of rv64_emu.

The 43 commits up to BASE are rewritten in order: each keeps its author,
committer, and dates, and its message except for the five that name code
that moved, and its files are those of fold() applied to the original.
The result is stored as refs/fold/master.
"""
import os
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fold import fold  # noqa: E402

REPO = os.environ['REPO']
BASE = os.environ.get('BASE', '5d83c6211b5ef9a5feb9202445a1a15e2a0a0f08')

MESSAGES = {
    'rv_emu: emulate arithmetic, jalr and branch instructions': [(
        'rv_one() dispatches on the opcode',
        'rv_exec() dispatches on the opcode')],
    'treewide: adopt c23 features': [(
        '''unsupported one is reported by its whole instruction word.  The
encodings live in rv_emu.c, since only the emulator decodes
instructions.
''', '''unsupported one is reported by its whole instruction word.  The
encodings move from rv_emu.h to the files whose switches use them, since
only the emulator decodes instructions.
''')],
    'treewide: adopt c2y features': [(
        '''countof from <stdcountof.h> gives the lengths of arrays, a switch
statement in rv_one() declares the decoded instruction, and an if
statement in main() declares the selected test program.

The fields that all instruction formats share are decoded once into
struct rv_insn instead of again in every emulation function.  They are
unsigned _BitInt types as wide as the fields, so the conversions in
rv_decode() do the masking, and conversions to signed _BitInt types of
the right width sign-extend the immediates.  Bit-precise types are not
promoted, so funct7 is widened before it is shifted into the OP key.
Each emulation function now computes its result and writes it through
set_rd(), which discards writes to x0, so rv_one() no longer clears x0
afterward.  The functions are named after their major opcodes.
''', '''countof from <stdcountof.h> gives the lengths of arrays, and an if
statement in run_test() declares the selected test program.

The fields that all instruction formats share are decoded once into
struct rv_insn, in decode.c, instead of again in every emulation
function.  They are unsigned _BitInt types as wide as the fields, so the
conversions in rv_decode() do the masking, and conversions to signed
_BitInt types of the right width sign-extend the immediates.
Bit-precise types are not promoted, so funct7 is widened before it is
shifted into the OP key.  Each emulation function now computes its
result and writes it through set_rd(), which discards writes to x0, so
rv_exec() no longer clears x0 afterward.  The functions are named after
their major opcodes.
''')],
    'treewide: tidy up the headers, prototypes and parameters': [(
        'rv64_emu.h, so their definitions are checked',
        'progs.h, so their definitions are checked')],
    'rv64_emu: validate the command line and the cache geometry': [(
        '''argument counts, which main() checks in the condition of the if
statement that declares the program.  The option values are checked as
well, and any error prints the options and the programs to stderr.
''', '''argument counts, which run_test() checks in the condition of the if
statement that declares the program.  A run that rejects its arguments
returns false before it prints anything, and main() then prints the
usage.  The option values are checked as well, and any error prints the
options and the programs to stderr.
''')],
}

# Files that fold() reads or rewrites, rather than passing them through
TEXT = re.compile(r'(src/.*|progs/\w+_c\.c|Makefile)$')


class Blob:
    def __init__(self, mode, sha):
        self.mode, self.sha = mode, sha


def git(*args, stdin=None, env=None):
    res = subprocess.run(['git', '-C', REPO, *args], input=stdin, capture_output=True,
                         env={**os.environ, **(env or {})})
    if res.returncode:
        sys.exit(f'git {" ".join(args)}: {res.stderr.decode()}')
    return res.stdout.decode()


def read_tree(rev):
    files, modes = {}, {}
    for line in git('ls-tree', '-r', rev).splitlines():
        meta, path = line.split('\t', 1)
        mode, kind, sha = meta.split()
        assert kind == 'blob', line
        modes[path] = mode
        files[path] = git('cat-file', 'blob', sha) if TEXT.match(path) else Blob(mode, sha)
    return files, modes


def write_tree(files, modes):
    """Tree of files, where text files that keep their name or move keep
    the mode of the original."""
    sources = {'src/emu/bits.c': 'src/bits.c', 'src/emu/bits.h': 'src/bits.h',
               'src/cache/cache.c': 'src/cache.c', 'src/cache/cache.h': 'src/cache.h',
               'progs/progs.h': 'src/rv64_emu.h'}
    entries = []
    for path in sorted(files):
        obj = files[path]
        if isinstance(obj, Blob):
            entries.append(f'{obj.mode} {obj.sha}\t{path}')
            continue
        sha = git('hash-object', '-w', '--stdin', stdin=obj.encode()).strip()
        mode = modes.get(sources.get(path, path), '100644')
        assert mode == '100644', path
        entries.append(f'{mode} {sha}\t{path}')
    fd, index = tempfile.mkstemp()
    os.close(fd)
    os.unlink(index)
    env = {'GIT_INDEX_FILE': index}
    git('update-index', '--add', '--index-info', stdin=('\n'.join(entries) + '\n').encode(), env=env)
    tree = git('write-tree', env=env).strip()
    os.unlink(index)
    return tree


def commit(tree, parent, rev):
    head, msg = git('cat-file', 'commit', rev).split('\n\n', 1)
    subject = msg.split('\n', 1)[0]
    for old, new in MESSAGES.pop(subject, []):
        assert msg.count(old) == 1, (subject, old)
        msg = msg.replace(old, new)
    env = {}
    for line in head.splitlines():
        key, _, value = line.partition(' ')
        if key in ('author', 'committer'):
            name, email, date = re.match(r'(.*) <(.*)> (\d+ [+-]\d{4})$', value).groups()
            role = key.upper()
            env.update({f'GIT_{role}_NAME': name, f'GIT_{role}_EMAIL': email, f'GIT_{role}_DATE': date})
    args = ['commit-tree', tree] + (['-p', parent] if parent else [])
    return git(*args, stdin=msg.encode(), env=env).strip()


def main():
    revs = git('rev-list', '--reverse', BASE).split()
    assert len(revs) == 43, len(revs)
    parent = None
    for n, rev in enumerate(revs, 1):
        files, modes = read_tree(rev)
        try:
            new = fold(files, n)
        except Exception:
            print(f'commit {n} {rev}', file=sys.stderr)
            raise
        tree = write_tree(new, modes)
        assert parent is None or tree != git('rev-parse', f'{parent}^{{tree}}').strip(), rev
        parent = commit(tree, parent, rev)
        print(n, rev[:7], parent[:7], git('log', '-1', '--format=%s', rev).strip())
    assert not MESSAGES, MESSAGES
    git('update-ref', 'refs/fold/master', parent)
    print('master', parent)


main()
