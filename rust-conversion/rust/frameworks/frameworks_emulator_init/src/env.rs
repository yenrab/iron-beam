//! Environment Variable Setup Module
//!
//! Provides environment variable setup functionality to replace erlexec environment setup.
//! Handles ROOTDIR, BINDIR, PROGNAME, and PATH manipulation.

use std::env;
use std::path::{Path, PathBuf};

/// Determine rootdir and bindir from binary location or environment
pub fn determine_paths() -> Result<(String, String), String> {
    // First, check if ROOTDIR and BINDIR are already set
    if let (Ok(rootdir), Ok(bindir)) = (env::var("ROOTDIR"), env::var("BINDIR")) {
        return Ok((rootdir, bindir));
    }

    // Try to determine from binary location
    if let Ok(exe_path) = env::current_exe() {
        if let Some(bindir_path) = exe_path.parent() {
            let bindir = bindir_path.to_string_lossy().to_string();
            
            // Try to determine rootdir from bindir (typically erts-VSN/bin)
            if let Some(rootdir_path) = bindir_path.parent() {
                if let Some(rootdir_parent) = rootdir_path.parent() {
                    let rootdir = rootdir_parent.to_string_lossy().to_string();
                    return Ok((rootdir, bindir));
                }
            }
        }
    }

    // Fallback to environment or defaults
    let rootdir = env::var("ROOTDIR")
        .unwrap_or_else(|_| "/usr/local/otp".to_string());
    let bindir = env::var("BINDIR")
        .unwrap_or_else(|_| format!("{}/erts-*/bin", rootdir));

    Ok((rootdir, bindir))
}

/// Set environment variables (ROOTDIR, BINDIR, PROGNAME)
pub fn set_env_vars(rootdir: &str, bindir: &str, progname: &str) {
    env::set_var("ROOTDIR", rootdir);
    env::set_var("BINDIR", bindir);
    env::set_var("PROGNAME", progname);
}

/// Manipulate PATH: add bindir to front, remove duplicates
pub fn manipulate_path(bindir: &str, rootdir: &str) {
    let current_path = env::var("PATH").unwrap_or_default();
    
    // Build new PATH: bindir + rootdir/bin + existing PATH (with duplicates removed)
    let mut path_components = Vec::new();
    
    // Add bindir first
    path_components.push(bindir.to_string());
    
    // Add rootdir/bin
    let rootdir_bin = format!("{}/bin", rootdir);
    path_components.push(rootdir_bin);
    
    // Add existing PATH components, removing duplicates of bindir
    let path_sep = if cfg!(windows) { ";" } else { ":" };
    for component in current_path.split(path_sep) {
        if !component.is_empty() && component != bindir {
            path_components.push(component.to_string());
        }
    }
    
    let new_path = path_components.join(path_sep);
    env::set_var("PATH", new_path);
}

/// Resolve boot script path
pub fn resolve_boot_path(boot: &str, rootdir: &str) -> String {
    if Path::new(boot).is_absolute() {
        boot.to_string()
    } else {
        // Resolve relative to rootdir
        PathBuf::from(rootdir)
            .join("releases")
            .join(boot)
            .to_string_lossy()
            .to_string()
    }
}

/// Resolve config file path
pub fn resolve_config_path(config: &str, rootdir: &str) -> String {
    if Path::new(config).is_absolute() {
        config.to_string()
    } else {
        // Resolve relative to rootdir or current directory
        if let Ok(abs_path) = env::current_dir() {
            let resolved = abs_path.join(config);
            if resolved.exists() {
                return resolved.to_string_lossy().to_string();
            }
        }
        
        // Fallback to rootdir/releases
        PathBuf::from(rootdir)
            .join("releases")
            .join(config)
            .to_string_lossy()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_paths() {
        // This test may fail if environment is not set up correctly
        // It's mainly to ensure the function doesn't panic
        let _result = determine_paths();
    }

    #[test]
    fn test_resolve_boot_path() {
        let boot = resolve_boot_path("start", "/usr/local/otp");
        assert!(boot.contains("start"));
    }

    #[test]
    fn test_resolve_config_path() {
        let config = resolve_config_path("sys.config", "/usr/local/otp");
        assert!(config.contains("sys.config"));
    }
}

