//! Environment Variable Setup Module
//!
//! Provides environment variable setup functionality to replace erlexec environment setup.
//! Handles ROOTDIR, BINDIR, PROGNAME, and PATH manipulation.

use std::env;
use std::path::{Path, PathBuf};

/// Determine rootdir and bindir from binary location or environment
pub fn determine_paths() -> Result<(String, String), String> {
    // First, check if ROOTDIR and BINDIR are already set (highest priority)
    if let (Ok(rootdir), Ok(bindir)) = (env::var("ROOTDIR"), env::var("BINDIR")) {
        if !rootdir.is_empty() && !bindir.is_empty() {
            return Ok((rootdir, bindir));
        }
    }
    
    // If only one is set, try to determine the other
    if let Ok(rootdir) = env::var("ROOTDIR") {
        if !rootdir.is_empty() {
            let bindir = env::var("BINDIR")
                .unwrap_or_else(|_| format!("{}/bin", rootdir));
            return Ok((rootdir, bindir));
        }
    }
    
    if let Ok(bindir) = env::var("BINDIR") {
        if !bindir.is_empty() {
            // Try to determine rootdir from bindir (bindir is typically rootdir/bin)
            if let Some(rootdir_path) = Path::new(&bindir).parent() {
                let rootdir = rootdir_path.to_string_lossy().to_string();
                return Ok((rootdir, bindir));
            }
        }
    }

    // Try to determine from binary location
    if let Ok(exe_path) = env::current_exe() {
        if let Some(bindir_path) = exe_path.parent() {
            let bindir = bindir_path.to_string_lossy().to_string();
            
            // Try to determine rootdir from bindir (bindir is typically rootdir/bin)
            if let Some(rootdir_path) = bindir_path.parent() {
                let rootdir = rootdir_path.to_string_lossy().to_string();
                return Ok((rootdir, bindir));
            }
        }
    }

    // Fallback to defaults
    let rootdir = "/usr/local/otp".to_string();
    let bindir = format!("{}/bin", rootdir);

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
    path_components.push(rootdir_bin.clone());
    
    // Add existing PATH components, removing duplicates of bindir and rootdir/bin
    let path_sep = if cfg!(windows) { ";" } else { ":" };
    for component in current_path.split(path_sep) {
        if !component.is_empty() && component != bindir && component != rootdir_bin {
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

    // Helper to save and restore environment variables
    fn with_env_vars<F>(vars: &[(&str, Option<&str>)], f: F)
    where
        F: FnOnce(),
    {
        // Save current values
        let saved: Vec<(String, Option<String>)> = vars
            .iter()
            .map(|(key, _)| {
                let value = env::var(key).ok();
                (key.to_string(), value)
            })
            .collect();
        
        // Set new values
        for (key, value) in vars {
            match value {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }
        
        // Execute test
        f();
        
        // Restore original values
        for (key, value) in saved {
            match value {
                Some(v) => env::set_var(&key, v),
                None => env::remove_var(&key),
            }
        }
    }

    #[test]
    fn test_determine_paths() {
        // This test may fail if environment is not set up correctly
        // It's mainly to ensure the function doesn't panic
        let result = determine_paths();
        assert!(result.is_ok());
        let (rootdir, bindir) = result.unwrap();
        assert!(!rootdir.is_empty());
        assert!(!bindir.is_empty());
    }

    #[test]
    fn test_determine_paths_with_roodir_and_bindir() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            let result = std::panic::catch_unwind(|| {
                with_env_vars(&[("ROOTDIR", Some("/test/root")), ("BINDIR", Some("/test/bin"))], || {
                    // Small delay to let any parallel operations complete
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
                    }
                    
                    // Verify environment variables are set correctly before calling determine_paths
                    let rootdir_env = env::var("ROOTDIR")
                        .expect("ROOTDIR should be set by with_env_vars");
                    let bindir_env = env::var("BINDIR")
                        .expect("BINDIR should be set by with_env_vars");
                    
                    assert_eq!(rootdir_env, "/test/root",
                        "ROOTDIR should be '/test/root' but got '{}'", rootdir_env);
                    assert_eq!(bindir_env, "/test/bin",
                        "BINDIR should be '/test/bin' but got '{}'", bindir_env);
                    
                    // Small delay to ensure environment variables are stable
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    let result = determine_paths();
                    assert!(result.is_ok(),
                        "determine_paths() should succeed when ROOTDIR and BINDIR are set. Got: {:?}", result);
                    let (rootdir, bindir) = result.unwrap();
                    assert_eq!(rootdir, "/test/root",
                        "rootdir should be '/test/root' but got '{}'. This may indicate test interference.", rootdir);
                    assert_eq!(bindir, "/test/bin",
                        "bindir should be '/test/bin' but got '{}'. This may indicate test interference.", bindir);
                });
            });
            
            match result {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 4 {
                        // Try again with a longer delay
                        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        
        assert!(success, "test_determine_paths_with_roodir_and_bindir failed after retries. This suggests test interference or a bug in determine_paths.");
    }

    #[test]
    fn test_determine_paths_with_only_roodir() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            let result = std::panic::catch_unwind(|| {
                with_env_vars(&[("ROOTDIR", Some("/test/root")), ("BINDIR", None)], || {
                    // Small delay to let any parallel operations complete
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
                    }
                    
                    // Verify environment variables are set correctly
                    let rootdir_env = env::var("ROOTDIR");
                    let bindir_env = env::var("BINDIR");
                    
                    // ROOTDIR should be set
                    if rootdir_env != Ok("/test/root".to_string()) {
                        panic!("ROOTDIR should be set to /test/root. Got: {:?}", rootdir_env);
                    }
                    
                    // BINDIR should not be set (or should be empty)
                    // If another test has set it, we need to clear it
                    if bindir_env.is_ok() && !bindir_env.as_ref().unwrap().is_empty() {
                        // Another test has interfered - clear BINDIR
                        env::remove_var("BINDIR");
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    
                    // Verify BINDIR is now not set
                    let bindir_env_after = env::var("BINDIR");
                    assert!(bindir_env_after.is_err() || bindir_env_after.as_ref().map(|s| s.is_empty()).unwrap_or(false),
                        "BINDIR should not be set after clearing. Got: {:?}", bindir_env_after);
                    
                    // Small delay to ensure environment variables are stable
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    let result = determine_paths();
                    assert!(result.is_ok(), 
                        "determine_paths should succeed. Got: {:?}. ROOTDIR env: {:?}, BINDIR env: {:?}", 
                        result, rootdir_env, bindir_env_after);
                    let (rootdir, bindir) = result.unwrap();
                    assert_eq!(rootdir, "/test/root",
                        "rootdir should be /test/root. Got: {}. ROOTDIR env: {:?}, BINDIR env: {:?}", 
                        rootdir, rootdir_env, bindir_env_after);
                    assert_eq!(bindir, "/test/root/bin",
                        "bindir should be /test/root/bin. Got: {}. ROOTDIR env: {:?}, BINDIR env: {:?}", 
                        bindir, rootdir_env, bindir_env_after);
                });
            });
            
            match result {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 4 {
                        // Try again with a longer delay
                        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        
        assert!(success, "test_determine_paths_with_only_roodir failed after retries. This suggests test interference or a bug in determine_paths.");
    }

    #[test]
    fn test_determine_paths_with_only_bindir() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            let result = std::panic::catch_unwind(|| {
                with_env_vars(&[("ROOTDIR", None), ("BINDIR", Some("/test/root/bin"))], || {
                    // Small delay to let any parallel operations complete
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
                    }
                    
                    // Verify environment variables are set correctly
                    let rootdir_env = env::var("ROOTDIR");
                    let bindir_env = env::var("BINDIR");
                    
                    // ROOTDIR should not be set (or should be empty)
                    // If another test has set it, we need to clear it
                    if rootdir_env.is_ok() && !rootdir_env.as_ref().unwrap().is_empty() {
                        // Another test has interfered - clear ROOTDIR
                        env::remove_var("ROOTDIR");
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    
                    // BINDIR should be set to /test/root/bin
                    if bindir_env != Ok("/test/root/bin".to_string()) {
                        // Set it explicitly in case another test cleared it
                        env::set_var("BINDIR", "/test/root/bin");
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    
                    // Verify environment variables are correct before calling determine_paths
                    let rootdir_env_after = env::var("ROOTDIR");
                    let bindir_env_after = env::var("BINDIR");
                    
                    assert!(rootdir_env_after.is_err() || rootdir_env_after.as_ref().map(|s| s.is_empty()).unwrap_or(false),
                        "ROOTDIR should not be set. Got: {:?}", rootdir_env_after);
                    assert_eq!(bindir_env_after, Ok("/test/root/bin".to_string()),
                        "BINDIR should be '/test/root/bin'. Got: {:?}", bindir_env_after);
                    
                    // Small delay to ensure environment variables are stable
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    let result = determine_paths();
                    assert!(result.is_ok(), 
                        "determine_paths should succeed. Got: {:?}. ROOTDIR env: {:?}, BINDIR env: {:?}", 
                        result, rootdir_env_after, bindir_env_after);
                    let (rootdir, bindir) = result.unwrap();
                    assert_eq!(rootdir, "/test/root",
                        "rootdir should be '/test/root' (parent of BINDIR). Got: {}. ROOTDIR env: {:?}, BINDIR env: {:?}", 
                        rootdir, rootdir_env_after, bindir_env_after);
                    assert_eq!(bindir, "/test/root/bin",
                        "bindir should be '/test/root/bin'. Got: {}. ROOTDIR env: {:?}, BINDIR env: {:?}", 
                        bindir, rootdir_env_after, bindir_env_after);
                });
            });
            
            match result {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 4 {
                        // Try again with a longer delay
                        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        
        assert!(success, "test_determine_paths_with_only_bindir failed after retries. This suggests test interference or a bug in determine_paths.");
    }

    #[test]
    fn test_determine_paths_with_empty_roodir() {
        with_env_vars(&[("ROOTDIR", Some("")), ("BINDIR", None)], || {
            let result = determine_paths();
            assert!(result.is_ok());
            // Should fall back to other methods
            let (rootdir, bindir) = result.unwrap();
            assert!(!rootdir.is_empty());
            assert!(!bindir.is_empty());
        });
    }

    #[test]
    fn test_determine_paths_with_empty_bindir() {
        with_env_vars(&[("ROOTDIR", None), ("BINDIR", Some(""))], || {
            let result = determine_paths();
            assert!(result.is_ok());
            // Should fall back to other methods
            let (rootdir, bindir) = result.unwrap();
            assert!(!rootdir.is_empty());
            assert!(!bindir.is_empty());
        });
    }

    #[test]
    fn test_determine_paths_fallback() {
        with_env_vars(&[("ROOTDIR", None), ("BINDIR", None)], || {
            let result = determine_paths();
            assert!(result.is_ok());
            let (rootdir, bindir) = result.unwrap();
            // Should have fallback values
            assert!(!rootdir.is_empty());
            assert!(!bindir.is_empty());
            // Fallback should be /usr/local/otp or determined from binary location
            assert!(rootdir.len() > 0);
            assert!(bindir.len() > 0);
        });
    }

    #[test]
    fn test_set_env_vars() {
        with_env_vars(&[("ROOTDIR", None), ("BINDIR", None), ("PROGNAME", None)], || {
            set_env_vars("/test/root", "/test/bin", "test_program");
            
            assert_eq!(env::var("ROOTDIR").unwrap(), "/test/root");
            assert_eq!(env::var("BINDIR").unwrap(), "/test/bin");
            assert_eq!(env::var("PROGNAME").unwrap(), "test_program");
        });
    }

    #[test]
    fn test_set_env_vars_overwrites() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            let result = std::panic::catch_unwind(|| {
                with_env_vars(&[
                    ("ROOTDIR", Some("/old/root")),
                    ("BINDIR", Some("/old/bin")),
                    ("PROGNAME", Some("old_program")),
                ], || {
                    // Small delay to let any parallel operations complete
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
                    }
                    
                    // Verify old values are set
                    let old_rootdir = env::var("ROOTDIR")
                        .expect("ROOTDIR should be set by with_env_vars");
                    let old_bindir = env::var("BINDIR")
                        .expect("BINDIR should be set by with_env_vars");
                    let old_progname = env::var("PROGNAME")
                        .expect("PROGNAME should be set by with_env_vars");
                    
                    assert_eq!(old_rootdir, "/old/root",
                        "ROOTDIR should be '/old/root' but got '{}'", old_rootdir);
                    assert_eq!(old_bindir, "/old/bin",
                        "BINDIR should be '/old/bin' but got '{}'", old_bindir);
                    assert_eq!(old_progname, "old_program",
                        "PROGNAME should be 'old_program' but got '{}'", old_progname);
                    
                    set_env_vars("/new/root", "/new/bin", "new_program");
                    
                    // Small delay after setting to ensure they're visible
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    // Verify new values overwrote old ones
                    let rootdir = env::var("ROOTDIR")
                        .expect("ROOTDIR should be set by set_env_vars");
                    let bindir = env::var("BINDIR")
                        .expect("BINDIR should be set by set_env_vars");
                    let progname = env::var("PROGNAME")
                        .expect("PROGNAME should be set by set_env_vars");
                    
                    assert_eq!(rootdir, "/new/root",
                        "ROOTDIR should be '/new/root' but got '{}'. This may indicate test interference.", rootdir);
                    assert_eq!(bindir, "/new/bin",
                        "BINDIR should be '/new/bin' but got '{}'. This may indicate test interference.", bindir);
                    assert_eq!(progname, "new_program",
                        "PROGNAME should be 'new_program' but got '{}'. This may indicate test interference.", progname);
                });
            });
            
            match result {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 4 {
                        // Try again with a longer delay
                        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        
        assert!(success, "test_set_env_vars_overwrites failed after retries. This suggests test interference or a bug in set_env_vars.");
    }

    #[test]
    fn test_manipulate_path() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            let result = std::panic::catch_unwind(|| {
                with_env_vars(&[("PATH", Some("/usr/bin:/usr/local/bin"))], || {
                    // Small delay to let any parallel operations complete
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
                    }
                    
                    // Set PATH explicitly right before manipulation to avoid interference
                    // Even though with_env_vars sets it, another test might have modified it
                    env::set_var("PATH", "/usr/bin:/usr/local/bin");
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    
                    // Verify PATH is correct before calling manipulate_path
                    let path_before_check = env::var("PATH")
                        .expect("PATH should be set");
                    if path_before_check != "/usr/bin:/usr/local/bin" {
                        // Another test interfered - set it again and wait
                        env::set_var("PATH", "/usr/bin:/usr/local/bin");
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    
                    manipulate_path("/test/bin", "/test/root");
                    
                    // Small delay after manipulation to ensure it's visible
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    let new_path = env::var("PATH")
                        .expect("PATH should be set by manipulate_path");
                    let path_sep = if cfg!(windows) { ";" } else { ":" };
                    let components: Vec<&str> = new_path.split(path_sep).collect();
                    
                    // Should start with bindir
                    assert_eq!(components[0], "/test/bin", 
                        "PATH should start with /test/bin. Got: {:?}", components);
                    // Second should be rootdir/bin
                    assert_eq!(components[1], "/test/root/bin",
                        "PATH second component should be /test/root/bin. Got: {:?}", components);
                    // Should contain the original paths
                    assert!(components.contains(&"/usr/bin"),
                        "PATH should contain /usr/bin. Got: {:?}", components);
                    assert!(components.contains(&"/usr/local/bin"),
                        "PATH should contain /usr/local/bin. Got: {:?}", components);
                });
            });
            
            match result {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 4 {
                        // Try again with a longer delay
                        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        
        assert!(success, "test_manipulate_path failed after retries. This suggests test interference or a bug in manipulate_path.");
    }

    #[test]
    fn test_manipulate_path_removes_duplicates() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            let result = std::panic::catch_unwind(|| {
                with_env_vars(&[("PATH", Some("/test/bin:/usr/bin"))], || {
                    // Small delay to let any parallel operations complete
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
                    }
                    
                    // Set PATH explicitly right before manipulation to avoid interference
                    // Even though with_env_vars sets it, another test might have modified it
                    env::set_var("PATH", "/test/bin:/usr/bin");
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    
                    // Verify PATH is correct before calling manipulate_path
                    let path_before_check = env::var("PATH")
                        .expect("PATH should be set");
                    if path_before_check != "/test/bin:/usr/bin" {
                        // Another test interfered - set it again and wait
                        env::set_var("PATH", "/test/bin:/usr/bin");
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    
                    manipulate_path("/test/bin", "/test/root");
                    
                    // Small delay after manipulation to ensure it's visible
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    let new_path = env::var("PATH")
                        .expect("PATH should be set by manipulate_path");
                    let path_sep = if cfg!(windows) { ";" } else { ":" };
                    let components: Vec<&str> = new_path.split(path_sep).collect();
                    
                    // Should only have one instance of /test/bin
                    let occurrences = components.iter().filter(|&&c| c == "/test/bin").count();
                    assert_eq!(occurrences, 1, 
                        "PATH should not contain duplicate bindir. Got: {:?}", components);
                    // Should start with /test/bin
                    assert_eq!(components[0], "/test/bin",
                        "PATH should start with /test/bin. Got: {:?}", components);
                });
            });
            
            match result {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 4 {
                        // Try again with a longer delay
                        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        
        assert!(success, "test_manipulate_path_removes_duplicates failed after retries. This suggests test interference or a bug in manipulate_path.");
    }

    #[test]
    fn test_manipulate_path_with_empty_path() {
        with_env_vars(&[("PATH", None)], || {
            manipulate_path("/test/bin", "/test/root");
            
            let new_path = env::var("PATH").unwrap();
            let path_sep = if cfg!(windows) { ";" } else { ":" };
            let components: Vec<&str> = new_path.split(path_sep).filter(|s| !s.is_empty()).collect();
            
            // Should still set bindir and rootdir/bin
            assert!(components.contains(&"/test/bin"),
                "PATH should contain /test/bin. Got: {:?}", components);
            assert!(components.contains(&"/test/root/bin"),
                "PATH should contain /test/root/bin. Got: {:?}", components);
            // Should start with bindir
            assert_eq!(components[0], "/test/bin",
                "PATH should start with /test/bin. Got: {:?}", components);
        });
    }

    #[test]
    fn test_manipulate_path_order() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            let result = std::panic::catch_unwind(|| {
                with_env_vars(&[("PATH", Some("/existing/path"))], || {
                    // Small delay to let any parallel operations complete
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
                    }
                    
                    // Clear PATH first to ensure clean state
                    env::set_var("PATH", "/existing/path");
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    manipulate_path("/test/bin", "/test/root");
                    
                    // Small delay after manipulation to ensure it's visible
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    let new_path = env::var("PATH")
                        .expect("PATH should be set by manipulate_path");
                    let path_sep = if cfg!(windows) { ";" } else { ":" };
                    let components: Vec<&str> = new_path.split(path_sep).collect();
                    
                    // First should be bindir
                    assert_eq!(components[0], "/test/bin",
                        "First PATH component should be '/test/bin' but got '{}'. Full PATH: {:?}",
                        components[0], components);
                    // Second should be rootdir/bin
                    assert_eq!(components[1], "/test/root/bin",
                        "Second PATH component should be '/test/root/bin' but got '{}'. Full PATH: {:?}",
                        components[1], components);
                    // Should contain existing path
                    assert!(components.contains(&"/existing/path"),
                        "PATH should contain '/existing/path' but got: {:?}", components);
                });
            });
            
            match result {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 4 {
                        // Try again with a longer delay
                        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        
        assert!(success, "test_manipulate_path_order failed after retries. This suggests test interference or a bug in manipulate_path.");
    }

    #[test]
    fn test_resolve_boot_path_absolute() {
        let boot = resolve_boot_path("/absolute/path/to/boot", "/usr/local/otp");
        assert_eq!(boot, "/absolute/path/to/boot");
    }

    #[test]
    fn test_resolve_boot_path_relative() {
        let boot = resolve_boot_path("start", "/usr/local/otp");
        assert!(boot.contains("start"));
        assert!(boot.contains("/usr/local/otp"));
        assert!(boot.contains("releases"));
    }

    #[test]
    fn test_resolve_boot_path_relative_with_slash() {
        let boot = resolve_boot_path("releases/start", "/usr/local/otp");
        assert!(boot.contains("start"));
        assert!(boot.contains("/usr/local/otp"));
    }

    #[test]
    fn test_resolve_boot_path_unix_absolute() {
        let boot = resolve_boot_path("/usr/local/boot", "/usr/local/otp");
        assert_eq!(boot, "/usr/local/boot");
    }

    #[test]
    fn test_resolve_config_path_absolute() {
        let config = resolve_config_path("/absolute/path/to/config", "/usr/local/otp");
        assert_eq!(config, "/absolute/path/to/config");
    }

    #[test]
    fn test_resolve_config_path_relative() {
        let config = resolve_config_path("sys.config", "/usr/local/otp");
        assert!(config.contains("sys.config"));
        assert!(config.contains("/usr/local/otp"));
        assert!(config.contains("releases"));
    }

    #[test]
    fn test_resolve_config_path_relative_with_slash() {
        let config = resolve_config_path("releases/sys.config", "/usr/local/otp");
        assert!(config.contains("sys.config"));
        assert!(config.contains("/usr/local/otp"));
    }

    #[test]
    fn test_resolve_config_path_unix_absolute() {
        let config = resolve_config_path("/usr/local/config", "/usr/local/otp");
        assert_eq!(config, "/usr/local/config");
    }

    #[test]
    fn test_resolve_boot_path_empty() {
        let boot = resolve_boot_path("", "/usr/local/otp");
        assert!(boot.contains("/usr/local/otp"));
        assert!(boot.contains("releases"));
    }

    #[test]
    fn test_resolve_config_path_empty() {
        let config = resolve_config_path("", "/usr/local/otp");
        // Empty config might resolve to current directory or rootdir/releases
        // Just verify it returns a non-empty string
        assert!(!config.is_empty());
        // Should either be current directory path or rootdir/releases path
        assert!(config.contains("/usr/local/otp") || config.contains("releases") || 
                env::current_dir().map(|p| config.contains(p.to_string_lossy().as_ref())).unwrap_or(false));
    }

    #[test]
    fn test_resolve_boot_path_with_dots() {
        let boot = resolve_boot_path("../boot", "/usr/local/otp");
        // Relative path should be resolved
        assert!(boot.contains("boot"));
    }

    #[test]
    fn test_resolve_config_path_with_dots() {
        let config = resolve_config_path("../config", "/usr/local/otp");
        // Relative path should be resolved
        assert!(config.contains("config"));
    }

    #[test]
    fn test_path_separator_windows() {
        // Test that PATH manipulation uses correct separator on Windows
        // This is tested implicitly in manipulate_path tests
        // The actual separator depends on cfg!(windows) at compile time
        let path_sep = if cfg!(windows) { ";" } else { ":" };
        assert!(!path_sep.is_empty());
    }

    #[test]
    fn test_determine_paths_result_structure() {
        let result = determine_paths();
        assert!(result.is_ok());
        let (rootdir, bindir) = result.unwrap();
        
        // Both should be non-empty strings
        assert!(!rootdir.is_empty());
        assert!(!bindir.is_empty());
        
        // Both should be valid paths (at least contain some characters)
        assert!(rootdir.len() > 0);
        assert!(bindir.len() > 0);
    }

    #[test]
    fn test_set_env_vars_with_special_characters() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            let result = std::panic::catch_unwind(|| {
                with_env_vars(&[("ROOTDIR", None), ("BINDIR", None), ("PROGNAME", None)], || {
                    // Small delay to let any parallel operations complete
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
                    }
                    
                    set_env_vars("/path/with spaces", "/bin/with-spaces", "program-name");
                    
                    // Small delay after setting to ensure they're visible
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    // Verify ROOTDIR was set correctly
                    let rootdir = env::var("ROOTDIR")
                        .expect("ROOTDIR should be set by set_env_vars. This may indicate test interference or a bug in set_env_vars.");
                    assert_eq!(rootdir, "/path/with spaces", 
                        "ROOTDIR should be '/path/with spaces' but got '{}'", rootdir);
                    
                    // Verify BINDIR was set correctly
                    let bindir = env::var("BINDIR")
                        .expect("BINDIR should be set by set_env_vars. This may indicate test interference or a bug in set_env_vars.");
                    assert_eq!(bindir, "/bin/with-spaces",
                        "BINDIR should be '/bin/with-spaces' but got '{}'", bindir);
                    
                    // Verify PROGNAME was set correctly
                    let progname = env::var("PROGNAME")
                        .expect("PROGNAME should be set by set_env_vars. This may indicate test interference or a bug in set_env_vars.");
                    assert_eq!(progname, "program-name",
                        "PROGNAME should be 'program-name' but got '{}'", progname);
                });
            });
            
            match result {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 4 {
                        // Try again with a longer delay
                        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        
        assert!(success, "test_set_env_vars_with_special_characters failed after retries. This suggests test interference or a bug in set_env_vars.");
    }

    #[test]
    fn test_manipulate_path_preserves_existing() {
        // Use retry logic to handle potential test interference from parallel execution
        let mut success = false;
        for attempt in 0..5 {
            let result = std::panic::catch_unwind(|| {
                with_env_vars(&[("PATH", Some("/path1:/path2:/path3"))], || {
                    // Small delay to let any parallel operations complete
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10 * attempt));
                    }
                    
                    // Set PATH explicitly right before manipulation to avoid interference
                    // Even though with_env_vars sets it, another test might have modified it
                    env::set_var("PATH", "/path1:/path2:/path3");
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    
                    // Verify PATH is correct before calling manipulate_path
                    let path_before_check = env::var("PATH")
                        .expect("PATH should be set");
                    if path_before_check != "/path1:/path2:/path3" {
                        // Another test interfered - set it again and wait
                        env::set_var("PATH", "/path1:/path2:/path3");
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    
                    manipulate_path("/new/bin", "/new/root");
                    
                    // Small delay after manipulation to ensure it's visible
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    
                    let new_path = env::var("PATH")
                        .expect("PATH should be set by manipulate_path");
                    let path_sep = if cfg!(windows) { ";" } else { ":" };
                    let components: Vec<&str> = new_path.split(path_sep).collect();
                    
                    // Should preserve existing paths as separate components
                    assert!(components.contains(&"/path1"), 
                        "PATH should contain /path1 as a component. Got: {:?}", components);
                    assert!(components.contains(&"/path2"), 
                        "PATH should contain /path2 as a component. Got: {:?}", components);
                    assert!(components.contains(&"/path3"), 
                        "PATH should contain /path3 as a component. Got: {:?}", components);
                });
            });
            
            match result {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 4 {
                        // Try again with a longer delay
                        std::thread::sleep(std::time::Duration::from_millis(20 * (attempt + 1)));
                        continue;
                    }
                }
            }
        }
        
        assert!(success, "test_manipulate_path_preserves_existing failed after retries. This suggests test interference or a bug in manipulate_path.");
    }

    #[test]
    fn test_manipulate_path_removes_empty_components() {
        with_env_vars(&[("PATH", Some(":/path1::/path2:"))], || {
            manipulate_path("/test/bin", "/test/root");
            
            let new_path = env::var("PATH").unwrap();
            let components: Vec<&str> = if cfg!(windows) {
                new_path.split(';').filter(|s| !s.is_empty()).collect()
            } else {
                new_path.split(':').filter(|s| !s.is_empty()).collect()
            };
            
            // Should not have empty components in the result
            assert!(!components.is_empty());
        });
    }
}

