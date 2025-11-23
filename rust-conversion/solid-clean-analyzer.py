#!/usr/bin/env python3
"""
C Code Analyzer for SOLID/CLEAN architecture - focused on minimizing circular dependencies.
Analyzes C code structure with emphasis on dependency direction and architectural layers.
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple
from collections import defaultdict

class SOLIDCleanAnalyzer:
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.c_files: List[Path] = []
        self.h_files: List[Path] = []
        self.functions: Dict[str, Dict] = {}
        self.structs: Dict[str, Dict] = {}
        self.dependencies: Dict[str, Set[str]] = defaultdict(set)
        self.external_callers: List[Dict] = []
        self.file_dependencies: Dict[str, List[str]] = {}
        
    def find_c_files(self):
        """Find all C and header files in the project."""
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
    
    def extract_function_signature(self, line: str, file_path: Path, line_num: int) -> Optional[Dict]:
        """Extract function signature from a line."""
        pattern = r'^(\w+(?:\s+\*+|\s+\w+)*)\s+(\w+)\s*\([^)]*\)\s*\{?\s*$'
        match = re.match(pattern, line.strip())
        if match:
            return_type = match.group(1).strip()
            func_name = match.group(2).strip()
            
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
        match = re.search(r'\(([^)]*)\)', line)
        if match:
            params_str = match.group(1).strip()
            if not params_str or params_str == 'void':
                return []
            return [p.strip() for p in params_str.split(',') if p.strip()]
        return []
    
    def extract_includes(self, content: str, file_path: Path) -> Set[str]:
        """Extract #include directives."""
        includes = set()
        pattern = r'#include\s+[<"]([^>"]+)[>"]'
        matches = re.findall(pattern, content)
        
        for include in matches:
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
        
        for i, line in enumerate(lines, 1):
            func = self.extract_function_signature(line, file_path, i)
            if func:
                file_info['functions'].append(func)
                func_id = f"{func['file_path']}:{func['name']}"
                self.functions[func_id] = func
        
        return file_info
    
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
                    self.file_dependencies[rel_path] = list(file_info['includes'])
                    for include in file_info['includes']:
                        self.dependencies[rel_path].add(include)
                
                analyzed += 1
            except Exception as e:
                errors.append((str(c_file.relative_to(self.project_root)), str(e)))
        
        print(f"Phase: Analysis Mode - CAnalystActor: Analysis complete. Analyzed {analyzed} files, {len(errors)} errors")
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
            except Exception as e:
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
                'total_structs': len(self.structs),
                'total_dependencies': sum(len(deps) for deps in self.dependencies.values()),
                'external_callers_count': len(self.external_callers),
            },
            'functions': self.functions,
            'structs': self.structs,
            'dependencies': {k: list(v) for k, v in self.dependencies.items()},
            'file_dependencies': self.file_dependencies,
            'external_callers': self.external_callers
        }
        
        with open(output_file, 'w') as f:
            json.dump(results, f, indent=2)
        
        print(f"Results saved to {output_file}")

def main():
    project_root = '/Volumes/Files_1/iron-beam'
    analyzer = SOLIDCleanAnalyzer(project_root)
    
    analyzer.find_c_files()
    summary = analyzer.analyze_all_files()
    analyzer.identify_external_callers()
    
    output_dir = Path(project_root) / 'rust-conversion' / 'solid-clean-design'
    output_dir.mkdir(exist_ok=True)
    
    output_file = output_dir / 'c_analysis_results.json'
    analyzer.save_results(str(output_file))
    
    print("\n" + "=" * 80)
    print("ANALYSIS SUMMARY")
    print("=" * 80)
    print(f"Total C files: {summary['total_files']}")
    print(f"Analyzed files: {summary['analyzed_files']}")
    print(f"Total functions: {summary['total_functions']}")
    print(f"External callers: {len(analyzer.external_callers)}")
    print("=" * 80)

if __name__ == '__main__':
    main()

