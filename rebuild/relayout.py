# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Squash the first three commits of rv64_emu into one initial commit by
its author, and move the sources into src and the test programs into
progs in every commit of master and c2y.

Every other commit keeps its author, committer and dates, takes its
message from messages.py, and has the tree of the original commit with
the files moved and the Makefile taught to find them, which the script
checks at both branch tips.
"""
import os
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from messages import MESSAGES  # noqa: E402

REPO = os.environ['REPO']
USER = 'Yiyu Zhou <yiyu@yiyuzhou.io>'


def git(*args, stdin=None, env=None):
    full = dict(os.environ)
    if env:
        full.update(env)
    res = subprocess.run(['git', '-C', REPO, *args], input=stdin,
                         capture_output=True, env=full)
    if res.returncode:
        sys.exit(f'git {" ".join(args)}: {res.stderr.decode()}')
    return res.stdout


def files_of(rev):
    out = {}
    for entry in filter(None, git('ls-tree', '-r', '-z', rev).decode().split('\0')):
        meta, path = entry.split('\t', 1)
        mode, kind, sha = meta.split()
        assert kind == 'blob', entry
        out[path] = (mode, git('cat-file', 'blob', sha))
    return out


def write_tree(files):
    fd, index = tempfile.mkstemp()
    os.close(fd)
    os.unlink(index)
    env = {'GIT_INDEX_FILE': index}
    lines = []
    for path in sorted(files):
        mode, content = files[path]
        sha = git('hash-object', '-w', '--stdin', stdin=content).decode().strip()
        lines.append(f'{mode} {sha}\t{path}')
    git('update-index', '--add', '--index-info', stdin=('\n'.join(lines) + '\n').encode(), env=env)
    tree = git('write-tree', env=env).decode().strip()
    os.unlink(index)
    return tree


def place(path):
    """Directory of a file in the new layout."""
    if '/' in path or not re.search(r'\.[chs]$', path):
        return path
    if re.search(r'_[cs]\.[cs]$', path) and not path.endswith('.h'):
        return 'progs/' + path
    return 'src/' + path


def makefile(text):
    def once(s, old, new):
        assert s.count(old) == 1, (old, s.count(old))
        return s.replace(old, new)
    text = once(text, 'HEADERS = ',
                '# Find the sources in src and the test programs in progs\n'
                'VPATH = src progs\n\nHEADERS = ')
    if '\tgcc -g -c -o $@ $<\n' in text:
        return once(text, '\tgcc -g -c -o $@ $<\n', '\tgcc -g -Isrc -c -o $@ $<\n')
    return once(text, '\t$(CC) $(CFLAGS) -c -o $@ $<\n', '\t$(CC) $(CFLAGS) -Isrc -c -o $@ $<\n')


def relayout(files):
    out = {}
    for path, (mode, content) in files.items():
        new = place(path)
        assert new not in out, new
        if path == 'Makefile':
            content = makefile(content.decode()).encode()
        out[new] = (mode, content)
    return out


def commit(tree, parent, msg, author, committer):
    a = re.match(r'(.*) <(.*)> (\d+ [+-]\d{4})$', author).groups()
    c = re.match(r'(.*) <(.*)> (\d+ [+-]\d{4})$', committer).groups()
    env = {'GIT_AUTHOR_NAME': a[0], 'GIT_AUTHOR_EMAIL': a[1], 'GIT_AUTHOR_DATE': a[2],
           'GIT_COMMITTER_NAME': c[0], 'GIT_COMMITTER_EMAIL': c[1], 'GIT_COMMITTER_DATE': c[2]}
    args = ['commit-tree', tree] + (['-p', parent] if parent else [])
    return git(*args, stdin=msg.encode(), env=env).decode().strip()


def header(rev):
    head, msg = git('cat-file', 'commit', rev).decode().split('\n\n', 1)
    h = dict(line.split(' ', 1) for line in head.splitlines() if not line.startswith(' ')
             and line.split(' ', 1)[0] in ('author', 'committer'))
    return h['author'], h['committer'], msg


def main():
    master = git('rev-list', '--reverse', 'origin/master').decode().split()
    c2y = git('rev-list', '--reverse', 'origin/master..origin/c2y').decode().split()
    assert len(master) == 27 and len(c2y) == 15, (len(master), len(c2y))

    # The initial commit has the tree of the third one, the date of the first
    first_date = header(master[0])[0].rsplit(' ', 2)[1]
    when = f'{first_date} -0700'
    used = {'treewide: initial commit'}
    parent = commit(write_tree(relayout(files_of(master[2]))), None,
                    MESSAGES['treewide: initial commit'],
                    f'{USER} {when}', f'{USER} {when}')
    print('init', parent)
    for name, revs in (('master', master[3:]), ('c2y', c2y)):
        for rev in revs:
            author, committer, msg = header(rev)
            subject = msg.split('\n', 1)[0]
            if subject in MESSAGES:
                assert subject not in used, subject
                used.add(subject)
                msg = MESSAGES[subject]
            parent = commit(write_tree(relayout(files_of(rev))), parent, msg, author, committer)
        git('update-ref', f'refs/heads/new-{name}', parent)
        print(name, parent)

    assert used == set(MESSAGES), set(MESSAGES) - used

    # Moving files and editing the Makefile must be the only differences
    for name in ('master', 'c2y'):
        old, new = f'origin/{name}', f'new-{name}'
        stat = git('diff', '-M100%', '--name-status', old, new).decode().split('\n')
        others = [l for l in stat if l and not l.startswith('R100') and l != 'M\tMakefile']
        assert not others, others
        print(name, 'differs from the original only by moved files and the Makefile')


main()
