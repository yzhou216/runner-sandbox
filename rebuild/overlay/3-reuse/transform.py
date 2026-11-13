def transform(tree):
    """Move the license into LICENSES and give every file that can hold a
    comment an SPDX header."""
    import re
    tree = dict(tree)
    tree['LICENSES/GPL-3.0-or-later.txt'] = tree.pop('LICENSE')

    def header(year):
        return (f'# SPDX-FileCopyrightText: {year} Yiyu Zhou <yiyu@yiyuzhou.io>\n'
                '#\n# SPDX-License-Identifier: GPL-3.0-or-later\n\n').encode()

    for path in list(tree):
        if (re.fullmatch(r'progs/[\w-]+\.s', path) or path.endswith('Makefile')
                or path.endswith('.py')):
            year = 2024
        elif path == '.gitignore':
            year = 2026
        else:
            continue
        mode, data = tree[path]
        assert b'SPDX' not in data, path
        tree[path] = (mode, header(year) + data)
    return tree
