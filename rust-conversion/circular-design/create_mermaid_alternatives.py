#!/usr/bin/env python3
"""
Create alternative Mermaid diagram versions:
1. Top-down (TD) layout for hierarchy
2. Grouped by architectural layers
3. Direct dependencies only (no transitive)
"""

import json
from pathlib import Path
from collections import defaultdict, deque
from typing import Dict, List, Set, Tuple

def load_behavior_groups(mapping_file: str) -> Tuple[Dict[str, Dict], Dict[str, str]]:
    """Load behavior groups from JSON-LD mapping."""
    with open(mapping_file, 'r') as f:
        data = json.load(f)
    
    groups = {}
    file_to_group = {}
    
    for node in data['@graph']:
        if '@id' in node and 'BehaviorGroup' in node.get('@id', ''):
            group_id = node['@id']
            groups[group_id] = node
            for file_path in node.get('files', []):
                file_to_group[file_path] = group_id
    
    return groups, file_to_group

def load_dependencies(analysis_file: str, groups: Dict[str, Dict], file_to_group: Dict[str, str], 
                     header_map: Dict[str, str], project_root: Path) -> Dict[str, Set[str]]:
    """Load and analyze group dependencies."""
    from analyze_group_dependencies import load_analysis_results, build_header_to_file_map, find_header_file, find_group_for_header
    
    analysis_results = load_analysis_results(analysis_file)
    file_dependencies = analysis_results.get('dependencies', {})
    group_deps = defaultdict(set)
    
    for group_id, group in groups.items():
        group_files = set(group.get('files', []))
        for file_path in group_files:
            deps = file_dependencies.get(file_path, [])
            for dep_header in deps:
                dep_file = find_header_file(dep_header, header_map, project_root)
                if dep_file:
                    dep_group = find_group_for_header(dep_file, file_to_group, project_root)
                    if dep_group and dep_group != group_id:
                        group_deps[group_id].add(dep_group)
    
    return dict(group_deps)

def classify_architectural_layers(groups: Dict[str, Dict]) -> Dict[str, str]:
    """Classify groups into architectural layers."""
    layer_map = {}
    
    # Foundational layer - core data structures and memory
    foundational = {'term_handling', 'memory_management', 'maps'}
    
    # Core layer - essential runtime components
    core = {'process_management', 'scheduling', 'code_management', 'bifs'}
    
    # Service layer - higher-level services
    service = {'io_ports', 'distribution', 'ets_tables', 'time_management', 'debugging_tracing'}
    
    # Integration layer - external interfaces
    integration = {'nifs', 'drivers', 'system_integration'}
    
    # Utility layer - supporting utilities
    utility = {'utils'}
    
    for group_id, group in groups.items():
        name = group.get('name', '')
        if name in foundational:
            layer_map[group_id] = 'Foundational'
        elif name in core:
            layer_map[group_id] = 'Core'
        elif name in service:
            layer_map[group_id] = 'Service'
        elif name in integration:
            layer_map[group_id] = 'Integration'
        elif name in utility:
            layer_map[group_id] = 'Utility'
        else:
            layer_map[group_id] = 'Other'
    
    return layer_map

def remove_transitive_dependencies(group_deps: Dict[str, Set[str]]) -> Dict[str, Set[str]]:
    """Remove transitive dependencies - keep only direct ones.
    
    A dependency A -> C is transitive if there exists a path A -> B -> C
    where B is also a direct dependency of A.
    """
    direct_deps = {}
    
    for group_id, deps in group_deps.items():
        direct = set()
        # For each dependency, check if it's transitive
        for dep in deps:
            is_direct = True
            # Check if this dependency can be reached through any other dependency
            # by doing a BFS to see if we can reach 'dep' through any intermediate node
            for other_dep in deps:
                if other_dep == dep:
                    continue
                # Check if we can reach 'dep' from 'other_dep' (transitive closure)
                visited = set()
                queue = deque([other_dep])
                visited.add(other_dep)
                
                while queue:
                    current = queue.popleft()
                    if current == dep:
                        # Found a path: group_id -> other_dep -> ... -> dep
                        is_direct = False
                        break
                    # Add all dependencies of current to queue
                    for next_dep in group_deps.get(current, set()):
                        if next_dep not in visited:
                            visited.add(next_dep)
                            queue.append(next_dep)
                
                if not is_direct:
                    break
            
            if is_direct:
                direct.add(dep)
        direct_deps[group_id] = direct
    
    return direct_deps

def create_td_layout_diagram(groups: Dict[str, Dict], group_deps: Dict[str, Set[str]], output_file: str):
    """Create top-down layout diagram."""
    active_groups = {gid: g for gid, g in groups.items() if len(g.get('functions', [])) > 0}
    
    group_names = {}
    for group_id, group in active_groups.items():
        name = group.get('name', group_id.split('_')[-1])
        if name == 'unknown':
            continue
        clean_name = name.replace('_', ' ').title().replace(' ', '')
        if clean_name in group_names.values():
            clean_name = f"{clean_name}{group_id.split('_')[-1]}"
        group_names[group_id] = clean_name
    
    mermaid = ["graph TD"]
    
    # Add nodes
    for group_id, group in active_groups.items():
        name = group.get('name', 'unknown')
        if name == 'unknown' or group_id not in group_names:
            continue
        clean_id = group_names[group_id]
        function_count = len(group.get('functions', []))
        file_count = len(group.get('files', []))
        label = f"{name.replace('_', ' ').title()}<br/>{function_count} funcs<br/>{file_count} files"
        mermaid.append(f"    {clean_id}[\"{label}\"]")
    
    # Add edges
    for group_id, deps in group_deps.items():
        if group_id not in group_names:
            continue
        source = group_names[group_id]
        for dep_group_id in deps:
            if dep_group_id in group_names:
                target = group_names[dep_group_id]
                mermaid.append(f"    {source} --> {target}")
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mermaid))
    
    print(f"Top-down diagram written to {output_file}")

def create_layered_diagram(groups: Dict[str, Dict], group_deps: Dict[str, Set[str]], 
                          layer_map: Dict[str, str], output_file: str):
    """Create diagram grouped by architectural layers."""
    active_groups = {gid: g for gid, g in groups.items() if len(g.get('functions', [])) > 0}
    
    group_names = {}
    for group_id, group in active_groups.items():
        name = group.get('name', group_id.split('_')[-1])
        if name == 'unknown':
            continue
        clean_name = name.replace('_', ' ').title().replace(' ', '')
        if clean_name in group_names.values():
            clean_name = f"{clean_name}{group_id.split('_')[-1]}"
        group_names[group_id] = clean_name
    
    mermaid = ["graph TD"]
    
    # Group by layers
    layers = ['Foundational', 'Core', 'Service', 'Integration', 'Utility', 'Other']
    layer_groups = {layer: [] for layer in layers}
    
    for group_id, group in active_groups.items():
        if group_id not in group_names:
            continue
        layer = layer_map.get(group_id, 'Other')
        layer_groups[layer].append((group_id, group))
    
    # Add subgraphs for each layer
    for layer in layers:
        if not layer_groups[layer]:
            continue
        mermaid.append(f"    subgraph {layer.replace(' ', '')}[\"{layer} Layer\"]")
        for group_id, group in layer_groups[layer]:
            name = group.get('name', 'unknown')
            clean_id = group_names[group_id]
            function_count = len(group.get('functions', []))
            file_count = len(group.get('files', []))
            label = f"{name.replace('_', ' ').title()}<br/>{function_count} funcs"
            mermaid.append(f"        {clean_id}[\"{label}\"]")
        mermaid.append("    end")
    
    # Add edges (only between different layers for clarity)
    for group_id, deps in group_deps.items():
        if group_id not in group_names:
            continue
        source = group_names[group_id]
        source_layer = layer_map.get(group_id, 'Other')
        for dep_group_id in deps:
            if dep_group_id in group_names:
                target = group_names[dep_group_id]
                target_layer = layer_map.get(dep_group_id, 'Other')
                # Only show cross-layer dependencies
                if source_layer != target_layer:
                    mermaid.append(f"    {source} --> {target}")
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mermaid))
    
    print(f"Layered diagram written to {output_file}")

def create_direct_deps_diagram(groups: Dict[str, Dict], group_deps: Dict[str, Set[str]], output_file: str):
    """Create diagram with only direct dependencies."""
    direct_deps = remove_transitive_dependencies(group_deps)
    
    active_groups = {gid: g for gid, g in groups.items() if len(g.get('functions', [])) > 0}
    
    group_names = {}
    for group_id, group in active_groups.items():
        name = group.get('name', group_id.split('_')[-1])
        if name == 'unknown':
            continue
        clean_name = name.replace('_', ' ').title().replace(' ', '')
        if clean_name in group_names.values():
            clean_name = f"{clean_name}{group_id.split('_')[-1]}"
        group_names[group_id] = clean_name
    
    mermaid = ["graph TD"]
    
    # Add nodes
    for group_id, group in active_groups.items():
        name = group.get('name', 'unknown')
        if name == 'unknown' or group_id not in group_names:
            continue
        clean_id = group_names[group_id]
        function_count = len(group.get('functions', []))
        file_count = len(group.get('files', []))
        label = f"{name.replace('_', ' ').title()}<br/>{function_count} funcs<br/>{file_count} files"
        mermaid.append(f"    {clean_id}[\"{label}\"]")
    
    # Add edges (only direct dependencies)
    for group_id, deps in direct_deps.items():
        if group_id not in group_names:
            continue
        source = group_names[group_id]
        for dep_group_id in deps:
            if dep_group_id in group_names:
                target = group_names[dep_group_id]
                mermaid.append(f"    {source} --> {target}")
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mermaid))
    
    print(f"Direct dependencies diagram written to {output_file}")
    print(f"Reduced from {sum(len(deps) for deps in group_deps.values())} to {sum(len(deps) for deps in direct_deps.values())} dependencies")

def main():
    project_root = Path('/Volumes/Files_1/iron-beam')
    mapping_file = project_root / 'rust-conversion' / 'behavior-groups-mapping.jsonld'
    analysis_file = project_root / 'rust-conversion' / 'c_analysis_results.json'
    
    print("Loading behavior groups...")
    groups, file_to_group = load_behavior_groups(str(mapping_file))
    
    print("Loading dependencies...")
    from analyze_group_dependencies import build_header_to_file_map
    header_map = build_header_to_file_map(project_root)
    group_deps = load_dependencies(str(analysis_file), groups, file_to_group, header_map, project_root)
    
    print("Classifying architectural layers...")
    layer_map = classify_architectural_layers(groups)
    
    print("\nCreating alternative diagrams...")
    
    # 1. Top-down layout
    td_file = project_root / 'rust-conversion' / 'group-dependencies-td.mmd'
    create_td_layout_diagram(groups, group_deps, str(td_file))
    
    # 2. Layered architecture
    layered_file = project_root / 'rust-conversion' / 'group-dependencies-layered.mmd'
    create_layered_diagram(groups, group_deps, layer_map, str(layered_file))
    
    # 3. Direct dependencies only
    direct_file = project_root / 'rust-conversion' / 'group-dependencies-direct.mmd'
    create_direct_deps_diagram(groups, group_deps, str(direct_file))
    
    print("\n=== SUMMARY ===")
    print(f"Total groups: {len([g for g in groups.values() if len(g.get('functions', [])) > 0])}")
    print(f"Total dependencies (all): {sum(len(deps) for deps in group_deps.values())}")
    direct_deps = remove_transitive_dependencies(group_deps)
    print(f"Direct dependencies: {sum(len(deps) for deps in direct_deps.values())}")
    
    # Print layer distribution
    print("\n=== LAYER DISTRIBUTION ===")
    for layer in ['Foundational', 'Core', 'Service', 'Integration', 'Utility', 'Other']:
        count = sum(1 for gid, g in groups.items() 
                   if len(g.get('functions', [])) > 0 and layer_map.get(gid) == layer)
        if count > 0:
            print(f"{layer}: {count} groups")

if __name__ == '__main__':
    main()

