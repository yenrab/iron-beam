#!/usr/bin/env python3
"""
Update Mermaid diagrams to include ✅ emoji for groups called from Erlang.
"""

import json
import re
from pathlib import Path

def load_behavior_groups(mapping_file: str) -> dict:
    """Load behavior groups and identify which have ✅ emoji."""
    with open(mapping_file, 'r') as f:
        data = json.load(f)
    
    groups_with_emoji = {}
    for node in data['@graph']:
        if 'BehaviorGroup' in node.get('@id', ''):
            group_id = node['@id']
            name = node.get('name', '')
            if '✅' in name:
                groups_with_emoji[group_id] = name
    
    return groups_with_emoji

def get_group_name_from_id(group_id: str, mapping_file: str) -> str:
    """Get the full name (with emoji) for a group ID."""
    with open(mapping_file, 'r') as f:
        data = json.load(f)
    
    for node in data['@graph']:
        if node.get('@id') == group_id:
            return node.get('name', 'unknown')
    return 'unknown'

def update_mermaid_file(mmd_file: str, groups_with_emoji: dict, mapping_file: str):
    """Update Mermaid file to include ✅ emoji in node labels."""
    with open(mmd_file, 'r') as f:
        content = f.read()
    
    # Create a mapping from clean names to group IDs
    # We need to match group names in the diagram to group IDs
    name_to_id = {}
    with open(mapping_file, 'r') as f:
        data = json.load(f)
    
    for node in data['@graph']:
        if 'BehaviorGroup' in node.get('@id', ''):
            name = node.get('name', '')
            # Remove emoji if present for matching
            clean_name = name.replace('✅', '').strip()
            # Convert to the format used in Mermaid (title case, no underscores)
            mermaid_name = clean_name.replace('_', ' ').title().replace(' ', '')
            name_to_id[mermaid_name] = node['@id']
    
    # Find all node definitions in the Mermaid file
    # Pattern: NodeName["Label<br/>..."]
    pattern = r'(\w+)\["([^"]+)"\]'
    
    def replace_node(match):
        node_name = match.group(1)
        label = match.group(2)
        
        # Check if this node corresponds to a group with emoji
        # Try to find the group ID by matching the label
        group_id = None
        for mermaid_name, gid in name_to_id.items():
            if mermaid_name == node_name or node_name.startswith(mermaid_name):
                group_id = gid
                break
        
        if group_id and group_id in groups_with_emoji:
            # Check if emoji is already in the label
            if '✅' not in label:
                # Add emoji at the beginning of the label
                # Find where the actual name starts (after any existing formatting)
                lines = label.split('<br/>')
                if lines:
                    # Add emoji to first line
                    lines[0] = '✅' + lines[0]
                    label = '<br/>'.join(lines)
        
        return f'{node_name}["{label}"]'
    
    updated_content = re.sub(pattern, replace_node, content)
    
    # Write back
    with open(mmd_file, 'w') as f:
        f.write(updated_content)
    
    return updated_content != content

def main():
    project_root = Path('/Volumes/Files_1/iron-beam')
    mapping_file = project_root / 'rust-conversion' / 'solid-clean-design' / 'behavior-groups-mapping.jsonld'
    
    print("Loading behavior groups...")
    groups_with_emoji = load_behavior_groups(str(mapping_file))
    print(f"Found {len(groups_with_emoji)} groups with ✅ emoji")
    
    # Find all .mmd files
    mmd_files = list((project_root / 'rust-conversion' / 'solid-clean-design').glob('*.mmd'))
    
    print(f"\nUpdating {len(mmd_files)} Mermaid diagram files...")
    for mmd_file in mmd_files:
        print(f"  Updating {mmd_file.name}...")
        updated = update_mermaid_file(str(mmd_file), groups_with_emoji, str(mapping_file))
        if updated:
            print(f"    ✅ Updated")
        else:
            print(f"    (no changes needed)")
    
    print("\n✅ All Mermaid diagrams updated!")

if __name__ == '__main__':
    main()

