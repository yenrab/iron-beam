//! Build script for infrastructure_beamasm
//!
//! This build script compiles the C++ wrapper and links against the asmjit C++ library.
//! asmjit is embedded in the Erlang source tree at erts/emulator/asmjit/

use std::env;
use std::path::PathBuf;

fn main() {
    // Get paths
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // Try to find asmjit directory - go up 4 levels from manifest_dir
    let asmjit_dir = manifest_dir
        .parent() // infrastructure_beamasm -> infrastructure
        .and_then(|p| p.parent()) // infrastructure -> rust
        .and_then(|p| p.parent()) // rust -> rust-conversion
        .and_then(|p| p.parent()) // rust-conversion -> project root
        .map(|p| p.join("erts/emulator/asmjit"))
        .and_then(|p| if p.exists() { Some(p) } else { None })
        .unwrap_or_else(|| {
            panic!("asmjit directory not found. Tried: {}", 
                   manifest_dir.parent()
                       .and_then(|p| p.parent())
                       .and_then(|p| p.parent())
                       .and_then(|p| p.parent())
                       .map(|p| p.join("erts/emulator/asmjit"))
                       .map(|p| p.display().to_string())
                       .unwrap_or_else(|| "unknown".to_string()));
        });
    
    let cpp_dir = manifest_dir.join("cpp");
    
    // Compile asmjit core source files
    let mut asmjit_build = cc::Build::new();
    asmjit_build.cpp(true);
    asmjit_build.std("c++17");
    asmjit_build.include(asmjit_dir.parent().unwrap()); // erts/emulator

    // Compiler flags (matching Erlang's Makefile exactly)
    asmjit_build.flag("-DASMJIT_EMBED=1");
    asmjit_build.flag("-DASMJIT_NO_BUILDER=1");
    asmjit_build.flag("-DASMJIT_NO_DEPRECATED=1");
    asmjit_build.flag("-DASMJIT_STATIC=1");
    asmjit_build.flag("-DASMJIT_NO_FOREIGN=1");

    // Match C build optimization: use -O2 with no-inline-functions (matching GEN_OPT_FLGS pattern)
    asmjit_build.opt_level(2);
    asmjit_build.flag("-fno-inline-functions");

    // Filter out format warnings like the C build does
    asmjit_build.flag("-Wno-format");
    asmjit_build.flag("-Wno-format=2");
    
    // Architecture-specific flags
    #[cfg(target_arch = "x86_64")]
    {
        asmjit_build.flag("-DASMJIT_BUILD_X86=1");
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        asmjit_build.flag("-DASMJIT_BUILD_ARM=1");
    }
    
    // Add all core source files
    let core_dir = asmjit_dir.join("core");
    for entry in std::fs::read_dir(&core_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("cpp") {
            asmjit_build.file(&path);
        }
    }
    
    // Add architecture-specific source files
    #[cfg(target_arch = "x86_64")]
    {
        let x86_dir = asmjit_dir.join("x86");
        for entry in std::fs::read_dir(&x86_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("cpp") {
                asmjit_build.file(&path);
            }
        }
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        let arm_dir = asmjit_dir.join("arm");
        for entry in std::fs::read_dir(&arm_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("cpp") {
                asmjit_build.file(&path);
            }
        }
    }
    
    // Compile asmjit
    asmjit_build.compile("asmjit");
    
    // Compile C++ wrapper
    let mut build = cc::Build::new();

    // Add C++ wrapper source
    build.file(cpp_dir.join("asmjit_wrapper.cpp"));

    // Set C++ standard
    build.cpp(true);
    build.std("c++17");

    // Add include paths
    build.include(asmjit_dir.parent().unwrap()); // erts/emulator
    build.include(&cpp_dir);

    // Same compiler flags as asmjit
    build.flag("-DASMJIT_EMBED=1");
    build.flag("-DASMJIT_NO_BUILDER=1");
    build.flag("-DASMJIT_NO_DEPRECATED=1");
    build.flag("-DASMJIT_STATIC=1");
    build.flag("-DASMJIT_NO_FOREIGN=1");

    // Match C build optimization for wrapper too
    build.opt_level(2);
    build.flag("-fno-inline-functions");

    // Filter out format warnings
    build.flag("-Wno-format");
    build.flag("-Wno-format=2");
    
    #[cfg(target_arch = "x86_64")]
    {
        build.flag("-DASMJIT_BUILD_X86=1");
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        build.flag("-DASMJIT_BUILD_ARM=1");
    }
    
    // Compile wrapper
    build.compile("asmjit_wrapper");
    
    // Link against C++ standard library
    // On macOS, use c++ instead of stdc++
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=c++");
    }
    #[cfg(not(target_os = "macos"))]
    {
        println!("cargo:rustc-link-lib=stdc++");
    }
    
    // Tell cargo to rerun this build script if these files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cpp/asmjit_wrapper.cpp");
    println!("cargo:rerun-if-changed=cpp/asmjit_wrapper.h");
}

