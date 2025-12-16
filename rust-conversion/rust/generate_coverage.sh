#!/usr/bin/env python3
"""
Rust Test Runner Script

A convenient wrapper for running Rust tests with various options.
Shows output as tests complete and supports parallel/serial execution.
Can generate HTML coverage reports using llvm-cov.
"""

import argparse
import subprocess
import sys
import os
import threading
import time
import glob
import shutil
from pathlib import Path
from datetime import datetime

# Default Rust workspace directory
RUST_DIR = Path(__file__).parent.absolute()
# Default coverage output directory
COVERAGE_DIR = RUST_DIR / "coverage-html" / "llvm-cov" / "html"


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


def generate_coverage_report(cwd, coverage_dir):
    """Generate HTML coverage report using llvm-cov."""
    print("\n" + "=" * 80)
    print("Generating coverage report...")
    print("=" * 80)
    
    # Create coverage directory
    coverage_dir.mkdir(parents=True, exist_ok=True)
    
    # Find all .profraw files
    profraw_files = []
    for pattern in ["**/*.profraw", "*.profraw"]:
        profraw_files.extend(glob.glob(str(cwd / pattern), recursive=True))
    
    if not profraw_files:
        print("Warning: No .profraw files found. Coverage data may not have been generated.")
        return 1
    
    print(f"Found {len(profraw_files)} profraw file(s)")
    
    # Merge profraw files
    profdata_file = cwd / "merged.profdata"
    merge_cmd = ["llvm-profdata", "merge", "-sparse"] + profraw_files + ["-o", str(profdata_file)]
    
    print(f"Running: {' '.join(merge_cmd[:5])}... (merging {len(profraw_files)} files)")
    result = subprocess.run(merge_cmd, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error merging profraw files: {result.stderr}", file=sys.stderr)
        return 1
    
    print("Profraw files merged successfully")
    
    # Find test binaries in target/debug/deps
    target_dir = cwd / "target" / "debug" / "deps"
    test_binaries = []
    
    if target_dir.exists():
        # Find all executable files (test binaries)
        for binary in target_dir.iterdir():
            if binary.is_file() and os.access(binary, os.X_OK):
                # Skip if it's not a Rust binary (basic check)
                if not binary.name.startswith('.'):
                    test_binaries.append(str(binary))
    
    if not test_binaries:
        print("Warning: No test binaries found. Trying alternative method...")
        # Try to use cargo-llvm-cov if available
        print("Attempting to use cargo-llvm-cov...")
        alt_cmd = ["cargo", "llvm-cov", "--html", "--output-dir", str(coverage_dir)]
        result = subprocess.run(alt_cmd, cwd=cwd, capture_output=True, text=True)
        if result.returncode == 0:
            index_html = coverage_dir / "index.html"
            if index_html.exists():
                print(f"\n{'=' * 80}")
                print(f"Coverage report generated successfully using cargo-llvm-cov!")
                print(f"HTML report location: {index_html}")
                print(f"{'=' * 80}\n")
                return 0
        print(f"cargo-llvm-cov failed: {result.stderr}", file=sys.stderr)
        return 1
    
    print(f"Found {len(test_binaries)} test binary(ies)")
    
    # Generate HTML report using llvm-cov show
    # We'll process binaries in batches to avoid command line length issues
    html_cmd = [
        "llvm-cov", "show",
        "--format", "html",
        "--output-dir", str(coverage_dir),
        "--show-line-counts-or-regions",
        "--show-instantiations",
        "--ignore-filename-regex", ".*/target/.*",
        "--ignore-filename-regex", ".*/\.cargo/.*",
        "--instr-profile", str(profdata_file),
    ]
    
    # Add binaries (limit to avoid command line length issues)
    # In practice, we can add many binaries
    html_cmd.extend(test_binaries[:50])  # Reasonable limit
    
    print(f"Generating HTML report with {len(test_binaries[:50])} binary(ies)...")
    result = subprocess.run(html_cmd, cwd=cwd, capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"Error generating HTML report: {result.stderr}", file=sys.stderr)
        if result.stdout:
            print(f"stdout: {result.stdout[:500]}", file=sys.stderr)
        return 1
    
    # Ensure index.html exists at the expected location
    index_html = coverage_dir / "index.html"
    if not index_html.exists():
        # Check if it's in a subdirectory
        for html_file in coverage_dir.rglob("index.html"):
            if html_file != index_html:
                shutil.copy2(html_file, index_html)
                print(f"Copied index.html from {html_file} to {index_html}")
                break
    
    if not index_html.exists():
        print("Warning: index.html not found in expected location", file=sys.stderr)
        # List what was created
        html_files = list(coverage_dir.rglob("*.html"))
        if html_files:
            print(f"Found HTML files: {[str(f) for f in html_files[:5]]}")
    
    print(f"\n{'=' * 80}")
    print(f"Coverage report generated successfully!")
    print(f"HTML report location: {index_html}")
    print(f"{'=' * 80}\n")
    
    return 0


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


def run_tests(cmd, cwd=None, background=False, is_parallel=True, generate_coverage=False, coverage_dir=None):
    """Run the cargo test command and stream output."""
    if cwd is None:
        cwd = RUST_DIR
    
    # Set up coverage environment if requested
    env = os.environ.copy()
    if generate_coverage:
        # Enable coverage instrumentation
        rustflags = env.get("RUSTFLAGS", "")
        if "-C instrument-coverage" not in rustflags:
            env["RUSTFLAGS"] = f"{rustflags} -C instrument-coverage".strip()
        
        # Set LLVM_PROFILE_FILE to generate profraw files
        # Use %p for process ID and %m for unique ID to avoid conflicts
        env["LLVM_PROFILE_FILE"] = str(cwd / "coverage-%p-%m.profraw")
        
        # Clean up old profraw files
        for old_profraw in cwd.glob("coverage-*.profraw"):
            try:
                old_profraw.unlink()
            except:
                pass
    
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
    
    if generate_coverage:
        print(f"Coverage instrumentation enabled")
        print(f"Profraw files will be written to: {cwd}")
    
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
                    bufsize=1,
                    env=env
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
                bufsize=1,
                env=env
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
        
        # Generate coverage report if requested
        if generate_coverage and return_code == 0:
            cov_result = generate_coverage_report(cwd, coverage_dir)
            if cov_result != 0:
                print("Warning: Coverage report generation had issues, but tests passed.", file=sys.stderr)
        
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
        description="Run Rust tests with various options. Output is shown as tests complete. "
                    "Can generate HTML coverage reports using llvm-cov.",
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

  # Generate HTML coverage report
  %(prog)s --coverage

  # Generate coverage report for specific package
  %(prog)s --coverage --package infrastructure_emulator_loop
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
    
    # Coverage generation
    parser.add_argument(
        "--coverage",
        action="store_true",
        help="Generate HTML coverage report using llvm-cov. "
             "Report will be written to coverage-html/llvm-cov/html/index.html"
    )
    
    parser.add_argument(
        "--coverage-dir",
        type=str,
        help=f"Directory for coverage HTML output (default: {COVERAGE_DIR})"
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
    
    # Determine coverage directory
    coverage_dir = Path(args.coverage_dir).absolute() if args.coverage_dir else COVERAGE_DIR
    
    # Build command
    cmd = build_cargo_command(args)
    
    # Determine if running in parallel (not serial)
    is_parallel = not args.serial
    
    # Run tests
    return run_tests(
        cmd, 
        cwd=cwd, 
        background=args.background, 
        is_parallel=is_parallel,
        generate_coverage=args.coverage,
        coverage_dir=coverage_dir
    )


if __name__ == "__main__":
    sys.exit(main())

