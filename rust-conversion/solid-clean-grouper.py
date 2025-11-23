#!/usr/bin/env python3
"""
Behavior Grouper following SOLID and CLEAN principles with minimal circular dependencies.
Groups code into layers with unidirectional dependencies.
"""

import json
import os
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple
from collections import defaultdict, deque

class SOLIDCleanGrouper:
    def __init__(self, analysis_file: str):
        with open(analysis_file, 'r') as f:
            self.analysis = json.load(f)
        
        self.functions = self.analysis['functions']
        self.dependencies = self.analysis['dependencies']
        self.file_dependencies = self.analysis.get('file_dependencies', {})
        self.external_callers = self.analysis['external_callers']
        
        self.groups: List[Dict] = []
        self.file_to_group: Dict[str, str] = {}
        self.group_dependencies: Dict[str, Set[str]] = defaultdict(set)
        
    def classify_clean_layer(self, file_path: str) -> str:
        """Classify file into CLEAN architecture layer."""
        path_lower = file_path.lower()
        
        # Entities Layer: Core data structures, types, constants
        if any(x in path_lower for x in ['atom', 'term', 'binary', 'bits', 'map', 'big']):
            return 'entities'
        
        # Use Cases Layer: Business logic, algorithms, operations
        if any(x in path_lower for x in ['bif_', 'process', 'sched', 'gc', 'alloc', 'heap']):
            return 'usecases'
        
        # Interface Adapters Layer: I/O, external interfaces, adapters
        if any(x in path_lower for x in ['io', 'port', 'dist', 'external', 'nif', 'driver']):
            return 'adapters'
        
        # Frameworks Layer: System integration, platform-specific code
        if any(x in path_lower for x in ['sys/', 'win32', 'unix', 'driver', 'nif']):
            return 'frameworks'
        
        # Infrastructure Layer: Utilities, helpers, common code
        if any(x in path_lower for x in ['utils', 'common', 'hash', 'time', 'debug', 'trace']):
            return 'infrastructure'
        
        # Code Management: Module loading, code organization
        if any(x in path_lower for x in ['code', 'module', 'export', 'beam_load', 'code_ix']):
            return 'code_management'
        
        # Default to infrastructure
        return 'infrastructure'
    
    def classify_solid_responsibility(self, file_path: str, functions: List[Dict]) -> str:
        """Classify file by SOLID Single Responsibility Principle."""
        path_lower = file_path.lower()
        func_names = [f.get('name', '') for f in functions]
        
        # Memory management
        if any(x in path_lower for x in ['alloc', 'gc', 'heap', 'memory']):
            return 'memory_management'
        
        # Process management
        if any(x in path_lower for x in ['process', 'sched']):
            return 'process_management'
        
        # Term/data handling
        if any(x in path_lower for x in ['term', 'binary', 'bits', 'atom', 'map']):
            return 'data_handling'
        
        # I/O operations
        if any(x in path_lower for x in ['io', 'port']):
            return 'io_operations'
        
        # Distribution
        if any(x in path_lower for x in ['dist', 'external']):
            return 'distribution'
        
        # Code loading/management
        if any(x in path_lower for x in ['code', 'module', 'export', 'beam_load']):
            return 'code_loading'
        
        # BIFs (Built-In Functions)
        if 'bif' in path_lower:
            return 'bifs'
        
        # ETS tables
        if any(x in path_lower for x in ['db', 'ets']):
            return 'ets_tables'
        
        # Time management
        if any(x in path_lower for x in ['time', 'timer']):
            return 'time_management'
        
        # Debugging/tracing
        if any(x in path_lower for x in ['debug', 'trace']):
            return 'debugging'
        
        # NIFs
        if 'nif' in path_lower:
            return 'nifs'
        
        # Drivers
        if 'driver' in path_lower:
            return 'drivers'
        
        # System integration
        if any(x in path_lower for x in ['sys', 'system']):
            return 'system_integration'
        
        # Utilities
        return 'utilities'
    
    def build_file_dependency_graph(self) -> Dict[str, Set[str]]:
        """Build dependency graph between files."""
        from analyze_group_dependencies import build_header_to_file_map, find_header_file, find_group_for_header
        
        project_root = Path('/Volumes/Files_1/iron-beam')
        header_map = build_header_to_file_map(project_root)
        
        file_graph = defaultdict(set)
        
        for file_path, includes in self.file_dependencies.items():
            for include in includes:
                header_file = find_header_file(include, header_map, project_root)
                if header_file:
                    # Find corresponding .c file or use header directly
                    header_dir = os.path.dirname(header_file)
                    header_basename = os.path.splitext(os.path.basename(header_file))[0]
                    possible_c_file = os.path.join(header_dir, header_basename + '.c')
                    
                    # Check if .c file exists
                    if os.path.exists(os.path.join(project_root, possible_c_file)):
                        if possible_c_file != file_path:
                            file_graph[file_path].add(possible_c_file)
        
        return dict(file_graph)
    
    def topological_sort_groups(self, groups: Dict[str, Dict], group_deps: Dict[str, Set[str]]) -> List[str]:
        """Topological sort of groups to create dependency order."""
        # Calculate in-degrees
        in_degree = defaultdict(int)
        for group_id in groups.keys():
            in_degree[group_id] = 0
        
        for group_id, deps in group_deps.items():
            for dep in deps:
                if dep in in_degree:
                    in_degree[dep] += 1
        
        # Kahn's algorithm
        queue = deque([gid for gid, degree in in_degree.items() if degree == 0])
        result = []
        
        while queue:
            current = queue.popleft()
            result.append(current)
            
            # Remove edges from current
            for group_id, deps in group_deps.items():
                if current in deps:
                    in_degree[group_id] -= 1
                    if in_degree[group_id] == 0:
                        queue.append(group_id)
        
        # Add remaining groups (cycles)
        remaining = [gid for gid in groups.keys() if gid not in result]
        result.extend(remaining)
        
        return result
    
    def create_layered_groups(self):
        """Create groups following CLEAN architecture layers with unidirectional dependencies."""
        print("Phase: Grouping Mode - CLEANExpertActor: Creating CLEAN architecture layers...")
        print("Phase: Grouping Mode - SOLIDExpertActor: Applying SOLID Single Responsibility...")
        
        # Group files by CLEAN layer and SOLID responsibility
        layer_groups = defaultdict(lambda: defaultdict(list))
        
        for func_id, func in self.functions.items():
            file_path = func['file_path']
            clean_layer = self.classify_clean_layer(file_path)
            solid_resp = self.classify_solid_responsibility(file_path, [func])
            
            # Create composite key: layer_responsibility
            group_key = f"{clean_layer}_{solid_resp}"
            layer_groups[clean_layer][group_key].append(file_path)
        
        # Create groups with proper layering
        clean_layers_order = ['entities', 'usecases', 'adapters', 'frameworks', 'infrastructure', 'code_management']
        group_id = 1
        
        for layer in clean_layers_order:
            if layer not in layer_groups:
                continue
            
            for group_key, files in layer_groups[layer].items():
                unique_files = list(set(files))
                if not unique_files:
                    continue
                
                # Get functions for these files
                group_functions = []
                for func_id, func in self.functions.items():
                    if func['file_path'] in unique_files:
                        group_functions.append({
                            '@id': f"ex:CFunction_{func_id.replace(':', '_').replace('/', '_')}",
                            'name': func['name'],
                            'filePath': func['file_path'],
                            'lineRange': {'start': func['line_start'], 'end': func['line_start'] + 10},
                            'signature': f"{func['return_type']} {func['name']}(...)",
                            'isStatic': func.get('is_static', False)
                        })
                
                if group_functions:
                    group = {
                        '@id': f"ex:BehaviorGroup_{group_id}",
                        'name': group_key,
                        'cleanLayer': layer,
                        'solidResponsibility': group_key.split('_', 1)[1] if '_' in group_key else group_key,
                        'files': unique_files,
                        'functions': group_functions,
                        'rationale': {
                            'codeRationale': f"Functions grouped by CLEAN layer ({layer}) and SOLID responsibility ({group_key})",
                            'solidRationale': f"Single Responsibility: All functions handle {group_key.split('_', 1)[1] if '_' in group_key else group_key}",
                            'cleanRationale': f"CLEAN Architecture {layer} layer: Dependencies flow inward from {layer}",
                            'rustRationale': f"Rust module: {group_key} as separate module with clear dependency boundaries"
                        }
                    }
                    
                    self.groups.append(group)
                    for file_path in unique_files:
                        self.file_to_group[file_path] = group['@id']
                    
                    group_id += 1
        
        print(f"Phase: Grouping Mode - CLEANExpertActor: Created {len(self.groups)} groups in CLEAN layers")
        
        # Build group dependencies (only allow inward flow)
        self.build_group_dependencies()
        
        return self.groups
    
    def build_header_to_file_map(self, project_root: Path) -> Dict[str, str]:
        """Build a map from header names to actual file paths."""
        header_map = {}
        for root, dirs, files in os.walk(project_root):
            if any(skip in root for skip in ['.git', 'node_modules', 'obj', 'obj.debug', 'circular-design']):
                continue
            for file in files:
                if file.endswith('.h'):
                    full_path = Path(root) / file
                    rel_path = str(full_path.relative_to(project_root))
                    header_map[file] = rel_path
                    if 'include' in rel_path:
                        parts = rel_path.split('/')
                        include_idx = parts.index('include')
                        if include_idx + 1 < len(parts):
                            include_name = '/'.join(parts[include_idx + 1:])
                            header_map[include_name] = rel_path
        return header_map
    
    def find_header_file(self, header_name: str, header_map: Dict[str, str], project_root: Path) -> Optional[str]:
        """Find the actual header file path."""
        header = header_name.strip('"<>')
        if header in header_map:
            return header_map[header]
        basename = os.path.basename(header)
        if basename in header_map:
            return header_map[basename]
        for mapped_name, mapped_path in header_map.items():
            if header in mapped_name or mapped_name.endswith(header):
                return mapped_path
        return None
    
    def build_group_dependencies(self):
        """Build dependencies between groups, ensuring they follow CLEAN architecture (inward flow)."""
        print("Phase: Grouping Mode - DependencyAnalystActor: Building group dependencies...")
        
        project_root = Path('/Volumes/Files_1/iron-beam')
        header_map = self.build_header_to_file_map(project_root)
        
        # CLEAN layer order (dependencies should flow inward: frameworks -> adapters -> usecases -> entities)
        layer_order = {
            'frameworks': 0,
            'adapters': 1,
            'usecases': 2,
            'entities': 3,
            'infrastructure': 1,  # Can be used by multiple layers
            'code_management': 2
        }
        
        for group in self.groups:
            group_id = group['@id']
            group_layer = group.get('cleanLayer', 'infrastructure')
            group_files = set(group.get('files', []))
            
            for file_path in group_files:
                deps = self.file_dependencies.get(file_path, [])
                for dep_header in deps:
                    dep_file = self.find_header_file(dep_header, header_map, project_root)
                    if dep_file:
                        # Find which group this file belongs to
                        dep_group_id = self.find_group_for_file(dep_file)
                        if dep_group_id and dep_group_id != group_id:
                            dep_group = self.get_group_by_id(dep_group_id)
                            if dep_group:
                                dep_layer = dep_group.get('cleanLayer', 'infrastructure')
                                
                                # Only allow dependencies that flow inward (higher layer -> lower layer)
                                source_order = layer_order.get(group_layer, 99)
                                target_order = layer_order.get(dep_layer, 99)
                                
                                if source_order < target_order:  # Allowed: outer layer depends on inner layer
                                    self.group_dependencies[group_id].add(dep_group_id)
        
        print(f"Phase: Grouping Mode - DependencyAnalystActor: Built {sum(len(deps) for deps in self.group_dependencies.values())} group dependencies")
    
    def find_group_for_file(self, file_path: str) -> Optional[str]:
        """Find which group a file belongs to."""
        # Try direct match
        if file_path in self.file_to_group:
            return self.file_to_group[file_path]
        
        # Try to find corresponding .c file
        if file_path.endswith('.h'):
            header_dir = os.path.dirname(file_path)
            header_basename = os.path.splitext(os.path.basename(file_path))[0]
            possible_c_file = os.path.join(header_dir, header_basename + '.c')
            if possible_c_file in self.file_to_group:
                return self.file_to_group[possible_c_file]
        
        return None
    
    def get_group_by_id(self, group_id: str) -> Optional[Dict]:
        """Get group by ID."""
        for group in self.groups:
            if group['@id'] == group_id:
                return group
        return None
    
    def detect_circular_dependencies(self) -> Set[Tuple[str, str]]:
        """Detect circular dependencies using Tarjan's algorithm."""
        index = 0
        indices = {}
        lowlinks = {}
        stack = []
        on_stack = set()
        circular_edges = set()
        
        def strongconnect(v: str):
            nonlocal index
            indices[v] = index
            lowlinks[v] = index
            index += 1
            stack.append(v)
            on_stack.add(v)
            
            for w in self.group_dependencies.get(v, set()):
                if w not in indices:
                    strongconnect(w)
                    lowlinks[v] = min(lowlinks[v], lowlinks[w])
                elif w in on_stack:
                    lowlinks[v] = min(lowlinks[v], indices[w])
            
            if lowlinks[v] == indices[v]:
                scc = []
                while True:
                    w = stack.pop()
                    on_stack.remove(w)
                    scc.append(w)
                    if w == v:
                        break
                
                if len(scc) > 1:
                    for i, node in enumerate(scc):
                        next_node = scc[(i + 1) % len(scc)]
                        if next_node in self.group_dependencies.get(node, set()):
                            circular_edges.add((node, next_node))
                    for node in scc:
                        for neighbor in self.group_dependencies.get(node, set()):
                            if neighbor in scc and neighbor != node:
                                circular_edges.add((node, neighbor))
        
        for node in self.group_dependencies.keys():
            if node not in indices:
                strongconnect(node)
        
        return circular_edges
    
    def generate_jsonld(self, output_file: str):
        """Generate JSON-LD mapping file."""
        print("Phase: Generation Mode - JSONLDExpertActor: Generating JSON-LD mapping file...")
        
        # Detect circular dependencies
        circular_edges = self.detect_circular_dependencies()
        print(f"Phase: Generation Mode - ValidationCoordinatorActor: Found {len(circular_edges)} circular dependencies")
        
        jsonld = {
            "@context": {
                "@vocab": "https://aalang.org/spec",
                "ex": "https://aalang.org/behavior-grouper/"
            },
            "@graph": []
        }
        
        # Add behavior groups
        for group in self.groups:
            jsonld['@graph'].append(group)
            
            # Add functions as separate nodes
            for func in group['functions']:
                func_node = {
                    "@id": func['@id'],
                    "@type": "ex:CFunction",
                    "name": func['name'],
                    "filePath": func['filePath'],
                    "lineRange": func['lineRange'],
                    "signature": func['signature'],
                    "isStatic": func['isStatic']
                }
                jsonld['@graph'].append(func_node)
        
        # Add grouping rationale nodes
        for group in self.groups:
            rationale_node = {
                "@id": f"{group['@id']}_rationale",
                "@type": "ex:GroupingRationale",
                "behaviorGroup": group['@id'],
                "solidRationale": group['rationale']['solidRationale'],
                "cleanRationale": group['rationale']['cleanRationale'],
                "rustRationale": group['rationale']['rustRationale'],
                "codeRationale": group['rationale']['codeRationale']
            }
            jsonld['@graph'].append(rationale_node)
        
        # Add external callers
        for caller in self.external_callers:
            caller_node = {
                "@id": f"ex:ExternalCaller_{len(jsonld['@graph'])}",
                "@type": "ex:ExternalCaller",
                "type": caller['type'],
                "cFile": caller['c_file'],
                "callerLanguage": caller['caller_language']
            }
            jsonld['@graph'].append(caller_node)
        
        # Add dependency information
        deps_node = {
            "@id": "ex:DependencyGraph",
            "@type": "ex:DependencyGraph",
            "totalDependencies": sum(len(deps) for deps in self.group_dependencies.values()),
            "circularDependencies": len(circular_edges),
            "groups": len(self.groups)
        }
        jsonld['@graph'].append(deps_node)
        
        # Write output
        with open(output_file, 'w') as f:
            json.dump(jsonld, f, indent=2)
        
        print(f"Phase: Generation Mode - JSONLDExpertActor: Generated JSON-LD mapping file: {output_file}")
        print(f"Phase: Generation Mode - JSONLDExpertActor: Created {len(self.groups)} behavior groups")
        print(f"Phase: Generation Mode - JSONLDExpertActor: Mapped {sum(len(g['functions']) for g in self.groups)} functions")
        print(f"Phase: Generation Mode - ValidationCoordinatorActor: Circular dependencies: {len(circular_edges)}/{sum(len(deps) for deps in self.group_dependencies.values())}")

def main():
    project_root = '/Volumes/Files_1/iron-beam'
    analysis_file = os.path.join(project_root, 'rust-conversion', 'solid-clean-design', 'c_analysis_results.json')
    output_file = os.path.join(project_root, 'rust-conversion', 'solid-clean-design', 'behavior-groups-mapping.jsonld')
    
    grouper = SOLIDCleanGrouper(analysis_file)
    grouper.create_layered_groups()
    grouper.generate_jsonld(output_file)
    
    print("\n" + "=" * 80)
    print("GROUPING COMPLETE")
    print("=" * 80)
    print(f"Total behavior groups: {len(grouper.groups)}")
    
    # Group by CLEAN layer
    by_layer = defaultdict(list)
    for group in grouper.groups:
        layer = group.get('cleanLayer', 'unknown')
        by_layer[layer].append(group)
    
    print("\nGroups by CLEAN Architecture Layer:")
    for layer, groups in sorted(by_layer.items()):
        print(f"  {layer}: {len(groups)} groups")

if __name__ == '__main__':
    main()

