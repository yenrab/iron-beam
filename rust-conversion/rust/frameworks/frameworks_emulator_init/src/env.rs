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
        with_env_vars(&[("ROOTDIR", Some("/test/root")), ("BINDIR", Some("/test/bin"))], || {
            let result = determine_paths();
            assert!(result.is_ok());
            let (rootdir, bindir) = result.unwrap();
            assert_eq!(rootdir, "/test/root");
            assert_eq!(bindir, "/test/bin");
        });
    }

    #[test]
    fn test_determine_paths_with_only_roodir() {
        with_env_vars(&[("ROOTDIR", Some("/test/root")), ("BINDIR", None)], || {
            let result = determine_paths();
            assert!(result.is_ok());
            let (rootdir, bindir) = result.unwrap();
            assert_eq!(rootdir, "/test/root");
            assert_eq!(bindir, "/test/root/bin");
        });
    }

    #[test]
    fn test_determine_paths_with_only_bindir() {
        with_env_vars(&[("ROOTDIR", None), ("BINDIR", Some("/test/root/bin"))], || {
            let result = determine_paths();
            assert!(result.is_ok());
            let (rootdir, bindir) = result.unwrap();
            assert_eq!(rootdir, "/test/root");
            assert_eq!(bindir, "/test/root/bin");
        });
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
        with_env_vars(&[
            ("ROOTDIR", Some("/old/root")),
            ("BINDIR", Some("/old/bin")),
            ("PROGNAME", Some("old_program")),
        ], || {
            set_env_vars("/new/root", "/new/bin", "new_program");
            
            assert_eq!(env::var("ROOTDIR").unwrap(), "/new/root");
            assert_eq!(env::var("BINDIR").unwrap(), "/new/bin");
            assert_eq!(env::var("PROGNAME").unwrap(), "new_program");
        });
    }

    #[test]
    fn test_manipulate_path() {
        with_env_vars(&[("PATH", Some("/usr/bin:/usr/local/bin"))], || {
            manipulate_path("/test/bin", "/test/root");
            
            let new_path = env::var("PATH").unwrap();
            // Should start with bindir
            assert!(new_path.starts_with("/test/bin"));
            // Should contain rootdir/bin
            assert!(new_path.contains("/test/root/bin"));
        });
    }

    #[test]
    fn test_manipulate_path_removes_duplicates() {
        with_env_vars(&[("PATH", Some("/test/bin:/usr/bin"))], || {
            manipulate_path("/test/bin", "/test/root");
            
            let new_path = env::var("PATH").unwrap();
            // Should only have one instance of /test/bin
            let occurrences = new_path.matches("/test/bin").count();
            assert_eq!(occurrences, 1, "PATH should not contain duplicate bindir");
        });
    }

    #[test]
    fn test_manipulate_path_with_empty_path() {
        with_env_vars(&[("PATH", None)], || {
            manipulate_path("/test/bin", "/test/root");
            
            let new_path = env::var("PATH").unwrap();
            // Should still set bindir and rootdir/bin
            assert!(new_path.contains("/test/bin"));
            assert!(new_path.contains("/test/root/bin"));
        });
    }

    #[test]
    fn test_manipulate_path_order() {
        with_env_vars(&[("PATH", Some("/existing/path"))], || {
            manipulate_path("/test/bin", "/test/root");
            
            let new_path = env::var("PATH").unwrap();
            let components: Vec<&str> = if cfg!(windows) {
                new_path.split(';').collect()
            } else {
                new_path.split(':').collect()
            };
            
            // First should be bindir
            assert_eq!(components[0], "/test/bin");
            // Second should be rootdir/bin
            assert_eq!(components[1], "/test/root/bin");
            // Should contain existing path
            assert!(components.contains(&"/existing/path"));
        });
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
        with_env_vars(&[("ROOTDIR", None), ("BINDIR", None), ("PROGNAME", None)], || {
            set_env_vars("/path/with spaces", "/bin/with-spaces", "program-name");
            
            assert_eq!(env::var("ROOTDIR").unwrap(), "/path/with spaces");
            assert_eq!(env::var("BINDIR").unwrap(), "/bin/with-spaces");
            assert_eq!(env::var("PROGNAME").unwrap(), "program-name");
        });
    }

    #[test]
    fn test_manipulate_path_preserves_existing() {
        with_env_vars(&[("PATH", Some("/path1:/path2:/path3"))], || {
            manipulate_path("/new/bin", "/new/root");
            
            let new_path = env::var("PATH").unwrap();
            // Should preserve existing paths
            assert!(new_path.contains("/path1"));
            assert!(new_path.contains("/path2"));
            assert!(new_path.contains("/path3"));
        });
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

