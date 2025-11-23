# Behavior Group Dependencies - Summary

This directory contains multiple Mermaid diagrams showing dependencies between behavior groups in the Erlang/OTP C codebase.

## Generated Diagrams

1. **group-dependencies.mmd** - Original left-to-right (LR) layout with all dependencies (116 total)
2. **group-dependencies-td.mmd** - Top-down (TD) layout for better hierarchy visualization
3. **group-dependencies-layered.mmd** - Grouped by architectural layers (Foundational, Core, Service, Integration, Utility)
4. **group-dependencies-direct.mmd** - Direct dependencies only (transitive dependencies removed - very few remain)
5. **group-dependencies-strong-2.mmd** - Strong dependencies appearing in 2+ files
6. **group-dependencies-strong-3.mmd** - Strong dependencies appearing in 3+ files

## Architecture Layers

### Foundational Layer (3 groups)
- **Memory Management** - 107 functions, 15 files
- **Term Handling** - 333 functions, 13 files  
- **Maps** - 76 functions, 3 files

### Core Layer (4 groups)
- **Process Management** - 45 functions, 5 files
- **Code Management** - 246 functions, 58 files
- **BIFs** - 416 functions, 16 files
- **Scheduling** - 5 functions, 1 file

### Service Layer (5 groups)
- **I/O Ports** - 229 functions, 29 files
- **Distribution** - 91 functions, 3 files
- **Time Management** - 124 functions, 13 files
- **Debugging/Tracing** - 70 functions, 11 files
- **ETS Tables** - 343 functions, 11 files

### Integration Layer (3 groups)
- **System Integration** - 1992 functions, 243 files
- **Drivers** - 200 functions, 17 files
- **NIFs** - 755 functions, 26 files

### Utility Layer (1 group)
- **Utils** - 172 functions, 20 files

## Dependency Statistics

- **Total Groups**: 16
- **Total Dependencies**: 116 (all dependencies)
- **Strong Dependencies (2+ files)**: 82 dependencies
- **Strong Dependencies (3+ files)**: 63 dependencies
- **Direct Dependencies**: Very few (most are transitive in this highly interconnected system)

## Key Observations

1. **System Integration** is the most central component, with dependencies from almost all other groups
2. **BIFs** is a core dependency used by many groups
3. **Process Management**, **Term Handling**, and **Memory Management** form the foundational layers
4. Most dependencies are transitive, indicating a highly interconnected architecture
5. The layered diagram shows clear separation between foundational/core layers and service/integration layers

## Usage

To view these diagrams:
- Use any Mermaid-compatible viewer (GitHub, GitLab, Mermaid Live Editor, VS Code extensions)
- Or use online tools like https://mermaid.live/

