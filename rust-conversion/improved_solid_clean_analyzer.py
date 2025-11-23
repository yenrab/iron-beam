#!/usr/bin/env python3
"""
Improved C Code Analyzer - captures transitive includes and macro dependencies.
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Set, Optional
from collections import defaultdict

class ImprovedSOLIDCleanAnalyzer:
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.c_files: List[Path] = []
        self.h_files: List[Path] = []
        self.functions: Dict[str, Dict] = {}
        self.structs: Dict[str, Dict] = {}
        self.dependencies: Dict[str, Set[str]] = defaultdict(set)
        self.transitive_dependencies: Dict[str, Set[str]] = defaultdict(set)
        self.external_callers: List[Dict] = []
        self.file_dependencies: Dict[str, List[str]] = {}
        self.header_map: Dict[str, str] = {}
        
    def find_c_files(self):
        """Find all C and header files."""
        print("Phase: Analysis Mode - CAnalystActor: Scanning directory for C files...")
        
        for root, dirs, files in os.walk(self.project_root):
            if any(skip in root for skip in ['.git', 'node_modules', 'obj', 'obj.debug', 'circular-design']):
                continue
                
            for file in files:
                if file.endswith('.c'):
                    self.c_files.append(Path(root) / file)
                elif file.endswith('.h'):
                    self.h_files.append(Path(root) / file)
        
        print(f"Phase: Analysis Mode - CAnalystActor: Found {len(self.c_files)} .c files and {len(self.h_files)} .h files")
        return len(self.c_files), len(self.h_files)
    
    def build_header_map(self):
        """Build comprehensive header file map."""
        print("Phase: Analysis Mode - CAnalystActor: Building header file map...")
        
        for h_file in self.h_files:
            rel_path = str(h_file.relative_to(self.project_root))
            basename = os.path.basename(rel_path)
            
            # Map by basename
            self.header_map[basename] = rel_path
            
            # Map by include path
            if 'include' in rel_path:
                parts = rel_path.split('/')
                try:
                    include_idx = parts.index('include')
                    if include_idx + 1 < len(parts):
                        include_name = '/'.join(parts[include_idx + 1:])
                        self.header_map[include_name] = rel_path
                except ValueError:
                    pass
            
            # Also map the full relative path
            self.header_map[rel_path] = rel_path
        
        print(f"Phase: Analysis Mode - CAnalystActor: Indexed {len(self.header_map)} header file mappings")
    
    def find_header_file(self, header_name: str) -> Optional[str]:
        """Find actual header file path."""
        header = header_name.strip('"<>')
        
        # Direct match
        if header in self.header_map:
            return self.header_map[header]
        
        # Basename match
        basename = os.path.basename(header)
        if basename in self.header_map:
            return self.header_map[basename]
        
        # Partial match
        for mapped_name, mapped_path in self.header_map.items():
            if header in mapped_name or mapped_name.endswith(header):
                return mapped_path
        
        return None
    
    def extract_includes(self, content: str) -> List[str]:
        """Extract #include directives."""
        includes = []
        pattern = r'#include\s+[<"]([^>"]+)[>"]'
        matches = re.findall(pattern, content)
        
        for include in matches:
            if not self._is_standard_header(include):
                includes.append(include)
        
        return includes
    
    def _is_standard_header(self, header: str) -> bool:
        """Check if header is standard C library."""
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
    
    def build_transitive_dependencies(self):
        """Build transitive dependency graph by following includes recursively."""
        print("Phase: Analysis Mode - DependencyAnalystActor: Building transitive dependency graph...")
        
        # Build direct dependencies first
        for file_path, includes in self.file_dependencies.items():
            for include in includes:
                header_file = self.find_header_file(include)
                if header_file:
                    self.dependencies[file_path].add(header_file)
        
        # Now build transitive dependencies by following header includes
        def get_transitive_deps(file_path: str, visited: Set[str] = None, depth: int = 0) -> Set[str]:
            """Recursively get all transitive dependencies."""
            if visited is None:
                visited = set()
            
            if depth > 15 or file_path in visited:  # Prevent infinite recursion
                return set()
            
            visited.add(file_path)
            all_deps = set()
            
            # Get direct dependencies
            direct_deps = self.dependencies.get(file_path, set())
            all_deps.update(direct_deps)
            
            # For each dependency, get its dependencies (if it's a header file)
            for dep_file in direct_deps:
                if dep_file.endswith('.h'):
                    # Get includes from this header file
                    if dep_file in self.file_dependencies:
                        header_includes = self.file_dependencies[dep_file]
                        for header_include in header_includes:
                            header_dep_file = self.find_header_file(header_include)
                            if header_dep_file and header_dep_file not in visited:
                                all_deps.add(header_dep_file)
                                # Recursively get dependencies
                                transitive = get_transitive_deps(header_dep_file, visited.copy(), depth + 1)
                                all_deps.update(transitive)
            
            return all_deps
        
        # Build transitive dependencies for all files
        for file_path in self.file_dependencies.keys():
            self.transitive_dependencies[file_path] = get_transitive_deps(file_path)
        
        total_transitive = sum(len(deps) for deps in self.transitive_dependencies.values())
        print(f"Phase: Analysis Mode - DependencyAnalystActor: Built transitive dependencies for {len(self.transitive_dependencies)} files")
        print(f"Phase: Analysis Mode - DependencyAnalystActor: Total transitive dependencies: {total_transitive}")
    
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
            'includes': self.extract_includes(content),
            'errors': []
        }
        
        # Extract functions
        for i, line in enumerate(lines, 1):
            pattern = r'^(\w+(?:\s+\*+|\s+\w+)*)\s+(\w+)\s*\([^)]*\)\s*\{?\s*$'
            match = re.match(pattern, line.strip())
            if match:
                return_type = match.group(1).strip()
                func_name = match.group(2).strip()
                
                if func_name.startswith('_') and len(func_name) > 1 and func_name[1].isupper():
                    continue
                
                func = {
                    'name': func_name,
                    'return_type': return_type,
                    'file_path': file_info['file_path'],
                    'line_start': i,
                    'is_static': 'static' in line
                }
                file_info['functions'].append(func)
                func_id = f"{func['file_path']}:{func_name}"
                self.functions[func_id] = func
        
        return file_info
    
    def analyze_all_files(self):
        """Analyze all C files."""
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
                    self.file_dependencies[rel_path] = file_info['includes']
                
                analyzed += 1
            except Exception as e:
                errors.append((str(c_file.relative_to(self.project_root)), str(e)))
        
        print(f"Phase: Analysis Mode - CAnalystActor: Analysis complete. Analyzed {analyzed} files, {len(errors)} errors")
        return {'total_files': len(self.c_files), 'analyzed_files': analyzed, 'errors': len(errors)}
    
    def identify_external_callers(self):
        """Identify where non-C code calls C code."""
        print("Phase: Analysis Mode - DependencyAnalystActor: Identifying external callers...")
        
        nif_pattern = r'ERL_NIF_INIT\s*\((\w+)'
        driver_pattern = r'driver_entry\s+(\w+)'
        
        for c_file in self.c_files:
            try:
                with open(c_file, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                
                rel_path = str(c_file.relative_to(self.project_root))
                
                nif_matches = re.findall(nif_pattern, content)
                if nif_matches:
                    for module_name in nif_matches:
                        self.external_callers.append({
                            'type': 'erlang_nif',
                            'module': module_name,
                            'c_file': rel_path,
                            'caller_language': 'erlang'
                        })
                
                driver_matches = re.findall(driver_pattern, content)
                if driver_matches:
                    for driver_name in driver_matches:
                        self.external_callers.append({
                            'type': 'erlang_driver',
                            'driver': driver_name,
                            'c_file': rel_path,
                            'caller_language': 'erlang'
                        })
            except Exception:
                pass
        
        print(f"Phase: Analysis Mode - DependencyAnalystActor: Found {len(self.external_callers)} external caller interfaces")
        return self.external_callers
    
    def save_results(self, output_file: str):
        """Save analysis results."""
        results = {
            'summary': {
                'total_c_files': len(self.c_files),
                'total_h_files': len(self.h_files),
                'total_functions': len(self.functions),
                'total_dependencies': sum(len(deps) for deps in self.dependencies.values()),
                'total_transitive_dependencies': sum(len(deps) for deps in self.transitive_dependencies.values()),
                'external_callers_count': len(self.external_callers),
            },
            'functions': self.functions,
            'structs': self.structs,
            'dependencies': {k: list(v) for k, v in self.dependencies.items()},
            'transitive_dependencies': {k: list(v) for k, v in self.transitive_dependencies.items()},
            'file_dependencies': self.file_dependencies,
            'external_callers': self.external_callers
        }
        
        with open(output_file, 'w') as f:
            json.dump(results, f, indent=2)
        
        print(f"Results saved to {output_file}")

def main():
    project_root = '/Volumes/Files_1/iron-beam'
    analyzer = ImprovedSOLIDCleanAnalyzer(project_root)
    
    analyzer.find_c_files()
    analyzer.build_header_map()
    summary = analyzer.analyze_all_files()
    analyzer.build_transitive_dependencies()
    analyzer.identify_external_callers()
    
    output_dir = Path(project_root) / 'rust-conversion' / 'solid-clean-design'
    output_dir.mkdir(exist_ok=True)
    
    output_file = output_dir / 'c_analysis_results.json'
    analyzer.save_results(str(output_file))
    
    print("\n" + "=" * 80)
    print("IMPROVED ANALYSIS SUMMARY")
    print("=" * 80)
    print(f"Total C files: {summary['total_files']}")
    print(f"Analyzed files: {summary['analyzed_files']}")
    print(f"Total functions: {len(analyzer.functions)}")
    print(f"Direct dependencies: {sum(len(deps) for deps in analyzer.dependencies.values())}")
    print(f"Transitive dependencies: {sum(len(deps) for deps in analyzer.transitive_dependencies.values())}")
    print(f"External callers: {len(analyzer.external_callers)}")
    print("=" * 80)

if __name__ == '__main__':
    main()

