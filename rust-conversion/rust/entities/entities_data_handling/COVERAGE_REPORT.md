# Code Coverage Report
## entities_data_handling Crate

**Generated:** 2025-11-27  
**Tool:** cargo-tarpaulin  
**Overall Coverage:** 88.24% (930/1054 lines)

---

## Summary by Module

| Module | Lines Covered | Total Lines | Coverage % |
|--------|--------------|-------------|------------|
| **map.rs** | 56/56 | 56 | **100.00%** ✅ |
| **binary.rs** | 3/3 | 3 | **100.00%** ✅ |
| **atomics.rs** | 8/9 | 9 | **88.89%** ✅ |
| **bits.rs** | 132/134 | 134 | **98.51%** ✅ |
| **atom.rs** | 36/59 | 59 | **61.02%** ⚠️ |
| **term_hashing.rs** | 695/772 | 772 | **90.03%** ✅ |

---

## map.rs Detailed Coverage

### Coverage: 100.00% (56/56 lines) ✅

**Uncovered Lines:** None

All lines are covered, including the `Default` trait implementation.

### Test Coverage Details

All public API methods are fully covered:
- ✅ `new()` - 100% covered
- ✅ `size()` - 100% covered
- ✅ `is_empty()` - 100% covered
- ✅ `is_key()` - 100% covered
- ✅ `get()` - 100% covered
- ✅ `find()` - 100% covered
- ✅ `put()` - 100% covered
- ✅ `update()` - 100% covered (including error path)
- ✅ `remove()` - 100% covered
- ✅ `take()` - 100% covered
- ✅ `keys()` - 100% covered
- ✅ `values()` - 100% covered
- ✅ `to_list()` - 100% covered
- ✅ `from_list()` - 100% covered (including duplicate key handling)
- ✅ `merge()` - 100% covered
- ✅ `find_index()` - 100% covered (through all public methods)

### Test Cases (15 tests, all passing)

1. ✅ `test_map_creation` - Tests empty map creation
2. ✅ `test_map_put_and_get` - Tests basic put/get operations
3. ✅ `test_map_remove` - Tests key removal
4. ✅ `test_map_update` - Tests update operation (success and error paths)
5. ✅ `test_map_take` - Tests take operation
6. ✅ `test_map_find` - Tests find operation
7. ✅ `test_map_keys_and_values` - Tests collection operations
8. ✅ `test_map_to_list` - Tests list conversion
9. ✅ `test_map_from_list` - Tests list creation
10. ✅ `test_map_from_list_duplicates` - Tests duplicate key handling
11. ✅ `test_map_merge` - Tests map merging
12. ✅ `test_map_atom_keys` - Tests with atom keys
13. ✅ `test_map_tuple_keys` - Tests with tuple keys
14. ✅ `test_map_multiple_operations` - Tests complex scenarios
15. ✅ `test_map_default` - Tests Default trait implementation

---

## Other Modules Coverage

### binary.rs: 100.00% ✅
- All 3 lines covered
- Basic binary operations fully tested

### atomics.rs: 88.89% ✅
- 8/9 lines covered
- One uncovered line (line 44) is a platform-specific check

### bits.rs: 98.51% ✅
- 132/134 lines covered
- Only 2 uncovered lines (223, 324) - likely edge case error paths

### atom.rs: 61.02% ⚠️
- 36/59 lines covered
- Several uncovered lines in validation and error handling paths
- **TODO:** Implement full UTF-8 validation (line 206)

### term_hashing.rs: 90.03% ✅
- 695/772 lines covered
- Most uncovered lines are in edge cases and platform-specific code paths

---

## Recommendations

### map.rs
- ✅ **Perfect coverage** - 100.00% coverage achieved
- All code paths are tested, including Default trait implementation
- No action needed

### atom.rs
- ⚠️ **Needs improvement** - 61.02% coverage
- Add tests for UTF-8 validation edge cases
- Complete the TODO for full UTF-8 validation implementation

### Overall
- ✅ **88.24% overall coverage** exceeds the 85% non-critical path target
- All critical path functions (those called from Erlang) should be at 100%
- Continue maintaining high test coverage as new features are added

---

## Test Execution Summary

**Total Tests:** 131 tests (129 unit tests + 2 integration tests)  
**Passing:** 131 ✅  
**Failing:** 0  
**Ignored:** 0

All tests pass successfully, ensuring code quality and correctness.

---

## HTML Coverage Report

A detailed HTML coverage report has been generated at:
`/Volumes/Files_1/iron-beam/rust-conversion/rust/coverage-html/tarpaulin-report.html`

You can open this file in a web browser to see line-by-line coverage details with color-coded source code.

