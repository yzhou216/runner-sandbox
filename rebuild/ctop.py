# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Split kernel-style C source into its top-level chunks.

A chunk is a run of lines that blank lines outside of any brace, comment,
or string delimit, such as a function with the comment above it, an
enum, or a block of includes.  Joining the chunks with blank lines gives
back the source.
"""
import re


def depth_after(line, state):
    """Return the brace depth and lexer state after line."""
    depth, comment = state
    i = 0
    while i < len(line):
        two = line[i:i + 2]
        if comment:
            if two == '*/':
                comment = False
                i += 2
                continue
            i += 1
            continue
        if two == '/*':
            comment = True
            i += 2
            continue
        if two == '//':
            break
        ch = line[i]
        if ch in '"\'':
            j = i + 1
            while j < len(line) and line[j] != ch:
                j += 2 if line[j] == '\\' else 1
            i = j + 1
            continue
        if ch == '{':
            depth += 1
        elif ch == '}':
            depth -= 1
        i += 1
    return depth, comment


def chunks(src):
    """Return the top-level chunks of src as strings without newlines at
    their ends."""
    assert src.endswith('\n'), 'missing final newline'
    out, cur, blank = [], [], 0
    state = (0, False)
    for line in src[:-1].split('\n'):
        if not line and state == (0, False):
            if cur:
                out.append('\n'.join(cur))
                cur = []
            else:
                blank += 1  # Each further blank line starts the next chunk
            continue
        if not cur:
            cur, blank = [''] * blank, 0
        cur.append(line)
        state = depth_after(line, state)
    assert cur and not blank, 'blank lines at the end'
    out.append('\n'.join(cur))
    assert join(out) == src, 'chunks do not give back the source'
    return out


def join(parts):
    return '\n\n'.join(p for p in parts if p is not None) + '\n'


def code_lines(chunk):
    """Lines of chunk outside of comments."""
    res, comment = [], False
    for line in chunk.split('\n'):
        stripped = line
        if comment:
            if '*/' not in line:
                continue
            stripped = line.split('*/', 1)[1]
            comment = False
        while '/*' in stripped:
            before, after = stripped.split('/*', 1)
            if '*/' in after:
                stripped = before + after.split('*/', 1)[1]
            else:
                stripped = before
                comment = True
                break
        if stripped.strip() and not stripped.lstrip().startswith('//'):
            res.append(stripped)
    return res


def name(chunk):
    """Name of what chunk defines or declares."""
    lines = code_lines(chunk)
    if not lines:
        return '#comment'
    first = lines[0]
    if first.startswith('#include'):
        return '#include'
    m = re.match(r'#\s*(define|ifndef|endif|if|else)\s*(\w*)', first)
    if m:
        return f'#{m.group(1)} {m.group(2)}'.strip()
    m = re.match(r'(?:static\s+)?(?:const\s+)?(enum|struct)\s+(\w+)\s*(?::\s*\w+\s*)?\{', first)
    if m and not re.search(r'\}\s*\w+\s*\[', chunk):
        return f'{m.group(1)} {m.group(2)}'
    text = ' '.join(lines)
    m = re.search(r'\}\s*(\w+)\s*\[\s*\]\s*=', text)
    if m:
        return m.group(1)
    m = re.match(r'(?:\[\[[^\]]*\]\]\s*)*[^(=]*?(\w+)\s*\(', text)
    if m:
        return m.group(1)
    m = re.match(r'[^=;]*?(\w+)\s*(?:=|;)', text)
    if m:
        return m.group(1)
    return '?' + first


def is_function(chunk):
    lines = code_lines(chunk)
    return bool(lines) and lines[-1] == '}' and '(' in lines[0] and not lines[0].startswith('#')
