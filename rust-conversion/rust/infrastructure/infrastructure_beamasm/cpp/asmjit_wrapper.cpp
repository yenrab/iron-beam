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

size_t asmjit_codeholder_code_size(const AsmjitCodeHolder* holder) {
    if (!holder) return 0;
    return holder->holder.codeSize();
}

const uint8_t* asmjit_codeholder_base_address(const AsmjitCodeHolder* holder) {
    if (!holder || !holder->holder.hasBaseAddress()) return nullptr;
    return (const uint8_t*)holder->holder.baseAddress();
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

} // extern "C"
