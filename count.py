#!/usr/bin/env python3
"""
Count lines in Rust source files.

Counts:
- Total non-empty lines
- Documentation lines (///, //!, /**, /*!)
- Source code lines (non-test, non-documentation)
- Test code lines (within #[cfg(test)] or #[test] blocks)
- Unsafe code lines (within unsafe blocks or unsafe functions)
"""

import os
import re
from pathlib import Path
from typing import Tuple


def is_documentation_line(line: str) -> bool:
    """Check if a line is a documentation comment."""
    stripped = line.strip()
    # Single-line doc comments
    if stripped.startswith('///') or stripped.startswith('//!'):
        return True
    # Multi-line doc comment start/end
    if stripped.startswith('/**') or stripped.startswith('/*!'):
        return True
    # Inside multi-line doc comment (lines starting with * or containing */)
    if stripped.startswith('*') and ('*/' in stripped or stripped.startswith('* ')):
        return True
    return False


def is_non_empty(line: str) -> bool:
    """Check if a line is non-empty (contains non-whitespace characters)."""
    return bool(line.strip())


def count_lines_in_file(file_path: Path) -> Tuple[int, int, int, int, int]:
    """
    Count lines in a Rust file.
    
    Returns:
        (total_non_empty, documentation, source_code, test_code, unsafe_code)
    """
    # If file is in a tests/ directory, count everything as test code
    if 'tests' in file_path.parts:
        return count_test_file(file_path)
    
    total_non_empty = 0
    documentation = 0
    source_code = 0
    test_code = 0
    unsafe_code = 0
    
    in_test_block = False
    in_test_function = False
    test_block_brace_depth = 0
    test_function_brace_depth = 0
    in_multiline_doc = False
    saw_cfg_test = False  # Track if we just saw #[cfg(test)]
    in_unsafe_block = False
    unsafe_block_brace_depth = 0
    in_unsafe_function = False
    unsafe_function_brace_depth = 0
    saw_unsafe_keyword = False  # Track if we just saw "unsafe" keyword
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            for line in f:
                stripped = line.strip()
                
                # Skip empty lines
                if not is_non_empty(line):
                    continue
                
                total_non_empty += 1
                
                # Check for multi-line doc comment boundaries
                if '/**' in stripped or '/*!' in stripped:
                    in_multiline_doc = True
                    documentation += 1
                    continue
                if '*/' in stripped and in_multiline_doc:
                    in_multiline_doc = False
                    documentation += 1
                    continue
                
                # If inside multi-line doc comment, count as documentation
                if in_multiline_doc:
                    documentation += 1
                    continue
                
                # Check for single-line documentation
                if is_documentation_line(line):
                    documentation += 1
                    continue
                
                # Track brace depth
                open_braces = stripped.count('{')
                close_braces = stripped.count('}')
                
                # Check for unsafe keyword (unsafe fn, unsafe trait, unsafe impl, unsafe {})
                # Match "unsafe" as a whole word (not part of another word)
                if re.search(r'\bunsafe\b', stripped):
                    saw_unsafe_keyword = True
                    # Check if it's an unsafe function/trait/impl (followed by fn/trait/impl)
                    if re.search(r'\bunsafe\s+(fn|trait|impl)\b', stripped):
                        in_unsafe_function = True
                        unsafe_function_brace_depth = 0
                        unsafe_code += 1
                        saw_unsafe_keyword = False
                
                # Check for unsafe block start (unsafe {)
                if saw_unsafe_keyword and open_braces > 0:
                    in_unsafe_block = True
                    unsafe_block_brace_depth = open_braces - close_braces
                    saw_unsafe_keyword = False
                    unsafe_code += 1
                
                # Track unsafe function braces
                if in_unsafe_function:
                    unsafe_function_brace_depth += open_braces - close_braces
                    unsafe_code += 1
                    # Exit unsafe function when braces balance
                    if unsafe_function_brace_depth <= 0 and close_braces > 0:
                        in_unsafe_function = False
                
                # Track unsafe block braces
                if in_unsafe_block:
                    unsafe_block_brace_depth += open_braces - close_braces
                    unsafe_code += 1
                    # Exit unsafe block when braces balance
                    if unsafe_block_brace_depth <= 0 and close_braces > 0:
                        in_unsafe_block = False
                    # If we're in an unsafe block, continue (don't count as regular source/test)
                    continue
                
                # If we saw "unsafe" but haven't entered the block yet, it's still unsafe code
                if saw_unsafe_keyword:
                    unsafe_code += 1
                    saw_unsafe_keyword = False
                    continue
                
                # Check for test block start
                if '#[cfg(test)]' in stripped:
                    saw_cfg_test = True
                    test_code += 1
                    continue
                
                # If we just saw #[cfg(test)], the next line with { starts the test block
                if saw_cfg_test and open_braces > 0:
                    in_test_block = True
                    test_block_brace_depth = open_braces - close_braces
                    saw_cfg_test = False
                    test_code += 1
                    continue
                
                # Check for test function
                if '#[test]' in stripped:
                    in_test_function = True
                    test_function_brace_depth = 0
                    test_code += 1
                    continue
                
                # If we're in a test function, track its braces
                if in_test_function:
                    test_function_brace_depth += open_braces - close_braces
                    test_code += 1
                    # Exit test function when braces balance
                    if test_function_brace_depth <= 0 and close_braces > 0:
                        in_test_function = False
                    # Also check if we're in unsafe code within test function
                    if in_unsafe_function:
                        unsafe_code += 1
                    continue
                
                # If we're in a test block, track its braces
                if in_test_block:
                    test_block_brace_depth += open_braces - close_braces
                    test_code += 1
                    # Exit test block when braces balance
                    if test_block_brace_depth <= 0 and close_braces > 0:
                        in_test_block = False
                    # Also check if we're in unsafe code within test block
                    if in_unsafe_function:
                        unsafe_code += 1
                    continue
                
                # If we saw #[cfg(test)] but haven't entered the block yet, it's still test code
                if saw_cfg_test:
                    test_code += 1
                    continue
                
                # Regular source code
                source_code += 1
                # Also check if we're in unsafe function (unsafe blocks are handled above)
                if in_unsafe_function:
                    unsafe_code += 1
                
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=os.sys.stderr)
        return (0, 0, 0, 0, 0)
    
    return (total_non_empty, documentation, source_code, test_code, unsafe_code)


def count_test_file(file_path: Path) -> Tuple[int, int, int, int, int]:
    """Count lines in a test file (files in tests/ directories)."""
    total_non_empty = 0
    documentation = 0
    source_code = 0
    test_code = 0
    unsafe_code = 0
    in_multiline_doc = False
    in_unsafe_block = False
    unsafe_block_brace_depth = 0
    in_unsafe_function = False
    unsafe_function_brace_depth = 0
    saw_unsafe_keyword = False
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            for line in f:
                stripped = line.strip()
                
                # Skip empty lines
                if not is_non_empty(line):
                    continue
                
                total_non_empty += 1
                
                # Check for multi-line doc comment boundaries
                if '/**' in stripped or '/*!' in stripped:
                    in_multiline_doc = True
                    documentation += 1
                    continue
                if '*/' in stripped and in_multiline_doc:
                    in_multiline_doc = False
                    documentation += 1
                    continue
                
                # If inside multi-line doc comment, count as documentation
                if in_multiline_doc:
                    documentation += 1
                    continue
                
                # Check for single-line documentation
                if is_documentation_line(line):
                    documentation += 1
                    continue
                
                # Track brace depth
                open_braces = stripped.count('{')
                close_braces = stripped.count('}')
                
                # Check for unsafe keyword
                if re.search(r'\bunsafe\b', stripped):
                    saw_unsafe_keyword = True
                    # Check if it's an unsafe function/trait/impl
                    if re.search(r'\bunsafe\s+(fn|trait|impl)\b', stripped):
                        in_unsafe_function = True
                        unsafe_function_brace_depth = 0
                        unsafe_code += 1
                        saw_unsafe_keyword = False
                
                # Check for unsafe block start
                if saw_unsafe_keyword and open_braces > 0:
                    in_unsafe_block = True
                    unsafe_block_brace_depth = open_braces - close_braces
                    saw_unsafe_keyword = False
                    unsafe_code += 1
                
                # Track unsafe function braces
                if in_unsafe_function:
                    unsafe_function_brace_depth += open_braces - close_braces
                    unsafe_code += 1
                    if unsafe_function_brace_depth <= 0 and close_braces > 0:
                        in_unsafe_function = False
                
                # Track unsafe block braces
                if in_unsafe_block:
                    unsafe_block_brace_depth += open_braces - close_braces
                    unsafe_code += 1
                    if unsafe_block_brace_depth <= 0 and close_braces > 0:
                        in_unsafe_block = False
                    # Continue if in unsafe block (don't double count)
                    test_code += 1
                    continue
                
                # If we saw "unsafe" but haven't entered the block yet
                if saw_unsafe_keyword:
                    unsafe_code += 1
                    saw_unsafe_keyword = False
                
                # Everything else in a test file is test code
                test_code += 1
                # Also check if we're in unsafe function (unsafe blocks are handled above)
                if in_unsafe_function:
                    unsafe_code += 1
                
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=os.sys.stderr)
        return (0, 0, 0, 0, 0)
    
    return (total_non_empty, documentation, source_code, test_code, unsafe_code)


def find_rust_files(root_dir: Path) -> list[Path]:
    """Find all .rs files in the directory tree."""
    rust_files = []
    for path in root_dir.rglob('*.rs'):
        rust_files.append(path)
    return sorted(rust_files)


def main():
    """Main function to count lines in all Rust files."""
    # Find the rust-conversion/rust directory
    script_dir = Path(__file__).parent
    rust_dir = script_dir / 'rust-conversion' / 'rust'
    
    if not rust_dir.exists():
        print(f"Error: {rust_dir} does not exist", file=os.sys.stderr)
        os.sys.exit(1)
    
    rust_files = find_rust_files(rust_dir)
    
    if not rust_files:
        print("No .rs files found", file=os.sys.stderr)
        os.sys.exit(1)
    
    total_non_empty = 0
    total_documentation = 0
    total_source_code = 0
    total_test_code = 0
    total_unsafe_code = 0
    files_with_unsafe = []  # Track files with unsafe code
    
    for file_path in rust_files:
        non_empty, doc, source, test, unsafe = count_lines_in_file(file_path)
        total_non_empty += non_empty
        total_documentation += doc
        total_source_code += source
        total_test_code += test
        total_unsafe_code += unsafe
        if unsafe > 0:
            # Store relative path and unsafe count
            rel_path = file_path.relative_to(rust_dir)
            files_with_unsafe.append((rel_path, unsafe))
    
    # Print results
    print(f"Total non-empty lines: {total_non_empty}")
    print(f"Documentation lines: {total_documentation}")
    print(f"Source code lines: {total_source_code}")
    print(f"Test code lines: {total_test_code}")
    print(f"Unsafe code lines: {total_unsafe_code}")
    
    # Print modules with unsafe code
    if files_with_unsafe:
        print(f"\nModules with unsafe code ({len(files_with_unsafe)} files):")
        # Sort by unsafe count (descending), then by path
        files_with_unsafe.sort(key=lambda x: (-x[1], str(x[0])))
        for file_path, unsafe_count in files_with_unsafe:
            print(f"  {file_path}: {unsafe_count} unsafe lines")
    
    # Verify totals
    calculated_total = total_documentation + total_source_code + total_test_code
    if calculated_total != total_non_empty:
        print(f"\nWarning: Sum mismatch! Documentation + Source + Test = {calculated_total}, "
              f"but total non-empty = {total_non_empty}", file=os.sys.stderr)
        print(f"Difference: {abs(calculated_total - total_non_empty)} lines", file=os.sys.stderr)


if __name__ == '__main__':
    main()

