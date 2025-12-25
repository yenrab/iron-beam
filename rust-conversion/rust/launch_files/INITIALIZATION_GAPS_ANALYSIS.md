# Initialization Sequence Gaps Analysis

**Date:** 2025-01-27  
**Issue:** The Rust code in `frameworks_emulator_init` doesn't follow the same initialization sequence as the C code and doesn't properly load .beam files.

---

## Critical Gaps Identified

### 1. Missing Preloaded Module Loading

**C Code Behavior (`erl_init.c:2524`):**
```c
load_preloaded();  // Loads preloaded modules (erl_init, init, etc.)
erts_init_process_id = erl_first_process_otp(init, boot_argc, boot_argv);
```

**Rust Code Behavior (`main_init.rs:177-201`):**
- ❌ **Missing:** No equivalent to `load_preloaded()`
- ❌ **Missing:** Preloaded modules (erl_init, init, etc.) are never loaded
- ❌ **Impact:** The init process cannot be created properly because `erl_init` module doesn't exist

**What Needs to Happen:**
1. Implement `load_preloaded()` equivalent that loads preloaded modules from embedded code
2. Load modules: `erl_init`, `init`, and other preloaded modules
3. Register these modules in the module table before creating init process

---

### 2. Init Process Creation Uses Placeholder Code

**C Code Behavior (`erl_init.c:353-396`):**
```c
erl_first_process_otp(char* mod_name, int argc, char** argv) {
    // Creates process by spawning erl_init:start/2 with boot arguments
    res = erl_spawn_system_process(&parent, am_erl_init, am_start, args, &so);
    // This spawns a process that will execute erl_init:start/2
    // The erl_init module must already be loaded (preloaded)
}
```

**Rust Code Behavior (`main_init.rs:254-321`):**
```rust
fn create_init_process() -> Result<(), String> {
    // Creates placeholder process with hardcoded test code
    let mut test_code = Vec::new();
    test_code.push(opcodes::MOVE as u64);  // ❌ Hardcoded test code!
    test_code.push(opcodes::RETURN as u64);
    // ...
}
```

**Problems:**
- ❌ **Uses hardcoded test code** instead of loading actual BEAM code
- ❌ **Doesn't spawn `erl_init:start/2`** like C code does
- ❌ **Doesn't pass boot arguments** to the init process
- ❌ **Doesn't use the code loading infrastructure** that exists in the codebase

**What Needs to Happen:**
1. Load `erl_init.beam` from filesystem (or preloaded code)
2. Create process that will execute `erl_init:start/2` with boot arguments
3. Use proper process spawning that loads BEAM code, not hardcoded instructions

---

### 3. Boot Script Loading Doesn't Use Code Loading System

**C Code Behavior:**
- Boot script is loaded by the init process (after it starts)
- Init process calls `init:boot/1` which loads modules from .beam files
- Uses the code loading infrastructure to load modules from filesystem

**Rust Code Behavior (`main_init.rs:230-244`):**
```rust
fn load_boot_script(boot_path: &str, rootdir: &str, bindir: &str) -> Result<(), String> {
    // Uses boot_script module to parse script
    // But doesn't actually load .beam files properly
    boot_script::execute_boot_script(&script)?;
}
```

**Problems:**
- ⚠️ **Boot script parsing exists** but execution is incomplete
- ❌ **Module loading in boot script** (`load_modules()`) doesn't properly integrate with code loading
- ❌ **Loaded modules aren't registered** in the module table correctly
- ❌ **Code isn't made executable** - modules are loaded but code isn't available to processes

**What Needs to Happen:**
1. Boot script execution should use the code loading infrastructure
2. Loaded modules must be registered in module table
3. Code must be made executable (code index management)
4. Process spawning must be able to find and execute loaded code

---

### 4. .beam Files Created by Makefile Are Not Used

**Makefile Behavior:**
- Creates .beam files in `target/otp_root/lib/stdlib-VSN/ebin/`
- Creates .beam files in `target/otp_root/lib/kernel-VSN/ebin/`
- Creates .beam files in `target/otp_root/lib/compiler-VSN/ebin/`
- Creates .beam files in `target/otp_root/lib/sasl-VSN/ebin/`

**Rust Code Behavior:**
- ❌ **Code paths don't include** `target/otp_root/lib/*/ebin/` directories
- ❌ **Module loading doesn't search** these directories
- ❌ **Boot script doesn't set code paths** to include these directories

**What Needs to Happen:**
1. Set code paths to include `target/otp_root/lib/*/ebin/` directories
2. Boot script should use these code paths when loading modules
3. Module loading should search these paths

---

### 5. Code Loading Infrastructure Exists But Isn't Integrated

**Existing Rust Code:**
- ✅ `code_management_code_loading::CodeLoader` - High-level code loading
- ✅ `code_management_code_loading::BeamLoader` - BEAM file parsing
- ✅ `code_management_code_loading::module_management` - Module table
- ✅ `usecases_bifs::load::LoadBif` - Module loading BIFs

**Problem:**
- ❌ **These aren't used** in the initialization sequence
- ❌ **Init process creation** doesn't use code loading
- ❌ **Boot script execution** doesn't properly use code loading

**What Needs to Happen:**
1. Use `CodeLoader::load_module()` to load .beam files
2. Use `BeamLoader::read_beam_file()` to parse BEAM files
3. Use module management to register loaded modules
4. Use code index management to make code executable

---

## Required Fixes

### Fix 1: Implement Preloaded Module Loading

**Location:** `frameworks_emulator_init/src/main_init.rs`

**Add function:**
```rust
/// Load preloaded modules
///
/// Based on load_preloaded() from erl_init.c
/// Loads preloaded modules (erl_init, init, etc.) from embedded code
fn load_preloaded() -> Result<(), String> {
    // TODO: Load preloaded modules from embedded code
    // For now, load from filesystem as fallback
    use code_management_code_loading::CodeLoader;
    use std::path::Path;
    
    let preloaded_modules = ["erl_init", "init"];
    let code_paths = get_preloaded_code_paths();
    
    for module_name in &preloaded_modules {
        // Try to load from filesystem
        for code_path in &code_paths {
            let beam_path = Path::new(code_path).join(format!("{}.beam", module_name));
            if beam_path.exists() {
                match CodeLoader::load_module(&beam_path) {
                    Ok(_) => {
                        eprintln!("Loaded preloaded module: {}", module_name);
                        break;
                    }
                    Err(_) => continue,
                }
            }
        }
    }
    
    Ok(())
}
```

**Call before creating init process:**
```rust
pub fn erl_start(...) -> Result<(), String> {
    // ... existing code ...
    
    // Load preloaded modules (must be before creating init process)
    load_preloaded()
        .map_err(|e| format!("Failed to load preloaded modules: {}", e))?;
    
    // Now create init process
    create_init_process()?;
    
    // ...
}
```

---

### Fix 2: Fix Init Process Creation to Use BEAM Code

**Location:** `frameworks_emulator_init/src/main_init.rs`

**Replace `create_init_process()`:**
```rust
/// Create init process
///
/// Based on erl_first_process_otp() from erl_init.c
/// Creates the init process by spawning erl_init:start/2 with boot arguments
fn create_init_process() -> Result<(), String> {
    use entities_process::Process;
    use infrastructure_utilities::process_table::get_global_process_table;
    use code_management_code_loading::module_management;
    use std::sync::Arc;
    
    // Verify erl_init module is loaded
    if !module_management::is_module_loaded("erl_init") {
        return Err("erl_init module not loaded (preloaded modules must be loaded first)".to_string());
    }
    
    // Get boot arguments from command line
    let boot_args = extract_boot_args();
    
    // Create process that will execute erl_init:start/2
    // This requires:
    // 1. Load erl_init module code
    // 2. Find erl_init:start/2 export
    // 3. Create process with that code
    // 4. Set up arguments (boot module name, boot args)
    // 5. Schedule process
    
    // TODO: Implement proper process spawning with BEAM code
    // For now, this is a placeholder that shows what needs to happen
    
    Err("Init process creation with BEAM code not yet implemented".to_string())
}
```

**This requires:**
1. Process spawning that loads BEAM code
2. Export table lookup to find `erl_init:start/2`
3. Code execution setup (instruction pointer, registers, etc.)

---

### Fix 3: Fix Boot Script to Use Code Loading Infrastructure

**Location:** `frameworks_emulator_init/src/boot_script.rs`

**Update `load_modules()`:**
```rust
fn load_modules(modules: &[String]) -> Result<(), String> {
    use code_management_code_loading::CodeLoader;
    use code_management_code_loading::BeamLoader;
    use code_management_code_loading::module_management;
    use std::path::Path;
    use std::fs;
    
    let code_paths = get_code_paths();  // Should include target/otp_root/lib/*/ebin/
    
    for module_name in modules {
        let mut found = false;
        
        for code_path in &code_paths {
            let beam_path = Path::new(code_path).join(format!("{}.beam", module_name));
            
            if !beam_path.exists() {
                continue;
            }
            
            // Read BEAM file
            match fs::read(&beam_path) {
                Ok(beam_data) => {
                    // Parse BEAM file
                    match BeamLoader::read_beam_file(&beam_data) {
                        Ok(beam_file) => {
                            // Load module using code loading infrastructure
                            match CodeLoader::load_module(&beam_path) {
                                Ok(_) => {
                                    // Register module in module table
                                    module_management::register_module(
                                        module_name,
                                        beam_file.exports,
                                        // ... other metadata
                                    )?;
                                    
                                    eprintln!("      ✓ Loaded: {} (from {})", 
                                             module_name, beam_path.display());
                                    found = true;
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("      ✗ Failed to load: {} - {:?}", module_name, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("      ✗ Invalid BEAM format: {} - {:?}", module_name, e);
                        }
                    }
                }
                Err(_) => continue,
            }
        }
        
        if !found {
            return Err(format!("Module {} not found in code paths: {:?}", module_name, code_paths));
        }
    }
    
    Ok(())
}
```

---

### Fix 4: Set Code Paths to Include Makefile Output

**Location:** `frameworks_emulator_init/src/boot_script.rs`

**Update `get_code_paths()`:**
```rust
fn get_code_paths() -> Vec<String> {
    use crate::env;
    
    let (rootdir, _bindir) = env::determine_paths().unwrap_or_else(|_| {
        (String::new(), String::new())
    });
    
    let mut paths = Vec::new();
    
    // Add paths from ROOTDIR/lib/*/ebin/
    if !rootdir.is_empty() {
        let lib_dir = Path::new(&rootdir).join("lib");
        if lib_dir.exists() {
            // Find all application directories
            if let Ok(entries) = fs::read_dir(&lib_dir) {
                for entry in entries.flatten() {
                    let app_dir = entry.path();
                    if app_dir.is_dir() {
                        let ebin_dir = app_dir.join("ebin");
                        if ebin_dir.exists() {
                            paths.push(ebin_dir.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    // Add default paths
    paths.push(format!("{}/lib/stdlib-*/ebin", rootdir));
    paths.push(format!("{}/lib/kernel-*/ebin", rootdir));
    paths.push(format!("{}/lib/compiler-*/ebin", rootdir));
    paths.push(format!("{}/lib/sasl-*/ebin", rootdir));
    
    paths
}
```

---

## Summary

The Rust code needs to:

1. ✅ **Load preloaded modules** before creating init process
2. ✅ **Create init process with actual BEAM code** (not placeholder)
3. ✅ **Use code loading infrastructure** to load .beam files
4. ✅ **Set code paths** to include Makefile output directories
5. ✅ **Integrate boot script** with code loading system
6. ✅ **Make loaded code executable** (code index management)

**Current Status:** The infrastructure exists but isn't integrated into the initialization sequence. The code needs to be refactored to use the existing code loading infrastructure instead of placeholders.

---

## Next Steps

1. Implement `load_preloaded()` function
2. Fix `create_init_process()` to use BEAM code
3. Update boot script execution to use code loading
4. Set code paths correctly
5. Test with actual .beam files from Makefile

