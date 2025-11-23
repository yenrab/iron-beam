#!/usr/bin/env python3
"""
Create a diagram showing only "strong" dependencies - those that appear
in multiple files or represent significant coupling.
"""

import json
from pathlib import Path
from collections import defaultdict
from typing import Dict, Set

def load_behavior_groups(mapping_file: str) -> Dict[str, Dict]:
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

def analyze_strong_dependencies(analysis_file: str, groups: Dict[str, Dict], 
                                file_to_group: Dict[str, str], header_map: Dict[str, str],
                                project_root: Path, min_files: int = 2) -> Dict[str, Set[str]]:
    """Analyze dependencies and return only those that appear in multiple files."""
    from analyze_group_dependencies import load_analysis_results, find_header_file, find_group_for_header
    
    analysis_results = load_analysis_results(analysis_file)
    file_dependencies = analysis_results.get('dependencies', {})
    
    # Count how many files in each group depend on each other group
    dep_counts = defaultdict(lambda: defaultdict(int))
    
    for group_id, group in groups.items():
        group_files = set(group.get('files', []))
        for file_path in group_files:
            deps = file_dependencies.get(file_path, [])
            for dep_header in deps:
                dep_file = find_header_file(dep_header, header_map, project_root)
                if dep_file:
                    dep_group = find_group_for_header(dep_file, file_to_group, project_root)
                    if dep_group and dep_group != group_id:
                        dep_counts[group_id][dep_group] += 1
    
    # Only keep dependencies that appear in at least min_files files
    strong_deps = {}
    for group_id, deps in dep_counts.items():
        strong = {dep_group for dep_group, count in deps.items() if count >= min_files}
        if strong:
            strong_deps[group_id] = strong
    
    return strong_deps

def create_strong_deps_diagram(groups: Dict[str, Dict], strong_deps: Dict[str, Set[str]], 
                               output_file: str, min_files: int):
    """Create diagram with only strong dependencies."""
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
    
    # Add edges (only strong dependencies)
    for group_id, deps in strong_deps.items():
        if group_id not in group_names:
            continue
        source = group_names[group_id]
        for dep_group_id in deps:
            if dep_group_id in group_names:
                target = group_names[dep_group_id]
                mermaid.append(f"    {source} --> {target}")
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(mermaid))
    
    print(f"Strong dependencies diagram written to {output_file} (min {min_files} files per dependency)")
    print(f"Total strong dependencies: {sum(len(deps) for deps in strong_deps.values())}")

def main():
    project_root = Path('/Volumes/Files_1/iron-beam')
    mapping_file = project_root / 'rust-conversion' / 'behavior-groups-mapping.jsonld'
    analysis_file = project_root / 'rust-conversion' / 'c_analysis_results.json'
    
    print("Loading behavior groups...")
    groups, file_to_group = load_behavior_groups(str(mapping_file))
    
    print("Building header map...")
    from analyze_group_dependencies import build_header_to_file_map
    header_map = build_header_to_file_map(project_root)
    
    print("Analyzing strong dependencies (min 2 files)...")
    strong_deps_2 = analyze_strong_dependencies(str(analysis_file), groups, file_to_group, 
                                               header_map, project_root, min_files=2)
    
    print("Analyzing strong dependencies (min 3 files)...")
    strong_deps_3 = analyze_strong_dependencies(str(analysis_file), groups, file_to_group,
                                               header_map, project_root, min_files=3)
    
    # Create diagrams
    output_file_2 = project_root / 'rust-conversion' / 'group-dependencies-strong-2.mmd'
    create_strong_deps_diagram(groups, strong_deps_2, str(output_file_2), min_files=2)
    
    output_file_3 = project_root / 'rust-conversion' / 'group-dependencies-strong-3.mmd'
    create_strong_deps_diagram(groups, strong_deps_3, str(output_file_3), min_files=3)
    
    print(f"\nStrong dependencies (min 2 files): {sum(len(deps) for deps in strong_deps_2.values())}")
    print(f"Strong dependencies (min 3 files): {sum(len(deps) for deps in strong_deps_3.values())}")

if __name__ == '__main__':
    main()

