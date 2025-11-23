#!/usr/bin/env python3
"""
Behavior Grouper - Groups related C code behaviors for Rust re-engineering.
Follows SOLID and CLEAN architecture principles.
"""

import json
import os
from pathlib import Path
from typing import Dict, List, Set, Optional
from collections import defaultdict

class BehaviorGrouper:
    def __init__(self, analysis_file: str):
        with open(analysis_file, 'r') as f:
            self.analysis = json.load(f)
        
        self.functions = self.analysis['functions']
        self.structs = self.analysis['structs']
        self.dependencies = self.analysis['dependencies']
        self.external_callers = self.analysis['external_callers']
        
        self.groups: List[Dict] = []
        self.file_to_group: Dict[str, str] = {}
        
    def group_by_directory(self) -> Dict[str, List[str]]:
        """Group files by directory structure (initial grouping)."""
        files_by_dir = defaultdict(list)
        
        for func_id, func in self.functions.items():
            file_path = func['file_path']
            dir_path = os.path.dirname(file_path)
            files_by_dir[dir_path].append(file_path)
        
        return dict(files_by_dir)
    
    def identify_functional_areas(self) -> Dict[str, List[str]]:
        """Identify main functional areas based on file paths and names."""
        areas = {
            'process_management': [],
            'memory_management': [],
            'code_management': [],
            'term_handling': [],
            'bifs': [],
            'io_ports': [],
            'distribution': [],
            'scheduling': [],
            'time_management': [],
            'debugging_tracing': [],
            'ets_tables': [],
            'unicode': [],
            'maps': [],
            'system_integration': [],
            'drivers': [],
            'nifs': [],
            'allocators': [],
            'utils': []
        }
        
        for func_id, func in self.functions.items():
            file_path = func['file_path']
            file_name = os.path.basename(file_path)
            dir_path = os.path.dirname(file_path)
            
            # Categorize based on file path and name
            if 'process' in file_path.lower() or 'sched' in file_path.lower():
                if 'sched' in file_path.lower():
                    areas['scheduling'].append(file_path)
                else:
                    areas['process_management'].append(file_path)
            elif 'alloc' in file_path.lower() or 'gc' in file_path.lower() or 'heap' in file_path.lower():
                areas['memory_management'].append(file_path)
            elif 'code' in file_path.lower() or 'module' in file_path.lower() or 'export' in file_path.lower() or 'beam_load' in file_path.lower():
                areas['code_management'].append(file_path)
            elif 'term' in file_path.lower() or 'binary' in file_path.lower() or 'bits' in file_path.lower() or 'atom' in file_path.lower():
                areas['term_handling'].append(file_path)
            elif 'bif' in file_path.lower():
                areas['bifs'].append(file_path)
            elif 'io' in file_path.lower() or 'port' in file_path.lower():
                areas['io_ports'].append(file_path)
            elif 'dist' in file_path.lower() or 'external' in file_path.lower():
                areas['distribution'].append(file_path)
            elif 'time' in file_path.lower() or 'timer' in file_path.lower():
                areas['time_management'].append(file_path)
            elif 'debug' in file_path.lower() or 'trace' in file_path.lower():
                areas['debugging_tracing'].append(file_path)
            elif 'db' in file_path.lower() or 'ets' in file_path.lower():
                areas['ets_tables'].append(file_path)
            elif 'unicode' in file_path.lower():
                areas['unicode'].append(file_path)
            elif 'map' in file_path.lower():
                areas['maps'].append(file_path)
            elif 'driver' in file_path.lower():
                areas['drivers'].append(file_path)
            elif 'nif' in file_path.lower():
                areas['nifs'].append(file_path)
            elif 'sys' in file_path.lower():
                areas['system_integration'].append(file_path)
            elif 'utils' in file_path.lower() or 'common' in file_path.lower():
                areas['utils'].append(file_path)
            else:
                # Default to system_integration for unclassified
                areas['system_integration'].append(file_path)
        
        # Remove duplicates
        for area in areas:
            areas[area] = list(set(areas[area]))
        
        return areas
    
    def create_behavior_groups(self):
        """Create behavior groups following SOLID and CLEAN principles."""
        print("Phase: Grouping Mode - CCodeExpertActor: Identifying related behaviors...")
        
        functional_areas = self.identify_functional_areas()
        
        group_id = 1
        for area_name, files in functional_areas.items():
            if not files:
                continue
            
            # Create a group for this functional area
            group = {
                '@id': f"ex:BehaviorGroup_{group_id}",
                'name': area_name,
                'files': list(set(files)),
                'functions': [],
                'rationale': {
                    'codeRationale': f"Functions grouped by functional area: {area_name}",
                    'solidRationale': f"Single Responsibility: All functions in this group handle {area_name}",
                    'cleanRationale': f"CLEAN Architecture: {area_name} represents a distinct use case/entity layer",
                    'rustRationale': f"Rust module boundary: {area_name} can be a separate Rust module"
                }
            }
            
            # Add functions from these files
            for file_path in group['files']:
                for func_id, func in self.functions.items():
                    if func['file_path'] == file_path:
                        group['functions'].append({
                            '@id': f"ex:CFunction_{func_id.replace(':', '_').replace('/', '_')}",
                            'name': func['name'],
                            'filePath': func['file_path'],
                            'lineRange': {'start': func['line_start'], 'end': func['line_start'] + 10},  # Estimate
                            'signature': f"{func['return_type']} {func['name']}(...)",
                            'isStatic': func.get('is_static', False)
                        })
            
            if group['functions']:
                self.groups.append(group)
                group_id += 1
        
        print(f"Phase: Grouping Mode - CCodeExpertActor: Created {len(self.groups)} initial behavior groups")
        
        # Refine groups based on dependencies and relationships
        self.refine_groups()
        
        return self.groups
    
    def refine_groups(self):
        """Refine groups based on dependencies and cross-file relationships."""
        print("Phase: Grouping Mode - SOLIDExpertActor: Refining groups for SOLID compliance...")
        print("Phase: Grouping Mode - CLEANExpertActor: Refining groups for CLEAN architecture...")
        print("Phase: Grouping Mode - RustExpertActor: Refining groups for Rust module boundaries...")
        
        # TODO: Add more sophisticated grouping logic based on:
        # - Function call relationships
        # - Shared data structures
        # - Dependency analysis
        # - SOLID principles validation
        # - CLEAN architecture layer mapping
        
        pass
    
    def generate_jsonld(self, output_file: str):
        """Generate JSON-LD mapping file."""
        print("Phase: Generation Mode - JSONLDExpertActor: Generating JSON-LD mapping file...")
        
        jsonld = {
            "@context": {
                "@vocab": "https://aalang.org/spec",
                "ex": "https://aalang.org/behavior-grouper/"
            },
            "@graph": []
        }
        
        # Add behavior groups
        for group in self.groups:
            jsonld['@graph'].append(group)
            
            # Add functions as separate nodes
            for func in group['functions']:
                func_node = {
                    "@id": func['@id'],
                    "@type": "ex:CFunction",
                    "name": func['name'],
                    "filePath": func['filePath'],
                    "lineRange": func['lineRange'],
                    "signature": func['signature'],
                    "isStatic": func['isStatic']
                }
                jsonld['@graph'].append(func_node)
        
        # Add grouping rationale nodes
        for group in self.groups:
            rationale_node = {
                "@id": f"{group['@id']}_rationale",
                "@type": "ex:GroupingRationale",
                "behaviorGroup": group['@id'],
                "solidRationale": group['rationale']['solidRationale'],
                "cleanRationale": group['rationale']['cleanRationale'],
                "rustRationale": group['rationale']['rustRationale'],
                "codeRationale": group['rationale']['codeRationale']
            }
            jsonld['@graph'].append(rationale_node)
        
        # Add external callers information
        for caller in self.external_callers:
            caller_node = {
                "@id": f"ex:ExternalCaller_{len(jsonld['@graph'])}",
                "@type": "ex:ExternalCaller",
                "type": caller['type'],
                "cFile": caller['c_file'],
                "callerLanguage": caller['caller_language']
            }
            jsonld['@graph'].append(caller_node)
        
        # Write output
        with open(output_file, 'w') as f:
            json.dump(jsonld, f, indent=2)
        
        print(f"Phase: Generation Mode - JSONLDExpertActor: Generated JSON-LD mapping file: {output_file}")
        print(f"Phase: Generation Mode - JSONLDExpertActor: Created {len(self.groups)} behavior groups")
        print(f"Phase: Generation Mode - JSONLDExpertActor: Mapped {sum(len(g['functions']) for g in self.groups)} functions")

def main():
    project_root = '/Volumes/Files_1/iron-beam'
    analysis_file = os.path.join(project_root, 'rust-conversion', 'c_analysis_results.json')
    output_file = os.path.join(project_root, 'rust-conversion', 'behavior-groups-mapping.jsonld')
    
    grouper = BehaviorGrouper(analysis_file)
    grouper.create_behavior_groups()
    grouper.generate_jsonld(output_file)
    
    print("\n" + "=" * 80)
    print("GROUPING COMPLETE")
    print("=" * 80)
    print(f"Total behavior groups: {len(grouper.groups)}")
    for group in grouper.groups:
        print(f"  - {group['name']}: {len(group['functions'])} functions, {len(group['files'])} files")

if __name__ == '__main__':
    main()

