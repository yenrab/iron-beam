# Requirements for Erlang REPL Execution

This document outlines what needs to be implemented to fully support launching and executing Erlang code in the REPL.

## Current Status

✅ **Completed:**
1. Scheduler threads started and running
2. Main execution loop entered (scheduler threads processing work)
3. Boot script loading attempted (placeholder)
4. Init process created and scheduled (placeholder)

⚠️ **Partially Implemented:**
- Process execution loop exists but doesn't actually execute BEAM instructions
- Boot script parsing not implemented
- Init process doesn't execute Erlang code

❌ **Missing:**
- BEAM instruction execution
- Erlang shell (user_drv) startup
- Code loading and module execution
- Process code execution

---

## What Needs to Happen

### 1. **BEAM Instruction Execution** (Critical)

**Current State:**
- `process_main()` exists but returns `None` (placeholder)
- `InstructionExecutor` trait exists but not implemented
- No actual BEAM instruction handlers

**What's Needed:**
1. Implement BEAM instruction execution in `process_main()`:
   ```rust
   // In infrastructure_emulator_loop/src/emulator_loop.rs
   pub fn process_main(
       emulator_loop: &mut EmulatorLoop,
       process: Arc<Process>,
   ) -> Result<InstructionResult, EmulatorLoopError> {
       // 1. Copy registers from process to emulator loop
       copy_in_registers(process, emulator_loop);
       
       // 2. Get instruction pointer from process
       let instruction_ptr = process.get_instruction_ptr();
       
       // 3. Execute instructions in a loop until process yields
       loop {
           // Decode instruction
           let instruction = decode_instruction(instruction_ptr);
           
           // Execute instruction
           let result = execute_instruction(instruction, emulator_loop)?;
           
           // Handle result
           match result {
               InstructionResult::Continue => {
                   // Move to next instruction
                   instruction_ptr = next_instruction(instruction_ptr);
               }
               InstructionResult::Yield => {
                   // Process out of reductions, reschedule
                   copy_out_registers(emulator_loop, process);
                   return Ok(InstructionResult::Yield);
               }
               InstructionResult::NormalExit => {
                   // Process finished normally
                   return Ok(InstructionResult::NormalExit);
               }
               // ... other cases
           }
       }
   }
   ```

2. Implement instruction handlers for all BEAM instructions:
   - Arithmetic operations (add, sub, mul, div)
   - Control flow (call, return, jump, branch)
   - Data operations (move, load, store)
   - Process operations (spawn, send, receive)
   - BIF calls
   - And ~200+ other instructions

**Files to Modify:**
- `infrastructure/infrastructure_emulator_loop/src/emulator_loop.rs`
- `infrastructure/infrastructure_emulator_loop/src/instruction_execution.rs`
- Create instruction handler modules

---

### 2. **Integrate Process Execution with Scheduler** (Critical)

**Current State:**
- Scheduler threads call `erts_schedule()` which dequeues processes
- But processes are not actually executed

**What's Needed:**
1. Update `scheduler_thread_func()` to call `process_main()`:
   ```rust
   // In usecases/usecases_scheduling/src/threads.rs
   fn scheduler_thread_func(...) {
       while running {
           // Dequeue process
           if let Some(process) = dequeue_process(...) {
               // Create emulator loop for this scheduler
               let mut emulator_loop = EmulatorLoop::new();
               
               // Execute the process
               match process_main(&mut emulator_loop, process.clone()) {
                   Ok(InstructionResult::Yield) => {
                       // Process yielded, reschedule if needed
                       if should_reschedule(&process) {
                           enqueue_process(...);
                       }
                   }
                   Ok(InstructionResult::NormalExit) => {
                       // Process finished, remove from table
                       process_table.remove(process.id());
                   }
                   // ... handle other cases
               }
           }
       }
   }
   ```

**Files to Modify:**
- `usecases/usecases_scheduling/src/threads.rs`
- `usecases/usecases_scheduling/src/scheduler.rs`

---

### 3. **Boot Script Loading and Execution** (High Priority)

**Current State:**
- Boot script path is extracted
- File existence is checked
- But script is not parsed or executed

**What's Needed:**
1. Parse boot script (`.boot` file is binary Erlang term):
   ```rust
   // In frameworks/frameworks_emulator_init/src/main_init.rs
   fn load_boot_script(boot_path: &str) -> Result<BootScript, String> {
       // 1. Read .boot file (binary Erlang term format)
       let boot_data = std::fs::read(boot_path)?;
       
       // 2. Decode binary Erlang term
       let boot_term = decode_erlang_term(&boot_data)?;
       
       // 3. Parse script structure
       // {script, {Name, Vsn}, [Commands]}
       match boot_term {
           ErlangTerm::Tuple([ErlangTerm::Atom("script"), 
                              ErlangTerm::Tuple([name, vsn]), 
                              ErlangTerm::List(commands)]) => {
               Ok(BootScript { name, vsn, commands })
           }
           _ => Err("Invalid boot script format".to_string())
       }
   }
   ```

2. Execute boot script commands:
   - `{preLoaded, [Mod1, Mod2, ...]}` - Mark modules as preloaded
   - `{path, [Dir1, ...]}` - Set code search path
   - `{primLoad, [Mod1, ...]}` - Load modules
   - `{kernelProcess, Name, {Mod, Func, Args}}` - Start kernel processes
   - `{apply, {Mod, Func, Args}}` - Call functions
   - `{progress, Status}` - Update progress

**Files to Create/Modify:**
- Create `frameworks/frameworks_emulator_init/src/boot_script.rs`
- Modify `frameworks/frameworks_emulator_init/src/main_init.rs`
- Use `code_management_code_loading` for module loading

---

### 4. **Init Process Code Execution** (High Priority)

**Current State:**
- Init process is created as a placeholder `Process::new(1)`
- Process has no code to execute
- Process is scheduled but doesn't run

**What's Needed:**
1. Create init process with actual Erlang code:
   ```rust
   // In frameworks/frameworks_emulator_init/src/main_init.rs
   fn create_init_process() -> Result<(), String> {
       // 1. Load init module code
       let init_module = load_module("init")?;
       
       // 2. Create process with init:boot/1 entry point
       let init_process = create_process_with_code(
           pid: 1,
           module: "init",
           function: "boot",
           args: [boot_script_path],
       )?;
       
       // 3. Set up process heap and stack
       setup_process_memory(&init_process, init_args)?;
       
       // 4. Set instruction pointer to init:boot/1
       init_process.set_instruction_ptr(
           get_function_entry_point("init", "boot", 1)
       );
       
       // 5. Add to process table and schedule
       process_table.insert(1, init_process);
       schedule_process(init_process, Priority::Max);
   }
   ```

2. The init process needs to:
   - Execute `init:boot/1` function
   - Load modules from boot script
   - Start kernel processes (code_server, erl_prim_loader, etc.)
   - Start applications
   - Start Erlang shell

**Files to Modify:**
- `frameworks/frameworks_emulator_init/src/main_init.rs`
- `entities/entities_process/src/process.rs` (add code execution setup)
- `code_management/code_management_code_loading/` (for module loading)

---

### 5. **Erlang Shell (user_drv) Startup** (High Priority)

**Current State:**
- Shell is not started
- No user interaction possible

**What's Needed:**
1. After boot script execution, start the shell:
   ```rust
   // This happens in init process after boot script
   // init:boot/1 calls shell:start_interactive()
   
   // In lib/stdlib/src/shell.erl (Erlang code):
   start_interactive() ->
       user_drv:start_shell().
   
   // user_drv:start_shell() creates:
   // 1. user_drv process (handles terminal I/O)
   // 2. group_leader process
   // 3. shell process (evaluates Erlang expressions)
   ```

2. The shell process needs:
   - Read input from terminal (stdin)
   - Parse Erlang expressions
   - Evaluate expressions using `erl_eval`
   - Print results to terminal (stdout)
   - Handle line editing (if supported)

**Files to Create:**
- Shell process creation (in init process after boot)
- Terminal I/O handling
- Expression parsing and evaluation

**Note:** The shell is Erlang code, so it needs the BEAM instruction execution to work first.

---

### 6. **Code Loading Infrastructure** (Medium Priority)

**Current State:**
- `code_management_code_loading` exists
- But modules can't be loaded into running processes

**What's Needed:**
1. Load BEAM files:
   - Read `.beam` file
   - Parse BEAM chunks (Code, Str, Imp, Exp, etc.)
   - Load code into memory
   - Register module in code server

2. Link code to processes:
   - Set process instruction pointer to module code
   - Set up function entry points
   - Handle code versioning

**Files to Use:**
- `code_management/code_management_code_loading/src/beam_loader.rs`
- `code_management/code_management_code_loading/src/code_loader.rs`

---

### 7. **Process Memory Management** (Medium Priority)

**Current State:**
- Process structure exists
- But heap/stack not properly managed

**What's Needed:**
1. Process heap allocation:
   - Allocate heap for process data
   - Grow heap when needed
   - Garbage collection

2. Process stack:
   - Function call stack
   - Return addresses
   - Local variables

**Files to Modify:**
- `entities/entities_process/src/process.rs`
- Memory management infrastructure

---

## Implementation Order

**Phase 1: Core Execution (Must Have)**
1. Implement basic BEAM instruction execution
   - Start with simple instructions (move, call, return)
   - Get a process to execute at least one instruction
2. Integrate process execution with scheduler
   - Make scheduler threads actually execute processes
3. Basic process creation with code
   - Create a process with a simple function to execute

**Phase 2: Boot and Init (High Priority)**
4. Boot script parsing
   - Parse binary boot script format
5. Init process with real code
   - Load init module
   - Execute init:boot/1
6. Module loading
   - Load modules from boot script

**Phase 3: Shell (High Priority)**
7. Start Erlang shell
   - After boot script completes
   - Start user_drv and shell processes
8. Terminal I/O
   - Read input, print output

**Phase 4: Full Execution (Nice to Have)**
9. Complete instruction set
   - Implement all BEAM instructions
10. Advanced features
    - Garbage collection
    - Hot code loading
    - Distribution

---

## Testing Strategy

1. **Unit Tests:**
   - Test individual instruction execution
   - Test process creation
   - Test boot script parsing

2. **Integration Tests:**
   - Test scheduler executing a process
   - Test init process loading boot script
   - Test shell startup

3. **End-to-End Tests:**
   - Launch emulator
   - Execute simple Erlang expression: `2 + 2.`
   - Verify output: `4`

---

## Summary

To get a working REPL, you need:

1. ✅ **Scheduler threads running** (DONE)
2. ✅ **Process scheduling** (DONE)
3. ❌ **BEAM instruction execution** (MISSING - Critical)
4. ❌ **Process code execution** (MISSING - Critical)
5. ⚠️ **Boot script execution** (PLACEHOLDER - High Priority)
6. ⚠️ **Init process with real code** (PLACEHOLDER - High Priority)
7. ❌ **Erlang shell startup** (MISSING - High Priority)
8. ❌ **Terminal I/O** (MISSING - High Priority)

The most critical missing piece is **BEAM instruction execution**. Without it, processes can't actually run Erlang code, which means:
- Init process can't execute `init:boot/1`
- Boot script can't be processed
- Shell can't start
- No REPL possible

Once BEAM instruction execution works, the rest can be built on top of it.

