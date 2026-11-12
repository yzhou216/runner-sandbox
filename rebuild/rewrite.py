# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Fold the renames of rename.py into every commit of master and c2y of
rv64_emu, so that each name appears in its new form in the commit that
introduces it.

Every commit keeps its author, committer, and dates, and its message
except for the two that named old identifiers.  The C files of every
commit are those of the original with the renames applied, and nothing
else changes.
"""
import os
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rename import rename  # noqa: E402

REPO = os.environ['REPO']
MESSAGES = {
    'treewide: rename cache structs and drop redundant braces': (
        None, '''treewide: drop redundant braces and tidy up comments

The remaining line comments become block comments, the spacing around
casts and pointer declarators is fixed, and a note explains the padding
of the '% Used' format string, which the expected test output relies
on.
'''),
    'cache: restructure the simulator': (
        'rv_state are removed, since nothing used them, and g_verbose is declared\nnext to verbose().\n',
        'rv_state are removed, since nothing used them, and verbose_on is\ndeclared next to verbose().\n'),
}


def git(*args, stdin=None, env=None):
    res = subprocess.run(['git', '-C', REPO, *args], input=stdin, capture_output=True,
                         env={**os.environ, **(env or {})})
    if res.returncode:
        sys.exit(f'git {" ".join(args)}: {res.stderr.decode()}')
    return res.stdout.decode()


def renamed_tree(rev):
    fd, index = tempfile.mkstemp()
    os.close(fd)
    env = {'GIT_INDEX_FILE': index}
    git('read-tree', f'{rev}^{{tree}}', env=env)
    for path in git('ls-tree', '-r', '--name-only', rev, 'src', 'progs').split():
        if path.endswith(('.c', '.h')):
            src = git('cat-file', 'blob', f'{rev}:{path}')
            new = rename(path, src)
            if new != src:
                blob = git('hash-object', '-w', '--stdin', stdin=new.encode()).strip()
                git('update-index', '--cacheinfo', f'100644,{blob},{path}', env=env)
    tree = git('write-tree', env=env).strip()
    os.unlink(index)
    return tree


def commit(tree, parent, rev):
    head, msg = git('cat-file', 'commit', rev).split('\n\n', 1)
    subject = msg.split('\n', 1)[0]
    if subject in MESSAGES:
        old, new = MESSAGES.pop(subject)
        if old is None:
            msg = new
        else:
            assert msg.count(old) == 1, subject
            msg = msg.replace(old, new)
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


def main():
    master = git('rev-list', '--reverse', 'origin/master').split()
    c2y = git('rev-list', '--reverse', 'origin/master..origin/c2y').split()
    assert len(master) == 28 and len(c2y) == 15, (len(master), len(c2y))
    parent = None
    pairs = []
    for name, revs in (('master', master), ('c2y', c2y)):
        for rev in revs:
            tree = renamed_tree(rev)
            assert parent is None or tree != git('rev-parse', f'{parent}^{{tree}}').strip(), rev
            parent = commit(tree, parent, rev)
            pairs.append((rev, parent))
        git('update-ref', f'refs/rewrite/{name}', parent)
        print(name, parent)
    assert not MESSAGES, MESSAGES
    for old, new in pairs:
        for path in git('diff', '--name-only', old, new).split():
            assert re.match(r'(src|progs)/.*\.[ch]$', path), (old, path)
    print(len(pairs), 'commits differ from the original in their C files alone')


main()
