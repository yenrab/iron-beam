# Circular Dependencies Diagram

This Mermaid diagram highlights circular dependencies between behavior groups in red.

## File

**group-dependencies-circular.mmd** - Diagram with circular dependencies highlighted in red

## Key Features

- **Red edges (thick, 3px)**: Circular dependencies - edges that are part of dependency cycles
- **Black edges (thin, 1px)**: Normal dependencies - not part of any cycle
- **Edge labels**: Circular edges are labeled with "circular" for clarity

## Statistics

- **Total dependencies**: 116
- **Circular dependencies**: 115 edges
- **Non-circular dependencies**: 1 edge

## Observations

The high number of circular dependencies (115 out of 116) indicates that the Erlang/OTP codebase is highly interconnected with many bidirectional dependencies. This is common in complex runtime systems where:

1. Core components (Process Management, Memory Management, BIFs) depend on each other
2. Service layers (I/O, Distribution, ETS) have mutual dependencies
3. Integration layers (System Integration, NIFs, Drivers) form dependency cycles

## Usage

View this diagram in any Mermaid-compatible viewer:
- GitHub/GitLab (renders automatically)
- VS Code with Mermaid extension
- Online: https://mermaid.live/
- Mermaid CLI: `mmdc -i group-dependencies-circular.mmd -o output.svg`

## Note

The cycle detection uses Tarjan's algorithm to find strongly connected components. Any edge that is part of a cycle (even if the cycle involves multiple nodes) is marked as circular.

