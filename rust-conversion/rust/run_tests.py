#!/usr/bin/env python3
"""
Rust Test Runner Script

A convenient wrapper for running Rust tests with various options.
Shows output as tests complete and supports parallel/serial execution.
"""

import argparse
import subprocess
import sys
import os
import threading
import time
from pathlib import Path
from datetime import datetime

# Default Rust workspace directory
RUST_DIR = Path(__file__).parent.absolute()


def build_cargo_command(args):
    """Build the cargo test command based on arguments."""
    cmd = ["cargo", "test"]
    
    # Package selection
    if args.package:
        cmd.extend(["--package", args.package])
    
    # Test type selection
    if args.lib:
        cmd.append("--lib")
    elif args.test_file:
        cmd.extend(["--test", args.test_file])
    elif args.bin:
        cmd.extend(["--bin", args.bin])
    elif args.example:
        cmd.extend(["--example", args.example])
    
    # Workspace flag
    if args.workspace:
        cmd.append("--workspace")
    
    # Specific test name
    if args.test:
        cmd.append(args.test)
    
    # Pass through additional cargo arguments
    if args.cargo_args:
        cmd.extend(args.cargo_args)
    
    # Test binary arguments (after --)
    test_args = []
    
    # Thread control
    if args.serial:
        test_args.extend(["--test-threads", "1"])
    elif args.test_threads:
        test_args.extend(["--test-threads", str(args.test_threads)])
    
    # Output control
    if args.nocapture:
        test_args.append("--nocapture")
    
    if args.show_output:
        test_args.append("--show-output")
    
    # Filter tests
    if args.filter:
        test_args.append(args.filter)
    
    # Pass through additional test arguments
    if args.test_args:
        test_args.extend(args.test_args)
    
    # Add test arguments after --
    if test_args:
        cmd.append("--")
        cmd.extend(test_args)
    
    return cmd


def tail_log_file(log_file, interval=5, lines=50, stop_event=None):
    """Periodically print the last N lines from a log file."""
    last_size = 0
    
    while not stop_event.is_set():
        try:
            if log_file.exists():
                # Get file size first
                current_size = log_file.stat().st_size
                
                # Only read if file has grown
                if current_size > last_size:
                    with open(log_file, 'r', encoding='utf-8', errors='ignore') as f:
                        # Read all lines
                        all_lines = f.readlines()
                        
                        if all_lines:
                            # Get last N lines
                            lines_to_print = all_lines[-lines:] if len(all_lines) > lines else all_lines
                            
                            # Print the lines
                            for line in lines_to_print:
                                print(line, end='')
                            print("\n")  # Two newlines as requested
                            sys.stdout.flush()
                            
                            last_size = current_size
            else:
                # File doesn't exist yet, wait a bit
                time.sleep(1)
        except (IOError, OSError):
            # File might be locked or not exist yet, wait and retry
            time.sleep(0.5)
        except Exception:
            # Silently handle other errors
            pass
        
        # Wait for interval or until stop event
        if stop_event.wait(interval):
            break
    
    # Print final output when stopping
    try:
        if log_file.exists():
            with open(log_file, 'r', encoding='utf-8', errors='ignore') as f:
                all_lines = f.readlines()
                if all_lines:
                    # Get last N lines
                    lines_to_print = all_lines[-lines:] if len(all_lines) > lines else all_lines
                    for line in lines_to_print:
                        print(line, end='')
                    print("\n")  # Two newlines as requested
                    sys.stdout.flush()
    except Exception:
        pass


def run_tests(cmd, cwd=None, background=False, is_parallel=True):
    """Run the cargo test command and stream output."""
    if cwd is None:
        cwd = RUST_DIR
    
    # Generate log file name if running in background with parallel execution
    log_file = None
    log_monitor_thread = None
    stop_event = None
    
    if background and is_parallel:
        # Create log file with timestamp
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        log_file = cwd / f"test_output_{timestamp}.log"
        stop_event = threading.Event()
        
        print(f"Running in background mode (parallel)")
        print(f"Log file: {log_file}")
        print(f"Monitoring log file every 5 seconds...")
        print("=" * 80)
        
        # Start log monitoring thread
        log_monitor_thread = threading.Thread(
            target=tail_log_file,
            args=(log_file, 5, 50, stop_event),
            daemon=True
        )
        log_monitor_thread.start()
    
    print(f"Running: {' '.join(cmd)}")
    print(f"Working directory: {cwd}")
    if not (background and is_parallel):
        print("=" * 80)
    
    try:
        if background and is_parallel:
            # Redirect output to log file
            with open(log_file, 'w', encoding='utf-8') as log:
                process = subprocess.Popen(
                    cmd,
                    cwd=cwd,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    universal_newlines=True,
                    bufsize=1
                )
                process.wait()
                return_code = process.returncode
        else:
            # Run with real-time output
            process = subprocess.Popen(
                cmd,
                cwd=cwd,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                universal_newlines=True,
                bufsize=1
            )
            
            # Stream output line by line
            for line in process.stdout:
                print(line, end='')
                sys.stdout.flush()
            
            process.wait()
            return_code = process.returncode
        
        # Stop log monitoring thread
        if stop_event:
            stop_event.set()
            if log_monitor_thread:
                log_monitor_thread.join(timeout=2)
        
        if background and is_parallel and log_file:
            print(f"\n{'=' * 80}")
            print(f"Test run completed. Full log available at: {log_file}")
        
        return return_code
        
    except KeyboardInterrupt:
        print("\n\nTest run interrupted by user")
        if stop_event:
            stop_event.set()
        if 'process' in locals() and process:
            process.terminate()
        return 130
    except Exception as e:
        print(f"Error running tests: {e}", file=sys.stderr)
        if stop_event:
            stop_event.set()
        return 1


def main():
    parser = argparse.ArgumentParser(
        description="Run Rust tests with various options. Output is shown as tests complete.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Run all tests in parallel (default)
  %(prog)s

  # Run all tests serially
  %(prog)s --serial

  # Run tests for a specific package
  %(prog)s --package infrastructure_emulator_loop

  # Run a specific test
  %(prog)s --package infrastructure_emulator_loop --test test_register_manager_integration

  # Run library tests for a package
  %(prog)s --package infrastructure_emulator_loop --lib

  # Run integration tests
  %(prog)s --package infrastructure_emulator_loop --test-file integration_test

  # Run with custom thread count
  %(prog)s --test-threads 4

  # Run with output shown
  %(prog)s --nocapture

  # Run all workspace tests
  %(prog)s --workspace

  # Run with filter
  %(prog)s --filter test_execute

  # Pass additional cargo arguments
  %(prog)s --cargo-args --release

  # Pass additional test arguments
  %(prog)s --test-args --skip slow_tests

  # Run in background mode (parallel only, logs to file)
  %(prog)s --background --package infrastructure_emulator_loop
        """
    )
    
    # Package and test selection
    parser.add_argument(
        "--package", "-p",
        help="Run tests for a specific package (crate)"
    )
    
    parser.add_argument(
        "--workspace",
        action="store_true",
        help="Run tests for all packages in the workspace"
    )
    
    parser.add_argument(
        "--test",
        help="Run a specific test by name (supports partial matching)"
    )
    
    parser.add_argument(
        "--lib",
        action="store_true",
        help="Run library tests (unit tests)"
    )
    
    parser.add_argument(
        "--test-file",
        help="Run integration tests from a specific test file (without .rs extension)"
    )
    
    parser.add_argument(
        "--bin",
        help="Run tests for a specific binary"
    )
    
    parser.add_argument(
        "--example",
        help="Run tests for a specific example"
    )
    
    # Execution control
    execution_group = parser.add_mutually_exclusive_group()
    execution_group.add_argument(
        "--serial",
        action="store_true",
        help="Run tests serially (equivalent to --test-threads 1)"
    )
    execution_group.add_argument(
        "--test-threads",
        type=int,
        metavar="N",
        help="Number of test threads to use (default: number of CPU cores)"
    )
    
    # Output control
    parser.add_argument(
        "--nocapture",
        action="store_true",
        help="Show output from test execution (don't capture stdout/stderr)"
    )
    
    parser.add_argument(
        "--show-output",
        action="store_true",
        help="Show output from passing tests (normally only failures are shown)"
    )
    
    parser.add_argument(
        "--filter",
        help="Filter tests by name pattern"
    )
    
    # Additional arguments
    parser.add_argument(
        "--cargo-args",
        nargs="+",
        default=[],
        help="Additional arguments to pass to cargo test (before --). Use multiple times or space-separated."
    )
    
    parser.add_argument(
        "--test-args",
        nargs="+",
        default=[],
        help="Additional arguments to pass to the test binary (after --). Use multiple times or space-separated."
    )
    
    # Working directory
    parser.add_argument(
        "--dir", "-d",
        type=str,
        help=f"Working directory (default: {RUST_DIR})"
    )
    
    # Background mode
    parser.add_argument(
        "--background", "-b",
        action="store_true",
        help="Run tests in background mode. When combined with parallel execution, "
             "output is logged to a file and a separate thread periodically prints "
             "the last 50 lines. Only works with parallel execution (not --serial)."
    )
    
    args = parser.parse_args()
    
    # Validate background mode
    if args.background and args.serial:
        print("Warning: --background mode requires parallel execution. Ignoring --serial.", file=sys.stderr)
        args.serial = False
    
    # Determine working directory
    cwd = Path(args.dir).absolute() if args.dir else RUST_DIR
    
    if not cwd.exists():
        print(f"Error: Directory does not exist: {cwd}", file=sys.stderr)
        return 1
    
    # Build command
    cmd = build_cargo_command(args)
    
    # Determine if running in parallel (not serial)
    is_parallel = not args.serial
    
    # Run tests
    return run_tests(cmd, cwd=cwd, background=args.background, is_parallel=is_parallel)


if __name__ == "__main__":
    sys.exit(main())

