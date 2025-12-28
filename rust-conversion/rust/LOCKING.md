# Lock Ordering Documentation

This document defines the **mandatory lock ordering** for all code in the Iron BEAM Rust implementation. **Violating these orders will cause deadlocks.**

## Critical Rule

**NEVER acquire locks in a different order than specified here. If you need multiple locks, you MUST acquire them in the documented order, even if it seems inefficient.**

## Lock Ordering Rules

### 1. Process Locks (`entities_process::Process`)

When acquiring multiple locks on a `Process` instance, use this order:

1. `heap_data` (Mutex<Vec<Eterm>>)
2. `heap_top_index` (Mutex<usize>)
3. `stack_top_index` (Mutex<Option<usize>>)

**Rationale:** The heap data is the most fundamental resource, and heap_top_index depends on it. Stack operations depend on both heap and heap_top_index.

**Examples:**
- ✅ `heap_data` → `heap_top_index` (e.g., `allocate_heap_words()`)
- ✅ `heap_data` → `stack_top_index` (e.g., `stack_pop()`, `stack_peek()`)
- ✅ `heap_data` → `heap_top_index` → `stack_top_index` (e.g., `stack_push()`)
- ✅ `heap_top_index` → `stack_top_index` (e.g., `stack_size_words()`)
- ❌ `heap_top_index` → `heap_data` (WRONG - violates order)
- ❌ `stack_top_index` → `heap_data` (WRONG - violates order)

### 2. Scheduler and Run Queue Locks

When acquiring scheduler and run queue locks, use this order:

1. `schedulers` (Mutex<Vec<Scheduler>>)
2. `runq` (Mutex<RunQueue>) - accessed via `scheduler.runq()`

**Rationale:** The schedulers collection must be locked first to access a scheduler, then the scheduler's run queue can be locked.

**Examples:**
- ✅ `schedulers` → `runq` (e.g., `create_init_process()`, `spawn_kernel_process()`)
- ❌ `runq` → `schedulers` (WRONG - violates order)

**Locations:**
- `frameworks_emulator_init/src/main_init.rs:create_init_process()`
- `frameworks_emulator_init/src/boot_script.rs:spawn_kernel_process()`
- `frameworks_emulator_init/src/boot_script.rs:apply_function()`

### 3. Global Literals Locks (`infrastructure_utilities::GlobalLiterals`)

When acquiring multiple locks for global literals, use this order:

1. `lock` (Mutex<()>) - main synchronization lock
2. `areas` (Mutex<Vec<GlobalLiteralArea>>)
3. `current_offset` (Mutex<usize>)
4. `current_size` (Mutex<usize>)

**Rationale:** The main `lock` provides coarse-grained synchronization, then finer-grained locks are acquired in dependency order.

**Examples:**
- ✅ `lock` → `areas` → `current_offset` → `current_size` (e.g., `expand_area()`)
- ❌ Any other order (WRONG - violates order)

**Locations:**
- `infrastructure_utilities/src/global_literals.rs:expand_area()`

### 4. Module Table Manager Locks (`code_management_code_loading::ModuleTableManager`)

When acquiring locks on multiple module tables (e.g., during staging operations), use this order:

1. Lock tables by **code index** (lower index first, then higher index)

**Rationale:** When copying modules between code indices (e.g., `start_staging()`), if two threads call the function with reversed index orders, they could deadlock. By always locking in index order, both threads will attempt to acquire the same lock first, preventing circular wait.

**Examples:**
- ✅ Lock table[0] → table[1] (e.g., `start_staging(0, 1)`)
- ✅ Lock table[1] → table[0] (e.g., `start_staging(1, 0)` - but locks table[0] first, then table[1])
- ❌ Lock table[1] → table[0] when called as `start_staging(1, 0)` without index ordering (WRONG - violates order)

**Locations:**
- `code_management_code_loading/src/module_management.rs:start_staging()`

### 5. Code Permission Manager Locks (`code_management_code_loading::CodePermissionManager`)

When acquiring multiple code permissions, use this order:

1. `stage_permission` (Mutex<CodePermission>)
2. `mod_permission` (Mutex<CodePermission>)

**Rationale:** Code load permission requires both staging and modification permissions. The order must be consistent to prevent deadlocks when multiple threads request permissions.

**Examples:**
- ✅ `stage_permission` → `mod_permission` (e.g., `try_seize_code_load_permission()`)
- ✅ `stage_permission` → `mod_permission` (e.g., `has_code_load_permission()`)
- ❌ `mod_permission` → `stage_permission` (WRONG - violates order)

**Locations:**
- `code_management_code_loading/src/code_permissions.rs:try_seize_code_load_permission()`
- `code_management_code_loading/src/code_permissions.rs:has_code_load_permission()`

**Note:** `release_code_load_permission()` releases in reverse order (mod_permission → stage_permission), which is acceptable for releases. However, acquisition must always follow the documented order.

### 6. BEAM Debug Tracer Locks (`code_management_code_loading::BeamDebugTracer`)

When acquiring multiple locks for BEAM debug tracing, use this order:

1. `traced_mfas` (Mutex<HashMap<Mfa, TracedMfa>>)
2. `traced_by_index` (Mutex<Vec<Option<Mfa>>>)
3. `next_index` (Mutex<usize>)

**Rationale:** The `traced_mfas` map is the primary data structure, `traced_by_index` provides index-based lookup, and `next_index` tracks the next available index. This order ensures consistent lock acquisition when multiple operations need to update the trace state.

**Examples:**
- ✅ `traced_mfas` → `traced_by_index` → `next_index` (e.g., `set_traced_mfa()`, `clear()`)
- ✅ `traced_mfas` only (e.g., `is_traced_mfa()`)
- ✅ `traced_by_index` only (e.g., `get_traced_mfa()`, `vtrace_mfa()`)
- ❌ `traced_by_index` → `traced_mfas` (WRONG - violates order)
- ❌ `next_index` → `traced_mfas` (WRONG - violates order)

**Locations:**
- `code_management_code_loading/src/beam_debug.rs:set_traced_mfa()`
- `code_management_code_loading/src/beam_debug.rs:clear()`

### 7. Process Table Locks (`infrastructure_utilities::ProcessTable`)

When acquiring multiple locks on ProcessTable, use this order:

1. `table` (RwLock<HashMap<ProcessId, Arc<Process>>>)
2. `free_ids` (RwLock<VecDeque<ProcessId>>)

**Rationale:** The main table must be locked first before accessing the free ID pool. This prevents deadlocks when multiple threads are removing processes or creating new ones.

**Examples:**
- ✅ `table.write()` → `free_ids.write()` (e.g., `remove()`, `clear()`)
- ✅ `table.read()` → `free_ids.write()` → `table.write()` (e.g., `new_element()`)
- ❌ `free_ids.write()` → `table.write()` (WRONG - violates order)

**Locations:**
- `infrastructure_utilities/src/process_table.rs:remove()`
- `infrastructure_utilities/src/process_table.rs:clear()`
- `infrastructure_utilities/src/process_table.rs:new_element()`

**Note:** In `new_element()`, the sequence is more complex: first `table.read()` for capacity check, then `free_ids.write()` to get a free ID, then `table.write()` to insert. If rollback is needed, `free_ids.write()` is acquired again. The key rule is: always acquire `table` locks before `free_ids` locks.

### 8. Atom Table Locks (`entities_data_handling::AtomTable`)

When acquiring multiple locks on AtomTable, use this order:

1. `atoms` (RwLock<HashMap<Vec<u8>, usize>>)
2. `index_to_name` (RwLock<Vec<Option<Vec<u8>>>>)

**Rationale:** The atoms map is the primary data structure, and index_to_name provides reverse lookup. This order ensures consistent lock acquisition when creating new atoms.

**Examples:**
- ✅ `atoms.write()` → `index_to_name.write()` (e.g., `put_index()`)
- ❌ `index_to_name.write()` → `atoms.write()` (WRONG - violates order)

**Locations:**
- `entities/entities_data_handling/src/atom.rs:put_index()`

### 9. Global Singleton and Module Table Locks (`code_management_code_loading`)

When accessing global singletons (via `OnceLock`) and module table locks together, use this order:

1. Global singleton access (`get_global_code_ix()`, `get_global_module_manager()`) - ensure initialization completes
2. Module table locks (`ModuleTable.modules` RwLock)

**Rationale:** `OnceLock::get_or_init()` uses internal synchronization during initialization. If module table locks are held while another thread initializes a global singleton, deadlock can occur. Always ensure global singletons are fully initialized before acquiring module table locks.

**Examples:**
- ✅ `get_global_code_ix()` → `module_manager.get_table().put_module()` (OnceLock init completes before RwLock)
- ✅ `get_global_module_manager()` → `table.put_module()` (OnceLock init completes before RwLock)
- ❌ `module_manager.get_table().put_module()` → `get_global_code_ix()` (WRONG - may deadlock if OnceLock is initializing)
- ❌ Holding module table lock while calling `get_global_code_ix()` (WRONG - may deadlock)

**Locations:**
- `code_management_code_loading/src/beam_loader.rs:prepare_loading()`
- `code_management_code_loading/src/beam_loader.rs:finalize_code()`
- `code_management_code_loading/src/beam_loader.rs:test_finalize_code_empty()`

**Note:** In production code, global singletons are typically initialized at startup, so this ordering is less critical. However, in tests that run in parallel, this ordering must be strictly followed. Use `ensure_globals_initialized()` in test setup to prevent initialization deadlocks.

## General Principles

### 1. Always Document Lock Order in Code

When acquiring multiple locks, add a comment indicating the lock order:

```rust
// LOCK ORDER: heap_data -> heap_top_index -> stack_top_index (see LOCKING.md)
let heap_data = self.heap_data.lock().unwrap();
let heap_top = *self.heap_top_index.lock().unwrap();
let stack_top = self.stack_top_index.lock().unwrap();
```

### 2. Minimize Lock Scope

Release locks as soon as possible. Don't hold locks while doing expensive operations:

```rust
// ✅ GOOD: Release lock before expensive operation
let value = {
    let guard = self.data.lock().unwrap();
    guard.clone()  // Clone while holding lock
};  // Lock released here
expensive_operation(value);  // No lock held

// ❌ BAD: Hold lock during expensive operation
let guard = self.data.lock().unwrap();
expensive_operation(&guard);  // Lock held unnecessarily
```

### 3. Avoid Nested Lock Calls

If possible, restructure code to avoid needing multiple locks. Consider:
- Combining related data into a single mutex
- Using atomic operations where possible
- Restructuring algorithms to reduce lock dependencies

### 4. Lock Ordering is Global

Lock ordering rules apply **across the entire codebase**. If function A acquires locks in order X→Y, and function B acquires locks in order Y→X, and both can be called from the same context, you have a deadlock risk.

## Deadlock Detection

### Signs of Potential Deadlocks

1. **Different lock orders in related functions** - If two functions that might be called together use different lock orders, there's a risk.

2. **Lock acquisition in callbacks** - If you acquire a lock, then call a function that might acquire another lock, ensure the order is consistent.

3. **Recursive lock acquisition** - If a function acquires lock A, then calls another function that also needs lock A, use `Mutex` (which allows re-entrancy) or restructure to avoid the need.

### Testing for Deadlocks

- Run code under high concurrency
- Use tools like `cargo test --test-threads=1` to force sequential execution (won't catch deadlocks, but helps with race conditions)
- Consider using `std::sync::Mutex::try_lock()` in tests to detect potential deadlocks

## Adding New Locks

When adding new locks to the codebase:

1. **Document the lock ordering** in this file
2. **Add comments** in code showing lock order
3. **Check for conflicts** with existing lock orders
4. **Update this document** with the new lock ordering rules

## Examples of Correct Lock Usage

### Example 1: Process Stack Operations

```rust
impl Process {
    pub fn stack_push(&self, value: Eterm) -> Result<(), String> {
        // LOCK ORDER: heap_data -> heap_top_index -> stack_top_index
        let mut heap_data = self.heap_data.lock().unwrap();
        let heap_top = *self.heap_top_index.lock().unwrap();
        let mut stack_top = self.stack_top_index.lock().unwrap();
        
        // Use all three locks here...
        Ok(())
    }
}
```

### Example 2: Scheduler Operations

```rust
fn schedule_process(process: Arc<Process>) -> Result<(), String> {
    // LOCK ORDER: schedulers -> runq
    let schedulers = get_global_schedulers()?;
    let schedulers_guard = schedulers.lock().unwrap();
    let scheduler = &schedulers_guard[0];
    let runq = scheduler.runq();
    let runq_guard = runq.lock().unwrap();
    
    // Use both locks here...
    Ok(())
}
```

## Summary

| Lock Group | Order | Critical Functions |
|------------|-------|-------------------|
| Process | `heap_data` → `heap_top_index` → `stack_top_index` | `stack_push()`, `stack_pop()`, `allocate_heap_words()` |
| Scheduler | `schedulers` → `runq` | `create_init_process()`, `spawn_kernel_process()` |
| Global Literals | `lock` → `areas` → `current_offset` → `current_size` | `expand_area()` |
| Module Table Manager | Lower code index → Higher code index | `start_staging()` |
| Code Permission Manager | `stage_permission` → `mod_permission` | `try_seize_code_load_permission()`, `has_code_load_permission()` |
| BEAM Debug Tracer | `traced_mfas` → `traced_by_index` → `next_index` | `set_traced_mfa()`, `clear()` |
| Process Table | `table` → `free_ids` | `remove()`, `clear()`, `new_element()` |
| Atom Table | `atoms` → `index_to_name` | `put_index()` |
| Global Singletons → Module Tables | Global singleton init → Module table locks | `prepare_loading()`, `finalize_code()` |

**Remember: Lock ordering violations are bugs that may not manifest until production under high concurrency. Always follow the documented order and DO NOT use one of the locks listed in this document without following the ENTIRE documented order!**

