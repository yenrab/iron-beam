#!/usr/bin/env python3
"""
Rust Coverage Report Generator

Generates HTML coverage reports using cargo llvm-cov.
Output is written to target/llvm-cov/html/index.html

Usage:
  ./generate_coverage.sh --workspace                    # Run coverage for all packages (parallel)
  ./generate_coverage.sh --workspace --serial           # Run coverage serially
  ./generate_coverage.sh --package my_crate             # Run coverage for specific package
  ./generate_coverage.sh --help                         # Show help
"""

import argparse
import subprocess
import sys
import os
import glob
from pathlib import Path

# Default Rust workspace directory
RUST_DIR = Path(__file__).parent.absolute()
# Default coverage output directory
COVERAGE_DIR = RUST_DIR / "target" / "llvm-cov" / "html"


def find_llvm_tools():
    """
    Find llvm-cov and llvm-profdata binaries in the rustup toolchain.
    Returns a dict with LLVM_COV and LLVM_PROFDATA paths, or empty dict if not found.
    """
    env_vars = {}
    
    # Check if already set in environment
    if os.environ.get("LLVM_COV") and os.environ.get("LLVM_PROFDATA"):
        return env_vars  # Already set, no need to override
    
    # Try to find rustup home
    rustup_home = os.environ.get("RUSTUP_HOME")
    if not rustup_home:
        home = Path.home()
        rustup_home = home / ".rustup"
    else:
        rustup_home = Path(rustup_home)
    
    if not rustup_home.exists():
        return env_vars
    
    # Find llvm-tools in the toolchain
    # Pattern: ~/.rustup/toolchains/*/lib/rustlib/*/bin/llvm-cov
    toolchains_dir = rustup_home / "toolchains"
    if not toolchains_dir.exists():
        return env_vars
    
    llvm_cov_path = None
    llvm_profdata_path = None
    
    # Search for llvm-cov and llvm-profdata
    for toolchain in toolchains_dir.iterdir():
        if not toolchain.is_dir():
            continue
        
        # Check lib/rustlib/*/bin/ for llvm tools
        rustlib_dir = toolchain / "lib" / "rustlib"
        if not rustlib_dir.exists():
            continue
        
        for target_dir in rustlib_dir.iterdir():
            if not target_dir.is_dir():
                continue
            
            bin_dir = target_dir / "bin"
            if not bin_dir.exists():
                continue
            
            # Look for llvm-cov
            for name in ["llvm-cov", "llvm-cov.exe"]:
                candidate = bin_dir / name
                if candidate.exists() and candidate.is_file():
                    llvm_cov_path = candidate
                    break
            
            # Look for llvm-profdata
            for name in ["llvm-profdata", "llvm-profdata.exe"]:
                candidate = bin_dir / name
                if candidate.exists() and candidate.is_file():
                    llvm_profdata_path = candidate
                    break
            
            if llvm_cov_path and llvm_profdata_path:
                break
        
        if llvm_cov_path and llvm_profdata_path:
            break
    
    if llvm_cov_path:
        env_vars["LLVM_COV"] = str(llvm_cov_path)
        print(f"Found llvm-cov: {llvm_cov_path}")
    
    if llvm_profdata_path:
        env_vars["LLVM_PROFDATA"] = str(llvm_profdata_path)
        print(f"Found llvm-profdata: {llvm_profdata_path}")
    
    return env_vars


def ensure_llvm_tools_installed():
    """
    Ensure llvm-tools-preview is installed.
    Returns True if tools are available, False otherwise.
    """
    # First try to find existing tools
    llvm_env = find_llvm_tools()
    if llvm_env:
        return llvm_env
    
    # Try to install llvm-tools-preview
    print("llvm-tools not found, attempting to install llvm-tools-preview...")
    result = subprocess.run(
        ["rustup", "component", "add", "llvm-tools-preview"],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        print("llvm-tools-preview installed successfully")
        # Try to find the tools again
        return find_llvm_tools()
    else:
        print(f"Warning: Failed to install llvm-tools-preview: {result.stderr}", file=sys.stderr)
        return {}


def generate_coverage_report(cwd, coverage_dir, package=None, workspace=False, all_features=False, serial=False):
    """Generate HTML coverage report using cargo llvm-cov."""
    print("\n" + "=" * 80)
    print("Generating coverage report using cargo llvm-cov...")
    print(f"Execution mode: {'serial' if serial else 'parallel'}")
    print("=" * 80)
    
    # Ensure llvm tools are available
    llvm_env = ensure_llvm_tools_installed()
    
    # Create coverage directory
    coverage_dir.mkdir(parents=True, exist_ok=True)
    
    # Build cargo llvm-cov command
    cmd = ["cargo", "llvm-cov", "--html", "--output-dir", str(coverage_dir)]
    
    if workspace:
        cmd.append("--workspace")
    elif package:
        cmd.extend(["--package", package])
    
    if all_features:
        cmd.append("--all-features")
    
    # Add test arguments after -- for serial execution
    if serial:
        cmd.extend(["--", "--test-threads=1"])
    
    print(f"Running: {' '.join(cmd)}")
    print(f"Working directory: {cwd}")
    print(f"Coverage output: {coverage_dir}")
    print("=" * 80)
    
    # Set up environment with LLVM tool paths
    env = os.environ.copy()
    env.update(llvm_env)
    
    # Run cargo llvm-cov
    result = subprocess.run(cmd, cwd=cwd, env=env, capture_output=False)
    
    if result.returncode != 0:
        print(f"\nError: cargo llvm-cov failed with exit code {result.returncode}", file=sys.stderr)
        return 1
    
    # Check if index.html was created
    index_html = coverage_dir / "index.html"
    if index_html.exists():
        print(f"\n{'=' * 80}")
        print(f"Coverage report generated successfully!")
        print(f"HTML report location: {index_html}")
        print(f"{'=' * 80}\n")
        return 0
    else:
        # Check for index.html in subdirectories
        html_files = list(coverage_dir.rglob("index.html"))
        if html_files:
            print(f"\n{'=' * 80}")
            print(f"Coverage report generated successfully!")
            print(f"HTML report location: {html_files[0]}")
            print(f"{'=' * 80}\n")
            return 0
        else:
            print(f"\nWarning: index.html not found in {coverage_dir}", file=sys.stderr)
            print("Coverage report may have been generated in a different location.", file=sys.stderr)
            return 1


def main():
    parser = argparse.ArgumentParser(
        description="Generate HTML coverage report using cargo llvm-cov. "
                    "Tests run in parallel by default. "
                    "Report will be written to target/llvm-cov/html/index.html",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate coverage for entire workspace (parallel, default)
  %(prog)s --workspace

  # Generate coverage for entire workspace (serial)
  %(prog)s --workspace --serial

  # Generate coverage for a specific package
  %(prog)s --package infrastructure_emulator_loop

  # Generate coverage for a specific package (serial)
  %(prog)s --package infrastructure_emulator_loop --serial

  # Generate coverage with all features enabled
  %(prog)s --workspace --all-features

  # Show this help message
  %(prog)s --help
        """
    )
    
    parser.add_argument(
        "--package", "-p",
        help="Generate coverage for a specific package (crate)"
    )
    
    parser.add_argument(
        "--workspace", "-w",
        action="store_true",
        help="Generate coverage for all packages in the workspace"
    )
    
    parser.add_argument(
        "--all-features",
        action="store_true",
        help="Build with all features enabled"
    )
    
    parser.add_argument(
        "--serial", "-s",
        action="store_true",
        help="Run tests serially (one at a time). Default is parallel execution."
    )
    
    parser.add_argument(
        "--dir", "-d",
        type=str,
        help=f"Working directory (default: {RUST_DIR})"
    )
    
    parser.add_argument(
        "--coverage-dir",
        type=str,
        help=f"Directory for coverage HTML output (default: {COVERAGE_DIR})"
    )
    
    args = parser.parse_args()
    
    # Validate arguments
    if not args.workspace and not args.package:
        print("Error: Must specify either --workspace or --package", file=sys.stderr)
        print("Use --help for usage information.", file=sys.stderr)
        return 1
    
    # Determine working directory
    cwd = Path(args.dir).absolute() if args.dir else RUST_DIR
    
    if not cwd.exists():
        print(f"Error: Directory does not exist: {cwd}", file=sys.stderr)
        return 1
    
    # Determine coverage directory
    coverage_dir = Path(args.coverage_dir).absolute() if args.coverage_dir else COVERAGE_DIR
    
    # Generate coverage report
    return generate_coverage_report(
        cwd=cwd,
        coverage_dir=coverage_dir,
        package=args.package,
        workspace=args.workspace,
        all_features=args.all_features,
        serial=args.serial
    )


if __name__ == "__main__":
    sys.exit(main())
