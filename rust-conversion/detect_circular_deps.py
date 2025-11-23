#!/usr/bin/env python3
"""
Detect circular dependencies using Tarjan's algorithm.
"""

from collections import defaultdict, deque
from typing import Dict, Set, Tuple, List

def detect_circular_dependencies(group_deps: Dict[str, Set[str]]) -> Set[Tuple[str, str]]:
    """
    Detect circular dependencies using Tarjan's strongly connected components algorithm.
    Returns set of (source, target) tuples representing circular edges.
    """
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
            
            # If SCC has more than one node, it's a cycle
            if len(scc) > 1:
                # Mark all edges within the SCC as circular
                for node in scc:
                    for neighbor in group_deps.get(node, set()):
                        if neighbor in scc and neighbor != node:
                            circular_edges.add((node, neighbor))
    
    # Process all nodes
    for node in group_deps.keys():
        if node not in indices:
            strongconnect(node)
    
    return circular_edges

