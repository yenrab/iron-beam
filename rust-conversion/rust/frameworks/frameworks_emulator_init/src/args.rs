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
        let args = EmulatorArgs::parse_from(&["beam", "-sname", "test@localhost"]);
        assert!(args.is_distributed());

        let args = EmulatorArgs::parse_from(&["beam"]);
        assert!(!args.is_distributed());
    }

    #[test]
    fn test_should_start_epmd() {
        let args = EmulatorArgs::parse_from(&["beam", "-sname", "test@localhost"]);
        assert!(args.should_start_epmd());

        let args = EmulatorArgs::parse_from(&["beam", "-sname", "test@localhost", "-no_epmd"]);
        assert!(!args.should_start_epmd());
    }

    #[test]
    fn test_validate() {
        let args = EmulatorArgs::parse_from(&["beam", "-no_epmd"]);
        assert!(args.validate().is_err());

        let args = EmulatorArgs::parse_from(&["beam", "-no_epmd", "-proto_dist", "inet_tcp"]);
        assert!(args.validate().is_ok());
    }
}

