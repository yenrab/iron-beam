#!/usr/bin/env python3
"""
Create Mermaid diagram showing dependencies between SOLID/CLEAN groups.
Highlights circular dependencies in red.
"""

import json
import os
from pathlib import Path
from typing import Dict, Set, Tuple
from collections import defaultdict

def load_behavior_groups(mapping_file: str) -> Dict[str, Dict]:
    """Load behavior groups from JSON-LD mapping."""
    with open(mapping_file, 'r') as f:
        data = json.load(f)
    
    groups = {}
    for node in data['@graph']:
        if '@id' in node and 'BehaviorGroup' in node.get('@id', ''):
            group_id = node['@id']
            groups[group_id] = node
    
    return groups

def build_header_to_file_map(project_root: Path) -> Dict[str, str]:
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

def find_header_file(header_name: str, header_map: Dict[str, str]) -> str:
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

def find_group_for_file(file_path: str, file_to_group: Dict[str, str], project_root: Path) -> str:
    """Find which group a file belongs to."""
    if file_path in file_to_group:
        return file_to_group[file_path]
    if file_path.endswith('.h'):
        header_dir = os.path.dirname(file_path)
        header_basename = os.path.splitext(os.path.basename(file_path))[0]
        possible_c_file = os.path.join(header_dir, header_basename + '.c')
        if possible_c_file in file_to_group:
            return file_to_group[possible_c_file]
    return None

def build_group_dependencies(groups: Dict[str, Dict], analysis_file: str, 
                             project_root: Path) -> Dict[str, Set[str]]:
    """Build dependencies between groups."""
    with open(analysis_file, 'r') as f:
        analysis = json.load(f)
    
    file_dependencies = analysis.get('file_dependencies', {})
    header_map = build_header_to_file_map(project_root)
    
    # Build file to group mapping
    file_to_group = {}
    for group_id, group in groups.items():
        for file_path in group.get('files', []):
            file_to_group[file_path] = group_id
    
    group_deps = defaultdict(set)
    
    for group_id, group in groups.items():
        group_files = set(group.get('files', []))
        
        for file_path in group_files:
            deps = file_dependencies.get(file_path, [])
            for dep_header in deps:
                dep_file = find_header_file(dep_header, header_map)
                if dep_file:
                    dep_group_id = find_group_for_file(dep_file, file_to_group, project_root)
                    if dep_group_id and dep_group_id != group_id:
                        # Check CLEAN layer ordering
                        group_layer = group.get('cleanLayer', 'infrastructure')
                        dep_group = groups.get(dep_group_id, {})
                        dep_layer = dep_group.get('cleanLayer', 'infrastructure')
                        
                        # Layer order (dependencies should flow inward)
                        layer_order = {
                            'frameworks': 0,
                            'adapters': 1,
                            'usecases': 2,
                            'entities': 3,
                            'infrastructure': 1,
                            'code_management': 2
                        }
                        
                        source_order = layer_order.get(group_layer, 99)
                        target_order = layer_order.get(dep_layer, 99)
                        
                        # Only allow dependencies that flow inward
                        if source_order < target_order:
                            group_deps[group_id].add(dep_group_id)
    
    return dict(group_deps)

def detect_circular_dependencies(group_deps: Dict[str, Set[str]]) -> Set[Tuple[str, str]]:
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
        
        for w in group_deps.get(v, set()):
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
                    if next_node in group_deps.get(node, set()):
                        circular_edges.add((node, next_node))
                for node in scc:
                    for neighbor in group_deps.get(node, set()):
                        if neighbor in scc and neighbor != node:
                            circular_edges.add((node, neighbor))
    
    for node in group_deps.keys():
        if node not in indices:
            strongconnect(node)
    
    return circular_edges

def create_diagram(groups: Dict[str, Dict], group_deps: Dict[str, Set[str]], 
                   circular_edges: Set[Tuple[str, str]], output_file: str):
    """Create Mermaid diagram with dependencies."""
    group_names = {}
    for group_id, group in groups.items():
        name = group.get('name', 'unknown')
        clean_name = name.replace('_', ' ').title().replace(' ', '')
        if clean_name in group_names.values():
            clean_name = f"{clean_name}{group_id.split('_')[-1]}"
        group_names[group_id] = clean_name
    
    mermaid = ["graph TD"]
    
    # Group by CLEAN layer
    by_layer = defaultdict(list)
    for group_id, group in groups.items():
        layer = group.get('cleanLayer', 'unknown')
        by_layer[layer].append((group_id, group))
    
    layer_order = ['entities', 'usecases', 'adapters', 'frameworks', 'infrastructure', 'code_management']
    layer_colors = {
        'entities': '#e1f5e1',
        'usecases': '#e1f5ff',
        'adapters': '#ffe1f5',
        'frameworks': '#fff5e1',
        'infrastructure': '#f5f5e1',
        'code_management': '#f5e1ff'
    }
    
    # Add subgraphs for each layer
    for layer in layer_order:
        if layer not in by_layer:
            continue
        
        layer_name = layer.replace('_', ' ').title()
        mermaid.append(f"    subgraph {layer.replace('_', '')}[\"{layer_name} Layer\"]")
        
        for group_id, group in by_layer[layer]:
            name = group.get('name', 'unknown')
            clean_id = group_names[group_id]
            function_count = len(group.get('functions', []))
            file_count = len(group.get('files', []))
            label = f"{name.replace('_', ' ').title()}<br/>{function_count} funcs, {file_count} files"
            mermaid.append(f"        {clean_id}[\"{label}\"]")
        
        mermaid.append("    end")
    
    # Add edges - track indices for styling
    circular_edge_indices = []
    edge_index = 0
    
    for group_id, deps in group_deps.items():
        if group_id not in group_names:
            continue
        source = group_names[group_id]
        for dep_group_id in deps:
            if dep_group_id in group_names:
                target = group_names[dep_group_id]
                edge = (group_id, dep_group_id)
                
                if edge in circular_edges:
                    mermaid.append(f"    {source} -->|circular| {target}")
                    circular_edge_indices.append(edge_index)
                else:
                    mermaid.append(f"    {source} --> {target}")
                edge_index += 1
    
    # Add linkStyle for circular edges
    if circular_edge_indices:
        mermaid.append("")
        for idx in circular_edge_indices:
            mermaid.append(f"    linkStyle {idx} stroke:#ff0000,stroke-width:3px")
    else:
        mermaid.append("")
        mermaid.append("    %% No circular dependencies found - all dependencies follow CLEAN architecture")
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mermaid))
    
    print(f"Diagram written to {output_file}")
    print(f"Total dependencies: {sum(len(deps) for deps in group_deps.values())}")
    print(f"Circular dependencies: {len(circular_edges)}")

def main():
    project_root = Path('/Volumes/Files_1/iron-beam')
    mapping_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'behavior-groups-mapping.jsonld'
    analysis_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'c_analysis_results.json'
    output_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'group-dependencies-detailed.mmd'
    
    print("Loading behavior groups...")
    groups = load_behavior_groups(str(mapping_file))
    print(f"Found {len(groups)} groups")
    
    print("Building group dependencies...")
    group_deps = build_group_dependencies(groups, str(analysis_file), project_root)
    
    print("Detecting circular dependencies...")
    circular_edges = detect_circular_dependencies(group_deps)
    
    print("Creating diagram...")
    create_diagram(groups, group_deps, circular_edges, str(output_file))
    
    if circular_edges:
        print(f"\n⚠️  Found {len(circular_edges)} circular dependencies:")
        for src, dst in list(circular_edges)[:10]:
            src_name = groups[src].get('name', 'unknown')
            dst_name = groups[dst].get('name', 'unknown')
            print(f"  {src_name} -> {dst_name}")
    else:
        print("\n✅ No circular dependencies - CLEAN architecture maintained!")

if __name__ == '__main__':
    main()

