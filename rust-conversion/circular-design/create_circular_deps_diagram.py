#!/usr/bin/env python3
"""
Create a Mermaid diagram with circular dependencies highlighted in red.
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

def find_cycles(group_deps: Dict[str, Set[str]]) -> Set[Tuple[str, str]]:
    """Find all edges that are part of cycles (circular dependencies)."""
    circular_edges = set()
    
    def has_cycle(start: str, current: str, visited: Set[str], path: List[str]) -> bool:
        """Check if there's a cycle starting from 'start'."""
        if current in visited:
            return False
        
        visited.add(current)
        path.append(current)
        
        for neighbor in group_deps.get(current, set()):
            if neighbor == start:
                # Found a cycle back to start
                # Mark all edges in this path as circular
                for i in range(len(path)):
                    if i + 1 < len(path):
                        circular_edges.add((path[i], path[i+1]))
                    circular_edges.add((path[-1], start))
                return True
            elif neighbor not in visited:
                if has_cycle(start, neighbor, visited, path[:]):
                    return True
        
        return False
    
    # Check for cycles from each node
    for group_id in group_deps.keys():
        visited = set()
        has_cycle(group_id, group_id, visited, [])
    
    return circular_edges

def find_all_cycles_tarjan(group_deps: Dict[str, Set[str]]) -> Set[Tuple[str, str]]:
    """Find all cycles using Tarjan's strongly connected components algorithm."""
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
        
        # If v is a root node, pop the stack and build an SCC
        if lowlinks[v] == indices[v]:
            scc = []
            while True:
                w = stack.pop()
                on_stack.remove(w)
                scc.append(w)
                if w == v:
                    break
            
            # If SCC has more than one node, it's a cycle
            # Mark all edges within the SCC as circular
            if len(scc) > 1:
                for i, node in enumerate(scc):
                    next_node = scc[(i + 1) % len(scc)]
                    if next_node in group_deps.get(node, set()):
                        circular_edges.add((node, next_node))
                # Also mark all edges between nodes in the SCC
                for node in scc:
                    for neighbor in group_deps.get(node, set()):
                        if neighbor in scc and neighbor != node:
                            circular_edges.add((node, neighbor))
    
    # Process all nodes
    for node in group_deps.keys():
        if node not in indices:
            strongconnect(node)
    
    return circular_edges

def load_dependencies(analysis_file: str, groups: Dict[str, Dict], file_to_group: Dict[str, str], 
                     header_map: Dict[str, str], project_root: Path) -> Dict[str, Set[str]]:
    """Load and analyze group dependencies."""
    from analyze_group_dependencies import load_analysis_results, find_header_file, find_group_for_header
    
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

def create_circular_deps_diagram(groups: Dict[str, Dict], group_deps: Dict[str, Set[str]], 
                                 circular_edges: Set[Tuple[str, str]], output_file: str):
    """Create Mermaid diagram with circular dependencies in red."""
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
    
    # Add edges - track which ones are circular for styling
    circular_edge_indices = []  # List of edge indices that are circular
    
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
                    # Red edge for circular dependency - add label
                    mermaid.append(f"    {source} -->|circular| {target}")
                    circular_edge_indices.append(edge_index)
                else:
                    # Normal black edge
                    mermaid.append(f"    {source} --> {target}")
                edge_index += 1
    
    # Add linkStyle for circular edges (0-indexed)
    mermaid.append("")
    for idx in circular_edge_indices:
        mermaid.append(f"    linkStyle {idx} stroke:#ff0000,stroke-width:3px")
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mermaid))
    
    print(f"Circular dependencies diagram written to {output_file}")
    print(f"Found {len(circular_edges)} circular dependency edges")

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
    
    print("Finding circular dependencies...")
    circular_edges = find_all_cycles_tarjan(group_deps)
    
    if circular_edges:
        print(f"\nFound {len(circular_edges)} circular dependency edges:")
        group_names = {}
        for group_id, group in groups.items():
            name = group.get('name', group_id.split('_')[-1])
            if name != 'unknown':
                clean_name = name.replace('_', ' ').title().replace(' ', '')
                group_names[group_id] = clean_name
        
        for src, dst in list(circular_edges)[:10]:  # Show first 10
            src_name = groups[src].get('name', 'unknown')
            dst_name = groups[dst].get('name', 'unknown')
            print(f"  {src_name} -> {dst_name}")
        if len(circular_edges) > 10:
            print(f"  ... and {len(circular_edges) - 10} more")
    else:
        print("No circular dependencies found!")
    
    print("\nCreating diagram with circular dependencies highlighted...")
    output_file = project_root / 'rust-conversion' / 'group-dependencies-circular.mmd'
    create_circular_deps_diagram(groups, group_deps, circular_edges, str(output_file))

if __name__ == '__main__':
    main()

