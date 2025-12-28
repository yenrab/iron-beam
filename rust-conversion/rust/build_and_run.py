#!/usr/bin/env python3
"""
Build and run the Rust Erlang emulator using the Makefile.

This script builds the emulator and optionally runs it with various configuration
options. All emulator command-line arguments are supported.
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path


def get_script_dir():
    """Get the directory where this script is located."""
    return Path(__file__).parent.absolute()


def run_command(cmd, cwd=None, check=True, shell=False, env=None):
    """Run a command and return the result."""
    print(f"Running: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    print("-" * 60)
    
    if isinstance(cmd, str) and shell:
        print(f"Running shell command: {cmd}")
        result = subprocess.run(cmd, cwd=cwd, shell=True, check=check, env=env)
    elif isinstance(cmd, list):
        print(f"Running command: {cmd}")
        result = subprocess.run(cmd, cwd=cwd, check=check, env=env)
    else:
        print(f"Running command: {cmd}")
        result = subprocess.run(cmd, cwd=cwd, shell=True, check=check, env=env)
    
    print(f"Result: {result}")
    print(f"Result.returncode: {result.returncode}")
    print(f"Result.stdout: {result.stdout}")
    print(f"Result.stderr: {result.stderr}")
    print("-" * 60)
    return result


def build_emulator(build_type="release", clean=False, skip_beam_build=False):
    """Build the emulator using make."""
    script_dir = get_script_dir()

    print(f"\n{'='*60}")
    print(f"Building Rust emulator ({build_type} mode)")
    print(f"{'='*60}\n")

    # Clean if requested
    if clean:
        print("Cleaning previous build...")
        run_command(["make", "clean"], cwd=script_dir)
        print()

    # Build with the specified build type
    # Pass RUST_BUILD_TYPE and SKIP_BEAM_BUILD as make variables
    try:
        make_vars = [f"RUST_BUILD_TYPE={build_type}"]
        if skip_beam_build:
            make_vars.append("SKIP_BEAM_BUILD=1")

        if build_type == "release":
            # For release, we can use the release target which does all + install
            run_command(["make"] + make_vars + ["release"], cwd=script_dir, check=True)
        else:
            # For debug, use all + install with RUST_BUILD_TYPE set as make variable
            run_command(
                ["make"] + make_vars + ["all", "install"],
                cwd=script_dir,
                check=True
            )

        print(f"\n✓ Build complete ({build_type} mode)")
        return True
    except subprocess.CalledProcessError as e:
        print(f"\n✗ Build failed with exit code {e.returncode}")
        return False


def build_emulator_args(emulator_args):
    """Build the command-line arguments for the emulator from parsed arguments."""
    cmd_args = []
    
    # Distribution options
    if emulator_args.sname:
        cmd_args.extend(["--sname", emulator_args.sname])
    
    if emulator_args.name:
        cmd_args.extend(["--name", emulator_args.name])
    
    if emulator_args.proto_dist:
        cmd_args.extend(["--proto-dist", emulator_args.proto_dist])
    
    if emulator_args.no_epmd:
        cmd_args.append("--no-epmd")
    
    if emulator_args.epmd:
        cmd_args.extend(["--epmd", emulator_args.epmd])
    
    # Boot and config
    if emulator_args.boot:
        cmd_args.extend(["--boot", emulator_args.boot])
    
    for config in emulator_args.config:
        cmd_args.extend(["--config", config])
    
    if emulator_args.args_file:
        cmd_args.extend(["--args-file", emulator_args.args_file])
    
    # SMP options
    if emulator_args.smp:
        cmd_args.extend(["--smp", emulator_args.smp])
    elif emulator_args.smpenable:
        cmd_args.append("--smpenable")
    elif emulator_args.smpauto:
        cmd_args.append("--smpauto")
    elif emulator_args.smpdisable:
        cmd_args.append("--smpdisable")
    
    # Emulator type/flavor
    if emulator_args.emu_type:
        cmd_args.extend(["--emu-type", emulator_args.emu_type])
    
    if emulator_args.emu_flavor:
        cmd_args.extend(["--emu-flavor", emulator_args.emu_flavor])
    
    # Special exit flags
    if emulator_args.emu_args_exit:
        cmd_args.append("--emu-args-exit")
    
    if emulator_args.emu_name_exit:
        cmd_args.append("--emu-name-exit")
    
    if emulator_args.emu_qouted_cmd_exit:
        cmd_args.append("--emu-qouted-cmd-exit")
    
    # Other flags
    if emulator_args.extra:
        cmd_args.append("--extra")
    
    if emulator_args.detached:
        cmd_args.append("--detached")
    
    # Remaining arguments
    if emulator_args.remaining:
        cmd_args.extend(emulator_args.remaining)
    
    return cmd_args


def validate_emulator_args(emulator_args):
    """Validate emulator argument combinations."""
    errors = []
    
    # no-epmd requires proto-dist
    if emulator_args.no_epmd and not emulator_args.proto_dist:
        errors.append("--no-epmd requires --proto-dist to be specified")
    
    # sname and name are mutually exclusive
    if emulator_args.sname and emulator_args.name:
        errors.append("Cannot specify both --sname and --name")
    
    if errors:
        return False, errors
    return True, []


def run_emulator(build_type="release", emulator_args=None):
    """Run the emulator."""
    script_dir = get_script_dir()
    rootdir = script_dir / "target" / "otp_root"
    beam_binary = rootdir / "bin" / "beam"
    
    if not beam_binary.exists():
        print(f"\n✗ Error: Binary not found at {beam_binary}")
        print("  Please build the emulator first.")
        return False
    
    print(f"\n{'='*60}")
    print(f"Running emulator ({build_type} mode)")
    print(f"{'='*60}\n")
    print(f"Binary: {beam_binary}")
    print(f"ROOTDIR: {rootdir}")
    
    # Validate arguments
    if emulator_args:
        valid, errors = validate_emulator_args(emulator_args)
        if not valid:
            print("\n✗ Invalid emulator arguments:")
            for error in errors:
                print(f"  - {error}")
            return False
        
        # Build command arguments
        cmd_args = build_emulator_args(emulator_args)
        if cmd_args:
            print(f"Arguments: {' '.join(cmd_args)}")
    else:
        cmd_args = []
    
    print()
    
    # Change to rootdir and run the emulator
    env = os.environ.copy()
    env["ROOTDIR"] = str(rootdir)
    
    # Prepare command
    cmd = [str(beam_binary)] + cmd_args
    
    try:
        # Run the emulator (this will block until it exits)
        run_command(cmd, cwd=rootdir, check=False, env=env)
        return True
    except KeyboardInterrupt:
        print("\n\nEmulator interrupted by user")
        return True
    except Exception as e:
        print(f"\n✗ Error running emulator: {e}")
        return False


def create_parser():
    """Create and configure the argument parser."""
    parser = argparse.ArgumentParser(
        description="Build and run the Rust Erlang emulator",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
EXAMPLES:
  # Build release and run
  python build_and_run.py

  # Build debug and run
  python build_and_run.py -t debug

  # Build only, don't run
  python build_and_run.py --build-only

  # Skip rebuild, just run (assumes binary exists)
  python build_and_run.py --run

  # Clean and rebuild
  python build_and_run.py --clean

  # Run with distribution name
  python build_and_run.py --sname test@localhost

  # Run with boot script and config
  python build_and_run.py --boot start.boot --config sys.config

  # Run with SMP enabled
  python build_and_run.py --smpenable

  # Run with multiple config files
  python build_and_run.py --config sys.config --config app.config

  # Pass remaining arguments to emulator
  python build_and_run.py -- -eval "io:format(\\\"Hello~n\\\")." -s init stop
  
  # Or without -- separator (if no conflicts)
  python build_and_run.py -eval "io:format(\\\"Hello~n\\\")." -s init stop

EMULATOR ARGUMENTS:
  All standard Erlang emulator arguments are supported. See below for details.
        """
    )
    
    # Build options
    build_group = parser.add_argument_group("Build Options")
    build_group.add_argument(
        "-t", "--build-type",
        choices=["release", "debug"],
        default="release",
        help="Build type: release (default) or debug"
    )
    build_group.add_argument(
        "--run",
        action="store_true",
        help="Skip building and run the currently built application (fails if not built)"
    )
    build_group.add_argument(
        "--build-only",
        action="store_true",
        help="Build only, do not run the emulator"
    )
    build_group.add_argument(
        "--clean",
        action="store_true",
        help="Clean previous build before building"
    )
    build_group.add_argument(
        "--skip-beam-build",
        action="store_true",
        help="Skip building .beam files if they already exist"
    )
    
    # Distribution options
    dist_group = parser.add_argument_group("Distribution Options")
    dist_group.add_argument(
        "--sname",
        metavar="NAME",
        help="Short name for distribution (e.g., 'node@host'). "
             "Mutually exclusive with --name."
    )
    dist_group.add_argument(
        "--name",
        metavar="NAME",
        help="Long name for distribution (e.g., 'node@host.domain'). "
             "Mutually exclusive with --sname."
    )
    dist_group.add_argument(
        "--proto-dist",
        metavar="PROTO",
        help="Distribution protocol (e.g., 'inet_tcp', 'inet_tls')"
    )
    dist_group.add_argument(
        "--no-epmd",
        action="store_true",
        help="Do not start epmd daemon (requires --proto-dist)"
    )
    dist_group.add_argument(
        "--epmd",
        metavar="PATH",
        help="Path to epmd program"
    )
    
    # Boot and configuration
    boot_group = parser.add_argument_group("Boot and Configuration")
    boot_group.add_argument(
        "--boot",
        metavar="FILE",
        help="Boot script path (e.g., 'start.boot')"
    )
    boot_group.add_argument(
        "--config",
        metavar="FILE",
        action="append",
        default=[],
        help="Config file path (can be specified multiple times)"
    )
    boot_group.add_argument(
        "--args-file",
        metavar="FILE",
        help="Arguments file path"
    )
    
    # SMP options
    smp_group = parser.add_argument_group("SMP Options")
    smp_group.add_argument(
        "--smp",
        metavar="MODE",
        help="SMP mode: number of schedulers, 'auto', or 'enable'"
    )
    smp_group.add_argument(
        "--smpenable",
        action="store_true",
        help="Enable SMP (mutually exclusive with other SMP options)"
    )
    smp_group.add_argument(
        "--smpdisable",
        action="store_true",
        help="Disable SMP (mutually exclusive with other SMP options)"
    )
    smp_group.add_argument(
        "--smpauto",
        action="store_true",
        help="Auto-detect SMP (mutually exclusive with other SMP options)"
    )
    
    # Emulator type/flavor
    emu_group = parser.add_argument_group("Emulator Type/Flavor")
    emu_group.add_argument(
        "--emu-type",
        metavar="TYPE",
        help="Emulator type (e.g., 'opt', 'debug', 'lcnt', 'valgrind')"
    )
    emu_group.add_argument(
        "--emu-flavor",
        metavar="FLAVOR",
        help="Emulator flavor (e.g., 'smp', 'jit', 'emu')"
    )
    
    # Special modes
    special_group = parser.add_argument_group("Special Modes")
    special_group.add_argument(
        "--emu-args-exit",
        action="store_true",
        help="Print emulator arguments and exit"
    )
    special_group.add_argument(
        "--emu-name-exit",
        action="store_true",
        help="Print emulator name and exit"
    )
    special_group.add_argument(
        "--emu-qouted-cmd-exit",
        action="store_true",
        help="Print quoted command line and exit"
    )
    
    # Other options
    other_group = parser.add_argument_group("Other Options")
    other_group.add_argument(
        "--extra",
        action="store_true",
        help="Extra flag: all remaining arguments after this"
    )
    other_group.add_argument(
        "--detached",
        action="store_true",
        help="Detached mode (Windows-specific)"
    )
    
    # Remaining arguments (everything else)
    parser.add_argument(
        "remaining",
        nargs=argparse.REMAINDER,
        metavar="...",
        help="Remaining arguments passed directly to the emulator. "
             "These can include Erlang flags like -eval, -s, -noshell, "
             "-pa, -pz, etc. Use '--' to separate script options from "
             "emulator arguments if needed."
    )
    
    return parser


def main():
    parser = create_parser()
    args = parser.parse_args()
    
    # Separate build options from emulator options
    build_type = args.build_type
    run_only = args.run
    build_only = args.build_only
    clean = args.clean
    skip_beam_build = args.skip_beam_build
    
    # Validate mutually exclusive options
    if run_only and build_only:
        print("✗ Error: --run and --build-only are mutually exclusive")
        sys.exit(1)
    
    # Create a namespace for emulator arguments
    class EmulatorArgs:
        def __init__(self, args):
            self.sname = args.sname
            self.name = args.name
            self.proto_dist = args.proto_dist
            self.no_epmd = args.no_epmd
            self.epmd = args.epmd
            self.boot = args.boot
            self.config = args.config
            self.args_file = args.args_file
            self.smp = args.smp
            self.smpenable = args.smpenable
            self.smpdisable = args.smpdisable
            self.smpauto = args.smpauto
            self.emu_type = args.emu_type
            self.emu_flavor = args.emu_flavor
            self.emu_args_exit = args.emu_args_exit
            self.emu_name_exit = args.emu_name_exit
            self.emu_qouted_cmd_exit = args.emu_qouted_cmd_exit
            self.extra = args.extra
            self.detached = args.detached
            self.remaining = args.remaining
    
    emulator_args = EmulatorArgs(args)
    
    # Build the emulator unless --run is specified
    if not run_only:
        if not build_emulator(build_type, clean, skip_beam_build):
            sys.exit(1)
    
    # Run the emulator unless --build-only is specified
    if not build_only:
        if not run_emulator(build_type, emulator_args):
            sys.exit(1)
    else:
        script_dir = get_script_dir()
        rootdir = script_dir / "target" / "otp_root"
        beam_binary = rootdir / "bin" / "beam"
        print(f"\n✓ Build complete. To run manually:")
        print(f"  cd {rootdir}")
        print(f"  ROOTDIR=$(pwd) ./bin/beam")
        if any(vars(emulator_args).values()):
            cmd_args = build_emulator_args(emulator_args)
            if cmd_args:
                print(f"  ROOTDIR=$(pwd) ./bin/beam {' '.join(cmd_args)}")


if __name__ == "__main__":
    main()
