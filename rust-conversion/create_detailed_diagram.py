#!/usr/bin/env python3
"""
Create detailed Mermaid diagram with CLEAN architecture layers as subgraphs.
"""

import json
import os
from pathlib import Path
from collections import defaultdict
from typing import Dict, Set, Tuple

def load_behavior_groups(mapping_file: str) -> tuple:
    """Load behavior groups from JSON-LD file."""
    with open(mapping_file, 'r') as f:
        data = json.load(f)
    
    groups = {}
    file_to_group = {}
    external_callers = set()
    
    # Get files with Erlang callers
    for node in data['@graph']:
        if node.get('@type') == 'ex:ExternalCaller':
            if node.get('callerLanguage') == 'erlang':
                external_callers.add(node.get('cFile', ''))
    
    # Get groups
    for node in data['@graph']:
        if 'BehaviorGroup' in node.get('@id', ''):
            group_id = node['@id']
            name = node.get('name', 'unknown')
            clean_layer = node.get('cleanLayer', 'unknown')
            files = set(node.get('files', []))
            functions = node.get('functions', [])
            
            # Check if group has Erlang callers
            has_erlang_caller = bool(files.intersection(external_callers))
            if has_erlang_caller and not name.startswith('✅'):
                name = '✅' + name
            
            groups[group_id] = {
                'name': name,
                'cleanLayer': clean_layer,
                'files': files,
                'functions': functions,
                'function_count': len(functions),
                'file_count': len(files)
            }
            
            for file_path in files:
                file_to_group[file_path] = group_id
    
    return groups, file_to_group

def analyze_group_dependencies(groups: Dict, file_to_group: Dict, analysis_file: str, project_root: Path) -> Dict[str, Set[str]]:
    """Analyze dependencies between groups."""
    with open(analysis_file, 'r') as f:
        analysis = json.load(f)
    
    file_dependencies = analysis.get('dependencies', {})
    transitive_dependencies = analysis.get('transitive_dependencies', {})
    
    # Build header map
    header_map = {}
    for root, dirs, files in os.walk(project_root):
        if any(skip in root for skip in ['.git', 'node_modules', 'obj', 'obj.debug']):
            continue
        for file in files:
            if file.endswith('.h'):
                full_path = Path(root) / file
                rel_path = str(full_path.relative_to(project_root))
                basename = os.path.basename(rel_path)
                header_map[basename] = rel_path
                header_map[rel_path] = rel_path
    
    def find_group_for_file(file_path: str) -> str:
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
    
    group_deps = defaultdict(set)
    
    for group_id, group in groups.items():
        group_files = set(group.get('files', []))
        
        all_deps = set()
        for file_path in group_files:
            direct_deps = file_dependencies.get(file_path, set())
            transitive_deps = transitive_dependencies.get(file_path, set())
            all_deps.update(direct_deps)
            all_deps.update(transitive_deps)
        
        for dep_file in all_deps:
            dep_group_id = find_group_for_file(dep_file)
            if dep_group_id and dep_group_id != group_id:
                group_deps[group_id].add(dep_group_id)
    
    return dict(group_deps)

def create_clean_node_id(name: str) -> str:
    """Create a clean node ID from a group name."""
    # Remove emojis and clean up
    clean_name = name.replace('✅', '').replace('❌', '').strip()
    clean_name = clean_name.replace('_', ' ').title().replace(' ', '')
    # Remove any remaining non-alphanumeric characters except underscores
    clean_name = ''.join(c for c in clean_name if c.isalnum() or c == '_')
    return clean_name

def format_display_name(name: str) -> str:
    """Format display name with proper capitalization."""
    # Keep emojis, format the rest
    emoji = '✅' if name.startswith('✅') else ''
    name_without_emoji = name.replace('✅', '').replace('❌', '').strip()
    formatted = name_without_emoji.replace('_', ' ').title()
    return emoji + formatted

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

def create_detailed_diagram(groups: Dict, group_deps: Dict, output_file: str):
    """Create detailed Mermaid diagram with CLEAN layers as subgraphs."""
    
    # Group by CLEAN layer
    by_layer = defaultdict(list)
    for group_id, group in groups.items():
        layer = group.get('cleanLayer', 'unknown')
        if layer != 'unknown' and group.get('function_count', 0) > 0:
            by_layer[layer].append((group_id, group))
    
    layer_order = ['entities', 'usecases', 'adapters', 'frameworks', 'infrastructure', 'code_management']
    layer_labels = {
        'entities': 'Entities Layer',
        'usecases': 'Usecases Layer',
        'adapters': 'Adapters Layer',
        'frameworks': 'Frameworks Layer',
        'infrastructure': 'Infrastructure Layer',
        'code_management': 'Code Management Layer'
    }
    
    mmd = ['graph TD']
    
    # Create node ID mapping
    node_ids = {}
    for group_id, group in groups.items():
        if group.get('function_count', 0) > 0:
            clean_id = create_clean_node_id(group['name'])
            # Ensure uniqueness
            base_id = clean_id
            counter = 1
            while clean_id in node_ids.values():
                clean_id = f"{base_id}{counter}"
                counter += 1
            node_ids[group_id] = clean_id
    
    # Create subgraphs for each layer
    for layer in layer_order:
        if layer not in by_layer:
            continue
        
        layer_groups = by_layer[layer]
        if not layer_groups:
            continue
        
        mmd.append(f'    subgraph {layer}["{layer_labels[layer]}"]')
        
        for group_id, group in sorted(layer_groups, key=lambda x: x[1]['name']):
            node_id = node_ids.get(group_id)
            if not node_id:
                continue
            
            display_name = format_display_name(group['name'])
            func_count = group['function_count']
            file_count = group['file_count']
            mmd.append(f'        {node_id}["{display_name}<br/>{func_count} funcs, {file_count} files"]')
        
        mmd.append('    end')
    
    # Detect circular dependencies
    circular_edges = detect_circular_dependencies(group_deps)
    
    # Add dependencies - track edge indices for styling
    edge_count = 0
    edge_to_index = {}  # Map (source_id, target_id) to edge index
    
    for source_id, target_ids in group_deps.items():
        source_node = node_ids.get(source_id)
        if not source_node:
            continue
        for target_id in target_ids:
            target_node = node_ids.get(target_id)
            if target_node:
                mmd.append(f'    {source_node} --> {target_node}')
                edge_to_index[(source_id, target_id)] = edge_count
                edge_count += 1
    
    # Add linkStyle statements for circular dependencies
    if circular_edges:
        mmd.append('')
        mmd.append('    %% Circular dependencies highlighted in red')
        for src_id, dst_id in circular_edges:
            if (src_id, dst_id) in edge_to_index:
                idx = edge_to_index[(src_id, dst_id)]
                mmd.append(f'    linkStyle {idx} stroke:#ff0000,stroke-width:3px')
    
    mmd.append('')
    circular_count = len(circular_edges)
    mmd.append('    %% Dependencies follow CLEAN architecture: outer layers depend on inner layers')
    mmd.append(f'    %% Total: {edge_count} dependencies, {circular_count} circular')
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mmd))
    
    print(f"Created detailed Mermaid diagram: {output_file}")
    print(f"  Groups: {len([g for g in groups.values() if g.get('function_count', 0) > 0])}")
    print(f"  Dependencies: {edge_count}")

def main():
    project_root = Path('/Volumes/Files_1/iron-beam')
    jsonld_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'behavior-groups-mapping.jsonld'
    analysis_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'c_analysis_results.json'
    output_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'group-dependencies-detailed.mmd'
    
    print("Loading behavior groups...")
    groups, file_to_group = load_behavior_groups(str(jsonld_file))
    print(f"Found {len(groups)} behavior groups")
    
    print("Analyzing group dependencies...")
    group_deps = analyze_group_dependencies(groups, file_to_group, str(analysis_file), project_root)
    print(f"Found {sum(len(deps) for deps in group_deps.values())} dependencies")
    
    print("Creating detailed diagram...")
    create_detailed_diagram(groups, group_deps, str(output_file))

if __name__ == '__main__':
    main()

