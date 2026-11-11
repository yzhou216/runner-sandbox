# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Fold "rv_emu: drop the remaining typedefs" into the initial commit of
rv64_emu, turn the rv_format typedef that it missed into a plain enum,
license every commit of master and c2y under the GPLv3 instead of the MIT
license, and replace the opening sentence of the README with a tagline.

The commits between the initial one and the typedef one are rebased onto
the folded initial commit, which must leave them with the tree of the
typedef commit, so that every later tree is unchanged.  Every commit keeps
its author, committer, dates, and message, except that the commit that
replaced the rv_format typedef no longer claims to.
"""
import os
import re
import subprocess
import sys
import tempfile

REPO = os.environ['REPO']
GPL = os.environ['GPL']
TYPEDEFS = 'rv_emu: drop the remaining typedefs'
OLD_TAG = 'A small RISC-V (RV64) emulator in C.\n'
NEW_TAG = '64-bit RISC-V emulator with instruction cache support\n'
RV_FORMAT = re.compile(r'typedef enum \{\n(.*?)\n\} rv_format;\n', re.S)
C23 = 'treewide: adopt c23 features'
OLD_C23 = """    instruction encodings, which replace the last typedef, with digit
    separators that group the opcode bits as the opcode map of the
    specification does
"""
NEW_C23 = """    instruction encodings, with digit separators that group the opcode
    bits as the opcode map of the specification does
"""


def run(*args, stdin=None, env=None):
    full = dict(os.environ)
    if env:
        full.update(env)
    return subprocess.run(['git', '-C', REPO, *args], input=stdin,
                          capture_output=True, env=full)


def git(*args, stdin=None, env=None):
    res = run(*args, stdin=stdin, env=env)
    if res.returncode:
        sys.exit(f'git {" ".join(args)}: {res.stderr.decode()}{res.stdout.decode()}')
    return res.stdout.decode()


def tree_of(rev):
    return git('rev-parse', f'{rev}^{{tree}}').strip()


def scratch(tree):
    """Parentless commit of a tree, for merge-tree to merge."""
    env = {'GIT_AUTHOR_DATE': '@1 +0000', 'GIT_COMMITTER_DATE': '@1 +0000'}
    return git('commit-tree', tree, stdin=b'scratch\n', env=env).strip()


def pick(rev, onto):
    """Tree of rev cherry-picked onto the tree onto."""
    return git('merge-tree', '--write-tree', f'--merge-base={rev}^',
               scratch(onto), rev).split('\n', 1)[0]


def edit(tree, gpl):
    """Tree with the GPLv3 as LICENSE, the README tagline, and rv_format
    as a plain enum."""
    fd, index = tempfile.mkstemp()
    os.close(fd)
    env = {'GIT_INDEX_FILE': index}
    git('read-tree', tree, env=env)
    git('update-index', '--cacheinfo', f'100644,{gpl},LICENSE', env=env)
    if run('cat-file', '-e', f'{tree}:README.md').returncode == 0:
        readme = git('cat-file', 'blob', f'{tree}:README.md')
        assert readme.count(OLD_TAG) == 1, tree
        blob = git('hash-object', '-w', '--stdin',
                   stdin=readme.replace(OLD_TAG, NEW_TAG).encode()).strip()
        git('update-index', '--cacheinfo', f'100644,{blob},README.md', env=env)
    header = git('cat-file', 'blob', f'{tree}:src/rv_emu.h')
    if 'rv_format' in header:
        header, n = RV_FORMAT.subn(r'enum rv_format {\n\1\n};\n', header)
        assert n == 1, tree
        blob = git('hash-object', '-w', '--stdin', stdin=header.encode()).strip()
        git('update-index', '--cacheinfo', f'100644,{blob},src/rv_emu.h', env=env)
    out = git('write-tree', env=env).strip()
    os.unlink(index)
    return out


def commit(tree, parent, rev):
    head, msg = git('cat-file', 'commit', rev).split('\n\n', 1)
    if msg.startswith(C23 + '\n'):
        assert msg.count(OLD_C23) == 1, rev
        msg = msg.replace(OLD_C23, NEW_C23)
    env = {}
    for line in head.splitlines():
        key, _, value = line.partition(' ')
        if key in ('author', 'committer'):
            name, email, date = re.match(r'(.*) <(.*)> (\d+ [+-]\d{4})$', value).groups()
            role = key.upper()
            env.update({f'GIT_{role}_NAME': name, f'GIT_{role}_EMAIL': email,
                        f'GIT_{role}_DATE': date})
    args = ['commit-tree', tree] + (['-p', parent] if parent else [])
    return git(*args, stdin=msg.encode(), env=env).strip()


def names(old, new):
    return set(filter(None, git('diff', '--name-only', old, new).split('\n')))


def main():
    master = git('rev-list', '--reverse', 'origin/master').split()
    c2y = git('rev-list', '--reverse', 'origin/master..origin/c2y').split()
    assert len(master) == 25 and len(c2y) == 15, (len(master), len(c2y))
    subjects = [git('log', '-1', '--format=%s', rev).strip() for rev in master]
    k = subjects.index(TYPEDEFS)
    with open(GPL, 'rb') as f:
        gpl = git('hash-object', '-w', '--stdin', stdin=f.read()).strip()

    # The typedef commit comes first, and the commits before it follow
    trees = [pick(master[k], tree_of(master[0]))]
    for rev in master[1:k]:
        trees.append(pick(rev, trees[-1]))
    assert trees[-1] == tree_of(master[k]), 'the reordered commits differ'
    trees += [tree_of(rev) for rev in master[k + 1:]]
    revs = master[:k] + master[k + 1:]

    parent = None
    for name, todo in (('master', list(zip(revs, trees))),
                       ('c2y', [(rev, tree_of(rev)) for rev in c2y])):
        for rev, tree in todo:
            parent = commit(edit(tree, gpl), parent, rev)
        git('update-ref', f'refs/heads/new-{name}', parent)
        print(name, parent)

    # The typedef changes, the license, the README, and rv_format are the
    # only differences
    folded = names(master[k - 1], master[k]) | {'src/rv_emu.h'}
    new = git('rev-list', '--reverse', 'new-c2y').split()
    assert len(new) == len(revs) + len(c2y)
    for i, (old, cur) in enumerate(zip(revs + c2y, new)):
        allowed = {'LICENSE', 'README.md', 'src/rv_emu.h'} | (folded if i < k else set())
        assert names(old, cur) <= allowed, (old, names(old, cur))
        assert git('show', f'{cur}:LICENSE') == open(GPL).read(), cur
        assert run('grep', '-q', '-w', 'typedef', cur, '--', 'src', 'progs').returncode == 1, cur
    print(len(new), 'commits differ from the original only by the license, the README,'
          ' rv_format, and, before the typedef commit, its changes; none has a typedef')


main()
