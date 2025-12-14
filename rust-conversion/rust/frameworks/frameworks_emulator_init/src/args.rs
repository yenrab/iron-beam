//! Command-Line Argument Parsing Module
//!
//! Provides argument parsing functionality to replace erlexec argument processing.
//! Uses clap for type-safe argument parsing.

use clap::Parser;

/// Erlang emulator command-line arguments
#[derive(Parser, Debug)]
#[command(name = "beam")]
#[command(about = "Erlang/OTP Emulator")]
pub struct EmulatorArgs {
    /// Distribution: short name (e.g., "node@host")
    #[arg(long)]
    pub sname: Option<String>,

    /// Distribution: long name (e.g., "node@host.domain")
    #[arg(long)]
    pub name: Option<String>,

    /// Distribution protocol (e.g., "inet_tcp", "inet_tls")
    #[arg(long)]
    pub proto_dist: Option<String>,

    /// Do not start epmd daemon (requires -proto_dist)
    #[arg(long)]
    pub no_epmd: bool,

    /// Path to epmd program
    #[arg(long)]
    pub epmd: Option<String>,

    /// Boot script path
    #[arg(long)]
    pub boot: Option<String>,

    /// Config file path(s) (can be specified multiple times)
    #[arg(long, num_args = 1..)]
    pub config: Vec<String>,

    /// Arguments file path
    #[arg(long)]
    pub args_file: Option<String>,

    /// SMP mode: auto, enable, or number
    #[arg(long)]
    pub smp: Option<String>,

    /// Enable SMP
    #[arg(long)]
    pub smpenable: bool,

    /// Disable SMP
    #[arg(long)]
    pub smpdisable: bool,

    /// Auto SMP
    #[arg(long)]
    pub smpauto: bool,

    /// Emulator type (e.g., "opt", "debug", "lcnt", "valgrind")
    #[arg(long)]
    pub emu_type: Option<String>,

    /// Emulator flavor (e.g., "smp", "jit", "emu")
    #[arg(long)]
    pub emu_flavor: Option<String>,

    /// Special mode: print arguments and exit
    #[arg(long)]
    pub emu_args_exit: bool,

    /// Special mode: print emulator name and exit
    #[arg(long)]
    pub emu_name_exit: bool,

    /// Special mode: print quoted command line and exit
    #[arg(long)]
    pub emu_qouted_cmd_exit: bool,

    /// Extra flag: all remaining arguments after this
    #[arg(long)]
    pub extra: bool,

    /// Detached mode (Windows-specific)
    #[arg(long)]
    pub detached: bool,

    /// Remaining arguments (everything after -- or -extra)
    #[arg(trailing_var_arg = true)]
    pub remaining: Vec<String>,
}

impl EmulatorArgs {
    /// Check if distribution is enabled
    pub fn is_distributed(&self) -> bool {
        self.sname.is_some() || self.name.is_some()
    }

    /// Check if epmd should be started
    pub fn should_start_epmd(&self) -> bool {
        self.is_distributed() && !self.no_epmd
    }

    /// Validate argument combinations
    pub fn validate(&self) -> Result<(), String> {
        if self.no_epmd && self.proto_dist.is_none() {
            return Err("-no_epmd requires -proto_dist flag".to_string());
        }

        if self.sname.is_some() && self.name.is_some() {
            return Err("Cannot specify both -sname and -name".to_string());
        }

        Ok(())
    }

    /// Build argument vector for erl_start()
    pub fn build_emulator_args(&self, rootdir: &str, bindir: &str) -> Vec<String> {
        let mut args = vec!["beam".to_string()];

        // Add rootdir and bindir
        args.push("-root".to_string());
        args.push(rootdir.to_string());
        args.push("-bindir".to_string());
        args.push(bindir.to_string());
        args.push("-progname".to_string());
        args.push("beam".to_string());

        // Add boot script
        if let Some(ref boot) = self.boot {
            args.push("-boot".to_string());
            args.push(boot.clone());
        }

        // Add config files
        for config in &self.config {
            args.push("-config".to_string());
            args.push(config.clone());
        }

        // Add distribution flags
        if let Some(ref sname) = self.sname {
            args.push("-sname".to_string());
            args.push(sname.clone());
        }

        if let Some(ref name) = self.name {
            args.push("-name".to_string());
            args.push(name.clone());
        }

        if let Some(ref proto_dist) = self.proto_dist {
            args.push("-proto_dist".to_string());
            args.push(proto_dist.clone());
        }

        if self.no_epmd {
            args.push("-no_epmd".to_string());
        }

        // Add SMP flags
        if let Some(ref smp) = self.smp {
            args.push("-smp".to_string());
            args.push(smp.clone());
        } else if self.smpenable {
            args.push("-smp".to_string());
            args.push("enable".to_string());
        } else if self.smpauto {
            args.push("-smp".to_string());
            args.push("auto".to_string());
        }

        // Add remaining arguments
        args.extend(self.remaining.clone());

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_distributed() {
        // Test with sname
        let args = EmulatorArgs::parse_from(&["beam", "--sname", "test@localhost"]);
        assert!(args.is_distributed());

        // Test with name
        let args = EmulatorArgs::parse_from(&["beam", "--name", "test@localhost.domain"]);
        assert!(args.is_distributed());

        // Test without distribution
        let args = EmulatorArgs::parse_from(&["beam"]);
        assert!(!args.is_distributed());
    }

    #[test]
    fn test_should_start_epmd() {
        // Test with sname (should start epmd)
        let args = EmulatorArgs::parse_from(&["beam", "--sname", "test@localhost"]);
        assert!(args.should_start_epmd());

        // Test with name (should start epmd)
        let args = EmulatorArgs::parse_from(&["beam", "--name", "test@localhost.domain"]);
        assert!(args.should_start_epmd());

        // Test with sname and no-epmd (should not start epmd)
        let args = EmulatorArgs::parse_from(&["beam", "--sname", "test@localhost", "--no-epmd"]);
        assert!(!args.should_start_epmd());

        // Test with name and no-epmd (should not start epmd)
        let args = EmulatorArgs::parse_from(&["beam", "--name", "test@localhost.domain", "--no-epmd"]);
        assert!(!args.should_start_epmd());

        // Test without distribution (should not start epmd)
        let args = EmulatorArgs::parse_from(&["beam"]);
        assert!(!args.should_start_epmd());
    }

    #[test]
    fn test_validate() {
        // Test no-epmd without proto-dist (should fail)
        let args = EmulatorArgs::parse_from(&["beam", "--no-epmd"]);
        assert!(args.validate().is_err());
        assert!(args.validate().unwrap_err().contains("-no_epmd requires -proto_dist"));

        // Test no-epmd with proto-dist (should succeed)
        let args = EmulatorArgs::parse_from(&["beam", "--no-epmd", "--proto-dist", "inet_tcp"]);
        assert!(args.validate().is_ok());

        // Test both sname and name (should fail)
        let args = EmulatorArgs::parse_from(&["beam", "--sname", "test1@localhost", "--name", "test2@localhost.domain"]);
        assert!(args.validate().is_err());
        assert!(args.validate().unwrap_err().contains("Cannot specify both -sname and -name"));

        // Test valid combinations
        let args = EmulatorArgs::parse_from(&["beam"]);
        assert!(args.validate().is_ok());

        let args = EmulatorArgs::parse_from(&["beam", "--sname", "test@localhost"]);
        assert!(args.validate().is_ok());

        let args = EmulatorArgs::parse_from(&["beam", "--name", "test@localhost.domain"]);
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_parse_all_fields() {
        // Test parsing all optional fields
        let args = EmulatorArgs::parse_from(&[
            "beam",
            "--sname", "node@host",
            "--name", "node@host.domain",
            "--proto-dist", "inet_tcp",
            "--no-epmd",
            "--epmd", "/path/to/epmd",
            "--boot", "/path/to/boot",
            "--config", "config1.config",
            "--config", "config2.config",
            "--args-file", "/path/to/args",
            "--smp", "4",
            "--smpenable",
            "--smpdisable",
            "--smpauto",
            "--emu-type", "opt",
            "--emu-flavor", "jit",
            "--emu-args-exit",
            "--emu-name-exit",
            "--emu-qouted-cmd-exit",
            "--extra",
            "--detached",
            "remaining1", "remaining2"
        ]);

        // Note: clap will parse the last value for conflicting flags
        // Test that fields are parsed (some may be overridden by later flags)
        assert!(args.sname.is_some() || args.name.is_some());
        assert!(args.proto_dist.is_some());
        assert!(args.no_epmd);
        assert!(args.epmd.is_some());
        assert!(args.boot.is_some());
        assert!(!args.config.is_empty());
        assert!(args.args_file.is_some());
        assert!(args.smp.is_some() || args.smpenable || args.smpauto);
        assert!(args.emu_type.is_some());
        assert!(args.emu_flavor.is_some());
        assert!(args.emu_args_exit);
        assert!(args.emu_name_exit);
        assert!(args.emu_qouted_cmd_exit);
        assert!(args.extra);
        assert!(args.detached);
        assert!(!args.remaining.is_empty());
    }

    #[test]
    fn test_parse_sname() {
        let args = EmulatorArgs::parse_from(&["beam", "--sname", "test@localhost"]);
        assert_eq!(args.sname, Some("test@localhost".to_string()));
        assert!(args.name.is_none());
    }

    #[test]
    fn test_parse_name() {
        let args = EmulatorArgs::parse_from(&["beam", "--name", "test@localhost.domain"]);
        assert_eq!(args.name, Some("test@localhost.domain".to_string()));
        assert!(args.sname.is_none());
    }

    #[test]
    fn test_parse_proto_dist() {
        let args = EmulatorArgs::parse_from(&["beam", "--proto-dist", "inet_tls"]);
        assert_eq!(args.proto_dist, Some("inet_tls".to_string()));
    }

    #[test]
    fn test_parse_no_epmd() {
        let args = EmulatorArgs::parse_from(&["beam", "--no-epmd"]);
        assert!(args.no_epmd);
    }

    #[test]
    fn test_parse_epmd() {
        let args = EmulatorArgs::parse_from(&["beam", "--epmd", "/usr/bin/epmd"]);
        assert_eq!(args.epmd, Some("/usr/bin/epmd".to_string()));
    }

    #[test]
    fn test_parse_boot() {
        let args = EmulatorArgs::parse_from(&["beam", "--boot", "/path/to/boot.script"]);
        assert_eq!(args.boot, Some("/path/to/boot.script".to_string()));
    }

    #[test]
    fn test_parse_config_multiple() {
        let args = EmulatorArgs::parse_from(&[
            "beam",
            "--config", "config1.config",
            "--config", "config2.config",
            "--config", "config3.config"
        ]);
        assert_eq!(args.config.len(), 3);
        assert_eq!(args.config[0], "config1.config");
        assert_eq!(args.config[1], "config2.config");
        assert_eq!(args.config[2], "config3.config");
    }

    #[test]
    fn test_parse_args_file() {
        let args = EmulatorArgs::parse_from(&["beam", "--args-file", "/path/to/args.txt"]);
        assert_eq!(args.args_file, Some("/path/to/args.txt".to_string()));
    }

    #[test]
    fn test_parse_smp() {
        let args = EmulatorArgs::parse_from(&["beam", "--smp", "8"]);
        assert_eq!(args.smp, Some("8".to_string()));
    }

    #[test]
    fn test_parse_smp_flags() {
        let args = EmulatorArgs::parse_from(&["beam", "--smpenable"]);
        assert!(args.smpenable);

        let args = EmulatorArgs::parse_from(&["beam", "--smpdisable"]);
        assert!(args.smpdisable);

        let args = EmulatorArgs::parse_from(&["beam", "--smpauto"]);
        assert!(args.smpauto);
    }

    #[test]
    fn test_parse_emu_type() {
        let args = EmulatorArgs::parse_from(&["beam", "--emu-type", "debug"]);
        assert_eq!(args.emu_type, Some("debug".to_string()));
    }

    #[test]
    fn test_parse_emu_flavor() {
        let args = EmulatorArgs::parse_from(&["beam", "--emu-flavor", "smp"]);
        assert_eq!(args.emu_flavor, Some("smp".to_string()));
    }

    #[test]
    fn test_parse_special_exit_flags() {
        let args = EmulatorArgs::parse_from(&["beam", "--emu-args-exit"]);
        assert!(args.emu_args_exit);

        let args = EmulatorArgs::parse_from(&["beam", "--emu-name-exit"]);
        assert!(args.emu_name_exit);

        let args = EmulatorArgs::parse_from(&["beam", "--emu-qouted-cmd-exit"]);
        assert!(args.emu_qouted_cmd_exit);
    }

    #[test]
    fn test_parse_extra() {
        let args = EmulatorArgs::parse_from(&["beam", "--extra"]);
        assert!(args.extra);
    }

    #[test]
    fn test_parse_detached() {
        let args = EmulatorArgs::parse_from(&["beam", "--detached"]);
        assert!(args.detached);
    }

    #[test]
    fn test_parse_remaining_args() {
        let args = EmulatorArgs::parse_from(&["beam", "arg1", "arg2", "arg3"]);
        assert_eq!(args.remaining.len(), 3);
        assert_eq!(args.remaining[0], "arg1");
        assert_eq!(args.remaining[1], "arg2");
        assert_eq!(args.remaining[2], "arg3");
    }

    #[test]
    fn test_build_emulator_args_minimal() {
        let args = EmulatorArgs::parse_from(&["beam"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        // Should contain base arguments
        assert!(result.contains(&"beam".to_string()));
        assert!(result.contains(&"-root".to_string()));
        assert!(result.contains(&"/root".to_string()));
        assert!(result.contains(&"-bindir".to_string()));
        assert!(result.contains(&"/bin".to_string()));
        assert!(result.contains(&"-progname".to_string()));
        assert!(result.contains(&"beam".to_string()));
    }

    #[test]
    fn test_build_emulator_args_with_boot() {
        let args = EmulatorArgs::parse_from(&["beam", "--boot", "/path/to/boot.script"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        assert!(result.contains(&"-boot".to_string()));
        assert!(result.contains(&"/path/to/boot.script".to_string()));
    }

    #[test]
    fn test_build_emulator_args_with_config() {
        let args = EmulatorArgs::parse_from(&[
            "beam",
            "--config", "config1.config",
            "--config", "config2.config"
        ]);
        let result = args.build_emulator_args("/root", "/bin");
        
        // Should have both config entries
        let config_indices: Vec<usize> = result.iter()
            .enumerate()
            .filter_map(|(i, s)| if s == "-config" { Some(i) } else { None })
            .collect();
        assert_eq!(config_indices.len(), 2);
        
        // Verify config values follow -config flags
        assert_eq!(result[config_indices[0] + 1], "config1.config");
        assert_eq!(result[config_indices[1] + 1], "config2.config");
    }

    #[test]
    fn test_build_emulator_args_with_sname() {
        let args = EmulatorArgs::parse_from(&["beam", "--sname", "node@host"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        assert!(result.contains(&"-sname".to_string()));
        assert!(result.contains(&"node@host".to_string()));
    }

    #[test]
    fn test_build_emulator_args_with_name() {
        let args = EmulatorArgs::parse_from(&["beam", "--name", "node@host.domain"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        assert!(result.contains(&"-name".to_string()));
        assert!(result.contains(&"node@host.domain".to_string()));
    }

    #[test]
    fn test_build_emulator_args_with_proto_dist() {
        let args = EmulatorArgs::parse_from(&["beam", "--proto-dist", "inet_tls"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        assert!(result.contains(&"-proto_dist".to_string()));
        assert!(result.contains(&"inet_tls".to_string()));
    }

    #[test]
    fn test_build_emulator_args_with_no_epmd() {
        let args = EmulatorArgs::parse_from(&["beam", "--no-epmd", "--proto-dist", "inet_tcp"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        assert!(result.contains(&"-no_epmd".to_string()));
    }

    #[test]
    fn test_build_emulator_args_with_smp_value() {
        let args = EmulatorArgs::parse_from(&["beam", "--smp", "4"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        assert!(result.contains(&"-smp".to_string()));
        assert!(result.contains(&"4".to_string()));
    }

    #[test]
    fn test_build_emulator_args_with_smpenable() {
        let args = EmulatorArgs::parse_from(&["beam", "--smpenable"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        assert!(result.contains(&"-smp".to_string()));
        assert!(result.contains(&"enable".to_string()));
    }

    #[test]
    fn test_build_emulator_args_with_smpauto() {
        let args = EmulatorArgs::parse_from(&["beam", "--smpauto"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        assert!(result.contains(&"-smp".to_string()));
        assert!(result.contains(&"auto".to_string()));
    }

    #[test]
    fn test_build_emulator_args_smp_priority() {
        // smp value should take priority over smpenable/smpauto
        let args = EmulatorArgs::parse_from(&["beam", "--smp", "8", "--smpenable"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        // Should use smp value, not smpenable
        let smp_indices: Vec<usize> = result.iter()
            .enumerate()
            .filter_map(|(i, s)| if s == "-smp" { Some(i) } else { None })
            .collect();
        assert_eq!(smp_indices.len(), 1);
        assert_eq!(result[smp_indices[0] + 1], "8");
    }

    #[test]
    fn test_build_emulator_args_with_remaining() {
        let args = EmulatorArgs::parse_from(&["beam", "arg1", "arg2", "arg3"]);
        let result = args.build_emulator_args("/root", "/bin");
        
        // Remaining args should be at the end
        let remaining_start = result.iter().position(|s| s == "arg1").unwrap();
        assert_eq!(result[remaining_start], "arg1");
        assert_eq!(result[remaining_start + 1], "arg2");
        assert_eq!(result[remaining_start + 2], "arg3");
    }

    #[test]
    fn test_build_emulator_args_comprehensive() {
        let args = EmulatorArgs::parse_from(&[
            "beam",
            "--boot", "/boot.script",
            "--config", "config1.config",
            "--config", "config2.config",
            "--sname", "node@host",
            "--proto-dist", "inet_tcp",
            "--no-epmd",
            "--smp", "4",
            "remaining1", "remaining2"
        ]);
        let result = args.build_emulator_args("/root", "/bin");
        
        // Verify all components are present
        assert!(result.contains(&"-boot".to_string()));
        assert!(result.contains(&"/boot.script".to_string()));
        assert!(result.contains(&"-sname".to_string()));
        assert!(result.contains(&"node@host".to_string()));
        assert!(result.contains(&"-proto_dist".to_string()));
        assert!(result.contains(&"inet_tcp".to_string()));
        assert!(result.contains(&"-no_epmd".to_string()));
        assert!(result.contains(&"-smp".to_string()));
        assert!(result.contains(&"4".to_string()));
        assert!(result.contains(&"remaining1".to_string()));
        assert!(result.contains(&"remaining2".to_string()));
    }

    #[test]
    fn test_debug_trait() {
        let args = EmulatorArgs::parse_from(&["beam", "--sname", "test@localhost"]);
        let debug_str = format!("{:?}", args);
        assert!(!debug_str.is_empty());
        // Debug should include the struct name
        assert!(debug_str.contains("EmulatorArgs"));
    }

    #[test]
    fn test_empty_remaining_args() {
        let args = EmulatorArgs::parse_from(&["beam"]);
        assert!(args.remaining.is_empty());
        
        let result = args.build_emulator_args("/root", "/bin");
        // Should not have any remaining args
        assert!(!result.iter().any(|s| s.starts_with("arg")));
    }

    #[test]
    fn test_multiple_config_files() {
        let args = EmulatorArgs::parse_from(&[
            "beam",
            "--config", "file1.config",
            "--config", "file2.config",
            "--config", "file3.config"
        ]);
        assert_eq!(args.config.len(), 3);
        
        let result = args.build_emulator_args("/root", "/bin");
        // Count -config occurrences
        let config_count = result.iter().filter(|s| *s == "-config").count();
        assert_eq!(config_count, 3);
    }
}

