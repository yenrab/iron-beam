# Process Table Performance Optimization Guide

## Overview

This document describes potential performance optimizations for the `ProcessTable` implementation. The current implementation uses a `HashMap`-based design which is sufficient for most use cases, but can be optimized for extreme performance scenarios similar to the C implementation in `erl_ptab.c`.

**Current Implementation**: HashMap-based with `RwLock` for thread safety  
**Target**: Lock-free array-based design with direct indexing (similar to C implementation)

---

## Current Performance Characteristics

### Strengths
- ✅ O(1) average-case lookups (HashMap)
- ✅ Thread-safe with `RwLock`
- ✅ Memory efficient for sparse data
- ✅ Idiomatic Rust design
- ✅ Easy to maintain

### Potential Bottlenecks
- ⚠️ Hash computation overhead on every lookup
- ⚠️ Lock contention on `RwLock` for concurrent operations
- ⚠️ HashMap bucket traversal (though minimal)
- ⚠️ Memory indirection through HashMap structure

---

## Optimization Roadmap

### Phase 1: Lock-Free Lookups (High Impact)

**Goal**: Eliminate lock contention for read operations (lookups are 99% of operations)

**Implementation Strategy**:
1. Replace `HashMap` with `Vec<AtomicPtr<Process>>` or `Vec<Option<Arc<Process>>>`
2. Use direct indexing: `index = id & mask` (where mask = `table_size - 1`)
3. Use atomic operations for lock-free reads:
   ```rust
   use std::sync::atomic::{AtomicPtr, Ordering};
   
   pub fn lookup(&self, id: ProcessId) -> Option<Arc<Process>> {
       let index = (id as usize) & self.mask;
       let ptr = self.table[index].load(Ordering::Acquire);
       // Convert pointer to Arc (requires careful lifetime management)
   }
   ```

**Benefits**:
- No lock contention on lookups
- Direct memory access (faster than hash computation)
- Similar to C implementation's 20,000%+ speedup for lookups

**Trade-offs**:
- Requires power-of-2 table size
- More complex memory management
- Need to handle pointer-to-Arc conversion safely

**Reference**: See `erts_ptab_pix2intptr_acqb()` in `erl_ptab.h` and `PTables.md` lines 89-118

---

### Phase 2: Cache-Line Optimization (Medium Impact)

**Goal**: Reduce false sharing when multiple threads insert simultaneously

**Implementation Strategy**:
1. Interleave slots across cache lines (typically 64 bytes)
2. Use cache-line aligned storage:
   ```rust
   use std::alloc::{Layout, alloc};
   
   // Allocate with cache-line alignment
   const CACHE_LINE_SIZE: usize = 64;
   let layout = Layout::from_size_align(
       size * mem::size_of::<AtomicPtr<Process>>(),
       CACHE_LINE_SIZE
   ).unwrap();
   ```

3. Map adjacent logical slots to different cache lines:
   ```rust
   // Instead of: slot[i] -> cache_line[i / slots_per_line]
   // Use: slot[i] -> cache_line[i % num_cache_lines][i / num_cache_lines]
   fn data_to_index(&self, id: ProcessId) -> usize {
       let data = id & self.data_mask;
       let cache_line_idx = data & self.cache_line_mask;
       let slot_in_line = (data >> self.cache_line_bits) & self.slot_mask;
       cache_line_idx * self.slots_per_cache_line + slot_in_line
   }
   ```

**Benefits**:
- Eliminates false sharing
- Better scalability with many threads
- Reduces cache line contention

**Trade-offs**:
- More complex index calculation
- Slightly more memory overhead

**Reference**: See `PTables.md` lines 192-216 and `erts_ptab_data2pix()` in `erl_ptab.h`

---

### Phase 3: Lock-Free Insertions (High Impact)

**Goal**: Eliminate write lock contention for insertions

**Implementation Strategy**:
1. Use atomic compare-and-swap (CAS) for slot reservation:
   ```rust
   use std::sync::atomic::{AtomicPtr, Ordering};
   use std::ptr;
   
   const RESERVED: *mut Process = ptr::null_mut();
   const EMPTY: *mut Process = ptr::null_mut();
   
   // Reserve slot
   loop {
       let current = self.table[index].load(Ordering::Acquire);
       if current == EMPTY {
           match self.table[index].compare_exchange_weak(
               EMPTY,
               RESERVED,
               Ordering::AcqRel,
               Ordering::Acquire
           ) {
               Ok(_) => break, // Reserved successfully
               Err(_) => continue, // Another thread got it, retry
           }
       } else {
           // Slot taken, try next
           index = (index + 1) & self.mask;
       }
   }
   ```

2. Use atomic operations for publishing:
   ```rust
   // After initializing process, publish it
   self.table[index].store(process_ptr, Ordering::Release);
   ```

**Benefits**:
- No write lock needed
- Better concurrent insertion performance
- Scales better with many threads

**Trade-offs**:
- More complex insertion logic
- Need to handle retry loops
- Requires careful memory ordering

**Reference**: See `erts_ptab_new_element()` in `erl_ptab.c` lines 499-641 and `PTables.md` lines 233-271

---

### Phase 4: Memory Ordering Optimization (Low-Medium Impact)

**Goal**: Minimize memory barrier overhead

**Implementation Strategy**:
1. Use appropriate memory ordering for each operation:
   ```rust
   // Lookup: Acquire (ensures we see initialized data)
   let ptr = self.table[index].load(Ordering::Acquire);
   
   // Insert: Release (ensures data is visible after)
   self.table[index].store(ptr, Ordering::Release);
   
   // Internal operations: Relaxed (no ordering needed)
   self.count.fetch_add(1, Ordering::Relaxed);
   ```

2. Provide different barrier variants for different use cases:
   ```rust
   pub fn lookup_acquire(&self, id: ProcessId) -> Option<Arc<Process>> {
       // Acquire barrier - ensures data visibility
   }
   
   pub fn lookup_relaxed(&self, id: ProcessId) -> Option<Arc<Process>> {
       // Relaxed - fastest, but caller must ensure ordering
   }
   ```

**Benefits**:
- Reduced memory barrier overhead
- Better performance on architectures with expensive barriers
- More control for performance-critical code

**Trade-offs**:
- More API surface
- Requires understanding of memory ordering
- Potential for bugs if used incorrectly

**Reference**: See `erts_ptab_pix2intptr_*()` variants in `erl_ptab.h` lines 344-366

---

### Phase 5: Reference Counting Optimization (Low Impact)

**Goal**: Optimize reference counting for lock-free operations

**Current**: Uses `Arc` which handles refcounting automatically  
**Optimization**: Co-locate reference count with process pointer (if using raw pointers)

**Implementation Strategy**:
1. If moving to raw pointers, embed refcount in process structure:
   ```rust
   struct ProcessWithRefcount {
       process: Process,
       refcount: AtomicU32,
   }
   ```

2. Use atomic operations for refcount:
   ```rust
   fn inc_ref(&self) {
       self.refcount.fetch_add(1, Ordering::Relaxed);
   }
   
   fn dec_ref(&self) -> bool {
       let prev = self.refcount.fetch_sub(1, Ordering::AcqRel);
       prev > 1
   }
   ```

**Benefits**:
- More control over refcounting
- Can optimize for specific patterns
- Better cache locality

**Trade-offs**:
- More complex than `Arc`
- Need to handle deallocation carefully
- Requires thread progress tracking (like C implementation)

**Reference**: See `erts_ptab_inc_refc()` and related functions in `erl_ptab.h`

---

## Implementation Checklist

### Prerequisites
- [ ] Benchmark current implementation to establish baseline
- [ ] Identify actual bottlenecks (profile with `perf` or `cargo flamegraph`)
- [ ] Determine if optimization is needed (measure first!)

### Phase 1: Lock-Free Lookups
- [ ] Replace `HashMap` with `Vec<AtomicPtr<Process>>` or similar
- [ ] Implement direct indexing with bitmask
- [ ] Add atomic load operations with Acquire ordering
- [ ] Handle pointer-to-Arc conversion safely
- [ ] Add tests for concurrent lookups
- [ ] Benchmark lookup performance improvement

### Phase 2: Cache-Line Optimization
- [ ] Calculate cache-line interleaving formula
- [ ] Implement `data_to_index()` with cache-line awareness
- [ ] Allocate memory with cache-line alignment
- [ ] Test with multiple threads inserting simultaneously
- [ ] Measure false sharing reduction

### Phase 3: Lock-Free Insertions
- [ ] Implement CAS-based slot reservation
- [ ] Add retry logic for contention
- [ ] Implement atomic publish operation
- [ ] Handle capacity limits atomically
- [ ] Test concurrent insertions
- [ ] Benchmark insertion performance

### Phase 4: Memory Ordering
- [ ] Audit all atomic operations for correct ordering
- [ ] Add relaxed variants where safe
- [ ] Document memory ordering requirements
- [ ] Test on different architectures (x86, ARM)

### Phase 5: Reference Counting (if needed)
- [ ] Evaluate if `Arc` is actually a bottleneck
- [ ] Implement custom refcounting if needed
- [ ] Add thread progress tracking
- [ ] Implement safe deallocation

---

## Performance Targets

Based on C implementation benchmarks (from `PTables.md`):

| Operation | Current (HashMap) | Target (Optimized) | Improvement |
|-----------|------------------|-------------------|-------------|
| **Lookup** | ~50-100ns | ~5-10ns | 10x faster |
| **Insert** | ~200-300ns | ~50-100ns | 3-4x faster |
| **Concurrent Lookups** | Limited by lock | Unlimited | 20,000%+ improvement |
| **Concurrent Inserts** | Limited by lock | Better scaling | 150-200% improvement |

**Note**: Actual improvements depend on workload, hardware, and contention levels.

---

## Code Structure Recommendations

### Suggested Module Organization

```
process_table/
├── mod.rs                 # Public API (current interface)
├── hashmap.rs            # Current HashMap implementation
├── array.rs              # New array-based implementation
├── atomic_ops.rs          # Atomic operation helpers
├── cache_line.rs         # Cache-line utilities
└── benchmarks.rs         # Performance benchmarks
```

### Feature Flags

Use Cargo features to allow switching implementations:

```toml
[features]
default = ["hashmap"]
hashmap = []  # Current HashMap implementation
array = []    # Optimized array implementation
```

```rust
#[cfg(feature = "hashmap")]
mod hashmap_impl;
#[cfg(feature = "array")]
mod array_impl;

#[cfg(feature = "hashmap")]
pub use hashmap_impl::ProcessTable;
#[cfg(feature = "array")]
pub use array_impl::ProcessTable;
```

---

## Testing Strategy

### Unit Tests
- [ ] Test all operations with single thread
- [ ] Test concurrent lookups
- [ ] Test concurrent insertions
- [ ] Test ID generation and reuse
- [ ] Test capacity limits

### Stress Tests
- [ ] High contention scenarios (many threads)
- [ ] Rapid create/destroy cycles
- [ ] Memory leak detection
- [ ] Long-running stability tests

### Performance Tests
- [ ] Benchmark lookup latency
- [ ] Benchmark insertion throughput
- [ ] Measure lock contention
- [ ] Profile memory usage
- [ ] Compare against C implementation (if possible)

---

## Safety Considerations

### Memory Safety
- ⚠️ **Raw Pointers**: If using `AtomicPtr`, must ensure processes outlive table
- ⚠️ **Arc Conversion**: Converting raw pointers to `Arc` requires careful lifetime management
- ⚠️ **Deallocation**: Need thread progress tracking to ensure safe deallocation

### Thread Safety
- ⚠️ **Memory Ordering**: Must use correct ordering to prevent data races
- ⚠️ **ABA Problem**: Need to handle if reusing IDs too quickly
- ⚠️ **Lock-Free Guarantees**: Must ensure operations always make progress

### Recommendations
1. Start with `Arc`-based array (safer than raw pointers)
2. Use `OnceCell` or `LazyLock` for initialization
3. Consider using `crossbeam` crate for lock-free data structures
4. Add extensive tests before optimizing
5. Profile before and after each optimization

---

## References

### C Implementation Files
- `erts/emulator/beam/erl_ptab.h` - Header with inline functions
- `erts/emulator/beam/erl_ptab.c` - Main implementation
- `erts/emulator/internal_doc/PTables.md` - Design documentation

### Key C Functions to Study
- `erts_ptab_new_element()` - Lock-free insertion
- `erts_ptab_pix2intptr_acqb()` - Lock-free lookup
- `erts_ptab_data2pix()` - Cache-line aware indexing
- `erts_ptab_delete_element()` - Lock-free deletion

### Rust Resources
- `std::sync::atomic` - Atomic operations
- `crossbeam` crate - Lock-free data structures
- `parking_lot` crate - Faster locks (if locks are needed)

---

## When to Optimize

**Don't optimize unless:**
1. ✅ You have measured a performance problem
2. ✅ The current implementation is a bottleneck
3. ✅ You understand the trade-offs
4. ✅ You have time for extensive testing

**Do optimize when:**
1. ✅ Lookups are >1% of total runtime
2. ✅ Lock contention is measurable
3. ✅ You need Erlang VM-level performance
4. ✅ You have benchmarks showing improvement potential

---

## Migration Path

If optimizing later:

1. **Keep current implementation** as fallback
2. **Add new implementation** behind feature flag
3. **Benchmark both** on real workloads
4. **Gradually migrate** if new implementation is better
5. **Maintain both** if needed for different use cases

---

## Summary

The current HashMap-based implementation is **sufficient for most use cases**. The optimizations described here are for **extreme performance scenarios** similar to the Erlang VM's requirements.

**Key Takeaway**: Measure first, optimize second. The C implementation's optimizations were driven by specific performance requirements in a VM runtime. Your use case may not need these optimizations.

If you do need extreme performance:
1. Start with **Phase 1** (lock-free lookups) - biggest impact
2. Add **Phase 2** (cache-line optimization) if you have many threads
3. Consider **Phase 3** (lock-free insertions) if insertions are frequent
4. Fine-tune with **Phase 4** (memory ordering) for specific architectures

The current design makes these optimizations possible without breaking the API.

