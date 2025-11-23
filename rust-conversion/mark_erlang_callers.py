#!/usr/bin/env python3
"""
Add '*' prefix to behavior group names that contain functions called from Erlang.
"""

import json
from pathlib import Path

def main():
    project_root = Path('/Volumes/Files_1/iron-beam')
    mapping_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'behavior-groups-mapping.jsonld'
    analysis_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'c_analysis_results.json'
    
    # Load files
    print("Loading behavior groups mapping...")
    with open(mapping_file, 'r') as f:
        mapping = json.load(f)
    
    print("Loading analysis results...")
    with open(analysis_file, 'r') as f:
        analysis = json.load(f)
    
    # Get external callers (Erlang callers)
    external_callers = analysis.get('external_callers', [])
    erlang_callers = [c for c in external_callers if c.get('caller_language') == 'erlang']
    
    # Build set of files that are called from Erlang
    files_called_from_erlang = set()
    for caller in erlang_callers:
        files_called_from_erlang.add(caller.get('c_file', ''))
    
    print(f"Found {len(files_called_from_erlang)} files called from Erlang")
    
    # Find which behavior groups contain these files and update their names
    groups_updated = 0
    
    for node in mapping['@graph']:
        if 'BehaviorGroup' in node.get('@id', ''):
            group_id = node['@id']
            group_files = set(node.get('files', []))
            
            # Check if any file in this group is called from Erlang
            if group_files.intersection(files_called_from_erlang):
                name = node.get('name', 'unknown')
                if not name.startswith('*'):
                    node['name'] = f'*{name}'
                    groups_updated += 1
                    print(f"  Updated: {group_id} -> *{name}")
    
    # Save updated mapping
    print(f"\nUpdating {mapping_file}...")
    with open(mapping_file, 'w') as f:
        json.dump(mapping, f, indent=2)
    
    print(f"✅ Updated {groups_updated} behavior groups with '*' prefix")
    print(f"Total groups with Erlang callers: {groups_updated}")

if __name__ == '__main__':
    main()

