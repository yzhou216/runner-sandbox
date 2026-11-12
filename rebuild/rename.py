# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Renames of the variables, struct members, and struct tags of rv64_emu.

GLOBAL applies to every identifier of that name in every C file: members,
tags, and variables whose name means the same thing everywhere.  SCOPED
applies to the ordinary identifiers, not members, in the parameter list
or the body of the named function, wherever it is declared or defined.
rename() refuses a rename whose new name the scope already uses.
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ctok  # noqa: E402

GLOBAL = {
    # Pointers named after their type
    'rsp': 'state',
    'csp': 'cache',
    'block_base_iw_ptr': 'src',
    # Struct tags with a suffix that says struct
    'cache_st': 'cache',
    'cache_slot_st': 'cache_slot',
    # Abbreviations
    'iw': 'word',
    'i_cache': 'icache',
    'i_count': 'insns',
    'ir_count': 'alu',
    'ld_count': 'loads',
    'st_count': 'stores',
    'j_count': 'jumps',
    'b_taken': 'taken',
    'b_not_taken': 'not_taken',
    'cache_sim': 'simulate_cache',
    'CONV_MAX_CHARS': 'MAX_DIGITS',
    # Scope in the name
    'g_verbose': 'verbose_on',
    # Signedness in the name
    'signed_imm': 'offset',
    'signed_rs1_val': 'rs1_val',
    'signed_rs2_val': 'rs2_val',
    # Cache addressing
    'b_index': 'offset',
    'b_base': 'base',
    'offset_rs1': 'addr',
}

TESTS = ['fib_rec_test', 'get_bitseq_test', 'pal_test', 'max3_test',
         'midpoint_test', 'quadratic_test', 'str_to_int_test']
SCOPED = {
    'rv_one': {'in': 'insn'},
    'unsupported': {'s': 'what', 'n': 'value'},
    'print_pct': {'numer': 'count', 'denom': 'total', 'pct': 'percent'},
    'rv_print': {'a': 'analysis', 'b_total': 'branches'},
    'get_bits': {'num': 'value'},
    'get_bit': {'num': 'value'},
    'sign_extend': {'num': 'value', 'start': 'width', 'dist': 'shift',
                    'shifted_bits': 'shifted', 'signed_bits': 'extended'},
    'cache_print': {'num_slots_used': 'used'},
    'find_lru_slot_in_set': {'low': 'lru'},
    'cache_lookup_sa': {'slot_found': 'found', 'slot_invalid': 'invalid'},
    'int_to_str_c': {'result_str': 'out', 'output_str': 'out', 'obase': 'base',
                     'div': 'quotient', 'rem': 'digit', 'tmp': 'digits',
                     'val': 'value'},
    'int_to_str_s': {'output_str': 'out', 'val': 'value'},
    'int_to_str_test': {'output_str': 'out'},
    'get_bitseq_c': {'num': 'value', 'n': 'value', 'val': 'bits'},
    'get_bitseq_s': {'n': 'value'},
    'pal_rec_c': {'s': 'str'},
    'pal_rec_s': {'s': 'str'},
    'find_max_index_c': {'arr': 'values', 'max_val': 'max'},
    'sort_c': {'arr': 'values', 'idx': 'max'},
    'sort_s': {'arr': 'values'},
    'print_array': {'arr': 'values'},
    'sort_test': {'tmp': 'sorted', 'a': 'values'},
    'to_upper_c': {'orig': 'src'},
    'to_upper_test': {'tmp': 'out'},
    'str_to_int_c': {'retval': 'value', 'place_val': 'place'},
    **{name: {'r': 'result'} for name in TESTS},
}
# The parameter of every function that takes a decoded instruction
INSN_FUNCTIONS = re.compile(r'(imm_[isbj]|set_rd|emu_[a-z_]+)$')


def scoped(scope):
    renames = dict(SCOPED.get(scope, {}))
    if scope and INSN_FUNCTIONS.match(scope):
        renames['in'] = 'insn'
    return renames


def rename(path, src):
    toks = ctok.tokens(src)
    # Ordinary identifiers that each scope uses, for the collision check
    used = {}
    for i, (kind, text, scope) in enumerate(toks):
        if kind == 'id' and not member(toks, i) and not tag(toks, i):
            used.setdefault(scope, set()).add(text)
    for scope, names in used.items():
        renames = {**{k: v for k, v in GLOBAL.items()}, **scoped(scope)}
        for old, new in renames.items():
            if old in names and new in names and new not in renames:
                raise SystemExit(f'{path}: {scope}: {new} for {old} is taken')
    for i, tok in enumerate(toks):
        kind, text, scope = tok
        if kind == 'pp' and not re.match(r'\s*#\s*include', text):
            tok[1] = re.sub(r'\b[A-Za-z_]\w*\b', lambda m: GLOBAL.get(m.group(), m.group()), text)
        if kind != 'id':
            continue
        if text in GLOBAL:
            tok[1] = GLOBAL[text]
        elif not member(toks, i) and text in scoped(scope):
            tok[1] = scoped(scope)[text]
    return restyle(''.join(t[1] for t in toks))


def member(toks, i):
    """Whether the identifier at i names a member after . or ->"""
    j = i - 1
    while j >= 0 and toks[j][0] in ('space', 'comment'):
        j -= 1
    return j >= 0 and toks[j][1] in ('.', '->')


def tag(toks, i):
    """Whether the identifier at i names a struct, union, or enum"""
    j = i - 1
    while j >= 0 and toks[j][0] in ('space', 'comment'):
        j -= 1
    return j >= 0 and toks[j][1] in ('struct', 'union', 'enum')


# Comments that name renamed identifiers, as exact text
COMMENTS = [
    ('Reverse tmp string and put into result string', 'Reverse digits and put into result string'),
    ('which is word b_index of the block.', 'which is word offset of the block.'),
    ('Convert block base to a pointer to the iw in memory', 'Convert block base to a pointer to the word in memory'),
    ('Move pointer by 32 bits (next iw)', 'Move pointer by 32 bits (next word)'),
    ('iw is now in the updated cache block', 'Word is now in the updated cache block'),
    ('cache_lookup function hits memory bus if iw is not cached', 'cache_lookup function hits memory bus if the word is not cached'),
    ('//iw = cache_lookup(&state->i_cache, (uint64_t) state->pc);', '//word = cache_lookup(&state->icache, (uint64_t) state->pc);'),
    ('Logging for -v, enabled by g_verbose', 'Logging for -v, enabled by verbose_on'),
    (""" * Return whether the simulator can model the cache that csp describes:
 * size words in blocks of block_size words, grouped into sets of ways
 * slots.  The masks in cache_init() need a power of two for both the
 * block size and the number of sets, and all slots have to fit.""",
     """ * Return whether the simulator can model the cache: size words in blocks
 * of block_size words, grouped into sets of ways slots.  The masks in
 * cache_init() need a power of two for both the block size and the number
 * of sets, and all slots have to fit."""),
]

# Lines that the longer names push past 80 columns, wrapped as their
# neighbors are
REWRAP = [
    ('\tcache->index_mask = ((cache->size / cache->block_size) / cache->ways) - 1;\n',
     '\tcache->index_mask =\n\t\t((cache->size / cache->block_size) / cache->ways) - 1;\n'),
    ('\t\tstate->regs[rd] = state->regs[rs1] << (state->regs[rs2] & 63);\n',
     '\t\tstate->regs[rd] = state->regs[rs1] <<\n\t\t\t\t  (state->regs[rs2] & 63);\n'),
    ('\t\tstate->regs[rd] = state->regs[rs1] >> (state->regs[rs2] & 63);\n',
     '\t\tstate->regs[rd] = state->regs[rs1] >>\n\t\t\t\t  (state->regs[rs2] & 63);\n'),
    ('\t\tstate->regs[rd] = emu_div(state->regs[rs1], state->regs[rs2]);\n',
     '\t\tstate->regs[rd] = emu_div(state->regs[rs1],\n\t\t\t\t\t  state->regs[rs2]);\n'),
    ('\t\tstate->regs[rd] = emu_rem(state->regs[rs1], state->regs[rs2]);\n',
     '\t\tstate->regs[rd] = emu_rem(state->regs[rs1],\n\t\t\t\t\t  state->regs[rs2]);\n'),
    ('\tuint64_t target = (state->regs[rs1] + sign_extend(imm, 12)) & ~(uint64_t)1;\n',
     '\tuint64_t target = (state->regs[rs1] + sign_extend(imm, 12)) &\n\t\t\t  ~(uint64_t)1;\n'),
    ('\tprint_pct("Instructions Executed  = %d\\n", analysis->insns, analysis->insns);\n',
     '\tprint_pct("Instructions Executed  = %d\\n", analysis->insns,\n\t\t  analysis->insns);\n'),
    ('\t    cache->ways < 1 || cache->size < 1 || cache->size % cache->block_size)\n',
     '\t    cache->ways < 1 || cache->size < 1 ||\n\t    cache->size % cache->block_size)\n'),
    ('\tif (struct cache_slot *hit = cache_find_tag(set, cache->ways, pos.tag)) {\n',
     '\tif (struct cache_slot *hit = cache_find_tag(set, cache->ways,\n\t\t\t\t\t\t    pos.tag)) {\n'),
    ('\t\t\tvictim->tag << (cache->index_bits + cache->block_bits + 2) |\n',
     '\t\t\tvictim->tag << (cache->index_bits +\n\t\t\t\t\tcache->block_bits + 2) |\n'),
    ('[[nodiscard]] int64_t sign_extend(uint64_t value, uint32_t width) [[unsequenced]];\n',
     '[[nodiscard]] int64_t sign_extend(uint64_t value,\n\t\t\t\t  uint32_t width) [[unsequenced]];\n'),
    ('\t\tdata = evict_data_in_cache_block(cache, slot, addr_word, offset);\n',
     '\t\tdata = evict_data_in_cache_block(cache, slot, addr_word,\n\t\t\t\t\t\t offset);\n'),
    ('\t\tvalue = evict_data_in_cache_block(cache, slot, addr_word, offset);\n',
     '\t\tvalue = evict_data_in_cache_block(cache, slot, addr_word,\n\t\t\t\t\t\t  offset);\n'),
    ('\tuint32_t imm = get_bits(insn->word, 7, 5) | get_bits(insn->word, 25, 7) << 5;\n',
     '\tuint32_t imm = get_bits(insn->word, 7, 5) |\n\t\t       get_bits(insn->word, 25, 7) << 5;\n'),
]


def restyle(src):
    """Comments and wrapping to go with the renames"""
    toks = ctok.tokens(src)
    for tok in toks:
        if tok[0] == 'comment':
            for old, new in COMMENTS:
                tok[1] = tok[1].replace(old, new)
    out = ''.join(t[1] for t in toks)
    for old, new in REWRAP:
        out = out.replace(old, new)
    return out
