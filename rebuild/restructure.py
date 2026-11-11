# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Rebuild master with fixes folded in, rebuild c2y on top of it as fewer
commits, and rename project04 to rv64_emu throughout.

Every tree is built explicitly from the trees of the original commits,
with transforms that assert they match exactly, so nothing can drift
silently.  Run in a scratch clone that has the original master and c2y.
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile

REPO = os.environ['REPO']
WORK = os.environ['WORK']
USER = ('Yiyu Zhou', 'yiyu@yiyuzhou.io')
# Directory with gcc and as for RISC-V, as the Makefiles of master expect
TOOLS = os.environ['TOOLS']


def git(*args, stdin=None, env=None, check=True):
    full = dict(os.environ)
    if env:
        full.update(env)
    res = subprocess.run(['git', '-C', REPO, *args], input=stdin,
                         capture_output=True, env=full)
    if check and res.returncode:
        sys.exit(f'git {" ".join(args)}: {res.stderr.decode()}')
    return res.stdout


def files_of(rev):
    out = {}
    for entry in filter(None, git('ls-tree', '-r', '-z', rev).decode().split('\0')):
        meta, path = entry.split('\t', 1)
        mode, kind, sha = meta.split()
        assert kind == 'blob', entry
        out[path] = (mode, git('cat-file', 'blob', sha).decode())
    return out


def text(files, path):
    return files[path][1]


def put(files, path, content, mode=None):
    old = files.get(path, ('100644', ''))
    files[path] = (mode or old[0], content)


def once(s, old, new):
    n = s.count(old)
    assert n == 1, (old[:80], n)
    return s.replace(old, new)


def write_tree(files):
    fd, index = tempfile.mkstemp()
    os.close(fd)
    os.unlink(index)
    env = {'GIT_INDEX_FILE': index}
    lines = []
    for path in sorted(files):
        mode, content = files[path]
        sha = git('hash-object', '-w', '--stdin', stdin=content.encode()).decode().strip()
        lines.append(f'{mode} {sha}\t{path}')
    git('update-index', '--add', '--index-info', stdin=('\n'.join(lines) + '\n').encode(), env=env)
    tree = git('write-tree', env=env).decode().strip()
    os.unlink(index)
    return tree


def header(rev):
    raw = git('cat-file', 'commit', rev).decode()
    head, msg = raw.split('\n\n', 1)
    h = {}
    for line in head.splitlines():
        k, v = line.split(' ', 1)
        h.setdefault(k, v)
    return h, msg


def ident(value):
    m = re.match(r'(.*) <(.*)> (\d+ [+-]\d{4})$', value)
    return m.groups()


def commit(tree, parents, msg, date_rev, keep_identity=False):
    h, _ = header(date_rev)
    a_name, a_mail, a_date = ident(h['author'])
    c_name, c_mail, c_date = ident(h['committer'])
    if not keep_identity:
        (a_name, a_mail), (c_name, c_mail) = USER, USER
    env = {'GIT_AUTHOR_NAME': a_name, 'GIT_AUTHOR_EMAIL': a_mail,
           'GIT_AUTHOR_DATE': a_date, 'GIT_COMMITTER_NAME': c_name,
           'GIT_COMMITTER_EMAIL': c_mail, 'GIT_COMMITTER_DATE': c_date}
    args = ['commit-tree', tree]
    for p in parents:
        args += ['-p', p]
    return git(*args, stdin=msg.encode(), env=env).decode().strip()


# ----------------------------------------------------------------------
# Folds of the emulator fixes, written against the code as master and the
# early c2y commits have it

DIVREM_REM = '''/*
 * Signed remainder as RISC-V defines it.  Unlike C, it defines the
 * result of dividing by zero and of dividing the most negative value by
 * -1, which overflows.
 */
static uint64_t emu_rem(int64_t dividend, int64_t divisor)
{
	if (divisor == 0)
		return dividend;
	if (dividend == INT64_MIN && divisor == -1)
		return 0;
	return dividend % divisor;
}

'''

DIVREM_BOTH = '''/*
 * Signed division and remainder as RISC-V defines them.  Unlike C, it
 * defines the results of dividing by zero and of dividing the most
 * negative value by -1, which overflows.
 */
static uint64_t emu_div(int64_t dividend, int64_t divisor)
{
	if (divisor == 0)
		return UINT64_MAX;
	if (dividend == INT64_MIN && divisor == -1)
		return dividend;
	return dividend / divisor;
}

static uint64_t emu_rem(int64_t dividend, int64_t divisor)
{
	if (divisor == 0)
		return dividend;
	if (dividend == INT64_MIN && divisor == -1)
		return 0;
	return dividend % divisor;
}

'''


def fold_x0(s):
    return once(s, '\tdefault:\n\t\tunsupported("Unknown opcode", opcode);\n\t}\n}\n',
                '\tdefault:\n\t\tunsupported("Unknown opcode", opcode);\n\t}\n\n'
                '\t/* x0 is hardwired to zero, so discard any write to it */\n'
                '\trsp->regs[RV_ZERO] = 0;\n}\n')


def fold_shifts(s, slli):
    s = once(s, 'rsp->regs[rd] = rsp->regs[rs1] << rsp->regs[rs2];',
             'rsp->regs[rd] = rsp->regs[rs1] << (rsp->regs[rs2] & 63);')
    s = once(s, 'rsp->regs[rd] = rsp->regs[rs1] >> rsp->regs[rs2];',
             'rsp->regs[rd] = rsp->regs[rs1] >> (rsp->regs[rs2] & 63);')
    s = once(s, '\tuint32_t shamt = get_bits(iw, 20, 5);\n'
                '\tuint32_t funct3 = get_bits(iw, 12, 3);\n'
                '\tuint32_t funct7 = get_bits(iw, 25, 7);\n',
             '\t/* RV64 shift amounts have six bits, leaving six bits of funct6 */\n'
             '\tuint32_t shamt = get_bits(iw, 20, 6);\n'
             '\tuint32_t funct3 = get_bits(iw, 12, 3);\n'
             '\tuint32_t funct6 = get_bits(iw, 26, 6);\n')
    if 'funct3 == 0b101 && funct7 == 0b0' in s:
        s = once(s, 'funct3 == 0b101 && funct7 == 0b0', 'funct3 == 0b101 && funct6 == 0b0')
        if slli:
            s = once(s, 'funct3 == 0b001 && funct7 == 0b0', 'funct3 == 0b001 && funct6 == 0b0')
    else:
        s = once(s, 'funct3 == OP_IMM_SRLI && funct7 == 0', 'funct3 == OP_IMM_SRLI && funct6 == 0')
        s = once(s, 'funct3 == OP_IMM_SLLI && funct7 == 0', 'funct3 == OP_IMM_SLLI && funct6 == 0')
    assert 'funct7 == 0' not in s.split('void emu_i_type')[1].split('\n}\n')[0]
    return s


def fold_jalr(s):
    return once(s, '\tuint32_t rs1 = get_bits(iw, 15, 5);  /* Will be ra (aka x1) */\n'
                   '\tuint64_t val = rsp->regs[rs1];  /* Value of regs[1] */\n\n'
                   '\trsp->pc = val;  /* PC = return address */\n',
                '\tuint32_t rd = get_bits(iw, 7, 5);\n'
                '\tuint32_t rs1 = get_bits(iw, 15, 5);\n'
                '\tuint32_t imm = get_bits(iw, 20, 12);\n'
                '\tuint64_t target = (rsp->regs[rs1] + sign_extend(imm, 12)) & ~(uint64_t)1;\n\n'
                '\t/* Compute the target before writing rd, which may be rs1 */\n'
                '\trsp->regs[rd] = rsp->pc + 4;\n'
                '\trsp->pc = target;\n')


def fold_store_offset(s):
    return once(s, '\tuint32_t funct3 = get_bits(iw, 12, 3);\n'
                   '\tuint64_t offset_rs1 = (uint64_t)((uint8_t *)rsp->regs[rs1] + imm);\n',
                '\tuint32_t funct3 = get_bits(iw, 12, 3);\n'
                '\tint64_t offset = sign_extend(imm, 12);\n'
                '\tuint64_t offset_rs1 = (uint64_t)((uint8_t *)rsp->regs[rs1] + offset);\n')


def fold_load(s):
    s = once(s, '\tuint32_t imm = get_bits(iw, 20, 12);\n'
                '\tuint64_t offset_rs1 = (uint64_t)((uint8_t *)rsp->regs[rs1] + imm);\n',
             '\tuint32_t imm = get_bits(iw, 20, 12);\n'
             '\tint64_t offset = sign_extend(imm, 12);\n'
             '\tuint64_t offset_rs1 = (uint64_t)((uint8_t *)rsp->regs[rs1] + offset);\n')
    return once(s, 'rsp->regs[rd] = *((uint8_t *)offset_rs1);',
                'rsp->regs[rd] = *((int8_t *)offset_rs1);')


def fold_lw(s):
    return once(s, 'rsp->regs[rd] = *((uint32_t *)offset_rs1);',
                'rsp->regs[rd] = *((int32_t *)offset_rs1);')


def fold_rem(s, with_div):
    s = once(s, 'void emu_r_type(struct rv_state *rsp, uint32_t iw)\n',
             (DIVREM_BOTH if with_div else DIVREM_REM) +
             'void emu_r_type(struct rv_state *rsp, uint32_t iw)\n')
    s = once(s, 'rsp->regs[rd] = rsp->regs[rs1] % rsp->regs[rs2];',
             'rsp->regs[rd] = emu_rem(rsp->regs[rs1], rsp->regs[rs2]);')
    if with_div:
        s = once(s, 'rsp->regs[rd] = rsp->regs[rs1] / rsp->regs[rs2];',
                 'rsp->regs[rd] = emu_div(rsp->regs[rs1], rsp->regs[rs2]);')
    return s


def fold_ways(s):
    return once(s, '\tfor (int i = 0; i < 4; i += 1) {', '\tfor (int i = 0; i < csp->ways; i += 1) {')


# Master commits are numbered from 1 for the starter code
M_INSTR, M_SBSD, M_LBLD, M_REM, M_DIV, M_LW, M_SLLI, M_LRU = 4, 5, 7, 9, 10, 12, 14, 22


def fold_rv_emu(s, k):
    """Fold the fixes into rv_emu.c of master commit k, or of a later tree
    with k at the end, which still has the master style of decoding."""
    if k >= M_INSTR:
        s = fold_x0(s)
        s = fold_shifts(s, slli=k >= M_SLLI)
        s = fold_jalr(s)
    if k >= M_SBSD:
        s = fold_store_offset(s)
    if k >= M_LBLD:
        s = fold_load(s)
    if k >= M_REM:
        s = fold_rem(s, with_div=k >= M_DIV)
    if k >= M_LW:
        s = fold_lw(s)
    return s


MASTER_MSGS = {
    1: '''treewide: import the cs315 starter code

Import the starter code for the RISC-V emulator project of CS 315
(Computer Architecture) at the University of San Francisco, as
distributed through GitHub Classroom, with the program renamed to
rv64_emu.  It provides the test harness, the C and RISC-V assembly
test programs, bit manipulation helpers, a skeleton of the instruction
cache simulator, and an emulator loop that rejects every opcode.
''',
    M_INSTR: '''rv_emu: emulate arithmetic, jalr and branch instructions

Dispatch on the opcode in rv_one() and emulate the first instructions
the test programs need:

  R-type  add, sub, mul, sll, srl, and
  I-type  addi, srli
  jalr
  B-type  blt, bne

Shifts take six bits of their amount, as RV64 does, jalr links into rd
and jumps to rs1 plus its offset, and rv_one() clears x0 after every
instruction, since writes to it are discarded.
''',
    M_SBSD: '''rv_emu: emulate sb and sd

Add the offset to the base register sign-extended, since it may be
negative.
''',
    M_LBLD: '''rv_emu: emulate lb and ld

Sign-extend both the loaded byte and the offset, which may be negative.
''',
    M_REM: '''rv_emu: emulate rem

Compute the remainder of signed values, and return what RISC-V defines
for a zero divisor and for overflow, which C leaves undefined.
''',
    M_DIV: '''rv_emu: emulate div

Divide signed values, and return what RISC-V defines for a zero divisor
and for overflow, which C leaves undefined.
''',
    M_LW: '''rv_emu: emulate lw

Sign-extend the loaded word, as lw loads a signed value.
''',
    M_SLLI: '''rv_emu: emulate slli

Like srli, slli takes a six-bit shift amount.
''',
    M_LRU: '''cache: use lru replacement in the set-associative cache

On a miss in a full set, evict the least recently used slot instead of
always the first one, and refill the whole block as the direct-mapped
cache does.  Take the word offset within the block from the address,
and search the configured number of ways, rather than hard-coding them.
''',
}

README_MSG = '''readme: describe the project

Say what the emulator does, how it compares the C, native and emulated
versions of the test programs, and how to build and run it.
'''


# ----------------------------------------------------------------------
# Rename of project04 to rv64_emu, applied to every tree and message last

def rename_content(s):
    s = s.replace('OBJS = project04.o rv_emu.o bits.o cache.o verbose.o \\\n',
                  'OBJS = rv64_emu.o rv_emu.o bits.o cache.o verbose.o  \\\n')
    s = s.replace('"                 <prog> [<arg> ...]\\n"',
                  '"                <prog> [<arg> ...]\\n"')
    s = s.replace('\n                 <prog> [<arg> ...]\n',
                  '\n                <prog> [<arg> ...]\n')
    s = s.replace('PROJECT04_H', 'RV64_EMU_H')
    s = s.replace('project04', 'rv64_emu')
    return s.replace('rv64-emu', 'rv64_emu')


def rename_files(files):
    out = {}
    for path, (mode, content) in files.items():
        new = {'project04.c': 'rv64_emu.c', 'project04.h': 'rv64_emu.h'}.get(path, path)
        out[new] = (mode, rename_content(content))
    return out


def rename_msg(msg):
    assert 'project04 of CS 315' not in msg and 'cs315 project04' not in msg, msg
    return msg.replace('project04', 'rv64_emu').replace('rv64-emu', 'rv64_emu')


# ----------------------------------------------------------------------

def master_readme():
    """The short README of master, before the rename."""
    old = git('rev-list', '--reverse', 'origin/master').decode().split()
    readme = text(files_of(old[27]), 'README.md')
    return once(readme, 'The `c2y` branch has a reworked version with bug fixes and a test\nsuite.\n',
                'The `c2y` branch reworks it in C2y and adds a test suite.\n')


def build_master():
    old = git('rev-list', '--reverse', 'origin/master').decode().split()
    assert len(old) == 28, len(old)
    new, parent, tips = {}, None, {}
    for k, rev in enumerate(old[:26], start=1):
        files = files_of(rev)
        put(files, 'rv_emu.c', fold_rv_emu(text(files, 'rv_emu.c'), k))
        if k >= M_LRU:
            put(files, 'cache.c', fold_ways(text(files, 'cache.c')))
        msg = MASTER_MSGS.get(k) or header(rev)[1]
        files = rename_files(files)
        parent = commit(write_tree(files), [parent] if parent else [],
                        rename_msg(msg), rev, keep_identity=(k == 1))
        new[k] = parent
        tips[k] = files
    files = dict(tips[26])
    put(files, 'README.md', rename_content(master_readme()))
    tips[27] = files
    parent = commit(write_tree(files), [parent], README_MSG, old[26])
    new[27] = parent
    git('update-ref', 'refs/heads/new-master', parent)
    return new, tips




# ----------------------------------------------------------------------
# c2y, rebuilt on the new master as fewer commits

def c2y_trees():
    base = git('rev-parse', 'origin/master~1').decode().strip()
    revs = git('rev-list', '--reverse', f'{base}..origin/c2y').decode().split()
    assert len(revs) == 72, len(revs)
    cache = {}

    def T(k):
        if k not in cache:
            cache[k] = files_of(revs[k - 1])
        return cache[k]
    return revs, T


ENC_START = '/*\n * Major opcodes.'
ENC_END = '\tBRANCH_GE = 0b101,\n};\n\n'


def enc_block(h):
    start = h.index(ENC_START)
    end = h.index(ENC_END, start) + len(ENC_END)
    return start, end


def remove_enc(h):
    start, end = enc_block(h)
    return h[:start] + h[end:]


def insert_enc(c, h):
    start, end = enc_block(h)
    return once(c, 'constexpr bool DEBUG = false;\n', h[start:end] + 'constexpr bool DEBUG = false;\n')


def unrev29_rv_emu(s):
    s = once(s, 'static void unsupported(const char *s, uint32_t n)', 'static void unsupported(char *s, uint32_t n)')
    s = once(s, 'static void print_pct(const char *fmt, int numer, int denom)',
             'static void print_pct(char *fmt, int numer, int denom)')
    return once(s, 'void rv_print(const struct rv_analysis *a)', 'void rv_print(struct rv_analysis *a)')


def unrev31_rv_emu(s):
    s = once(s, '(double)numer / (double)denom * 100.0;', '(double)numer / (double)denom *100.0;')
    s, n = re.subn(r'^\t\t  (a->i_count|b_total)\);$', '\t\t\t\t\t\t\t    \\1);', s, flags=re.M)
    assert n == 7, n
    return s


def unrev29_cache(s):
    return once(s, 'void cache_print(const struct cache *csp, const char *name)',
                'void cache_print(struct cache *csp, char *name)')


def apply71_rv_emu_h(s):
    return once(s, 'void rv_init(struct rv_state *state, uint32_t *target,',
                'void rv_init(struct rv_state *state, const uint32_t *target,')


def strip_check(mk):
    mk = once(mk, 'RUN =\n', '')
    mk = once(mk, "check: $(PROG)\n\tRUN='$(RUN)' tests/run.sh ./$(PROG)\n\n", '')
    return once(mk, '.PHONY: check clean\n', '.PHONY: clean\n')


def rv64g(mk):
    if 'ASFLAGS = -g -march=rv64g\n' in mk:
        return mk
    return once(mk, 'ASFLAGS = -g\n', 'ASFLAGS = -g -march=rv64g\n')


def self_checks(T):
    assert unrev29_rv_emu(T(29)['rv_emu.c'][1]) == T(28)['rv_emu.c'][1]
    assert unrev31_rv_emu(T(31)['rv_emu.c'][1]) == T(30)['rv_emu.c'][1]
    assert unrev29_cache(T(29)['cache.c'][1]) == T(28)['cache.c'][1]
    assert apply71_rv_emu_h(T(70)['rv_emu.h'][1]) == T(71)['rv_emu.h'][1]
    assert remove_enc(T(42)['rv_emu.h'][1]) == T(43)['rv_emu.h'][1]
    assert insert_enc(T(42)['rv_emu.c'][1], T(42)['rv_emu.h'][1]) == T(43)['rv_emu.c'][1]
    for k in (13, 14, 19, 24, 31, 39):
        h = T(k)['rv_emu.h'][1]
        s, e = enc_block(h)
        assert h[s:e] == T(42)['rv_emu.h'][1][slice(*enc_block(T(42)['rv_emu.h'][1]))], k
    # rv_emu.c is untouched between these commits, so they can share
    assert T(19)['rv_emu.c'] == T(24)['rv_emu.c'] == T(28)['rv_emu.c']
    assert T(13)['rv_emu.c'] == T(14)['rv_emu.c']
    assert T(4)['rv_emu.c'][1] == files_of('origin/master~2')['rv_emu.c'][1]


def common(files, T, k):
    files['README.md'] = ('100644', master_readme())
    files['tests/run.sh'] = T(55)['tests/run.sh']
    files['.gitattributes'] = T(41)['.gitattributes']
    files['.gitignore'] = T(47)['.gitignore']
    mk = T(25)['Makefile'][1] if k < 25 else files['Makefile'][1]
    put(files, 'Makefile', rv64g(mk))
    if 10 <= k < 43:
        h = files['rv_emu.h'][1]
        put(files, 'rv_emu.c', insert_enc(files['rv_emu.c'][1], h))
        put(files, 'rv_emu.h', remove_enc(h))
    return files


def groups(T):
    master_rv_emu = fold_rv_emu(T(4)['rv_emu.c'][1], 26)
    decode_pre29 = unrev29_rv_emu(unrev31_rv_emu(T(39)['rv_emu.c'][1]))

    def g1():
        f = dict(T(4))
        for p in [p for p in f if p.startswith('tests/')]:
            del f[p]
        put(f, 'Makefile', rv64g(strip_check(f['Makefile'][1])))
        f['.gitignore'] = T(47)['.gitignore']
        f['README.md'] = ('100644', master_readme())
        put(f, 'rv_emu.c', master_rv_emu)
        put(f, 'cache.c', fold_ways(f['cache.c'][1]))
        return f

    def g2():
        f = g1()
        f['tests/run.sh'] = T(55)['tests/run.sh']
        f['.gitattributes'] = T(41)['.gitattributes']
        put(f, 'Makefile', rv64g(T(25)['Makefile'][1]))
        return f

    def at(k, rv_emu=None, cache=None):
        def build():
            f = dict(T(k))
            if rv_emu is not None:
                put(f, 'rv_emu.c', rv_emu(f))
            if cache is not None:
                put(f, 'cache.c', cache(f))
            return common(f, T, k)
        return build

    def g11():
        f = common(dict(T(46)), T, 46)
        f['rv_emu.c'] = T(71)['rv_emu.c']
        put(f, 'rv_emu.h', apply71_rv_emu_h(f['rv_emu.h'][1]))
        return f

    def g12():
        f = g11()
        f['cache.c'] = T(70)['cache.c']
        return f

    def g13():
        f = g12()
        f['project04.c'] = T(69)['project04.c']
        f['rv_emu.h'] = T(71)['rv_emu.h']
        return f

    def g14():
        return common(dict(T(71)), T, 71)

    def g15():
        return dict(T(72))

    enum_rv_emu = lambda f: fold_rv_emu(f['rv_emu.c'][1], 26)
    loop_ways = lambda f: fold_ways(f['cache.c'][1])
    return [
        # name, builder, date from, original commits whose test cases it adds
        ('C1', g1, 1, []),
        ('C2', g2, 2, [2, 35]),
        ('C3', at(13, enum_rv_emu, loop_ways), 5, []),
        ('C4', at(14, enum_rv_emu, loop_ways), 14, []),
        ('C5', at(19, lambda f: decode_pre29, loop_ways), 15, []),
        ('C6', at(24, lambda f: decode_pre29, lambda f: unrev29_cache(T(35)['cache.c'][1])), 20, []),
        ('C7', at(31, lambda f: T(39)['rv_emu.c'][1], lambda f: T(35)['cache.c'][1]), 26, []),
        ('C8', at(39), 36, [36]),
        ('C9', at(42), 40, [40, 42]),
        ('C10', at(44), 44, []),
        ('C11', g11, 45, [58]),
        ('C12', g12, 49, []),
        ('C13', g13, 51, [51, 57, 63]),
        ('C14', g14, 59, [66]),
        ('C15', g15, 72, []),
    ]


def case_intro(T):
    final = T(72)['tests/cases'][1].rstrip('\n').split('\n')
    intro = {}
    for k in range(2, 73):
        for c in T(k)['tests/cases'][1].rstrip('\n').split('\n'):
            intro.setdefault(c, k)
    return final, intro


def regenerate(files):
    """Build the renamed tree and rewrite tests/expected from its output."""
    shutil.rmtree(WORK, ignore_errors=True)
    for path, (mode, content) in files.items():
        dst = os.path.join(WORK, path)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        with open(dst, 'w') as fp:
            fp.write(content)
        os.chmod(dst, 0o755 if mode == '100755' else 0o644)
    env = dict(os.environ, PATH=TOOLS + ':' + os.environ['PATH'])
    build = subprocess.run(['make', '-s', '-C', WORK, 'CC=gcc', 'AS=as'],
                           capture_output=True, text=True, env=env)
    warnings = build.stdout.count('warning:') + build.stderr.count('warning:')
    if build.returncode:
        sys.exit(f'build failed:\n{build.stdout}{build.stderr}')
    if 'tests/cases' in files:
        env['RUN'] = 'qemu-riscv64'
        subprocess.run(['sh', '-c', 'ulimit -c 0; tests/run.sh -u ./rv64_emu'],
                       cwd=WORK, check=True, env=env)
        files['tests/expected'] = ('100644', open(os.path.join(WORK, 'tests/expected')).read())
    return warnings


def blocks(expected):
    out, cur, case = {}, [], None
    for line in expected.splitlines(keepends=True):
        if line.startswith('$ '):
            case, cur = line[2:].rstrip('\n'), []
        elif line.startswith('[exit '):
            out[case] = ''.join(cur) + line
        else:
            cur.append(line)
    return out


C2Y_MSGS = {
    'C1': 'build: compile as c2y with a configurable toolchain\n\nInvoke the compiler and the assembler through CC and AS and pass\nCFLAGS, ASFLAGS and LDFLAGS, so that a cross toolchain can be selected\non the command line:\n\n    make CC=riscv64-linux-gnu-gcc AS=riscv64-linux-gnu-as\n\nCompile the C sources as C2y with -Wall and -Wextra, and fix what they\nreport: sort_c() returned int against its prototype, char_to_digit()\nfell off its end for characters that are not digits, and\ncache_lookup_dm() and str_to_int_test() had an unused variable and\nparameter.  Assemble with -march=rv64g, since the emulator only decodes\n32-bit instructions and an rv64gc default would compress them, and\nignore the built program in git.\n',
    'C2': 'tests: add golden-output regression tests\n\nRun the program once for each of 371 argument lists, which cover every\ntest program without a cache, with -a, and with direct-mapped and 2-,\n4- and 8-way set-associative caches, and compare the combined output\nand exit statuses with tests/expected.  make check runs the suite, RUN\nselects an emulator such as qemu-riscv64 for cross builds, and a run\nthat hangs is killed after TIMEOUT seconds.\n\nThe cache statistics of sort depend on how far apart the linker puts\nsort_s and find_max_index_s, which depended on every C object linked\nbetween them, so link all assembly objects first and next to each\nother.\n',
    'C3': 'treewide: adopt c23 features\n\nUse what C23 offers in place of older idioms:\n\n  - bool, true and false as keywords instead of <stdbool.h>\n  - nullptr instead of NULL\n  - constexpr objects instead of constant macros, including DEBUG,\n    which turns the conditional debug print into an if statement\n  - enums with fixed underlying types for the ABI registers and the\n    instruction encodings, which replace the last typedef and group\n    the opcode bits with digit separators as the opcode map of the\n    specification does\n  - [[noreturn]], [[nodiscard]] and [[unsequenced]], and [[gnu::format]]\n    on verbose(), which found uint64_t values printed with %d and %X,\n    now printed with %w64u and %w64X\n  - empty initializers instead of memset() and per-field resets, which\n    also initializes the emulator state in main()\n\nOP instructions are dispatched on funct7 and funct3 together, which\nturns the nested if-else chains in emu_r_type() into one switch and\nreports an unsupported one by its whole word.  The encodings live in\nrv_emu.c, since only the emulator decodes instructions.\n',
    'C4': 'rv_emu: align the emulated stack to 16 bytes\n\nThe RISC-V calling convention keeps sp 16-byte aligned, but the stack\narray only had the 8-byte alignment of the registers before it, so\nrv_init() started the emulated programs with sp misaligned by 8.\nDeclare the array with the C23 alignas keyword and check with a\nstatic_assert that its size keeps the top of the stack aligned as well.\n',
    'C5': 'treewide: adopt c2y features\n\nUse countof from <stdcountof.h> for the lengths of arrays, a switch\nstatement that declares the decoded instruction in rv_one(), and an if\nstatement that declares the selected test program in main().\n\nDecode the fields that all instruction formats share once into struct\nrv_insn instead of extracting them again in every emulation function.\nThe fields are unsigned _BitInt types as wide as the fields, so the\nconversions in rv_decode() do the masking, and the immediates are\nsign-extended by conversions to signed _BitInt types of their width.\nBit-precise types are not promoted, so funct7 is widened before it is\nshifted into the OP key.  Each emulation function computes its result\nand writes it through set_rd(), which discards writes to x0, so\nrv_one() no longer clears x0 afterwards, and the functions are named\nafter their major opcodes.\n\nCount the index and offset bits of the cache with stdc_trailing_ones()\nfrom <stdbit.h>, and look the test programs up in a table, which gives\nall of them one signature.\n',
    'C6': "cache: restructure the simulator\n\nRename evict_data_in_cache_block() to cache_fill_block(), since it\nfills a slot rather than evicting anything, and fix its indentation.\nReturn from a hit directly in both lookups, search a set with\ncache_find_tag(), cache_find_invalid() and cache_find_lru(), and scope\nthe set-associative hit with an if statement that declares it.\n\nStore the valid flag as bool, explain the padding of the '% Used' line\ninstead of blaming the test cases for it, drop verbose_cache(), the\nslots_used field and the verbose field of struct rv_state, which\nnothing used, and declare g_verbose next to verbose().\n",
    'C7': 'treewide: tidy up the headers, prototypes and parameters\n\nAdd include guards to the headers, include rv64_emu.h in the C test\nprograms so that their definitions are checked against the prototypes\nthat the harness calls them through, and correct the four that\ndisagreed.  Mark the pointer parameters that are only read const,\nclassify the digits in char_to_digit() with C2y case ranges, end the\noption loop in its condition, and align the continuation lines in\nrv_print().\n',
    'C8': 'tests: exercise instruction corner cases with insn_s\n\nAdd insn_s, an assembly test program for the corner cases that the\nother programs do not reach: loads and stores through negative offsets,\njalr through an odd offset and with rd equal to rs1, shifts by 32 or\nmore, and writes to x0.  It stores one result per case into an array.\nIt has no C version, so its test compares the native results with the\nemulated ones only, and it is linked after the other assembly programs\nso that it does not move them.\n',
    'C9': 'rv64_emu: validate the command line and the cache geometry\n\nA missing program name, a missing argument of a test program, or a\nmissing value of -dm or -sa made rv64_emu dereference a null argv\nentry and crash, int_to_str crashed unless -o was given although it\ndefaults to base 10, and unknown options were ignored.  Record the\narguments and the range of argument counts of each test program in the\ntable, check them in the condition of the if statement that declares\nthe program, check the option values, and print the options and the\nprograms to stderr when something is wrong.\n\ncache_init() derives its masks on the assumption that the block size\nand the number of sets are powers of two, and the slots and blocks\nhave fixed sizes, but nothing checked the sizes given to -dm and -sa.\nReject the geometries that cache_valid() cannot model, using\nstdc_has_single_bit() for the powers of two, and count the used slots\nover the slots instead of over the words.\n',
    'C10': 'treewide: give the cache and the logging their own headers\n\nrv_emu.h declared the cache simulator and the logging functions next to\nthe emulator, so cache.c and verbose.c had to include the whole\nemulator interface to get at their own declarations.  Move the cache\ntypes and functions into cache.h and the logging into verbose.h, and\ninclude only what each file uses.  The Makefile now lists every header\nas a dependency, including bits.h, which it missed before.\n',
    'C11': 'rv_emu: harden the emulator loop\n\nClear all registers in rv_init() instead of only the ones it sets, and\nstop with an error when sp moves below the emulated stack, before a\nstore through it can overwrite the registers that lie below.  Report\nunsupported instructions on stderr after flushing stdout, so that the\nresults of the runs before come out first, and report unsupported\nOP-IMM instructions by their word, as OP ones are.  Take the code to\nemulate through a const pointer, and build the mask in get_bits() from\na 64-bit one, so that it does not overflow for 31 bits and more.\n',
    'C12': 'cache: simplify the lookups and report the usage in slots\n\nSplit an address into its tag, index and word offset in one place,\nreturn the result of each lookup straight from the dispatch, include\nthe block offset bits in the evicted address that -v logs, and brace\nthe loops whose body is an if statement.\n\ncache_print() labeled the size as slots, although it counts words, and\ndivided the valid slots by that word count, so a full cache with\nfour-word blocks showed 25% used.  Label the size as words and divide\nthe valid slots by the number of slots.\n',
    'C13': 'rv64_emu: check more arguments and tidy up the harness\n\nReject bases other than 2, 10 and 16 for int_to_str, which loops\nforever for bases 0 and 1, strings that do not fit the buffer of\nto_upper, and str_to_int strings that are not numbers in their base,\non which the C and assembly versions disagree.  Keep the -a flag in\nrv64_emu.c instead of struct rv_state, tell whether a cache is\nconfigured from its type, align the parameters of the test functions,\nand brace the loop in find_test().\n',
    'C14': 'treewide: fix and tidy up the test programs\n\nint_to_str_s terminated the string after the middle of the reversed\ndigits, which cut off every number with more than two digits, and\nwrote no digit for zero.  It also divided the sign-extended register\nwhile int_to_str_c() converts an unsigned 32-bit value, and the two\nwrote hexadecimal digits in different cases.  Terminate the digits\nbefore reversing them, zero-extend the value, and write uppercase\ndigits in both versions.\n\nget_bitseq_c() shifted the int constant 1 by the length of the\nsequence, which overflows for 31 bits, so shift an unsigned one.  Pick\nthe digit characters in int_to_str_c() with case ranges, and follow\nthe brace rules of the kernel coding style in all C test programs.\n',
    'C15': 'readme: describe building and testing the c2y version\n\nSay that the code needs a C2y toolchain, how to cross-compile it and\nrun the regression tests under QEMU, and that insn_s only exists in\nassembly.\n',
}


def build_c2y(master_tip):
    revs, T = c2y_trees()
    self_checks(T)
    final, intro = case_intro(T)
    added = set()
    parent, made = master_tip, []
    for name, builder, date_k, case_ks in groups(T):
        files = builder()
        added |= set(case_ks)
        # Every commit from the one that adds the tests on has test cases
        if name != 'C1':
            cases = [c for c in final if intro[c] in added]
            files['tests/cases'] = ('100644', '\n'.join(cases) + '\n')
        files = rename_files(files)
        warnings = regenerate(files)
        anchor = {'C1': 4, 'C2': 4, 'C3': 13, 'C4': 14, 'C5': 19, 'C6': 24, 'C7': 31,
                  'C8': 39, 'C9': 42, 'C10': 44, 'C11': 46, 'C12': 46, 'C13': 46,
                  'C14': 71, 'C15': 72}[name]
        report = ''
        if 'tests/expected' in T(anchor) and 'tests/expected' in files:
            orig = rename_content(T(anchor)['tests/expected'][1])
            a, b = blocks(orig), blocks(files['tests/expected'][1])
            diff = [c for c in b if c in a and a[c] != b[c]]
            report = f'{len(b)} cases, {len(diff)} differ from original {anchor}'
            if diff:
                report += ': ' + ', '.join(diff[:6]) + (' ...' if len(diff) > 6 else '')
        msg = C2Y_MSGS[name]
        parent = commit(write_tree(files), [parent], rename_msg(msg), revs[date_k - 1])
        made.append(parent)
        print(f'{name:4} {parent[:7]} warnings={warnings} {report}', flush=True)
    git('update-ref', 'refs/heads/new-c2y', parent)
    final_tree = git('rev-parse', 'new-c2y^{tree}').decode().strip()
    want = write_tree(rename_files(dict(T(72))))
    print('final c2y tree matches the original tip:', final_tree == want)
    return made


if __name__ == '__main__':
    new, tips = build_master()
    print('new master', new[27], flush=True)
    build_c2y(new[27])
