# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Give one commit of rv64_emu the modular layout.

fold() maps the files of the nth commit of the history, counted from one,
to the files of its rewritten counterpart: the emulator in src/emu, split
into rv_emu.c, exec.c, alu.c, mem.c, control.c, decode.c, and analysis.c,
the cache simulator in src/cache, the runs of the test programs in
harness.c, and progs.h next to the programs.  Code moves verbatim, except
that functions that other files call lose static, the dispatch of
rv_one() becomes rv_exec(), the dispatch of main() becomes run_test(),
and from the commit whose runs first reject arguments on, the runs
return whether they accepted them instead of calling usage().
"""
import re

import ctop

SPDX = ('/*\n * SPDX-FileCopyrightText: {} Yiyu Zhou <yiyu@yiyuzhou.io>\n'
        ' *\n * SPDX-License-Identifier: GPL-3.0-or-later\n */')

# Commits, counted from one, at which the code changes shape
C23 = 31        # treewide: adopt c23 features
C2Y = 33        # treewide: adopt c2y features, struct rv_insn, the table
GUARDS = 35     # treewide: tidy up the headers, prototypes and parameters
REJECT = 37     # rv64_emu: validate the command line and the cache geometry

SYMS = {
    'stdarg.h': r'\bva_(list|start|end|arg)\b',
    'stdbit.h': r'\bstdc_\w+',
    'stdbool.h': r'\b(bool|true|false)\b',
    'stdcountof.h': r'\bcountof\b',
    'stddef.h': r'\b(size_t|ptrdiff_t|NULL|offsetof)\b',
    'stdint.h': r'\b(u?int(8|16|32|64)_t|U?INT(8|16|32|64)_(MAX|MIN|C))\b',
    'stdio.h': r'\b(printf|fprintf|fputs|puts|fflush|vprintf|stdout|stderr|FILE)\b',
    'stdlib.h': r'\b(exit|atoi|abs|EXIT_FAILURE|EXIT_SUCCESS)\b',
    'string.h': r'\b(memset|memcpy|strcmp|strncmp|strlen|strncpy|strspn|strcpy)\b',
}

TOKEN = re.compile(r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\\n])*"|\'(?:\\.|[^\'\\\n])*\'', re.S)


def code(text):
    """text without its comments and the contents of its literals."""
    return TOKEN.sub(lambda m: ' ' if m.group().startswith('/') else '""', text)


def uses(text, word):
    return re.search(r'\b' + re.escape(word) + r'\b', code(text)) is not None


def once(src, old, new):
    assert src.count(old) == 1, (old, src.count(old))
    return src.replace(old, new)


def width(line):
    return len(line.expandtabs(8))


def indent(col):
    return '\t' * (col // 8) + ' ' * (col % 8)


def wrap_call(line):
    """Break a declaration longer than 80 columns after commas, aligning
    the continuation lines with the opening parenthesis."""
    if width(line) <= 80:
        return line
    col = width(line[:line.index('(') + 1])
    out, rest = [], line
    while width(rest) > 80:
        cut = max(i for i in range(len(rest)) if rest.startswith(', ', i) and width(rest[:i + 1]) <= 80)
        out.append(rest[:cut + 1])
        rest = indent(col) + rest[cut + 2:]
    return '\n'.join(out + [rest])


def wrap_words(head, words, cont, limit=77):
    """Fill words after head into lines of at most limit columns, each but
    the last ending in a backslash, with cont before continuation lines."""
    lines, cur = [], head + words[0]
    for k, w in enumerate(words[1:], 2):
        tail = ' \\' if k < len(words) else ''
        if width(cur + ' ' + w + tail) > limit:
            lines.append(cur + ' \\')
            cur = cont + w
        else:
            cur += ' ' + w
    return '\n'.join(lines + [cur])


# Function chunks: signatures, static, and prototypes

def sig_range(lines):
    """Indices of the first and last line of the signature of a function
    chunk, after any comment above it."""
    start = 0
    comment = False
    for i, line in enumerate(lines):
        if comment:
            comment = '*/' not in line
            continue
        if line.startswith('/*'):
            comment = '*/' not in line
            continue
        start = i
        break
    end = lines.index('{', start) - 1
    return start, end


def unstatic(chunk):
    lines = chunk.split('\n')
    start, end = sig_range(lines)
    old = lines[start]
    new = re.sub(r'^((?:\[\[\w+\]\] )?)static ', r'\1', old)
    assert new != old, old
    lines[start] = new
    old_col, new_col = width(old[:old.index('(') + 1]), width(new[:new.index('(') + 1])
    for i in range(start + 1, end + 1):
        body = lines[i].lstrip('\t ')
        if width(lines[i][:len(lines[i]) - len(body)]) == old_col:
            lines[i] = indent(new_col) + body
    return '\n'.join(lines)


def prototype(chunk, prefix=''):
    """Prototype of a function chunk that has lost static, broken into
    lines where its definition is."""
    lines = chunk.split('\n')
    start, end = sig_range(lines)
    sig = lines[start:end + 1]
    assert not is_static(chunk)
    if prefix:
        col = width(sig[0][:sig[0].index('(') + 1])
        sig[0] = prefix + sig[0]
        sig[1:] = [indent(col + len(prefix)) + l.lstrip('\t ')
                   if width(l[:len(l) - len(l.lstrip('\t '))]) == col else l for l in sig[1:]]
    sig[-1] += ';'
    assert all(width(l) <= 80 for l in sig), sig
    return '\n'.join(sig)


def is_static(chunk):
    lines = chunk.split('\n')
    start, _ = sig_range(lines)
    return re.match(r'(?:\[\[\w+\]\] )?static ', lines[start]) is not None


# Includes

def sys_includes(chunk_list):
    return [m for c in chunk_list for m in re.findall(r'#include <([\w/.]+)>', c)]


def includes(original, text, forced=()):
    """System headers of original that text uses, and forced ones."""
    keep = {h for h in original if h in SYMS and re.search(SYMS[h], code(text))}
    keep |= set(forced)
    unknown = [h for h in original if h not in SYMS]
    assert not unknown, unknown
    return sorted(keep)


def c_file(year, sys, local, chunks):
    parts = [SPDX.format(year)] if year else []
    if sys:
        parts.append('\n'.join(f'#include <{h}>' for h in sys))
    if local:
        parts.append('\n'.join(f'#include "{h}"' for h in local))
    return ctop.join(parts + chunks)


def h_file(year, label, guard, body):
    parts = [SPDX.format(year)] if year else []
    parts += [label, f'#ifndef {guard}\n#define {guard}'] + body + [f'#endif /* {guard} */']
    return ctop.join(parts)


def spdx_year(src):
    m = re.match(r'/\*\n \* SPDX-FileCopyrightText: (\d{4}) ', src)
    return m and m.group(1)


# The emulator

EMU = {
    'DEBUG': 'rv_emu', '#define DEBUG': 'rv_emu',
    'rv_one': 'rv_emu', 'rv_init': 'rv_emu', 'rv_emulate': 'rv_emu',
    'enum rv_opcode': 'exec', 'unsupported': 'exec', 'set_rd': 'exec',
    'enum rv_op': 'alu', 'enum rv_op_imm': 'alu', 'emu_div': 'alu', 'emu_rem': 'alu',
    'emu_r_type': 'alu', 'emu_i_type': 'alu', 'emu_op': 'alu', 'emu_op_imm': 'alu',
    'enum rv_width': 'mem', 'emu_load': 'mem', 'emu_s_type': 'mem', 'emu_store': 'mem',
    'enum rv_branch': 'control', 'emu_ucond_j': 'control', 'emu_jal': 'control',
    'emu_jalr': 'control', 'emu_b_type': 'control', 'emu_branch': 'control',
    'struct rv_insn': 'decode.h', 'rv_decode': 'decode',
    'imm_i': 'decode', 'imm_s': 'decode', 'imm_b': 'decode', 'imm_j': 'decode',
    'print_pct': 'analysis', 'rv_print': 'analysis',
}
EMU_FILES = ['rv_emu', 'decode', 'exec', 'alu', 'mem', 'control', 'analysis']
UNITS = ['alu', 'mem', 'control']


def split_one(chunk, n):
    """Split rv_one() into the fetch, which stays, and rv_exec()."""
    lines = chunk.split('\n')
    i = next(k for k, l in enumerate(lines) if l.startswith('\tswitch ('))
    assert lines[-1] == '}'
    before, after = lines[:i], lines[i:-1]
    assert before[0].startswith('static void rv_one(')
    before.insert(0, '/* Fetch and execute the instruction at pc */' if n < C2Y
                  else '/* Fetch, decode, and execute the instruction at pc */')
    if n < C2Y:
        op = '\tuint32_t opcode = get_bits(word, 0, 7);'
        k = before.index(op)
        assert before[k + 1] == ''
        del before[k:k + 2]
        one = before + ['\trv_exec(state, word);', '}']
        assert after[0] == '\tswitch (opcode) {'
        body = [op, ''] + after
        head = ['/* Execute word, leaving pc at the instruction to run next */',
                'void rv_exec(struct rv_state *state, uint32_t word)']
    else:
        assert after[0] == '\tswitch (auto insn = rv_decode(word); insn.opcode) {'
        one = before + ['\tauto insn = rv_decode(word);', '\trv_exec(state, &insn);', '}']
        body = ['\tswitch (insn->opcode) {'] + [
            l.replace('&insn', 'insn').replace('insn.opcode', 'insn->opcode') for l in after[1:]]
        head = ['/* Execute insn, leaving pc at the instruction to run next */',
                'void rv_exec(struct rv_state *state, const struct rv_insn *insn)']
    return '\n'.join(one), '\n'.join(head + ['{'] + body + ['}'])


def fold_emu(src, header, n):
    """Files of the emulator from rv_emu.c and rv_emu.h."""
    year = spdx_year(src)
    cs = ctop.chunks(src)
    assert '' not in cs, 'double blank line'
    if year:
        cs = cs[1:]
    sys = sys_includes([c for c in cs if ctop.name(c) == '#include'])
    files = {f: [] for f in EMU_FILES + ['decode.h']}
    for c in cs:
        name = ctop.name(c)
        if name == '#include':
            continue
        target = EMU[name]
        if name == 'rv_one':
            one, exe = split_one(c, n)
            files['rv_emu'].append(one)
            rv_exec = exe
            continue
        files[target].append(c)
    files['exec'].append(rv_exec)

    texts = {f: '\n\n'.join(files[f]) for f in files}

    def callers(name, home):
        return [f for f in texts if f != home and uses(texts[f], name)]

    # Functions that other files call lose static and get prototypes
    protos = {f: [] for f in EMU_FILES}
    for f in EMU_FILES:
        for k, c in enumerate(files[f]):
            if not ctop.is_function(c):
                continue
            name = ctop.name(c)
            if name == 'rv_exec' or callers(name, f):
                if is_static(c):
                    c = unstatic(c)
                    files[f][k] = c
                protos[f].append(prototype(c, '[[nodiscard]] ' if f == 'decode' else ''))
    texts = {f: '\n\n'.join(files[f]) for f in files}

    units = [u for u in UNITS if files[u]]
    out = {}
    emu_year = year and '2024'
    for f in EMU_FILES:
        if not files[f]:
            continue
        local = {
            'rv_emu': (['rv_emu.h'] if n >= GUARDS else []) + (['decode.h'] if n >= C2Y else []) + ['exec.h'],
            'decode': ['decode.h'],
            'analysis': ['analysis.h'],
        }.get(f, ['exec.h'])
        if uses(texts[f], 'get_bits') or uses(texts[f], 'get_bit'):
            local.append('bits.h')
        forced = ['stdint.h'] if f in UNITS + ['decode'] else []
        fyear = year if f == 'rv_emu' else ('2026' if f == 'decode' else emu_year)
        out[f'src/emu/{f}.c'] = c_file(fyear, includes(sys, texts[f], forced), local, files[f])

    exec_body = ['#include <stdint.h>',
                 ('#include "decode.h"\n' if n >= C2Y else '') + '#include "rv_emu.h"',
                 protos['exec'][-1]]
    assert protos['exec'][-1].startswith('void rv_exec(')
    if units:
        names = [f'{u}.c' for u in units]
        listed = names[0] if len(names) == 1 else (
            ' and '.join(names) if len(names) == 2 else ', '.join(names[:-1]) + ', and ' + names[-1])
        exec_body.append(f'/* Instructions by major opcode, in {listed} */\n' +
                         '\n'.join(p for u in units for p in protos[u]))
    if protos['exec'][:-1]:
        exec_body.append('/* Helpers of the instructions, in exec.c */\n' + '\n'.join(protos['exec'][:-1]))
    label = '/* Execution of decoded instructions */' if n >= C2Y else '/* Execution of instructions */'
    out['src/emu/exec.h'] = h_file(emu_year, label, 'EXEC_H', exec_body)

    if files['decode']:
        decode_protos = protos['decode']
        assert decode_protos[0].startswith('[[nodiscard]] struct rv_insn rv_decode(')
        body = ['#include <stdint.h>'] + files['decode.h'] + [
            decode_protos[0],
            '/* Sign-extended immediates of the I, S, B, and J formats */\n' + '\n'.join(decode_protos[1:])]
        out['src/emu/decode.h'] = h_file('2026' if year else None, '/* RISC-V instruction formats */',
                                         'DECODE_H', body)

    # struct rv_analysis and rv_print() move from rv_emu.h to analysis.h
    hcs = ctop.chunks(header)
    k = hcs.index('/* Dynamic analysis structs */')
    struct = hcs[k + 1]
    assert ctop.name(struct) == 'struct rv_analysis'
    del hcs[k:k + 2]
    print_proto = re.search(r'^void rv_print\(.*\);$', header, re.M).group()
    hcs = ['\n'.join(l for l in c.split('\n') if l != print_proto) for c in hcs]
    local = [i for i, c in enumerate(hcs) if c.startswith('#include "')]
    if local:
        (i,) = local
        hcs[i] = once(hcs[i], '#include "cache.h"', '#include "analysis.h"\n#include "cache/cache.h"')
    else:
        i = max(i for i, c in enumerate(hcs) if c.startswith('#include <'))
        hcs.insert(i + 1, '#include "analysis.h"')
    out['src/emu/rv_emu.h'] = '\n\n'.join(hcs) + '\n'
    struct = struct.replace('struct rv_analysis {', '/* Executed instructions by kind */\nstruct rv_analysis {', 1)
    out['src/emu/analysis.h'] = h_file(emu_year, '/* Dynamic instruction mix, which -a prints */',
                                       'ANALYSIS_H', [struct, print_proto])
    return out


# The front end and the harness

FRONT = {'verbose_on', 'analyze', 'parse_params', 'usage', 'main'}
HARNESS = {'copy_array', 'print_array', 'print_results', 'tests', 'find_test'}


def is_run(name):
    return name.endswith('_test') and name not in HARNESS

RUN_A = '''/*
 * Run the test program called name on the arguments in argv from
 * start_arg on, and return false if there is no such program.
 */
bool run_test(const char *name, int start_arg, int argc, char **argv,
	      struct rv_state *state)
{
%s
	return true;
}'''

RUN_B = '''/*
 * Run the test program called name with the argc arguments in argv, and
 * return false if there is no such program.
 */
bool run_test(const char *name, int argc, char **argv,
	      struct rv_state *state)
{
	if (const struct test *test = find_test(name)) {
		test->run(argc, argv, state);
		return true;
	}
	return false;
}'''

RUN_C = '''/*
 * Run the test program called name with the argc arguments in argv, and
 * return false if there is no such program or it rejects the arguments.
 */
bool run_test(const char *name, int argc, char **argv,
	      struct rv_state *state)
{
	if (const struct test *test = find_test(name);
	    test && argc >= test->min_args && argc <= test->max_args)
		return test->run(argc, argv, state);
	return false;
}'''

PRINT_TESTS = '''/* List the test programs and their arguments for the usage message */
void print_tests(FILE *stream)
{
%s
}'''


def accept(chunk):
    """Make a run return whether it accepted its arguments."""
    chunk = re.sub(r'^static void (\w+_test)\(', r'static bool \1(', chunk, count=1, flags=re.M)
    assert 'static bool ' in chunk, chunk[:80]
    chunk = chunk.replace('\t\tusage();\n', '\t\treturn false;\n')
    assert 'usage' not in code(chunk), chunk
    lines = chunk.split('\n')
    assert lines[-1] == '}'
    if lines[-2] == '':
        del lines[-2]
    return '\n'.join(lines[:-1] + ['\treturn true;', '}'])


def fold_front(src, n, max_array):
    year = spdx_year(src)
    cs = ctop.chunks(src)
    assert '' not in cs, 'double blank line'
    if year:
        cs = cs[1:]
    sys = sys_includes([c for c in cs if ctop.name(c) == '#include'])
    front, harness = {}, []
    for c in cs:
        name = ctop.name(c)
        if name == '#include':
            continue
        if name == 'usage' and not ctop.is_function(c):
            continue  # The runs no longer call usage() before its definition
        if name in FRONT:
            front[name] = c
        else:
            assert name in HARNESS or is_run(name), name
            harness.append(c)

    main, usage = front['main'], front['usage']
    if n < C2Y:
        m = re.search(r'^\tif \(!strcmp\(argv\[i\], "fib_rec"\)\)( \{)?\n(.*?)\n\t(\} )?else( \{)?\n\t\tusage\(\);\n(\t\}\n)?',
                      main, re.M | re.S)
        braces = m.group(1) is not None
        chain = m.group()
        new = ('\tif (!run_test(argv[i], i + 1, argc, argv, &state)) {\n\t\tusage();\n\t}\n' if braces
               else '\tif (!run_test(argv[i], i + 1, argc, argv, &state))\n\t\tusage();\n')
        main = once(main, chain, new)
        chain = chain.replace('argv[i]', 'name').replace('i + 1', 'start_arg').replace('&state', 'state')
        chain = once(chain, '\t\tusage();\n', '\t\treturn false;\n')
        harness.append(RUN_A % chain.rstrip('\n'))
    elif n < REJECT:
        main = once(main, '\tif (const struct test *test = find_test(argv[i]))\n'
                    '\t\ttest->run(argc - i - 1, &argv[i + 1], &state);\n\telse\n\t\tusage();\n',
                    '\tif (!run_test(argv[i], argc - i - 1, &argv[i + 1], &state))\n\t\tusage();\n')
        harness.append(RUN_B)
    else:
        main = once(main, '\tif (const struct test *test = i < argc ? find_test(argv[i]) : nullptr;\n'
                    '\t    test && nargs >= test->min_args && nargs <= test->max_args)\n'
                    '\t\ttest->run(nargs, &argv[i + 1], &state);\n\telse\n\t\tusage();\n',
                    '\tif (i >= argc || !run_test(argv[i], nargs, &argv[i + 1], &state))\n\t\tusage();\n')
        m = re.search(r'^\tfor \(size_t i = 0; i < countof\(tests\); i\+\+\) \{\n.*?^\t\}\n', usage, re.M | re.S)
        usage = once(usage, m.group(), '\tprint_tests(stderr);\n')
        harness = [accept(c) if is_run(ctop.name(c)) else c for c in harness]
        k = next(i for i, c in enumerate(harness) if ctop.name(c) == 'tests')
        harness[k] = once(harness[k], ' * a summary of their arguments and the range of their argument count.\n */',
                          ' * a summary of their arguments and the range of their argument count.\n'
                          ' * run() returns false, before it prints anything, if it rejects its\n'
                          ' * arguments.\n */')
        harness[k] = once(harness[k], '\tvoid (*run)(int argc, char **argv, struct rv_state *state);',
                          '\tbool (*run)(int argc, char **argv, struct rv_state *state);')
        harness.append(RUN_C)
        harness.append(PRINT_TESTS % m.group().rstrip('\n').replace('stderr', 'stream'))
    assert not uses(main, 'find_test') and not uses(usage, 'tests')

    harness = [max_array] + harness
    text = '\n\n'.join(harness)
    out = {}
    hyear = year and '2024'
    out['src/harness.c'] = c_file(hyear, includes(sys, text), ['harness.h', 'progs.h'], harness)

    front_chunks = [front[k] for k in ('verbose_on', 'analyze') if k in front]
    front_chunks += [usage, front['parse_params'], main]
    text = '\n\n'.join(front_chunks)
    local = ['harness.h'] + (['emu/rv_emu.h'] if n >= GUARDS else [])
    if '#include "verbose.h"' in src:
        local.append('verbose.h')
    out['src/rv64_emu.c'] = c_file(year, includes(sys, text), local, front_chunks)

    attr = '[[nodiscard]] ' if n >= C23 else ''
    params = ('const char *name, int start_arg, int argc, char **argv, struct rv_state *state'
              if n < C2Y else 'const char *name, int argc, char **argv, struct rv_state *state')
    body = (['#include <stdio.h>'] if n >= REJECT else []) + ['#include "emu/rv_emu.h"']
    protos = wrap_call(f'{attr}bool run_test({params});')
    if n >= REJECT:
        protos += '\nvoid print_tests(FILE *stream);'
    out['src/harness.h'] = h_file(hyear, '/* Runs of the test programs in C, in assembly, and in the emulator */',
                                  'HARNESS_H', body + [protos])
    return out


def fold_progs(src):
    """progs.h from rv64_emu.h, and the chunk of MAX_ARRAY."""
    cs = ctop.chunks(src)
    (k,) = [i for i, c in enumerate(cs) if 'MAX_ARRAY' in c]
    max_array = cs.pop(k)
    assert max_array in ('#define MAX_ARRAY 1024', 'constexpr int MAX_ARRAY = 1024;'), max_array
    return '\n\n'.join(cs).replace('RV64_EMU_H', 'PROGS_H') + '\n', max_array


def fold_makefile(src, n, has):
    src = once(src, '# Find the sources in src and the test programs in progs\nVPATH = src progs\n',
               '# Find the sources under src and the test programs in progs\nVPATH = src src/emu src/cache progs\n')
    src = once(src, ' -Isrc -c ', ' -Isrc -Iprogs -c ')
    objs = ['rv64_emu.o', 'harness.o', 'rv_emu.o'] + [f'{f}.o' for f in ('decode', 'exec', 'alu', 'mem', 'control')
                                                      if has(f'src/emu/{f}.c')] + ['analysis.o']
    old = 'OBJS = rv64_emu.o rv_emu.o bits.o cache.o verbose.o  \\\n'
    if old in src:
        words, lines, cur = objs + ['bits.o', 'cache.o', 'verbose.o'], [], ''
        for w in words:
            if cur and len(cur + ' ' + w) > 45:
                lines.append(cur)
                cur = w
            else:
                cur = (cur + ' ' + w).strip()
        lines.append(cur)
        new = ''.join(('OBJS = ' if i == 0 else '       ') + l.ljust(46) + '\\\n' for i, l in enumerate(lines))
        src = once(src, old, new)
    else:
        m = re.search(r'^OBJS = (\$\(TESTS:=_s\.o\)(?: insn_s\.o)?) rv64_emu\.o rv_emu\.o bits\.o cache\.o '
                      r'verbose\.o \\\n {7}\$\(TESTS:=_c\.o\)\n', src, re.M)
        words = m.group(1).split() + objs + ['bits.o', 'cache.o', 'verbose.o', '$(TESTS:=_c.o)']
        src = once(src, m.group(), wrap_words('OBJS = ', words, '       ') + '\n')
    m = re.search(r'^HEADERS = (.*)\n', src, re.M)
    headers = {'progs.h' if h == 'rv64_emu.h' else h for h in m.group(1).split()}
    headers |= {'analysis.h', 'exec.h', 'harness.h'} | ({'decode.h'} if has('src/emu/decode.h') else set())
    src = once(src, m.group(), wrap_words('HEADERS = ', sorted(headers), '\t') + '\n')
    return src


def fold(files, n):
    """Files of the nth commit in the modular layout."""
    out = {}
    progs_h, max_array = fold_progs(files['src/rv64_emu.h'])
    new = {}
    new.update(fold_emu(files['src/rv_emu.c'], files['src/rv_emu.h'], n))
    new.update(fold_front(files['src/rv64_emu.c'], n, max_array))
    new['progs/progs.h'] = progs_h
    moves = {'src/bits.c': 'src/emu/bits.c', 'src/bits.h': 'src/emu/bits.h',
             'src/cache.c': 'src/cache/cache.c', 'src/cache.h': 'src/cache/cache.h'}
    gone = {'src/rv_emu.c', 'src/rv_emu.h', 'src/rv64_emu.c', 'src/rv64_emu.h', 'Makefile'}
    for path, src in files.items():
        if path in gone:
            continue
        dst = moves.get(path, path)
        if path in ('src/cache.c', 'src/verbose.c') and '#include "rv_emu.h"' in src:
            src = once(src, '#include "rv_emu.h"', '#include "emu/rv_emu.h"')
        if re.match(r'progs/\w+_c\.c$', path) and '#include "rv64_emu.h"' in src:
            src = once(src, '#include "rv64_emu.h"', '#include "progs.h"')
        out[dst] = src
    for path, src in new.items():
        assert path not in out, path
        out[path] = src
    out['Makefile'] = fold_makefile(files['Makefile'], n, lambda p: p in out)
    return out
