#!/usr/bin/env python3
"""
Create Mermaid diagrams using GROUP-LEVEL dependencies from the cycle-minimizing grouper.
This shows the actual behavior group structure, not file-level dependencies.
"""

import json
import os
from pathlib import Path
from typing import Dict, Set, Tuple, List
from collections import defaultdict
import sys

# Import the grouper to get group-level dependencies
sys.path.insert(0, str(Path(__file__).parent))
from cycle_minimizing_grouper import CycleMinimizingGrouper

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

def create_clean_node_id(name: str) -> str:
    """Create a clean node ID from a group name."""
    clean_name = name.replace('✅', '').replace('❌', '').replace('[Erlang]', '').strip()
    clean_name = clean_name.replace('_', ' ').title().replace(' ', '')
    clean_name = ''.join(c for c in clean_name if c.isalnum() or c == '_')
    return clean_name

def format_display_name(name: str) -> str:
    """Format display name with proper capitalization."""
    # Keep emoji for those who can see it, but also ensure visibility
    has_erlang = name.startswith('✅') or '[Erlang]' in name
    name_without_emoji = name.replace('✅', '').replace('❌', '').replace('[Erlang]', '').strip()
    formatted = name_without_emoji.replace('_', ' ').title()
    if has_erlang:
        # Use both emoji and text marker for maximum compatibility
        return '✅ [Erlang] ' + formatted
    return formatted

def create_detailed_diagram(groups: List[Dict], group_deps: Dict[str, Set[str]], output_file: str):
    """Create detailed TD diagram with subgraphs."""
    # Convert groups list to dict for easier lookup
    groups_dict = {g['@id']: g for g in groups}
    
    # Identify groups called by Erlang - these need API facades
    erlang_called_groups = set()
    for group in groups:
        name = group.get('name', '')
        if name.startswith('✅') or '[Erlang]' in name or '✅' in name:
            erlang_called_groups.add(group['@id'])
    
    # Create API facade groups for each Erlang-called group
    api_facades = []
    for group_id in erlang_called_groups:
        group = groups_dict[group_id]
        original_name = group.get('name', '').replace('✅', '').replace('[Erlang]', '').strip()
        facade_id = f"{group_id}_api_facade"
        facade_name = f"{original_name}_api"
        api_facades.append({
            '@id': facade_id,
            'name': facade_name,
            'cleanLayer': 'api_facades',
            'original_group': group_id,
            'function_count': 0,
            'file_count': 0,
            'files': [],
            'functions': []
        })
    
    by_layer = defaultdict(list)
    
    # Add API facades layer
    for facade in api_facades:
        by_layer['api_facades'].append(facade)
    
    # Add other groups
    for group in groups:
        layer = group.get('cleanLayer', 'unknown')
        if layer != 'unknown' and len(group.get('functions', [])) > 0:
            by_layer[layer].append(group)
    
    layer_order = ['api_facades', 'entities', 'usecases', 'adapters', 'frameworks', 'infrastructure', 'code_management']
    layer_labels = {
        'api_facades': 'API Facades Layer (NEW)',
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
    
    # First, add API facades
    for facade in api_facades:
        facade_id = facade['@id']
        original_group = groups_dict[facade['original_group']]
        original_name = original_group.get('name', '').replace('✅', '').replace('[Erlang]', '').strip()
        clean_id = create_clean_node_id(original_name + '_api')
        node_ids[facade_id] = clean_id
    
    # Then add regular groups
    for group in groups:
        group_id = group['@id']
        if len(group.get('functions', [])) > 0:
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
        
        for group in sorted(layer_groups, key=lambda x: x.get('name', '')):
            group_id = group['@id']
            node_id = node_ids.get(group_id)
            if not node_id:
                continue
            
            # Handle API facades differently
            if layer == 'api_facades':
                original_group_id = group.get('original_group')
                original_group = groups_dict.get(original_group_id, {})
                original_name = original_group.get('name', '').replace('✅', '').replace('[Erlang]', '').strip()
                display_name = format_display_name(original_name).replace('✅ [Erlang] ', '') + ' API'
                mmd.append(f'        {node_id}["{display_name}<br/>(Facade)"]')
                # Blue background for new API layer
                mmd.append(f'        style {node_id} fill:#87CEEB,stroke:#0066CC,stroke-width:2px')
            else:
                display_name = format_display_name(group.get('name', ''))
                func_count = len(group.get('functions', []))
                file_count = len(group.get('files', []))
                has_erlang = group.get('name', '').startswith('✅') or '[Erlang]' in group.get('name', '')
                mmd.append(f'        {node_id}["{display_name}<br/>{func_count} funcs, {file_count} files"]')
                # Add styling for Erlang-callable groups (green background)
                if has_erlang:
                    mmd.append(f'        style {node_id} fill:#90EE90,stroke:#006400,stroke-width:2px')
        
        mmd.append('    end')
    
    # Detect circular dependencies
    circular_edges = detect_circular_dependencies(group_deps)
    
    # Add API facade dependencies (API Facades → Original Groups)
    # First, add edges from API facades to their corresponding groups
    api_deps = {}
    for facade in api_facades:
        facade_id = facade['@id']
        original_group_id = facade['original_group']
        facade_node = node_ids.get(facade_id)
        group_node = node_ids.get(original_group_id)
        if facade_node and group_node:
            mmd.append(f'    {facade_node} --> {group_node}')
            api_deps[facade_id] = {original_group_id}
    
    # Add dependencies - track indices
    edge_count = len(api_deps)
    edge_to_index = {}
    
    # Track API facade edges
    for facade_id, target_ids in api_deps.items():
        source_node = node_ids.get(facade_id)
        if not source_node:
            continue
        for target_id in target_ids:
            target_node = node_ids.get(target_id)
            if target_node:
                idx = list(api_deps.keys()).index(facade_id)
                edge_to_index[(facade_id, target_id)] = idx
    
    # Add original group dependencies
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
    mmd.append('    %% GROUP-LEVEL dependencies (from cycle-minimizing grouper)')
    mmd.append('    %% Dependencies follow CLEAN architecture: outer layers depend on inner layers')
    mmd.append('    %% API Facades Layer (NEW): Facades that Erlang calls, which then call Rust modules')
    mmd.append(f'    %% Total: {edge_count} dependencies, {len(circular_edges)} circular')
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mmd))
    
    print(f"Created detailed diagram: {output_file}")
    print(f"  Groups: {len([g for g in groups if len(g.get('functions', [])) > 0])}")
    print(f"  API Facades: {len(api_facades)}")
    print(f"  Dependencies: {edge_count}")
    print(f"  Circular: {len(circular_edges)}")

def main():
    project_root = Path('/Volumes/Files_1/iron-beam')
    analysis_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'c_analysis_results.json'
    
    print("Creating cycle-minimizing grouper...")
    grouper = CycleMinimizingGrouper(str(analysis_file))
    grouper.create_layered_groups()
    
    # Mark groups with Erlang callers (add ✅ emoji)
    files_with_erlang_callers = set()
    for caller in grouper.external_callers:
        if caller.get('caller_language') == 'erlang':
            files_with_erlang_callers.add(caller.get('c_file', ''))
    
    for group in grouper.groups:
        group_files = set(group.get('files', []))
        if group_files.intersection(files_with_erlang_callers):
            if not group.get('name', '').startswith('✅'):
                group['name'] = '✅' + group.get('name', '')
    
    print("\nGenerating diagram from GROUP-LEVEL dependencies...")
    
    # Create detailed diagram using group-level dependencies
    td_output = project_root / 'rust-conversion' / 'solid-clean-design' / 'group-dependencies-detailed.mmd'
    
    create_detailed_diagram(grouper.groups, dict(grouper.group_dependencies), str(td_output))
    
    # Summary
    circular_edges = grouper.detect_circular_dependencies()
    total_deps = sum(len(deps) for deps in grouper.group_dependencies.values())
    print(f"\n=== SUMMARY ===")
    print(f"Total groups: {len(grouper.groups)}")
    print(f"Total GROUP-LEVEL dependencies: {total_deps}")
    print(f"Circular GROUP-LEVEL dependencies: {len(circular_edges)}")
    if total_deps > 0:
        print(f"Percentage circular: {len(circular_edges)/total_deps*100:.1f}%")
    if len(circular_edges) == 0:
        print("\n✅ Behavior groups have ZERO circular dependencies!")
        print("   The grouping structure is acyclic (DAG).")

if __name__ == '__main__':
    main()
