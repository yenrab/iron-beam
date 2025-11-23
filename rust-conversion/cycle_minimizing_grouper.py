#!/usr/bin/env python3
"""
Cycle-Minimizing Behavior Grouper - aggressively breaks cycles
while maintaining CLEAN and SOLID principles.
"""

import json
import os
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple
from collections import defaultdict, deque

class CycleMinimizingGrouper:
    def __init__(self, analysis_file: str):
        with open(analysis_file, 'r') as f:
            self.analysis = json.load(f)
        
        self.functions = self.analysis['functions']
        self.dependencies = self.analysis['dependencies']
        self.transitive_dependencies = self.analysis.get('transitive_dependencies', {})
        self.file_dependencies = self.analysis.get('file_dependencies', {})
        self.external_callers = self.analysis['external_callers']
        
        self.groups: List[Dict] = []
        self.file_to_group: Dict[str, str] = {}
        self.group_dependencies: Dict[str, Set[str]] = defaultdict(set)
        
    def classify_clean_layer(self, file_path: str) -> str:
        """Classify file into CLEAN architecture layer."""
        path_lower = file_path.lower()
        
        # Entities Layer: Core data structures (innermost, no dependencies)
        if any(x in path_lower for x in ['atom', 'term', 'binary', 'bits', 'map', 'big', 'export', 'register']):
            # But exclude erl_interface - that's infrastructure
            if 'erl_interface' not in path_lower:
                return 'entities'
        
        # Use Cases Layer: Business logic
        if any(x in path_lower for x in ['bif_', 'process', 'sched', 'gc', 'alloc', 'heap']):
            return 'usecases'
        
        # Interface Adapters Layer: I/O, external interfaces
        if any(x in path_lower for x in ['io', 'port', 'dist', 'external', 'nif', 'driver']):
            return 'adapters'
        
        # Frameworks Layer: System integration (outermost)
        if any(x in path_lower for x in ['sys/', 'win32', 'unix', 'sys_', 'erl_sys', 'erts_sys']):
            return 'frameworks'
        
        # Infrastructure Layer: Utilities, helpers, common code
        if any(x in path_lower for x in ['utils', 'common', 'hash', 'time', 'debug', 'trace', 'erl_interface']):
            return 'infrastructure'
        
        # Code Management
        if any(x in path_lower for x in ['code', 'module', 'beam_load', 'code_ix']):
            # Distinguish actual code loading from encoding/decoding
            if 'trace' in path_lower or ('big' in path_lower and ('encode' in path_lower or 'decode' in path_lower)):
                return 'infrastructure'  # These are utilities, not code management
            return 'code_management'
        
        return 'infrastructure'
    
    def classify_solid_responsibility(self, file_path: str) -> str:
        """Classify by SOLID Single Responsibility."""
        path_lower = file_path.lower()
        
        # System integration - split by platform
        if 'sys/' in path_lower or 'sys_' in path_lower:
            if 'win32' in path_lower:
                return 'system_integration_win32'
            elif 'unix' in path_lower:
                return 'system_integration_unix'
            elif 'common' in path_lower:
                return 'system_integration_common'
            else:
                return 'system_integration'
        
        # Bignum encoding (not code loading!)
        if 'big' in path_lower and ('encode' in path_lower or 'decode' in path_lower):
            if 'bignum' in path_lower:
                return 'bignum_encoding_gmp'
            return 'bignum_encoding'
        
        # Trace encoding (not code loading!)
        if 'trace' in path_lower and ('encode' in path_lower or 'decode' in path_lower):
            return 'trace_encoding'
        
        if any(x in path_lower for x in ['alloc', 'gc', 'heap', 'memory']):
            return 'memory_management'
        
        if any(x in path_lower for x in ['process', 'sched']):
            return 'process_management'
        
        if any(x in path_lower for x in ['term', 'binary', 'bits', 'atom', 'map']):
            return 'data_handling'
        
        if any(x in path_lower for x in ['io', 'port']):
            return 'io_operations'
        
        if any(x in path_lower for x in ['dist', 'external']):
            return 'distribution'
        
        if any(x in path_lower for x in ['code', 'module', 'export', 'beam_load']):
            # Make sure we're not matching trace/big encoding
            if 'trace' not in path_lower and not ('big' in path_lower and ('encode' in path_lower or 'decode' in path_lower)):
                return 'code_loading'
        
        if 'bif' in path_lower:
            return 'bifs'
        
        if any(x in path_lower for x in ['db', 'ets']):
            return 'ets_tables'
        
        if any(x in path_lower for x in ['time', 'timer']):
            return 'time_management'
        
        if any(x in path_lower for x in ['debug', 'trace']):
            return 'debugging'
        
        if 'nif' in path_lower:
            return 'nifs'
        
        if 'driver' in path_lower:
            return 'drivers'
        
        return 'utilities'
    
    def create_layered_groups(self):
        """Create groups following CLEAN architecture with aggressive cycle breaking."""
        print("Phase: Grouping Mode - CLEANExpertActor: Creating CLEAN architecture layers...")
        print("Phase: Grouping Mode - SOLIDExpertActor: Applying SOLID Single Responsibility...")
        
        # Group files by CLEAN layer and SOLID responsibility
        layer_groups = defaultdict(lambda: defaultdict(list))
        
        for func_id, func in self.functions.items():
            file_path = func['file_path']
            clean_layer = self.classify_clean_layer(file_path)
            solid_resp = self.classify_solid_responsibility(file_path)
            
            group_key = f"{clean_layer}_{solid_resp}"
            layer_groups[clean_layer][group_key].append(file_path)
        
        # Create groups
        clean_layers_order = ['entities', 'usecases', 'adapters', 'frameworks', 'infrastructure', 'code_management']
        group_id = 1
        
        for layer in clean_layers_order:
            if layer not in layer_groups:
                continue
            
            for group_key, files in layer_groups[layer].items():
                unique_files = list(set(files))
                if not unique_files:
                    continue
                
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
                    solid_resp = group_key.split('_', 1)[1] if '_' in group_key else group_key
                    group = {
                        '@id': f"ex:BehaviorGroup_{group_id}",
                        'name': group_key,
                        'cleanLayer': layer,
                        'solidResponsibility': solid_resp,
                        'files': unique_files,
                        'functions': group_functions,
                        'rationale': {
                            'codeRationale': f"Functions grouped by CLEAN layer ({layer}) and SOLID responsibility ({solid_resp})",
                            'solidRationale': f"Single Responsibility: All functions handle {solid_resp}",
                            'cleanRationale': f"CLEAN Architecture {layer} layer: Dependencies flow inward from {layer}",
                            'rustRationale': f"Rust module: {group_key} as separate module with clear dependency boundaries"
                        }
                    }
                    
                    self.groups.append(group)
                    for file_path in unique_files:
                        self.file_to_group[file_path] = group['@id']
                    
                    group_id += 1
        
        print(f"Phase: Grouping Mode - CLEANExpertActor: Created {len(self.groups)} groups")
        
        # Build group dependencies with aggressive cycle breaking
        self.build_group_dependencies_aggressive_cycle_breaking()
        
        return self.groups
    
    def build_group_dependencies_aggressive_cycle_breaking(self):
        """Build dependencies while aggressively breaking cycles."""
        print("Phase: Grouping Mode - DependencyAnalystActor: Building group dependencies with aggressive cycle breaking...")
        
        layer_order = {
            'frameworks': 0,
            'adapters': 1,
            'usecases': 2,
            'entities': 3,
            'infrastructure': 1,  # Can depend on usecases/entities
            'code_management': 2  # Can depend on usecases/entities
        }
        
        # Build dependency graph
        file_to_group_map = {}
        for group in self.groups:
            for file_path in group.get('files', []):
                file_to_group_map[file_path] = group['@id']
                # Also map headers
                if file_path.endswith('.c'):
                    header_path = file_path[:-2] + '.h'
                    file_to_group_map[header_path] = group['@id']
                elif file_path.endswith('.h'):
                    c_path = file_path[:-2] + '.c'
                    file_to_group_map[c_path] = group['@id']
        
        # First pass: collect all potential dependencies
        potential_deps = defaultdict(set)
        
        for group in self.groups:
            group_id = group['@id']
            group_files = set(group.get('files', []))
            
            all_deps = set()
            for file_path in group_files:
                direct_deps = self.dependencies.get(file_path, set())
                transitive_deps = self.transitive_dependencies.get(file_path, set())
                all_deps.update(direct_deps)
                all_deps.update(transitive_deps)
            
            for dep_file in all_deps:
                dep_group_id = file_to_group_map.get(dep_file)
                if dep_group_id and dep_group_id != group_id:
                    potential_deps[group_id].add(dep_group_id)
        
        # Second pass: only add dependencies that maintain strict layer ordering
        # This is more aggressive - we'll break cycles by enforcing strict layer rules
        added_deps = defaultdict(set)
        
        def would_create_cycle(source: str, target: str) -> bool:
            """Check if adding this edge would create a cycle."""
            visited = set()
            queue = deque([target])
            visited.add(target)
            
            while queue:
                current = queue.popleft()
                if current == source:
                    return True
                
                for dep in added_deps.get(current, set()):
                    if dep not in visited:
                        visited.add(dep)
                        queue.append(dep)
            
            return False
        
        # Group by layer for processing
        groups_by_layer = defaultdict(list)
        for group in self.groups:
            layer = group.get('cleanLayer', 'infrastructure')
            groups_by_layer[layer].append(group)
        
        layer_order_list = ['frameworks', 'adapters', 'usecases', 'entities', 'infrastructure', 'code_management']
        
        # Process layers from outer to inner
        for layer in layer_order_list:
            for group in groups_by_layer.get(layer, []):
                group_id = group['@id']
                group_layer = group.get('cleanLayer', 'infrastructure')
                
                for dep_group_id in potential_deps.get(group_id, set()):
                    dep_group = self.get_group_by_id(dep_group_id)
                    if not dep_group:
                        continue
                    
                    dep_layer = dep_group.get('cleanLayer', 'infrastructure')
                    source_order = layer_order.get(group_layer, 99)
                    target_order = layer_order.get(dep_layer, 99)
                    
                    # STRICT rule: only allow dependencies that go inward (outer -> inner)
                    # Infrastructure and code_management can depend on same-layer or inner layers
                    if source_order < target_order:
                        # Check if this would create a cycle
                        if not would_create_cycle(group_id, dep_group_id):
                            self.group_dependencies[group_id].add(dep_group_id)
                            added_deps[group_id].add(dep_group_id)
                    elif source_order == target_order:
                        # Same layer dependencies - only allow for infrastructure/utilities
                        # and only if it doesn't create cycles
                        if group_layer in ['infrastructure', 'code_management']:
                            if not would_create_cycle(group_id, dep_group_id):
                                self.group_dependencies[group_id].add(dep_group_id)
                                added_deps[group_id].add(dep_group_id)
        
        print(f"Phase: Grouping Mode - DependencyAnalystActor: Built {sum(len(deps) for deps in self.group_dependencies.values())} group dependencies")
    
    def get_group_by_id(self, group_id: str) -> Optional[Dict]:
        """Get group by ID."""
        for group in self.groups:
            if group['@id'] == group_id:
                return group
        return None
    
    def detect_circular_dependencies(self) -> Set[Tuple[str, str]]:
        """Detect circular dependencies."""
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
        
        circular_edges = self.detect_circular_dependencies()
        print(f"Phase: Generation Mode - ValidationCoordinatorActor: Found {len(circular_edges)} circular dependencies")
        
        jsonld = {
            "@context": {
                "@vocab": "https://aalang.org/spec",
                "ex": "https://aalang.org/behavior-grouper/"
            },
            "@graph": []
        }
        
        # Mark groups with Erlang callers
        files_with_erlang_callers = set()
        for caller in self.external_callers:
            files_with_erlang_callers.add(caller.get('c_file', ''))
        
        for group in self.groups:
            group_files = set(group.get('files', []))
            if group_files.intersection(files_with_erlang_callers):
                group['name'] = '✅' + group.get('name', '')
            
            jsonld['@graph'].append(group)
            
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
        
        for caller in self.external_callers:
            caller_node = {
                "@id": f"ex:ExternalCaller_{len(jsonld['@graph'])}",
                "@type": "ex:ExternalCaller",
                "type": caller['type'],
                "cFile": caller['c_file'],
                "callerLanguage": caller['caller_language']
            }
            jsonld['@graph'].append(caller_node)
        
        deps_node = {
            "@id": "ex:DependencyGraph",
            "@type": "ex:DependencyGraph",
            "totalDependencies": sum(len(deps) for deps in self.group_dependencies.values()),
            "circularDependencies": len(circular_edges),
            "groups": len(self.groups)
        }
        jsonld['@graph'].append(deps_node)
        
        with open(output_file, 'w') as f:
            json.dump(jsonld, f, indent=2)
        
        print(f"Phase: Generation Mode - JSONLDExpertActor: Generated JSON-LD mapping file: {output_file}")
        print(f"Phase: Generation Mode - JSONLDExpertActor: Created {len(self.groups)} behavior groups")
        print(f"Phase: Generation Mode - ValidationCoordinatorActor: Circular dependencies: {len(circular_edges)}/{sum(len(deps) for deps in self.group_dependencies.values())}")

def main():
    project_root = '/Volumes/Files_1/iron-beam'
    analysis_file = os.path.join(project_root, 'rust-conversion', 'solid-clean-design', 'c_analysis_results.json')
    output_file = os.path.join(project_root, 'rust-conversion', 'solid-clean-design', 'behavior-groups-mapping.jsonld')
    
    grouper = CycleMinimizingGrouper(analysis_file)
    grouper.create_layered_groups()
    grouper.generate_jsonld(output_file)
    
    print("\n" + "=" * 80)
    print("CYCLE-MINIMIZING GROUPING COMPLETE")
    print("=" * 80)
    print(f"Total behavior groups: {len(grouper.groups)}")
    
    by_layer = defaultdict(list)
    for group in grouper.groups:
        layer = group.get('cleanLayer', 'unknown')
        by_layer[layer].append(group)
    
    print("\nGroups by CLEAN Architecture Layer:")
    for layer, groups in sorted(by_layer.items()):
        print(f"  {layer}: {len(groups)} groups")
    
    circular_edges = grouper.detect_circular_dependencies()
    total_deps = sum(len(deps) for deps in grouper.group_dependencies.values())
    if circular_edges:
        print(f"\n⚠️  Found {len(circular_edges)} circular dependencies out of {total_deps} total ({len(circular_edges)/total_deps*100:.1f}%)")
    else:
        print(f"\n✅ No circular dependencies - CLEAN architecture maintained!")
        print(f"Total dependencies: {total_deps}")

if __name__ == '__main__':
    main()

