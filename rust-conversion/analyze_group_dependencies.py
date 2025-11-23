#!/usr/bin/env python3
"""
Analyze dependencies between behavior groups based on file dependencies.
"""

import json
import os
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Set, Tuple, Optional

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
                for node in scc:
                    for neighbor in group_deps.get(node, set()):
                        if neighbor in scc and neighbor != node:
                            circular_edges.add((node, neighbor))
    
    for node in group_deps.keys():
        if node not in indices:
            strongconnect(node)
    
    return circular_edges

def load_analysis_results(analysis_file: str) -> Dict:
    """Load the C analysis results to get file dependencies."""
    with open(analysis_file, 'r') as f:
        return json.load(f)

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
            # Map files to groups
            for file_path in node.get('files', []):
                file_to_group[file_path] = group_id
    
    return groups, file_to_group

def build_header_to_file_map(project_root: Path) -> Dict[str, str]:
    """Build a map from header names to actual file paths."""
    header_map = {}
    
    # Walk through the project and index all .h files
    for root, dirs, files in os.walk(project_root):
        # Skip certain directories
        if any(skip in root for skip in ['.git', 'node_modules', 'obj', 'obj.debug']):
            continue
        
        for file in files:
            if file.endswith('.h'):
                full_path = Path(root) / file
                rel_path = str(full_path.relative_to(project_root))
                
                # Map by basename
                header_map[file] = rel_path
                # Also map by relative path from common include dirs
                if 'include' in rel_path:
                    parts = rel_path.split('/')
                    include_idx = parts.index('include')
                    if include_idx + 1 < len(parts):
                        include_name = '/'.join(parts[include_idx + 1:])
                        header_map[include_name] = rel_path
    
    return header_map

def find_header_file(header_name: str, header_map: Dict[str, str], project_root: Path) -> Optional[str]:
    """Find the actual header file path."""
    # Remove quotes/angle brackets
    header = header_name.strip('"<>')
    
    # Direct match
    if header in header_map:
        return header_map[header]
    
    # Try basename match
    basename = os.path.basename(header)
    if basename in header_map:
        return header_map[basename]
    
    # Try to find by searching
    for mapped_name, mapped_path in header_map.items():
        if header in mapped_name or mapped_name.endswith(header):
            return mapped_path
    
    return None

def find_group_for_header(header_path: str, file_to_group: Dict[str, str], project_root: Path) -> Optional[str]:
    """Find which group a header file belongs to."""
    # Check if header itself is in a group (some groups might include headers)
    if header_path in file_to_group:
        return file_to_group[header_path]
    
    # Try to find corresponding .c file in same directory
    header_dir = os.path.dirname(header_path)
    header_basename = os.path.splitext(os.path.basename(header_path))[0]
    
    # Look for .c file with same basename
    possible_c_file = os.path.join(header_dir, header_basename + '.c')
    if possible_c_file in file_to_group:
        return file_to_group[possible_c_file]
    
    # Look for any .c file in the same directory that's in a group
    # This handles cases where headers are shared across multiple .c files
    header_dir_path = project_root / header_dir
    if header_dir_path.exists():
        for c_file in header_dir_path.glob('*.c'):
            c_file_rel = str(c_file.relative_to(project_root))
            if c_file_rel in file_to_group:
                return file_to_group[c_file_rel]
    
    return None

def analyze_group_dependencies(
    groups: Dict[str, Dict],
    file_to_group: Dict[str, str],
    analysis_results: Dict,
    header_map: Dict[str, str],
    project_root: Path
) -> Dict[str, Set[str]]:
    """Analyze dependencies between groups based on file includes."""
    group_deps = defaultdict(set)
    
    # Get file dependencies from analysis
    file_dependencies = analysis_results.get('dependencies', {})
    
    # For each group, check which other groups its files depend on
    for group_id, group in groups.items():
        group_files = set(group.get('files', []))
        
        for file_path in group_files:
            # Get dependencies for this file
            deps = file_dependencies.get(file_path, [])
            
            for dep_header in deps:
                # Find the actual header file path
                dep_file = find_header_file(dep_header, header_map, project_root)
                
                if dep_file:
                    # Find which group this header belongs to
                    dep_group = find_group_for_header(dep_file, file_to_group, project_root)
                    
                    if dep_group and dep_group != group_id:  # Don't count self-dependencies
                        group_deps[group_id].add(dep_group)
    
    return dict(group_deps)

def create_mermaid_diagram(
    groups: Dict[str, Dict],
    group_deps: Dict[str, Set[str]],
    output_file: str
):
    """Create a Mermaid diagram showing dependencies between groups."""
    
    # Filter out groups with no functions (these are likely rationale nodes)
    active_groups = {gid: g for gid, g in groups.items() if len(g.get('functions', [])) > 0}
    
    # Create a mapping from group IDs to readable names
    group_names = {}
    for group_id, group in active_groups.items():
        name = group.get('name', group_id.split('_')[-1])
        if name == 'unknown':
            continue
        # Remove emojis from node ID (but keep in label)
        name_without_emoji = name.replace('✅', '').replace('❌', '').strip()
        # Clean up name for Mermaid (remove underscores, capitalize)
        clean_name = name_without_emoji.replace('_', ' ').title().replace(' ', '')
        # Remove any remaining non-alphanumeric characters except underscores
        clean_name = ''.join(c for c in clean_name if c.isalnum() or c == '_')
        # Ensure unique names
        if clean_name in group_names.values():
            clean_name = f"{clean_name}{group_id.split('_')[-1]}"
        group_names[group_id] = clean_name
    
    # Detect circular dependencies
    circular_edges = detect_circular_dependencies(group_deps)
    print(f"Found {len(circular_edges)} circular dependencies")
    
    # Use LR (left-to-right) layout for better readability
    mermaid = ["graph LR"]
    
    # Add nodes
    for group_id, group in active_groups.items():
        name = group.get('name', 'unknown')
        if name == 'unknown':
            continue
        clean_id = group_names.get(group_id)
        if not clean_id:
            continue
        function_count = len(group.get('functions', []))
        file_count = len(group.get('files', []))
        # Keep emojis in label but format nicely
        display_name = name.replace('_', ' ').title()
        label = f"{display_name}<br/>{function_count} funcs<br/>{file_count} files"
        mermaid.append(f"    {clean_id}[\"{label}\"]")
    
    # Add edges (dependencies) - track edge indices for styling
    edge_index = 0
    edge_to_index = {}  # Map (source, target) to edge index
    
    for group_id, deps in group_deps.items():
        if group_id not in group_names:
            continue
        source = group_names[group_id]
        for dep_group_id in deps:
            if dep_group_id in group_names:
                target = group_names[dep_group_id]
                mermaid.append(f"    {source} --> {target}")
                edge_to_index[(group_id, dep_group_id)] = edge_index
                edge_index += 1
    
    # Add linkStyle statements for circular dependencies
    if circular_edges:
        mermaid.append('')
        mermaid.append('    %% Circular dependencies highlighted in red')
        for src_id, dst_id in circular_edges:
            if (src_id, dst_id) in edge_to_index:
                idx = edge_to_index[(src_id, dst_id)]
                mermaid.append(f'    linkStyle {idx} stroke:#ff0000,stroke-width:3px')
    
    # Write to file
    with open(output_file, 'w') as f:
        f.write('\n'.join(mermaid))
    
    print(f"Mermaid diagram written to {output_file}")
    print(f"Total active groups: {len(active_groups)}")
    total_deps = sum(len(deps) for deps in group_deps.values())
    print(f"Total dependencies: {total_deps}")
    if circular_edges:
        print(f"Circular dependencies: {len(circular_edges)} (highlighted in red)")
        if total_deps > 0:
            print(f"Percentage circular: {len(circular_edges)/total_deps*100:.1f}%")
    else:
        print("✅ No circular dependencies found!")

def main():
    project_root = Path('/Volumes/Files_1/iron-beam')
    mapping_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'behavior-groups-mapping.jsonld'
    analysis_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'c_analysis_results.json'
    # Note: group-dependencies.mmd is no longer generated - use group-dependencies-detailed.mmd instead
    output_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'group-dependencies-detailed.mmd'
    
    print("Loading behavior groups...")
    groups, file_to_group = load_behavior_groups(str(mapping_file))
    print(f"Found {len(groups)} behavior groups")
    print(f"Mapped {len(file_to_group)} files to groups")
    
    print("Building header file map...")
    header_map = build_header_to_file_map(project_root)
    print(f"Indexed {len(header_map)} header files")
    
    print("Loading analysis results...")
    analysis_results = load_analysis_results(str(analysis_file))
    
    print("Analyzing group dependencies...")
    group_deps = analyze_group_dependencies(
        groups, file_to_group, analysis_results, header_map, project_root
    )
    
    groups_with_deps = [g for g, d in group_deps.items() if d]
    print(f"Found dependencies between {len(groups_with_deps)} groups")
    
    print("Creating Mermaid diagram...")
    # Note: We now only generate group-dependencies-detailed.mmd (with subgraphs)
    # The LR layout diagram is no longer generated as it provides no additional information
    # Use create_detailed_diagram.py or create_group_level_diagrams.py instead
    print("Note: This script analyzes file-level dependencies.")
    print("For group-level dependencies, use create_group_level_diagrams.py")
    # create_mermaid_diagram(groups, group_deps, str(output_file))  # Disabled - use detailed diagram instead
    
    # Print summary
    print("\n=== DEPENDENCY SUMMARY ===")
    for group_id, deps in sorted(group_deps.items()):
        if deps:
            group_name = groups[group_id].get('name', 'unknown')
            dep_names = [groups[d].get('name', 'unknown') for d in deps]
            print(f"{group_name} depends on: {', '.join(dep_names)}")

if __name__ == '__main__':
    main()
