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

// C++ wrapper for asmjit to expose C functions for Rust FFI
// This file wraps asmjit C++ API in C functions that can be called from Rust

// Include asmjit headers
// asmjit.h includes core.h, and conditionally includes x86.h or a64.h
#include "asmjit/asmjit.h"
#include <cstring>
#include <cstdlib>

using namespace asmjit;

// Ensure architecture-specific headers are available
#ifndef ASMJIT_NO_X86
#include "asmjit/x86.h"
#endif

#ifndef ASMJIT_NO_ARM
#include "asmjit/a64.h"
#endif

// Forward declarations for opaque types
struct AsmjitCodeHolder {
    CodeHolder holder;
};

struct AsmjitAssembler {
    CodeHolder* holder;
    bool is_x86;
    // Architecture-specific assemblers - only one is used based on is_x86
    #if defined(__x86_64__) || defined(_M_X64)
    x86::Assembler* x86_asm;
    #else
    void* x86_asm; // Placeholder for non-x86 builds
    #endif
    #if defined(__aarch64__) || defined(_M_ARM64)
    a64::Assembler* a64_asm;
    #else
    void* a64_asm; // Placeholder for non-aarch64 builds
    #endif
};

struct AsmjitLabel {
    Label label;
};

struct AsmjitSection {
    Section* section;
};

// CodeHolder operations
extern "C" {

AsmjitCodeHolder* asmjit_codeholder_new() {
    try {
        return new AsmjitCodeHolder();
    } catch (...) {
        return nullptr;
    }
}

void asmjit_codeholder_delete(AsmjitCodeHolder* holder) {
    if (holder) {
        delete holder;
    }
}

int asmjit_codeholder_init(AsmjitCodeHolder* holder) {
    if (!holder) return -1;
    try {
        Error err = holder->holder.init(Environment::host());
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
}

int asmjit_codeholder_attach(AsmjitCodeHolder* holder, AsmjitAssembler* assembler) {
    if (!holder || !assembler) return -1;
    try {
        // The assembler is already created with the code holder reference,
        // but asmjit requires explicit attachment for proper linking
        fprintf(stderr, "[CPP DEBUG] Attaching assembler to code holder\n");
        Error err = holder->holder.attach(assembler->is_x86 ?
            static_cast<BaseAssembler*>(assembler->x86_asm) :
            static_cast<BaseAssembler*>(assembler->a64_asm));
        fprintf(stderr, "[CPP DEBUG] Attach result: %d\n", err ? -1 : 0);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
}

void asmjit_codeholder_reset(AsmjitCodeHolder* holder) {
    if (holder) {
        holder->holder.reset();
    }
}

int asmjit_codeholder_flatten(AsmjitCodeHolder* holder) {
    if (!holder) return -1;
    try {
        Error err = holder->holder.flatten();
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
}

int asmjit_codeholder_resolve_unresolved_links(AsmjitCodeHolder* holder) {
    if (!holder) return -1;
    try {
        Error err = holder->holder.resolveUnresolvedLinks();
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
}

int asmjit_codeholder_relocate_to_base(AsmjitCodeHolder* holder, uint8_t* base_address) {
    if (!holder) return -1;
    try {
        Error err = holder->holder.relocateToBase((uintptr_t)base_address);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
}


size_t asmjit_codeholder_code_size(const AsmjitCodeHolder* holder) {
    if (!holder) return 0;
    return holder->holder.codeSize();
}

const uint8_t* asmjit_codeholder_base_address(const AsmjitCodeHolder* holder) {
    if (!holder || !holder->holder.hasBaseAddress()) return nullptr;
    return (const uint8_t*)holder->holder.baseAddress();
}

int asmjit_codeholder_copy_flattened_data(AsmjitCodeHolder* holder, uint8_t* buffer, size_t size) {
    printf("[CPP DEBUG] copy_flattened_data: holder=%p, buffer=%p, size=%zu\n", holder, buffer, size);
    if (!holder) {
        printf("[CPP DEBUG] copy_flattened_data: holder is null\n");
        return -1;
    }
    if (!buffer) {
        printf("[CPP DEBUG] copy_flattened_data: buffer is null\n");
        return -1;
    }

    // Additional validation
    printf("[CPP DEBUG] copy_flattened_data: validating buffer alignment\n");
    uintptr_t buffer_addr = (uintptr_t)buffer;
    if (buffer_addr % 4 != 0) {
        printf("[CPP DEBUG] copy_flattened_data: WARNING - buffer not 4-byte aligned: %p\n", buffer);
    }

    printf("[CPP DEBUG] copy_flattened_data: checking CodeHolder state\n");
    size_t code_size = holder->holder.codeSize();
    printf("[CPP DEBUG] copy_flattened_data: CodeHolder codeSize=%zu, requested size=%zu\n", code_size, size);

    if (code_size == 0) {
        printf("[CPP DEBUG] copy_flattened_data: ERROR - CodeHolder has zero code size\n");
        return -1;
    }

    if (size > code_size) {
        printf("[CPP DEBUG] copy_flattened_data: WARNING - requested size (%zu) > CodeHolder code size (%zu)\n", size, code_size);
    }

    // Check if buffer has base address set
    uintptr_t base_addr_raw = holder->holder.baseAddress();
    const uint8_t* base_addr = reinterpret_cast<const uint8_t*>(base_addr_raw);
    printf("[CPP DEBUG] copy_flattened_data: CodeHolder baseAddress=%p (raw=%llu)\n", base_addr, (unsigned long long)base_addr_raw);

    try {
        printf("[CPP DEBUG] copy_flattened_data: calling copyFlattenedData WITHOUT flags\n");
        // Try without flags first (safer)
        Error err = holder->holder.copyFlattenedData(buffer, size);
        printf("[CPP DEBUG] copy_flattened_data: result without flags=%d\n", err);
        if (err) {
            printf("[CPP DEBUG] copy_flattened_data: ERROR - failed even without flags\n");
            // Don't try with flags since that crashes
        }
        return err ? -1 : 0;
    } catch (...) {
        printf("[CPP DEBUG] copy_flattened_data: exception caught\n");
        return -1;
    }
}

AsmjitSection* asmjit_codeholder_new_section(
    AsmjitCodeHolder* holder,
    const char* name,
    size_t size,
    uint32_t flags,
    uint32_t alignment) {
    if (!holder) return nullptr;
    try {
        Section* section;
        Error err = holder->holder.newSection(&section, name, size, 
                                              (SectionFlags)flags, alignment);
        if (err) return nullptr;
        
        AsmjitSection* wrapper = new AsmjitSection();
        wrapper->section = section;
        return wrapper;
    } catch (...) {
        return nullptr;
    }
}

// Assembler operations
// Note: 'asm' is a C++ keyword, so we use 'assembler' instead
AsmjitAssembler* asmjit_assembler_new(AsmjitCodeHolder* holder) {
    if (!holder) return nullptr;
    try {
        AsmjitAssembler* asm_wrapper = new AsmjitAssembler();
        asm_wrapper->holder = &holder->holder;

        // Determine architecture and create appropriate assembler
        const Environment& env = Environment::host();
        #if defined(__x86_64__) || defined(_M_X64)
        if (env.arch() == Arch::kX64) {
            asm_wrapper->x86_asm = new x86::Assembler(&holder->holder);
            asm_wrapper->a64_asm = nullptr;
            asm_wrapper->is_x86 = true;
        } else
        #endif
        #if defined(__aarch64__) || defined(_M_ARM64)
        if (env.arch() == Arch::kAArch64) {
            asm_wrapper->x86_asm = nullptr;
            asm_wrapper->a64_asm = new a64::Assembler(&holder->holder);
            asm_wrapper->is_x86 = false;
            fprintf(stderr, "[CPP DEBUG] Created AArch64 assembler: %p\n", asm_wrapper->a64_asm);
        } else
        #endif
        {
            delete asm_wrapper;
            return nullptr;
        }

        return asm_wrapper;
    } catch (...) {
        return nullptr;
    }
}

void asmjit_assembler_delete(AsmjitAssembler* assembler) {
    if (assembler) {
        #if defined(__x86_64__) || defined(_M_X64)
        if (assembler->x86_asm) delete static_cast<x86::Assembler*>(assembler->x86_asm);
        #endif
        #if defined(__aarch64__) || defined(_M_ARM64)
        if (assembler->a64_asm) delete static_cast<a64::Assembler*>(assembler->a64_asm);
        #endif
        delete assembler;
    }
}

size_t asmjit_assembler_offset(const AsmjitAssembler* assembler) {
    if (!assembler) return 0;
    #if defined(__x86_64__) || defined(_M_X64)
    if (assembler->is_x86 && assembler->x86_asm) {
        return static_cast<x86::Assembler*>(assembler->x86_asm)->offset();
    }
    #endif
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler->is_x86 && assembler->a64_asm) {
        return static_cast<a64::Assembler*>(assembler->a64_asm)->offset();
    }
    #endif
    return 0;
}

AsmjitLabel* asmjit_assembler_new_label(AsmjitAssembler* assembler) {
    if (!assembler) return nullptr;
    try {
        AsmjitLabel* label_wrapper = new AsmjitLabel();
        #if defined(__x86_64__) || defined(_M_X64)
        if (assembler->is_x86 && assembler->x86_asm) {
            label_wrapper->label = static_cast<x86::Assembler*>(assembler->x86_asm)->newLabel();
            return label_wrapper;
        }
        #endif
        #if defined(__aarch64__) || defined(_M_ARM64)
        if (!assembler->is_x86 && assembler->a64_asm) {
            label_wrapper->label = static_cast<a64::Assembler*>(assembler->a64_asm)->newLabel();
            return label_wrapper;
        }
        #endif
        delete label_wrapper;
        return nullptr;
    } catch (...) {
        return nullptr;
    }
}

int asmjit_assembler_bind_label(AsmjitAssembler* assembler, AsmjitLabel* label) {
    if (!assembler || !label) return -1;
    try {
        Error err;
        #if defined(__x86_64__) || defined(_M_X64)
        if (assembler->is_x86 && assembler->x86_asm) {
            err = static_cast<x86::Assembler*>(assembler->x86_asm)->bind(label->label);
            return err ? -1 : 0;
        }
        #endif
        #if defined(__aarch64__) || defined(_M_ARM64)
        if (!assembler->is_x86 && assembler->a64_asm) {
            err = static_cast<a64::Assembler*>(assembler->a64_asm)->bind(label->label);
            return err ? -1 : 0;
        }
        #endif
        return -1;
    } catch (...) {
        return -1;
    }
}

uint32_t asmjit_assembler_label_id(const AsmjitLabel* label) {
    if (!label) return 0;
    return label->label.id();
}

// x86-64 specific operations
#if defined(__x86_64__) || defined(_M_X64)
int asmjit_x86_assembler_emit_mov_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src) {
    #if defined(__x86_64__) || defined(_M_X64)
    if (!assembler || !assembler->is_x86 || !assembler->x86_asm) return -1;
    try {
        // x86::Gp is constructed from a register ID
        x86::Gp dst_reg = x86::gpq(dst); // Assuming 64-bit registers
        x86::Gp src_reg = x86::gpq(src);
        Error err = static_cast<x86::Assembler*>(assembler->x86_asm)->mov(dst_reg, src_reg);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)dst; (void)src;
    return -1; // Not x86-64
    #endif
}

int asmjit_x86_assembler_emit_ret(AsmjitAssembler* assembler) {
    #if defined(__x86_64__) || defined(_M_X64)
    if (!assembler || !assembler->is_x86 || !assembler->x86_asm) return -1;
    try {
        Error err = static_cast<x86::Assembler*>(assembler->x86_asm)->ret();
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler;
    return -1; // Not x86-64
    #endif
}
#else
int asmjit_x86_assembler_emit_mov_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src) {
    (void)assembler; (void)dst; (void)src;
    return -1; // Not x86-64
}

int asmjit_x86_assembler_emit_ret(AsmjitAssembler* assembler) {
    (void)assembler;
    return -1; // Not x86-64
}
#endif

// aarch64 specific operations
#if defined(__aarch64__) || defined(_M_ARM64)
int asmjit_a64_assembler_emit_mov_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        // a64::GpX is for 64-bit registers
        a64::GpX dst_reg(dst);
        a64::GpX src_reg(src);
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->mov(dst_reg, src_reg);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)dst; (void)src;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_ret(AsmjitAssembler* assembler) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        // a64 ret() requires a register argument (typically x30/lr)
        a64::GpX lr(30); // Link register
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->ret(lr);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_ldr_reg_offset(AsmjitAssembler* assembler, uint32_t dst, uint32_t base, int32_t offset) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) {
        fprintf(stderr, "[CPP DEBUG] ldr: invalid assembler state\n");
        return -1;
    }
    try {
        fprintf(stderr, "[CPP DEBUG] ldr: emitting ldr x%d, [x%d, #%d]\n", dst, base, offset);
        a64::GpX dst_reg(dst);
        a64::GpX base_reg(base);
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->ldr(dst_reg, a64::ptr(base_reg, offset));
        fprintf(stderr, "[CPP DEBUG] ldr: result %d\n", err ? -1 : 0);
        return err ? -1 : 0;
    } catch (...) {
        fprintf(stderr, "[CPP DEBUG] ldr: exception\n");
        return -1;
    }
    #else
    (void)assembler; (void)dst; (void)base; (void)offset;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_str_reg_offset(AsmjitAssembler* assembler, uint32_t src, uint32_t base, int32_t offset) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        a64::GpX src_reg(src);
        a64::GpX base_reg(base);
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->str(src_reg, a64::ptr(base_reg, offset));
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)src; (void)base; (void)offset;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_add_reg_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src1, uint32_t src2) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        a64::GpX dst_reg(dst);
        a64::GpX src1_reg(src1);
        a64::GpX src2_reg(src2);
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->add(dst_reg, src1_reg, src2_reg);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)dst; (void)src1; (void)src2;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_sub_reg_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src1, uint32_t src2) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        a64::GpX dst_reg(dst);
        a64::GpX src1_reg(src1);
        a64::GpX src2_reg(src2);
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->sub(dst_reg, src1_reg, src2_reg);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)dst; (void)src1; (void)src2;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_stp_pre_idx(AsmjitAssembler* assembler, uint32_t reg1, uint32_t reg2, uint32_t base, int32_t offset) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        a64::GpX reg1_gpx(reg1);
        a64::GpX reg2_gpx(reg2);
        a64::GpX base_reg(base);
        // STP with pre-index addressing: stp reg1, reg2, [base, offset]!
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->stp(reg1_gpx, reg2_gpx, a64::ptr(base_reg, offset).pre());
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)reg1; (void)reg2; (void)base; (void)offset;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_ldp_post_idx(AsmjitAssembler* assembler, uint32_t reg1, uint32_t reg2, uint32_t base, int32_t offset) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        a64::GpX reg1_gpx(reg1);
        a64::GpX reg2_gpx(reg2);
        a64::GpX base_reg(base);
        // LDP with post-index addressing: ldp reg1, reg2, [base], offset
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->ldp(reg1_gpx, reg2_gpx, a64::ptr(base_reg).post(offset));
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)reg1; (void)reg2; (void)base; (void)offset;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_blr(AsmjitAssembler* assembler, uint32_t reg) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        a64::GpX reg_gpx(reg);
        // BLR (branch with link to register) for function calls
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->blr(reg_gpx);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)reg;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_mov_imm(AsmjitAssembler* assembler, uint32_t dst, uint64_t imm) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        a64::GpX dst_reg(dst);
        // MOV immediate
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->mov(dst_reg, imm);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)dst; (void)imm;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_add_imm(AsmjitAssembler* assembler, uint32_t dst, uint32_t src, uint32_t imm) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) {
        fprintf(stderr, "[CPP DEBUG] add_imm: invalid assembler state\n");
        return -1;
    }
    try {
        fprintf(stderr, "[CPP DEBUG] add_imm: emitting add x%d, x%d, #%d\n", dst, src, imm);
        a64::GpX dst_reg(dst);
        a64::GpX src_reg(src);
        // ADD immediate
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->add(dst_reg, src_reg, imm);
        fprintf(stderr, "[CPP DEBUG] add_imm: result %d\n", err ? -1 : 0);
        return err ? -1 : 0;
    } catch (...) {
        fprintf(stderr, "[CPP DEBUG] add_imm: exception\n");
        return -1;
    }
    #else
    (void)assembler; (void)dst; (void)src; (void)imm;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_sub_imm(AsmjitAssembler* assembler, uint32_t dst, uint32_t src, uint32_t imm) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        a64::GpX dst_reg(dst);
        a64::GpX src_reg(src);
        // SUB immediate
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->sub(dst_reg, src_reg, imm);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)dst; (void)src; (void)imm;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_cmp_reg_reg(AsmjitAssembler* assembler, uint32_t reg1, uint32_t reg2) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        a64::GpX reg1_gpx(reg1);
        a64::GpX reg2_gpx(reg2);
        // CMP (compare)
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->cmp(reg1_gpx, reg2_gpx);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)reg1; (void)reg2;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_b_eq(AsmjitAssembler* assembler, uint32_t label_id) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        Label label(label_id);
        // B.EQ (branch if equal)
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->b_eq(label);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)label_id;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_b_ne(AsmjitAssembler* assembler, uint32_t label_id) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        Label label(label_id);
        // B.NE (branch if not equal)
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->b_ne(label);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)label_id;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_b_lt(AsmjitAssembler* assembler, uint32_t label_id) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        Label label(label_id);
        // B.LT (branch if less than)
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->b_lt(label);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)label_id;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_b_ge(AsmjitAssembler* assembler, uint32_t label_id) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        Label label(label_id);
        // B.GE (branch if greater than or equal)
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->b_ge(label);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)label_id;
    return -1; // Not aarch64
    #endif
}

int asmjit_a64_assembler_emit_b(AsmjitAssembler* assembler, uint32_t label_id) {
    #if defined(__aarch64__) || defined(_M_ARM64)
    if (!assembler || assembler->is_x86 || !assembler->a64_asm) return -1;
    try {
        Label label(label_id);
        // B (unconditional branch)
        Error err = static_cast<a64::Assembler*>(assembler->a64_asm)->b(label);
        return err ? -1 : 0;
    } catch (...) {
        return -1;
    }
    #else
    (void)assembler; (void)label_id;
    return -1; // Not aarch64
    #endif
}
#else
int asmjit_a64_assembler_emit_mov_reg_reg(AsmjitAssembler* assembler, uint32_t dst, uint32_t src) {
    (void)assembler; (void)dst; (void)src;
    return -1; // Not aarch64
}

int asmjit_a64_assembler_emit_ret(AsmjitAssembler* assembler) {
    (void)assembler;
    return -1; // Not aarch64
}
#endif

// Memory protection functions
int asmjit_virtmem_protect_jit_memory(int access) {
    try {
        printf("[CPP DEBUG] protectJitMemory: calling VirtMem::protectJitMemory with access=%d\n", access);
        if (access == 0) {
            VirtMem::protectJitMemory(VirtMem::ProtectJitAccess::kReadWrite);
            printf("[CPP DEBUG] protectJitMemory: set kReadWrite\n");
        } else if (access == 1) {
            VirtMem::protectJitMemory(VirtMem::ProtectJitAccess::kReadExecute);
            printf("[CPP DEBUG] protectJitMemory: set kReadExecute\n");
        } else {
            printf("[CPP DEBUG] protectJitMemory: invalid access value %d\n", access);
            return -1;
        }
        return 0;
    } catch (...) {
        printf("[CPP DEBUG] protectJitMemory: exception caught\n");
        return -1;
    }
}

} // extern "C"
