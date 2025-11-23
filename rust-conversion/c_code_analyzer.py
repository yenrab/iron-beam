#!/usr/bin/env python3
"""
C Code Analyzer for behavior-grouper tool.
Extracts function signatures, data structures, dependencies, and external callers.
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple
from collections import defaultdict

class CCodeAnalyzer:
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.c_files: List[Path] = []
        self.h_files: List[Path] = []
        self.functions: Dict[str, Dict] = {}
        self.structs: Dict[str, Dict] = {}
        self.dependencies: Dict[str, Set[str]] = defaultdict(set)
        self.external_callers: List[Dict] = []
        
    def find_c_files(self):
        """Find all C and header files in the project."""
        print("Phase: Analysis Mode - CAnalystActor: Scanning directory for C files...")
        
        for root, dirs, files in os.walk(self.project_root):
            # Skip certain directories
            if any(skip in root for skip in ['.git', 'node_modules', 'obj', 'obj.debug']):
                continue
                
            for file in files:
                if file.endswith('.c'):
                    self.c_files.append(Path(root) / file)
                elif file.endswith('.h'):
                    self.h_files.append(Path(root) / file)
        
        print(f"Phase: Analysis Mode - CAnalystActor: Found {len(self.c_files)} .c files and {len(self.h_files)} .h files")
        return len(self.c_files), len(self.h_files)
    
    def extract_function_signature(self, line: str, file_path: Path, line_num: int) -> Optional[Dict]:
        """Extract function signature from a line."""
        # Match function definitions: return_type function_name(params)
        # Handle multi-line signatures
        pattern = r'^(\w+(?:\s+\*+|\s+\w+)*)\s+(\w+)\s*\([^)]*\)\s*\{?\s*$'
        match = re.match(pattern, line.strip())
        if match:
            return_type = match.group(1).strip()
            func_name = match.group(2).strip()
            
            # Skip if it's a macro or typedef
            if func_name.startswith('_') and len(func_name) > 1 and func_name[1].isupper():
                return None
            
            return {
                'name': func_name,
                'return_type': return_type,
                'file_path': str(file_path.relative_to(self.project_root)),
                'line_start': line_num,
                'is_static': 'static' in line,
                'parameters': self._extract_parameters(line)
            }
        return None
    
    def _extract_parameters(self, line: str) -> List[str]:
        """Extract parameter list from function signature."""
        # Simple extraction - find content between parentheses
        match = re.search(r'\(([^)]*)\)', line)
        if match:
            params_str = match.group(1).strip()
            if not params_str or params_str == 'void':
                return []
            # Split by comma, but be careful of function pointers
            params = []
            for param in params_str.split(','):
                param = param.strip()
                if param:
                    params.append(param)
            return params
        return []
    
    def extract_struct_definition(self, line: str, file_path: Path, line_num: int) -> Optional[Dict]:
        """Extract struct definition from a line."""
        # Match: struct struct_name { or typedef struct struct_name {
        pattern = r'(?:typedef\s+)?struct\s+(\w+)(?:\s+\w+)?\s*\{'
        match = re.search(pattern, line)
        if match:
            struct_name = match.group(1)
            return {
                'name': struct_name,
                'file_path': str(file_path.relative_to(self.project_root)),
                'line_start': line_num
            }
        return None
    
    def extract_includes(self, content: str, file_path: Path) -> Set[str]:
        """Extract #include directives."""
        includes = set()
        pattern = r'#include\s+[<"]([^>"]+)[>"]'
        matches = re.findall(pattern, content)
        
        for include in matches:
            # Check if it's a project header (not standard library)
            if not self._is_standard_header(include):
                includes.add(include)
        
        return includes
    
    def _is_standard_header(self, header: str) -> bool:
        """Check if header is a standard C library header."""
        standard_headers = {
            'stdio.h', 'stdlib.h', 'string.h', 'stddef.h', 'stdint.h',
            'stdbool.h', 'stdarg.h', 'ctype.h', 'errno.h', 'limits.h',
            'math.h', 'time.h', 'signal.h', 'assert.h', 'setjmp.h',
            'unistd.h', 'fcntl.h', 'pthread.h', 'sys/types.h', 'sys/stat.h',
            'sys/socket.h', 'sys/time.h', 'sys/wait.h', 'sys/mman.h',
            'sys/resource.h', 'sys/utsname.h', 'sys/un.h', 'sys/ioctl.h',
            'sys/select.h', 'netinet/in.h', 'arpa/inet.h', 'netdb.h',
            'dirent.h', 'termios.h', 'dlfcn.h', 'poll.h', 'semaphore.h'
        }
        return header in standard_headers or any(header.startswith(f'{s}/') for s in ['sys', 'linux', 'netinet', 'arpa'])
    
    def analyze_file(self, file_path: Path) -> Dict:
        """Analyze a single C file."""
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                lines = content.split('\n')
        except Exception as e:
            return {'error': str(e)}
        
        file_info = {
            'file_path': str(file_path.relative_to(self.project_root)),
            'functions': [],
            'structs': [],
            'includes': list(self.extract_includes(content, file_path)),
            'errors': []
        }
        
        # Extract functions
        for i, line in enumerate(lines, 1):
            # Check for function definition
            func = self.extract_function_signature(line, file_path, i)
            if func:
                file_info['functions'].append(func)
                func_id = f"{func['file_path']}:{func['name']}"
                self.functions[func_id] = func
            
            # Check for struct definition
            struct = self.extract_struct_definition(line, file_path, i)
            if struct:
                file_info['structs'].append(struct)
                struct_id = f"{struct['file_path']}:{struct['name']}"
                self.structs[struct_id] = struct
        
        # Extract function calls
        for func_id, func in self.functions.items():
            if func['file_path'] == file_info['file_path']:
                # Find function calls in this file
                calls = self._extract_function_calls(content, func['name'])
                if calls:
                    func['calls'] = calls
        
        return file_info
    
    def _extract_function_calls(self, content: str, func_name: str) -> List[str]:
        """Extract function calls from content."""
        # Simple pattern: function_name( or function_name (
        pattern = rf'\b{re.escape(func_name)}\s*\('
        matches = re.findall(pattern, content)
        return list(set(matches)) if matches else []
    
    def analyze_all_files(self):
        """Analyze all C files in the project."""
        print(f"Phase: Analysis Mode - CAnalystActor: Analyzing {len(self.c_files)} C files...")
        
        analyzed = 0
        errors = []
        
        for c_file in self.c_files:
            try:
                rel_path = str(c_file.relative_to(self.project_root))
                if analyzed % 50 == 0:
                    print(f"Phase: Analysis Mode - CAnalystActor: Analyzing {rel_path}... ({analyzed}/{len(self.c_files)})")
                
                file_info = self.analyze_file(c_file)
                if 'error' in file_info:
                    errors.append((rel_path, file_info['error']))
                else:
                    # Store dependencies
                    for include in file_info['includes']:
                        self.dependencies[rel_path].add(include)
                
                analyzed += 1
            except Exception as e:
                errors.append((str(c_file.relative_to(self.project_root)), str(e)))
        
        print(f"Phase: Analysis Mode - CAnalystActor: Analysis complete. Analyzed {analyzed} files, {len(errors)} errors")
        
        if errors:
            print(f"Phase: Analysis Mode - CAnalystActor: Errors encountered in {len(errors)} files")
            for file_path, error in errors[:10]:  # Show first 10 errors
                print(f"  Error in {file_path}: {error}")
            if len(errors) > 10:
                print(f"  ... and {len(errors) - 10} more errors")
        
        return {
            'total_files': len(self.c_files),
            'analyzed_files': analyzed,
            'total_functions': len(self.functions),
            'total_structs': len(self.structs),
            'errors': len(errors)
        }
    
    def identify_external_callers(self):
        """Identify where non-C code calls C code."""
        print("Phase: Analysis Mode - DependencyAnalystActor: Identifying external callers...")
        
        # Look for Erlang NIF exports, driver exports, port driver exports
        # These indicate Erlang code calling C code
        nif_pattern = r'ERL_NIF_INIT\s*\((\w+)'
        driver_pattern = r'driver_entry\s+(\w+)'
        
        for c_file in self.c_files:
            try:
                with open(c_file, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                
                rel_path = str(c_file.relative_to(self.project_root))
                
                # Check for NIF exports
                nif_matches = re.findall(nif_pattern, content)
                if nif_matches:
                    for module_name in nif_matches:
                        self.external_callers.append({
                            'type': 'erlang_nif',
                            'module': module_name,
                            'c_file': rel_path,
                            'caller_language': 'erlang'
                        })
                
                # Check for driver exports
                driver_matches = re.findall(driver_pattern, content)
                if driver_matches:
                    for driver_name in driver_matches:
                        self.external_callers.append({
                            'type': 'erlang_driver',
                            'driver': driver_name,
                            'c_file': rel_path,
                            'caller_language': 'erlang'
                        })
            except Exception as e:
                pass
        
        print(f"Phase: Analysis Mode - DependencyAnalystActor: Found {len(self.external_callers)} external caller interfaces")
        return self.external_callers
    
    def get_summary(self) -> Dict:
        """Get analysis summary."""
        return {
            'total_c_files': len(self.c_files),
            'total_h_files': len(self.h_files),
            'total_functions': len(self.functions),
            'total_structs': len(self.structs),
            'total_dependencies': sum(len(deps) for deps in self.dependencies.values()),
            'external_callers_count': len(self.external_callers),
            'functions_by_file': defaultdict(int),
            'structs_by_file': defaultdict(int)
        }

def main():
    project_root = '/Volumes/Files_1/iron-beam'
    analyzer = CCodeAnalyzer(project_root)
    
    # Find all C files
    analyzer.find_c_files()
    
    # Analyze all files
    summary = analyzer.analyze_all_files()
    
    # Identify external callers
    analyzer.identify_external_callers()
    
    # Print summary
    print("\n" + "=" * 80)
    print("ANALYSIS SUMMARY")
    print("=" * 80)
    print(f"Total C files: {summary['total_files']}")
    print(f"Analyzed files: {summary['analyzed_files']}")
    print(f"Total functions: {summary['total_functions']}")
    print(f"Total structs: {summary['total_structs']}")
    print(f"External callers: {len(analyzer.external_callers)}")
    print("=" * 80)
    
    # Save results
    output_dir = Path(project_root) / 'rust-conversion'
    output_dir.mkdir(exist_ok=True)
    
    results = {
        'summary': analyzer.get_summary(),
        'functions': analyzer.functions,
        'structs': analyzer.structs,
        'dependencies': {k: list(v) for k, v in analyzer.dependencies.items()},
        'external_callers': analyzer.external_callers
    }
    
    output_file = output_dir / 'c_analysis_results.json'
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\nResults saved to {output_file}")

if __name__ == '__main__':
    main()

