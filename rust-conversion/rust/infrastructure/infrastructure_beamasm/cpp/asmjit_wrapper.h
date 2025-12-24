/*
 * %CopyrightBegin%
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Copyright Ericsson AB 2025. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * %CopyrightEnd%
 */

// C header for asmjit wrapper - used by Rust FFI

#ifndef ASMJIT_WRAPPER_H
#define ASMJIT_WRAPPER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque types
typedef struct AsmjitCodeHolder AsmjitCodeHolder;
typedef struct AsmjitAssembler AsmjitAssembler;
typedef struct AsmjitLabel AsmjitLabel;
typedef struct AsmjitSection AsmjitSection;

// CodeHolder operations
AsmjitCodeHolder* asmjit_codeholder_new(void);
void asmjit_codeholder_delete(AsmjitCodeHolder* holder);
int asmjit_codeholder_init(AsmjitCodeHolder* holder);
void asmjit_codeholder_reset(AsmjitCodeHolder* holder);
int asmjit_codeholder_flatten(AsmjitCodeHolder* holder);
int asmjit_codeholder_resolve_unresolved_links(AsmjitCodeHolder* holder);
int asmjit_codeholder_relocate_to_base(AsmjitCodeHolder* holder, uint8_t* base_address);
int asmjit_codeholder_copy_flattened_data(AsmjitCodeHolder* holder, uint8_t* buffer, size_t size);
size_t asmjit_codeholder_code_size(const AsmjitCodeHolder* holder);
const uint8_t* asmjit_codeholder_base_address(const AsmjitCodeHolder* holder);
AsmjitSection* asmjit_codeholder_new_section(
    AsmjitCodeHolder* holder,
    const char* name,
    size_t size,
    uint32_t flags,
    uint32_t alignment);

// Assembler operations
// Note: Using 'assembler' instead of 'asm' because 'asm' is a C++ keyword
AsmjitAssembler* asmjit_assembler_new(AsmjitCodeHolder* holder);
void asmjit_assembler_delete(AsmjitAssembler* assembler);
size_t asmjit_assembler_offset(const AsmjitAssembler* assembler);
AsmjitLabel* asmjit_assembler_new_label(AsmjitAssembler* assembler);
int asmjit_assembler_bind_label(AsmjitAssembler* assembler, AsmjitLabel* label);
uint32_t asmjit_assembler_label_id(const AsmjitLabel* label);

// x86-64 specific operations
int asmjit_x86_assembler_emit_mov_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src);
int asmjit_x86_assembler_emit_ret(AsmjitAssembler* assembler);

// aarch64 specific operations
int asmjit_a64_assembler_emit_mov_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src);
int asmjit_a64_assembler_emit_ret(AsmjitAssembler* assembler);
int asmjit_a64_assembler_emit_ldr_reg_offset(AsmjitAssembler* assembler, uint32_t dst, uint32_t base, int32_t offset);
int asmjit_a64_assembler_emit_tst_imm(AsmjitAssembler* assembler, uint32_t reg, uint32_t imm);
int asmjit_a64_assembler_emit_str_reg_offset(AsmjitAssembler* assembler, uint32_t src, uint32_t base, int32_t offset);
int asmjit_a64_assembler_emit_add_reg_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src1, uint32_t src2);
int asmjit_a64_assembler_emit_and_imm(AsmjitAssembler* assembler, uint32_t dst, uint32_t src, uint32_t imm);
int asmjit_a64_assembler_emit_sub_reg_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src1, uint32_t src2);
int asmjit_a64_assembler_emit_subs_imm(AsmjitAssembler* assembler, uint32_t dst, uint32_t src, uint32_t imm);
int asmjit_a64_assembler_emit_stp_pre_idx(AsmjitAssembler* assembler, uint32_t reg1, uint32_t reg2, uint32_t base, int32_t offset);
int asmjit_a64_assembler_emit_ldp_post_idx(AsmjitAssembler* assembler, uint32_t reg1, uint32_t reg2, uint32_t base, int32_t offset);
int asmjit_a64_assembler_emit_stp(AsmjitAssembler* assembler, uint32_t reg1, uint32_t reg2, uint32_t base, int32_t offset);
int asmjit_a64_assembler_emit_ldp(AsmjitAssembler* assembler, uint32_t reg1, uint32_t reg2, uint32_t base, int32_t offset);
int asmjit_a64_assembler_emit_blr(AsmjitAssembler* assembler, uint32_t reg);
int asmjit_a64_assembler_emit_blr_imm(AsmjitAssembler* assembler, uint64_t addr);
int asmjit_a64_assembler_emit_mov_imm(AsmjitAssembler* assembler, uint32_t dst, uint64_t imm);
int asmjit_a64_assembler_emit_add_imm(AsmjitAssembler* assembler, uint32_t dst, uint32_t src, uint32_t imm);
int asmjit_a64_assembler_emit_adds_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src1, uint32_t src2);
int asmjit_a64_assembler_emit_adds_imm(AsmjitAssembler* assembler, uint32_t dst, uint32_t src, uint32_t imm);
int asmjit_a64_assembler_emit_sub_imm(AsmjitAssembler* assembler, uint32_t dst, uint32_t src, uint32_t imm);
int asmjit_a64_assembler_emit_cmp_reg_reg(AsmjitAssembler* assembler, uint32_t reg1, uint32_t reg2);
int asmjit_a64_assembler_emit_b_eq(AsmjitAssembler* assembler, uint32_t label_id);
int asmjit_a64_assembler_emit_b_ne(AsmjitAssembler* assembler, uint32_t label_id);
int asmjit_a64_assembler_emit_b_lt(AsmjitAssembler* assembler, uint32_t label_id);
int asmjit_a64_assembler_emit_b_ge(AsmjitAssembler* assembler, uint32_t label_id);
int asmjit_a64_assembler_emit_b(AsmjitAssembler* assembler, uint32_t label_id);

// Memory protection functions
int asmjit_virtmem_protect_jit_memory(int access);

#ifdef __cplusplus
}
#endif

// Additional ARM64 functions
int asmjit_a64_assembler_emit_cmp_imm(AsmjitAssembler* assembler, uint32_t reg, uint64_t imm);
int asmjit_a64_assembler_emit_b_cond(AsmjitAssembler* assembler, uint32_t condition, uint32_t target);
int asmjit_a64_assembler_emit_nop(AsmjitAssembler* assembler);

// Additional ARM64 arithmetic and shift operations
int asmjit_a64_assembler_emit_lsr_imm(AsmjitAssembler* assembler, uint32_t dst, uint32_t src, uint32_t shift);
int asmjit_a64_assembler_emit_lsl_imm(AsmjitAssembler* assembler, uint32_t dst, uint32_t src, uint32_t shift);
int asmjit_a64_assembler_emit_stur_reg_offset(AsmjitAssembler* assembler, uint32_t src, uint32_t base, int32_t offset);
int asmjit_a64_assembler_emit_ldur_reg_offset(AsmjitAssembler* assembler, uint32_t dst, uint32_t base, int32_t offset);
int asmjit_a64_assembler_emit_udiv_reg_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t dividend, uint32_t divisor);
int asmjit_a64_assembler_emit_mul_reg_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src1, uint32_t src2);
int asmjit_a64_assembler_emit_msub_reg_reg_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src1, uint32_t src2, uint32_t src3);
int asmjit_a64_assembler_emit_eor_reg_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src1, uint32_t src2);

#endif // ASMJIT_WRAPPER_H

