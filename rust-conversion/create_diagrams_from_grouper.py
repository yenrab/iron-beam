#!/usr/bin/env python3
"""
Create Mermaid diagrams using group-level dependencies from the grouper,
not file-level dependencies. This shows the actual grouping structure.
"""

import json
import os
from pathlib import Path
from typing import Dict, Set, Tuple
from collections import defaultdict

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

def load_groups_and_dependencies(jsonld_file: str) -> tuple:
    """Load groups and their dependencies from JSON-LD."""
    with open(jsonld_file, 'r') as f:
        data = json.load(f)
    
    groups = {}
    group_deps = defaultdict(set)
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
    
    # Now we need to rebuild dependencies from file-level analysis
    # but we'll use the grouping structure
    return groups, group_deps

def create_clean_node_id(name: str) -> str:
    """Create a clean node ID from a group name."""
    clean_name = name.replace('✅', '').replace('❌', '').strip()
    clean_name = clean_name.replace('_', ' ').title().replace(' ', '')
    clean_name = ''.join(c for c in clean_name if c.isalnum() or c == '_')
    return clean_name

def format_display_name(name: str) -> str:
    """Format display name with proper capitalization."""
    emoji = '✅' if name.startswith('✅') else ''
    name_without_emoji = name.replace('✅', '').replace('❌', '').strip()
    formatted = name_without_emoji.replace('_', ' ').title()
    return emoji + formatted

def create_lr_diagram(groups: Dict, group_deps: Dict, output_file: str):
    """Create left-to-right Mermaid diagram."""
    active_groups = {gid: g for gid, g in groups.items() if g.get('function_count', 0) > 0}
    
    # Create node ID mapping
    group_names = {}
    for group_id, group in active_groups.items():
        name = group.get('name', 'unknown')
        if name == 'unknown':
            continue
        clean_name = create_clean_node_id(name)
        if clean_name in group_names.values():
            clean_name = f"{clean_name}{group_id.split('_')[-1]}"
        group_names[group_id] = clean_name
    
    # Detect circular dependencies
    circular_edges = detect_circular_dependencies(group_deps)
    
    mmd = ['graph LR']
    
    # Add nodes
    for group_id, group in active_groups.items():
        name = group.get('name', 'unknown')
        if name == 'unknown':
            continue
        clean_id = group_names.get(group_id)
        if not clean_id:
            continue
        function_count = group.get('function_count', 0)
        file_count = group.get('file_count', 0)
        display_name = format_display_name(name)
        label = f"{display_name}<br/>{function_count} funcs<br/>{file_count} files"
        mmd.append(f"    {clean_id}[\"{label}\"]")
    
    # Add edges - track indices
    edge_index = 0
    edge_to_index = {}
    
    for group_id, deps in group_deps.items():
        if group_id not in group_names:
            continue
        source = group_names[group_id]
        for dep_group_id in deps:
            if dep_group_id in group_names:
                target = group_names[dep_group_id]
                mmd.append(f"    {source} --> {target}")
                edge_to_index[(group_id, dep_group_id)] = edge_index
                edge_index += 1
    
    # Add linkStyle for circular dependencies
    if circular_edges:
        mmd.append('')
        mmd.append('    %% Circular dependencies highlighted in red')
        for src_id, dst_id in circular_edges:
            if (src_id, dst_id) in edge_to_index:
                idx = edge_to_index[(src_id, dst_id)]
                mmd.append(f'    linkStyle {idx} stroke:#ff0000,stroke-width:3px')
    
    mmd.append('')
    mmd.append(f'    %% Total: {edge_index} dependencies, {len(circular_edges)} circular')
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mmd))
    
    print(f"Created LR diagram: {output_file}")
    print(f"  Groups: {len(active_groups)}")
    print(f"  Dependencies: {edge_index}")
    print(f"  Circular: {len(circular_edges)}")

def create_detailed_diagram(groups: Dict, group_deps: Dict, output_file: str):
    """Create detailed TD diagram with subgraphs."""
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
            base_id = clean_id
            counter = 1
            while clean_id in node_ids.values():
                clean_id = f"{base_id}{counter}"
                counter += 1
            node_ids[group_id] = clean_id
    
    # Create subgraphs
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
    
    # Add dependencies - track indices
    edge_count = 0
    edge_to_index = {}
    
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
    
    # Add linkStyle for circular dependencies
    if circular_edges:
        mmd.append('')
        mmd.append('    %% Circular dependencies highlighted in red')
        for src_id, dst_id in circular_edges:
            if (src_id, dst_id) in edge_to_index:
                idx = edge_to_index[(src_id, dst_id)]
                mmd.append(f'    linkStyle {idx} stroke:#ff0000,stroke-width:3px')
    
    mmd.append('')
    mmd.append('    %% Dependencies follow CLEAN architecture: outer layers depend on inner layers')
    mmd.append(f'    %% Total: {edge_count} dependencies, {len(circular_edges)} circular')
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mmd))
    
    print(f"Created detailed diagram: {output_file}")
    print(f"  Groups: {len([g for g in groups.values() if g.get('function_count', 0) > 0])}")
    print(f"  Dependencies: {edge_count}")
    print(f"  Circular: {len(circular_edges)}")

def main():
    project_root = Path('/Volumes/Files_1/iron-beam')
    jsonld_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'behavior-groups-mapping.jsonld'
    analysis_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'c_analysis_results.json'
    
    # Load groups
    groups, _ = load_groups_and_dependencies(str(jsonld_file))
    
    # Rebuild dependencies from file analysis but respect grouping
    from rust_conversion.analyze_group_dependencies import analyze_group_dependencies, load_behavior_groups
    
    # Actually, let's use the cycle-minimizing grouper's dependencies
    # We need to extract them from the JSON-LD or regenerate
    print("Loading groups and rebuilding dependencies...")
    
    # For now, use analyze_group_dependencies but we'll filter to show group-level structure
    file_to_group = {}
    for group_id, group in groups.items():
        for file_path in group.get('files', []):
            file_to_group[file_path] = group_id
    
    # Build group dependencies from file dependencies
    file_deps = {}
    transitive_deps = {}
    with open(analysis_file, 'r') as f:
        analysis = json.load(f)
        file_deps = analysis.get('dependencies', {})
        transitive_deps = analysis.get('transitive_dependencies', {})
    
    group_deps = defaultdict(set)
    
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
        if file_path in file_to_group:
            return file_to_group[file_path]
        if file_path.endswith('.h'):
            header_dir = os.path.dirname(file_path)
            header_basename = os.path.splitext(os.path.basename(file_path))[0]
            possible_c_file = os.path.join(header_dir, header_basename + '.c')
            if possible_c_file in file_to_group:
                return file_to_group[possible_c_file]
        return None
    
    # Build group dependencies
    for group_id, group in groups.items():
        group_files = set(group.get('files', []))
        
        all_deps = set()
        for file_path in group_files:
            direct_deps = file_deps.get(file_path, set())
            transitive_deps_list = transitive_deps.get(file_path, set())
            all_deps.update(direct_deps)
            all_deps.update(transitive_deps_list)
        
        for dep_file in all_deps:
            dep_group_id = find_group_for_file(dep_file)
            if dep_group_id and dep_group_id != group_id:
                group_deps[group_id].add(dep_group_id)
    
    # Create diagrams
    lr_output = project_root / 'rust-conversion' / 'solid-clean-design' / 'group-dependencies.mmd'
    td_output = project_root / 'rust-conversion' / 'solid-clean-design' / 'group-dependencies-detailed.mmd'
    
    print("\nCreating diagrams...")
    create_lr_diagram(groups, dict(group_deps), str(lr_output))
    create_detailed_diagram(groups, dict(group_deps), str(td_output))
    
    # Summary
    circular_edges = detect_circular_dependencies(dict(group_deps))
    total_deps = sum(len(deps) for deps in group_deps.values())
    print(f"\n=== SUMMARY ===")
    print(f"Total groups: {len(groups)}")
    print(f"Total dependencies: {total_deps}")
    print(f"Circular dependencies: {len(circular_edges)}")
    if total_deps > 0:
        print(f"Percentage circular: {len(circular_edges)/total_deps*100:.1f}%")

if __name__ == '__main__':
    main()

