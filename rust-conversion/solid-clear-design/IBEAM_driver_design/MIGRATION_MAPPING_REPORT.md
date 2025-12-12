# Migration Mapping Report

## Overview

This report maps existing NIF code locations to specific migration points in each design alternative. It identifies which functions, data structures, and code sections need modification for each design.

## Migration Points Summary

From the analysis, the following migration points were identified:

1. **Library Loading**: Dynamic library loading mechanism
2. **Function Discovery**: NIF function discovery and registration
3. **Heap Allocation**: Direct heap allocation on process heap
4. **Process Access**: Direct Process struct access
5. **Heap Queries**: Direct heap size/top queries
6. **Pointer Tracking**: NIF pointer storage in Process struct
7. **Library Reference Tracking**: NIF library reference storage
8. **NIF Environment Creation**: NifEnv creation from Process
9. **Term Creation/Decoding**: Direct heap manipulation for terms
10. **NIF Execution**: NIF function execution flow

## Design A Mapping: Primary DriverKit

### Library Loading Migration

**Current Code Location**:
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::load_nif_library()` (lines 449-484)
- **Lines**: 459-463 (dynamic library loading)

**Migration Point**: Replace `libloading::Library::new()` with isolated process creation

**Required Changes**:
- Replace `Library::new(path)` with process creation API
- Create isolated process for NIF library
- Load library into isolated process instead of kernel address space
- Register isolated process with kernel proxy

**Migration Complexity**: High - Requires new process creation infrastructure

### Function Discovery Migration

**Current Code Location**:
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::discover_nif_functions()` (lines 486-617)
- **Lines**: 510-614 (metadata lookup and symbol resolution)

**Migration Point**: Replace direct symbol lookup with IPC-based function discovery

**Required Changes**:
- Replace direct `library.get()` symbol lookup with IPC metadata exchange
- Isolated process provides metadata via IPC
- Kernel receives metadata and registers functions
- Function pointers replaced with process identifiers

**Migration Complexity**: High - Requires IPC protocol design

### Heap Allocation Migration

**Current Code Location**:
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::allocate_heap()` (lines 79-81)
- **Line**: 80 (`self.process.allocate_heap_words(words)`)

**Migration Point**: Replace direct heap allocation with IPC-based allocation

**Required Changes**:
- Replace `Process::allocate_heap_words()` call with IPC allocation request
- Serialize allocation request (size in words)
- Send IPC message to kernel
- Kernel allocates on process heap and returns handle
- Store allocation handle instead of direct heap index

**Migration Complexity**: High - Requires IPC protocol and serialization

### Process Access Migration

**Current Code Location**:
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::from_process()` (lines 49-51)
- **Function**: `NifEnv::from_process_id()` (lines 35-40)
- **Field**: `process: Arc<Process>` (line 21)

**Migration Point**: Replace direct Process access with IPC connection

**Required Changes**:
- Replace `Arc<Process>` with IPC connection handle
- Replace `NifEnv::from_process()` with IPC connection setup
- Replace `NifEnv::from_process_id()` with IPC connection lookup
- Remove direct Process struct access
- Access process state via IPC serialization

**Migration Complexity**: High - Requires complete NifEnv redesign

### Heap Queries Migration

**Current Code Location**:
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::available_heap_space()` (lines 89-95)
- **Function**: `NifEnv::heap_top_index()` (lines 100-102)
- **Function**: `NifEnv::heap_size()` (lines 105-107)

**Migration Point**: Replace direct heap queries with IPC-based queries

**Required Changes**:
- Replace direct `Process` method calls with IPC query requests
- Serialize query type (available_space, heap_top, heap_size)
- Send IPC message to kernel
- Receive serialized response
- Deserialize and return result

**Migration Complexity**: High - Requires IPC query protocol

### Pointer Tracking Migration

**Current Code Location**:
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::associate_nif_with_process()` (lines 665-695)
- **Line**: 675 (`process.add_nif_pointer(nif_pointer)`)

**Migration Point**: Replace pointer storage with process identifier storage

**Required Changes**:
- Replace `*const u8` pointer storage with isolated process identifier
- Store process ID or handle instead of function pointer
- Update `Process::add_nif_pointer()` to accept process identifier
- Modify pointer lookup to use process identifier

**Migration Complexity**: Medium - Requires Process struct modification

**Current Code Location**:
- **File**: `entities/entities_process/src/process.rs`
- **Field**: `nif_pointers: Vec<*const u8>` (line 177)
- **Function**: `Process::add_nif_pointer()` (lines 397-406)

**Migration Point**: Replace pointer vector with process identifier vector

**Required Changes**:
- Replace `Vec<*const u8>` with `Vec<NifProcessId>` or similar
- Update `add_nif_pointer()` to accept process identifier
- Update `remove_nif_pointer()` to use process identifier
- Update `get_nif_pointers()` to return process identifiers

**Migration Complexity**: Medium - Requires Process struct refactoring

### Library Reference Tracking Migration

**Current Code Location**:
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::associate_nif_with_process()` (lines 686-690)
- **Line**: 686 (`process.add_nif_library(library_any)`)

**Migration Point**: Replace library reference storage with process identifier

**Required Changes**:
- Replace `Arc<NifLibrary>` storage with isolated process identifier
- Store process ID instead of library reference
- Update `Process::add_nif_library()` to accept process identifier
- Modify library lookup to use process identifier

**Migration Complexity**: Medium - Requires Process struct modification

**Current Code Location**:
- **File**: `entities/entities_process/src/process.rs`
- **Field**: `nif_libraries: Vec<Arc<dyn Any + Send + Sync>>` (line 181)
- **Function**: `Process::add_nif_library()` (lines 444-459)

**Migration Point**: Replace library reference vector with process identifier vector

**Required Changes**:
- Replace `Vec<Arc<dyn Any + Send + Sync>>` with `Vec<NifProcessId>`
- Update `add_nif_library()` to accept process identifier
- Update `remove_nif_library()` to use process identifier
- Update `get_nif_libraries()` to return process identifiers

**Migration Complexity**: Medium - Requires Process struct refactoring

### NIF Environment Creation Migration

**Current Code Location**:
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::from_process_id()` (lines 35-40)
- **Function**: `NifEnv::from_process()` (lines 49-51)

**Migration Point**: Replace Process lookup with IPC connection setup

**Required Changes**:
- Replace process table lookup with IPC connection lookup
- Create IPC connection to isolated NIF process
- Initialize IPC client in NifEnv
- Remove direct Process struct access

**Migration Complexity**: High - Requires IPC infrastructure

### Term Creation/Decoding Migration

**Current Code Location**:
- **Files**: `infrastructure/infrastructure_nif_api/src/term_creation.rs`, `term_decoding.rs`
- **Functions**: All `enif_make_*` and `enif_get_*` functions

**Migration Point**: Replace direct heap manipulation with serialized term transfer

**Required Changes**:
- Replace direct heap manipulation with IPC term creation requests
- Serialize term data for IPC transfer
- Send IPC message to kernel for term creation
- Kernel creates term on process heap
- Return term handle or serialized term

**Migration Complexity**: High - Requires term serialization protocol

### NIF Execution Migration

**Current Code Location**:
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: NIF function call flow (implicit in function pointers)

**Migration Point**: Replace direct function calls with IPC routing

**Required Changes**:
- Replace direct function pointer calls with IPC function call requests
- Serialize function arguments
- Send IPC message to isolated NIF process
- Isolated process executes function
- Receive serialized result via IPC
- Deserialize and return result

**Migration Complexity**: High - Requires complete execution flow redesign

## Design B Mapping: Hybrid Isolation

### Library Loading Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::load_nif_library()` (lines 449-484)

**Migration Point**: Replace library loading with hybrid loading (process + shared memory)

**Required Changes**:
- Create isolated process for NIF library
- Create shared memory regions for read-only data
- Map shared memory into isolated process
- Load library into isolated process
- Register process and shared memory handles

**Migration Complexity**: Medium-High - Requires shared memory management

### Function Discovery Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::discover_nif_functions()` (lines 486-617)

**Migration Point**: Hybrid function discovery (IPC + shared memory)

**Required Changes**:
- Use IPC for function metadata exchange
- Store function metadata in shared memory for fast access
- Register functions with both IPC and shared memory references

**Migration Complexity**: Medium-High - Requires hybrid access patterns

### Heap Allocation Migration

**Current Code Location**: Same as Design A
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::allocate_heap()` (lines 79-81)

**Migration Point**: Hybrid heap allocation (IPC for writes, shared memory for reads)

**Required Changes**:
- Check shared memory heap snapshot for available space
- Request allocation via IPC (write operation)
- Kernel allocates and updates shared memory snapshot
- Return allocation handle

**Migration Complexity**: Medium-High - Requires shared memory synchronization

### Process Access Migration

**Current Code Location**: Same as Design A
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::from_process()` (lines 49-51)

**Migration Point**: Hybrid process access (shared memory reads, IPC writes)

**Required Changes**:
- Replace `Arc<Process>` with shared memory client + IPC client
- Access process state via shared memory (read-only)
- Access process state via IPC for write operations
- Synchronize shared memory updates

**Migration Complexity**: Medium-High - Requires hybrid access patterns

### Heap Queries Migration

**Current Code Location**: Same as Design A
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::available_heap_space()` (lines 89-95)

**Migration Point**: Hybrid heap queries (shared memory for reads)

**Required Changes**:
- Read heap information from shared memory snapshot
- No IPC required for read-only queries
- Kernel updates shared memory periodically

**Migration Complexity**: Medium - Shared memory access only

### Pointer Tracking Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::associate_nif_with_process()` (lines 665-695)

**Migration Point**: Store process identifier (same as Design A)

**Required Changes**: Same as Design A

**Migration Complexity**: Medium - Same as Design A

### Library Reference Tracking Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::associate_nif_with_process()` (lines 686-690)

**Migration Point**: Store process identifier (same as Design A)

**Required Changes**: Same as Design A

**Migration Complexity**: Medium - Same as Design A

### NIF Environment Creation Migration

**Current Code Location**: Same as Design A
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::from_process_id()` (lines 35-40)

**Migration Point**: Hybrid environment creation (shared memory + IPC)

**Required Changes**:
- Create IPC connection to isolated NIF process
- Map shared memory regions into NifEnv
- Initialize both shared memory client and IPC client

**Migration Complexity**: Medium-High - Requires shared memory mapping

### Term Creation/Decoding Migration

**Current Code Location**: Same as Design A
- **Files**: `infrastructure/infrastructure_nif_api/src/term_creation.rs`, `term_decoding.rs`

**Migration Point**: Hybrid term operations (shared memory for reads, IPC for writes)

**Required Changes**:
- Read terms from shared memory when possible
- Use IPC for term creation (write operations)
- Optimize for read-heavy workloads

**Migration Complexity**: Medium-High - Requires hybrid access patterns

### NIF Execution Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`

**Migration Point**: Hybrid execution (shared memory reads, IPC for function calls)

**Required Changes**:
- Route function calls via IPC
- Access process state via shared memory
- Optimize read-heavy NIF functions

**Migration Complexity**: Medium-High - Requires hybrid execution flow

## Design C Mapping: Minimal Isolation

### Library Loading Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::load_nif_library()` (lines 449-484)

**Migration Point**: Minimal isolation loading (process + shared heap)

**Required Changes**:
- Create isolated process for NIF library
- Create shared heap region
- Map shared heap into isolated process (read-write)
- Load library into isolated process
- Register process with shared heap handle

**Migration Complexity**: Low-Medium - Requires shared heap management

### Function Discovery Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::discover_nif_functions()` (lines 486-617)

**Migration Point**: Minimal IPC function discovery

**Required Changes**:
- Use minimal IPC for function metadata exchange
- Store function metadata in isolated process
- Register functions with kernel via minimal IPC

**Migration Complexity**: Low - Minimal IPC only

### Heap Allocation Migration

**Current Code Location**: Same as Design A
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::allocate_heap()` (lines 79-81)

**Migration Point**: Direct shared heap allocation with locks

**Required Changes**:
- Replace `Process::allocate_heap_words()` with shared heap allocation
- Acquire heap lock before allocation
- Allocate directly on shared heap
- Release heap lock after allocation
- Maintain compatibility with current API

**Migration Complexity**: Low - Compatibility layer maintains API

### Process Access Migration

**Current Code Location**: Same as Design A
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::from_process()` (lines 49-51)

**Migration Point**: Compatibility layer with shared heap

**Required Changes**:
- Replace `Arc<Process>` with shared heap client
- Provide NifEnv-like API using shared heap
- Maintain compatibility with existing code
- Add synchronization for heap access

**Migration Complexity**: Low - Compatibility layer minimizes changes

### Heap Queries Migration

**Current Code Location**: Same as Design A
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::available_heap_space()` (lines 89-95)

**Migration Point**: Direct shared heap queries with locks

**Required Changes**:
- Query shared heap directly (with lock)
- Maintain same API interface
- Add synchronization for thread safety

**Migration Complexity**: Low - Compatibility layer maintains API

### Pointer Tracking Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::associate_nif_with_process()` (lines 665-695)

**Migration Point**: Store process identifier (same as Design A)

**Required Changes**: Same as Design A

**Migration Complexity**: Medium - Same as Design A

### Library Reference Tracking Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`
- **Function**: `NifLoader::associate_nif_with_process()` (lines 686-690)

**Migration Point**: Store process identifier (same as Design A)

**Required Changes**: Same as Design A

**Migration Complexity**: Medium - Same as Design A

### NIF Environment Creation Migration

**Current Code Location**: Same as Design A
- **File**: `infrastructure/infrastructure_nif_api/src/nif_env.rs`
- **Function**: `NifEnv::from_process_id()` (lines 35-40)

**Migration Point**: Compatibility layer with shared heap

**Required Changes**:
- Create compatibility NifEnv using shared heap
- Maintain same API interface
- Map shared heap into NifEnv

**Migration Complexity**: Low - Compatibility layer maintains API

### Term Creation/Decoding Migration

**Current Code Location**: Same as Design A
- **Files**: `infrastructure/infrastructure_nif_api/src/term_creation.rs`, `term_decoding.rs`

**Migration Point**: Direct shared heap manipulation with locks

**Required Changes**:
- Use shared heap directly for term creation
- Acquire locks before heap manipulation
- Maintain compatibility with current API
- Add synchronization for thread safety

**Migration Complexity**: Low - Compatibility layer maintains API

### NIF Execution Migration

**Current Code Location**: Same as Design A
- **File**: `adapters/adapters_nifs/src/nif_loader.rs`

**Migration Point**: Minimal IPC routing with shared heap access

**Required Changes**:
- Route function calls via minimal IPC
- Execute function with shared heap access
- Maintain compatibility with current execution flow

**Migration Complexity**: Low - Minimal IPC, compatibility maintained

## Unmapped Migration Points

All identified migration points have been mapped to each design alternative. No unmapped migration points remain.

## Summary

### Design A (Primary DriverKit)
- **Total Migration Points**: 10
- **High Complexity**: 8 points
- **Medium Complexity**: 2 points
- **Low Complexity**: 0 points

### Design B (Hybrid Isolation)
- **Total Migration Points**: 10
- **High Complexity**: 0 points
- **Medium-High Complexity**: 8 points
- **Medium Complexity**: 2 points
- **Low Complexity**: 0 points

### Design C (Minimal Isolation)
- **Total Migration Points**: 10
- **High Complexity**: 0 points
- **Medium-High Complexity**: 0 points
- **Medium Complexity**: 2 points
- **Low Complexity**: 8 points

