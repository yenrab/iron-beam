# Function Reorganization Analysis for New Behavior Groups

This document identifies functions in the new C files that should be moved to different behavior groups based on CLEAN architecture principles and SOLID responsibilities.

## 1. `bif.c` → Split into Multiple Groups

### Current Assignment: `infrastructure_bif_dispatcher`
**Issue**: `bif.c` contains both BIF dispatcher infrastructure AND BIF implementations (business logic).

### Functions to MOVE to `usecases_bifs`:
All BIF implementations (business logic) should move to `usecases_bifs`:

**Process Management BIFs:**
- `spawn_3()` - Process spawning
- `spawn_link_3()` - Process spawning with link
- `spawn_opt_4()` - Process spawning with options
- `erts_internal_spawn_request_4()` - Internal spawn request
- `spawn_request_abandon_1()` - Abandon spawn request
- `link_1()`, `link_2()` - Process linking
- `unlink_1()` - Process unlinking
- `monitor_2()`, `monitor_3()` - Process monitoring
- `demonitor_1()`, `demonitor_2()` - Process demonitoring
- `exit_1()`, `exit_2()`, `exit_3()` - Process exit
- `exit_signal_2()` - Send exit signal
- `process_flag_2()` - Process flags
- `erts_internal_process_flag_3()` - Internal process flags
- `hibernate_3()` - Process hibernation
- `self_0()` - Get self process
- `group_leader_0()` - Get group leader
- `erts_internal_group_leader_2()`, `erts_internal_group_leader_3()` - Internal group leader

**Message Passing BIFs:**
- `send_2()`, `send_3()` - Send messages
- `erl_send()` - Internal send function
- `dsend_continue_trap_1()` - Distributed send continuation

**Registration BIFs:**
- `register_2()` - Register process/port
- `unregister_1()` - Unregister
- `whereis_1()` - Find registered process/port

**Data Structure BIFs:**
- `hd_1()`, `tl_1()` - List operations
- `element_2()` - Tuple element access
- `tuple_size_1()` - Tuple size
- `setelement_3()` - Set tuple element
- `make_tuple_2()`, `make_tuple_3()` - Create tuples
- `append_element_2()` - Append to tuple
- `insert_element_3()` - Insert into tuple
- `delete_element_2()` - Delete from tuple
- `tuple_to_list_1()`, `list_to_tuple_1()` - Tuple/list conversion

**Type Conversion BIFs:**
- `atom_to_list_1()` - Atom to list
- `list_to_atom_1()` - List to atom
- `list_to_existing_atom_1()` - List to existing atom
- `integer_to_list_1()`, `integer_to_list_2()` - Integer to list
- `float_to_list_1()`, `float_to_list_2()` - Float to list
- `float_to_binary_1()`, `float_to_binary_2()` - Float to binary
- `list_to_float_1()` - List to float
- `binary_to_float_1()` - Binary to float
- `string_list_to_float_1()` - String list to float
- `pid_to_list_1()`, `list_to_pid_1()` - PID conversion
- `port_to_list_1()`, `list_to_port_1()` - Port conversion
- `ref_to_list_1()`, `list_to_ref_1()` - Reference conversion
- `fun_to_list_1()` - Function to list
- `make_fun_3()` - Create function

**Error Handling BIFs:**
- `error_1()`, `error_2()`, `error_3()` - Error generation
- `nif_error_1()`, `nif_error_2()` - NIF error generation
- `throw_1()` - Throw exception
- `raise_3()` - Raise exception

**System BIFs:**
- `time_0()`, `date_0()` - Time/date
- `universaltime_0()`, `localtime_0()` - Time conversion
- `universaltime_to_localtime_1()` - Timezone conversion
- `universaltime_to_posixtime_1()` - Time conversion
- `posixtime_to_universaltime_1()` - Time conversion
- `now_0()` - Current time
- `processes_0()` - List processes
- `erts_internal_processes_next_1()` - Process iteration
- `ports_0()` - List ports
- `display_1()`, `display_string_2()` - Debug display
- `erts_internal_term_to_string_2()` - Term to string
- `halt_2()` - System halt
- `function_exported_3()` - Check function export
- `is_builtin_3()` - Check if builtin
- `system_flag_2()` - System flags
- `erts_internal_scheduler_wall_time_1()` - Scheduler wall time
- `phash_2()`, `phash2_1()` - Hashing

**Helper Functions (also move to usecases_bifs):**
- `link_opt()` - Link with options
- `monitor()` - Monitor helper
- `send_exit_signal_bif()` - Send exit signal helper
- `process_flag_aux()` - Process flag helper
- `remote_send()` - Remote send helper
- `integer_to_list()` - Integer conversion helper
- `do_float_to_list()` - Float conversion helper
- `do_float_to_binary()` - Float binary conversion helper
- `do_charbuf_to_float()` - Buffer conversion helper

### Functions to KEEP in `infrastructure_bif_dispatcher`:
**BIF Dispatcher Infrastructure:**
- `erts_init_bif()` - BIF system initialization
- `erts_init_trap_export()` - Trap export initialization
- `call_bif()` - BIF call dispatcher
- `erts_call_dirty_bif()` - Dirty BIF dispatcher
- `bif_return_trap()` - BIF return trap handler
- `bif_handle_signals_return()` - Signal return handler
- `erts_internal_await_exit_trap()` - Await exit trap
- All trap export setup code

**Rationale**: The dispatcher functions are infrastructure that routes calls to BIFs, while the BIF implementations themselves are business logic (use cases).

---

## 2. `external.c` → Split into Multiple Groups

### Current Assignment: `infrastructure_external_format`
**Issue**: `external.c` contains both encoding/decoding infrastructure AND BIF implementations.

### Functions to MOVE to `usecases_bifs`:
**BIF Wrappers (these are BIFs, not infrastructure):**
- `term_to_binary_1()`, `term_to_binary_2()` - Term to binary BIF
- `binary_to_term_1()`, `binary_to_term_2()` - Binary to term BIF
- `term_to_iovec_1()`, `term_to_iovec_2()` - Term to I/O vector BIF
- `term_to_binary_trap_1()` - Term to binary trap
- `binary_to_term_trap_1()` - Binary to term trap
- `erts_term_to_binary_int()` - Internal term to binary (used by BIFs)
- `binary_to_term_int()` - Internal binary to term (used by BIFs)
- `erts_debug_dist_ext_to_term_2()` - Debug BIF

### Functions to KEEP in `infrastructure_external_format`:
**Encoding Infrastructure:**
- `enc_term()` - Encode term
- `enc_atom()` - Encode atom
- `enc_pid()` - Encode PID
- `enc_term_int()` - Internal encoding
- `erts_encode_ext()` - Encode external format
- `erts_encode_ext_size()` - Calculate encoding size
- `erts_encode_ext_size_2()` - Calculate encoding size with flags
- `erts_encode_ext_size_ets()` - ETS encoding size
- `erts_encode_dist_ext()` - Encode for distribution
- `erts_encode_ext_dist_header_finalize()` - Finalize distribution header
- `encode_size_struct_int()` - Size calculation
- `is_external_string()` - Check if external string
- `store_in_vec()` - Store in I/O vector
- `calc_iovec_fun_size()` - Calculate I/O vector size
- `transcode_dist_obuf()` - Transcode distribution buffer

**Decoding Infrastructure:**
- `dec_term()` - Decode term
- `dec_atom()` - Decode atom
- `dec_pid()` - Decode PID
- `decoded_size()` - Calculate decoded size
- `erts_decode_ext()` - Decode external format
- `erts_decode_ext_ets()` - Decode for ETS
- `erts_decode_ext_size()` - Calculate decode size
- `erts_decode_ext_size_ets()` - ETS decode size
- `dec_is_this_node()` - Check if this node

**Initialization:**
- `erts_init_external()` - External format initialization
- `erts_late_init_external()` - Late initialization
- `erts_debug_max_atom_out_cache_index()` - Debug function

**Context Management:**
- `b2t_export_context()` - Binary-to-term context export
- `b2t_destroy_context()` - Destroy context
- `b2t_context_destructor()` - Context destructor
- `b2t_rand()` - Random number for context
- `ttb_context_destructor()` - TTB context destructor
- `binary2term_uncomp_size()` - Uncompressed size

**Rationale**: The encoding/decoding functions are infrastructure, but the BIF wrappers that call them are business logic (use cases).

---

## 3. `erl_process.c` → Split into Multiple Groups

### Current Assignment: `usecases_scheduling`
**Issue**: `erl_process.c` contains scheduling, process creation, and process lifecycle management - these are different responsibilities.

### Functions to MOVE to `usecases_process_management`:
**Process Creation:**
- `erl_create_process()` - Create new process
- `alloc_process()` - Allocate process structure
- `early_init_process_struct()` - Early initialization
- `erts_parse_spawn_opts()` - Parse spawn options
- `erts_send_local_spawn_reply()` - Send spawn reply

**Process Lifecycle:**
- `erts_free_proc()` - Free process
- `delete_process()` - Delete process
- `erts_set_self_exiting()` - Set exiting state
- `erts_do_exit_process()` - Exit process
- `erts_continue_exit_process()` - Continue exit
- `erts_proc_exit_link()` - Exit with link
- `erts_init_empty_process()` - Initialize empty process
- `erts_cleanup_empty_process()` - Cleanup empty process
- `erts_debug_verify_clean_empty_process()` - Debug verification

**Process State Management:**
- `erts_get_cpu_topology_term()` - Get CPU topology
- `erts_get_schedulers_binds()` - Get scheduler binds
- `erts_set_cpu_topology()` - Set CPU topology
- `erts_bind_schedulers()` - Bind schedulers

### Functions to KEEP in `usecases_scheduling`:
**Scheduling:**
- `erts_schedule()` - Main scheduler function
- `erts_schedid2runq()` - Scheduler ID to run queue
- `dequeue_process()` - Dequeue from run queue
- `enqueue_process()` - Enqueue to run queue
- `check_requeue_process()` - Check if should requeue
- `free_proxy_proc()` - Free proxy process
- `clear_proc_dirty_queue_bit()` - Clear dirty queue bit
- `trace_schedule_in()` - Schedule in tracing
- `trace_schedule_out()` - Schedule out tracing

**Scheduler Management:**
- `init_scheduler_suspend()` - Initialize scheduler suspend
- `setup_aux_work_timer()` - Setup aux work timer
- `wake_scheduler()` - Wake scheduler
- `init_aux_work_data()` - Initialize aux work data
- `check_sleepers_list()` - Check sleepers
- `print_current_process_info()` - Print process info

**Run Queue Management:**
- All run queue manipulation functions
- Process priority queue functions
- Scheduler sleep/wake functions

**Rationale**: Process creation and lifecycle are separate from scheduling. Scheduling is about when to run processes, while process management is about creating/destroying processes.

---

## 4. `utils.c` → Keep in Infrastructure, but Consider Splitting

### Current Assignment: `infrastructure_runtime_utils`
**Status**: Mostly correct, but some functions might belong elsewhere.

### Functions to CONSIDER MOVING:

**To `entities_data_handling` (if term building is considered entity-level):**
- `erts_bld_atom()` - Build atom term
- `erts_bld_uint()` - Build uint term
- `erts_bld_uword()` - Build uword term
- `erts_bld_uint64()` - Build uint64 term
- `erts_bld_sint64()` - Build sint64 term
- `erts_bld_cons()` - Build cons cell
- `erts_bld_tuple()` - Build tuple
- `erts_bld_tuplev()` - Build tuple from vector
- `erts_bld_string_n()` - Build string
- `erts_bld_list()` - Build list
- `erts_bld_2tup_list()` - Build 2-tuple list
- `erts_bld_atom_uword_2tup_list()` - Build atom-uword list
- `erts_bld_atom_2uint_3tup_list()` - Build atom-2uint list

**Rationale**: Term building could be considered entity-level operations, but they're also utilities. Current placement is acceptable.

### Functions to KEEP in `infrastructure_runtime_utils`:
- All comparison functions (`eq()`, `erts_cmp()`, etc.)
- All printing/formatting functions
- All string utilities
- All initialization functions
- All memory allocation utilities
- All general-purpose utilities

---

## 5. `io.c` → Removed from Design

### Previous Assignment: `adapters_io_subsystem`
**Status**: Removed from design.

**Note**: This implementation does not include ports and drivers. The `io.c` file and `adapters_io_subsystem` group have been removed from the design.

---

## 6. `beam_emu.c` → Keep in Infrastructure

### Current Assignment: `infrastructure_emulator_loop`
**Status**: Correct placement.

**Note**: The emulator loop (`process_main()`) is core infrastructure. No reorganization needed.

---

## 7. `erl_init.c` → Keep in Frameworks

### Current Assignment: `frameworks_emulator_init`
**Status**: Correct placement.

**Note**: Initialization is framework-level. No reorganization needed.

---

## Summary of Required Reorganizations

1. **`bif.c`**: Move ~80+ BIF implementation functions to `usecases_bifs`, keep ~10 dispatcher functions in `infrastructure_bif_dispatcher`

2. **`external.c`**: Move ~8 BIF wrapper functions to `usecases_bifs`, keep ~30 encoding/decoding functions in `infrastructure_external_format`

3. **`erl_process.c`**: Move ~15 process creation/lifecycle functions to `usecases_process_management`, keep ~20 scheduling functions in `usecases_scheduling`

4. **`utils.c`**: Consider moving term building functions to `entities_data_handling`, but current placement is acceptable

5. **`io.c`**: No changes needed

6. **`beam_emu.c`**: No changes needed

7. **`erl_init.c`**: No changes needed

---

## Impact on Design Documents

After reorganization:
- `infrastructure_bif_dispatcher` will be much smaller (only dispatcher functions)
- `usecases_bifs` will grow significantly (adds ~80+ functions from `bif.c` and ~8 from `external.c`)
- `usecases_process_management` will grow (adds ~15 functions from `erl_process.c`)
- `usecases_scheduling` will be more focused (only scheduling, not process creation)

