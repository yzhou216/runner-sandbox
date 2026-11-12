# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Tokenizer for the C of rv64_emu that knows which function each token
belongs to.

Tokens are (kind, text, scope), where kind is id, comment, string, char,
number, pp (a preprocessor directive), space, or punct, and scope is the
name of the function whose parameter list or body holds the token, or
None at file level.  Joining the texts gives back the source exactly.
"""
import re

TOKEN = re.compile(r'''
    (?P<comment>/\*.*?\*/|//[^\n]*)
  | (?P<pp>^[ \t]*\#[^\n]*(?:\\\n[^\n]*)*)
  | (?P<string>"(?:\\.|[^"\\\n])*")
  | (?P<number>\.?[0-9](?:[0-9a-zA-Z_.]|'(?=[0-9a-fA-F])|[eEpP][+-])*)
  | (?P<char>'(?:\\.|[^'\\\n])+')
  | (?P<id>[A-Za-z_][A-Za-z0-9_]*)
  | (?P<space>\s+)
  | (?P<punct>->|\.\.\.|[^\sA-Za-z0-9_])
''', re.S | re.M | re.X)


def tokens(src):
    out = []
    pos = 0
    while pos < len(src):
        m = TOKEN.match(src, pos)
        assert m, src[pos:pos + 40]
        out.append([m.lastgroup, m.group(), None])
        pos = m.end()
    assert ''.join(t[1] for t in out) == src
    scope(out)
    return out


def scope(toks):
    """Mark the parameters and the body of every function with its name."""
    depth = 0
    paren = 0
    name = None      # Identifier before the last top-level '('
    start = None     # Index of that identifier
    end = None       # Index of the ')' that closes its parameters
    body_name = None # Function whose body is open
    body_start = None
    for i, (kind, text, _) in enumerate(toks):
        if kind != 'punct':
            continue
        if depth == 0 and paren == 0 and text == '(':
            j = i - 1
            while j >= 0 and toks[j][0] in ('space', 'comment'):
                j -= 1
            if j >= 0 and toks[j][0] == 'id':
                name, start = toks[j][1], j
        if text == '(':
            paren += 1
        elif text == ')':
            paren -= 1
            if depth == 0 and paren == 0 and name is not None:
                end = i
        elif text == '{':
            if depth == 0 and paren == 0 and name is not None and start is not None:
                # A body if only attributes or space follow the parameter list
                between = ''.join(t[1] for t in toks[end + 1:i] if t[0] not in ('space', 'comment'))
                if re.fullmatch(r'(\[\[[^\]]*\]\])*', between):
                    body_start, body_name = start, name
                else:
                    body_name = None
            depth += 1
        elif text == '}':
            depth -= 1
            if depth == 0 and body_name is not None:
                for t in toks[body_start:i + 1]:
                    t[2] = body_name
                body_name = None
                name = start = None
        elif text == ';' and depth == 0 and paren == 0:
            # A prototype: its parameters take the function's name too
            if name is not None and start is not None:
                for t in toks[start:i + 1]:
                    t[2] = name
            name = start = None
