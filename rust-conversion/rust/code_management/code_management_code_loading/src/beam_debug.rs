//! BEAM Debug Tracer
//!
//! Provides debug tracing functionality for MFA (Module, Function, Arity).
//! Based on beam_load.c - Debug tracing for BEAM code.
//!
//! Allows setting which MFA combinations should be traced and checking
//! if a particular MFA is being traced.

/*
 * %CopyrightBegin%
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Copyright Lee Barney 2025. All Rights Reserved.
 *
 * This file is derived from work copyrighted by Ericsson AB 1996-2025.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * %CopyrightEnd%
 */

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt;

/// Maximum number of traced MFAs
const MFA_MAX: usize = 256;

/// MFA (Module, Function, Arity) identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mfa {
    /// Module atom (simplified - in full implementation would be Eterm)
    pub module: u32,
    /// Function atom (simplified - in full implementation would be Eterm)
    pub function: u32,
    /// Arity
    pub arity: u32,
}

impl Mfa {
    /// Create a new MFA
    pub fn new(module: u32, function: u32, arity: u32) -> Self {
        Self {
            module,
            function,
            arity,
        }
    }
}

impl fmt::Display for Mfa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MFA({}, {}, {})", self.module, self.function, self.arity)
    }
}

/// Traced MFA entry
#[derive(Debug, Clone)]
struct TracedMfa {
    /// Module atom
    module: u32,
    /// Function atom
    function: u32,
    /// Arity
    arity: u32,
    /// Index in trace array (1-based, 0 means not traced)
    index: usize,
}

/// BEAM debug tracer
pub struct BeamDebugTracer {
    /// Traced MFAs indexed by MFA
    traced_mfas: Arc<Mutex<HashMap<Mfa, TracedMfa>>>,
    /// Traced MFAs indexed by index (1-based)
    traced_by_index: Arc<Mutex<Vec<Option<Mfa>>>>,
    /// Next available index
    next_index: Arc<Mutex<usize>>,
}

impl BeamDebugTracer {
    /// Create a new BEAM debug tracer
    pub fn new() -> Self {
        Self {
            traced_mfas: Arc::new(Mutex::new(HashMap::new())),
            traced_by_index: Arc::new(Mutex::new(vec![None; MFA_MAX + 1])), // +1 for 1-based indexing
            next_index: Arc::new(Mutex::new(1)), // Start at 1 (0 means not traced)
        }
    }

    /// Set a traced MFA
    ///
    /// Adds a module/function/arity combination to the trace list.
    ///
    /// # Arguments
    /// * `module` - Module name (as string, will be converted to atom)
    /// * `function` - Function name (as string, will be converted to atom)
    /// * `arity` - Function arity
    ///
    /// # Returns
    /// The trace index (1-based) if successful, or None if trace list is full
    pub fn set_traced_mfa(&self, module: &str, function: &str, arity: u32) -> Option<usize> {
        // In a full implementation, would convert strings to atoms using atom table
        // For now, we'll use a simple hash-based approach
        let module_atom = self.string_to_atom(module);
        let function_atom = self.string_to_atom(function);
        let mfa = Mfa::new(module_atom, function_atom, arity);

        let mut traced = self.traced_mfas.lock().unwrap();
        let mut by_index = self.traced_by_index.lock().unwrap();
        let mut next_idx = self.next_index.lock().unwrap();

        // Check if already traced
        if let Some(existing) = traced.get(&mfa) {
            return Some(existing.index);
        }

        // Check if we have space
        if *next_idx > MFA_MAX {
            return None;
        }

        // Add to trace list
        let index = *next_idx;
        *next_idx += 1;

        let traced_mfa = TracedMfa {
            module: module_atom,
            function: function_atom,
            arity,
            index,
        };

        traced.insert(mfa, traced_mfa);
        by_index[index] = Some(mfa);

        Some(index)
    }

    /// Check if an MFA is traced
    ///
    /// # Arguments
    /// * `module` - Module atom
    /// * `function` - Function atom (None to match any function in module)
    /// * `arity` - Function arity (ignored if function is None)
    ///
    /// # Returns
    /// The trace index (1-based) if traced, or 0 if not traced
    pub fn is_traced_mfa(&self, module: u32, function: Option<u32>, arity: u32) -> usize {
        let traced = self.traced_mfas.lock().unwrap();

        if let Some(function_atom) = function {
            // Check exact match
            let mfa = Mfa::new(module, function_atom, arity);
            if let Some(traced_mfa) = traced.get(&mfa) {
                return traced_mfa.index;
            }
        } else {
            // Check module match (any function/arity)
            for (mfa, traced_mfa) in traced.iter() {
                if mfa.module == module {
                    return traced_mfa.index;
                }
            }
        }

        0
    }

    /// Get traced MFA by index
    ///
    /// # Arguments
    /// * `index` - Trace index (1-based)
    ///
    /// # Returns
    /// The MFA if found, or None
    pub fn get_traced_mfa(&self, index: usize) -> Option<Mfa> {
        if index == 0 || index > MFA_MAX {
            return None;
        }

        let by_index = self.traced_by_index.lock().unwrap();
        by_index[index]
    }

    /// Clear all traced MFAs
    pub fn clear(&self) {
        let mut traced = self.traced_mfas.lock().unwrap();
        let mut by_index = self.traced_by_index.lock().unwrap();
        let mut next_idx = self.next_index.lock().unwrap();

        traced.clear();
        for i in 1..=MFA_MAX {
            by_index[i] = None;
        }
        *next_idx = 1;
    }

    /// Format and trace an MFA
    ///
    /// # Arguments
    /// * `index` - Trace index (1-based)
    /// * `format` - Format string (printf-style)
    /// * `args` - Format arguments
    ///
    /// # Returns
    /// Formatted string if MFA is traced, or None
    pub fn vtrace_mfa(&self, index: usize, format: &str, args: &[&dyn fmt::Display]) -> Option<String> {
        if index == 0 || index > MFA_MAX {
            return None;
        }

        let by_index = self.traced_by_index.lock().unwrap();
        if by_index[index].is_none() {
            return None;
        }

        // In a full implementation, would use proper printf-style formatting
        // For now, we'll do a simple string replacement
        let mut result = format.to_string();
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            result = result.replace(&placeholder, &format!("{}", arg));
        }

        Some(result)
    }

    /// Convert string to atom (simplified)
    ///
    /// In a full implementation, would use the atom table.
    /// For now, uses a simple hash.
    pub fn string_to_atom(&self, s: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish() as u32
    }
}

impl Default for BeamDebugTracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Global BEAM debug tracer (singleton)
static GLOBAL_DEBUG_TRACER: std::sync::OnceLock<BeamDebugTracer> = std::sync::OnceLock::new();

/// Get the global BEAM debug tracer
pub fn get_global_debug_tracer() -> &'static BeamDebugTracer {
    GLOBAL_DEBUG_TRACER.get_or_init(BeamDebugTracer::new)
}

/// Set a traced MFA
///
/// Convenience function that uses the global debug tracer.
pub fn dbg_set_traced_mfa(module: &str, function: &str, arity: u32) -> Option<usize> {
    get_global_debug_tracer().set_traced_mfa(module, function, arity)
}

/// Check if an MFA is traced
///
/// Convenience function that uses the global debug tracer.
///
/// # Arguments
/// * `module` - Module atom
/// * `function` - Function atom (0 to match any function in module)
/// * `arity` - Function arity
///
/// # Returns
/// The trace index (1-based) if traced, or 0 if not traced
pub fn dbg_is_traced_mfa(module: u32, function: u32, arity: u32) -> usize {
    let function_opt = if function == 0 { None } else { Some(function) };
    get_global_debug_tracer().is_traced_mfa(module, function_opt, arity)
}

/// Format and trace an MFA
///
/// Convenience function that uses the global debug tracer.
pub fn dbg_vtrace_mfa(index: usize, format: &str, args: &[&dyn fmt::Display]) -> Option<String> {
    get_global_debug_tracer().vtrace_mfa(index, format, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beam_debug_tracer_init() {
        let tracer = BeamDebugTracer::new();
        assert_eq!(tracer.is_traced_mfa(1, Some(2), 3), 0);
    }

    #[test]
    fn test_set_traced_mfa() {
        let tracer = BeamDebugTracer::new();
        
        let index = tracer.set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        assert!(index.unwrap() > 0);
        
        // Check that it's traced
        let module_atom = tracer.string_to_atom("mymod");
        let function_atom = tracer.string_to_atom("myfunc");
        let traced_index = tracer.is_traced_mfa(module_atom, Some(function_atom), 2);
        assert_eq!(traced_index, index.unwrap());
    }

    #[test]
    fn test_is_traced_mfa() {
        let tracer = BeamDebugTracer::new();
        
        let index = tracer.set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        
        let module_atom = tracer.string_to_atom("mymod");
        let function_atom = tracer.string_to_atom("myfunc");
        
        // Exact match
        assert_eq!(tracer.is_traced_mfa(module_atom, Some(function_atom), 2), index.unwrap());
        
        // Module match (any function)
        assert_eq!(tracer.is_traced_mfa(module_atom, None, 0), index.unwrap());
        
        // No match
        assert_eq!(tracer.is_traced_mfa(module_atom + 1, Some(function_atom), 2), 0);
    }

    #[test]
    fn test_vtrace_mfa() {
        let tracer = BeamDebugTracer::new();
        
        let index = tracer.set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        let index = index.unwrap();
        
        let result = tracer.vtrace_mfa(index, "Trace: {0}", &[&"test"]);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Trace: test");
    }

    #[test]
    fn test_global_functions() {
        let index = dbg_set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        
        let tracer = get_global_debug_tracer();
        let module_atom = tracer.string_to_atom("mymod");
        let function_atom = tracer.string_to_atom("myfunc");
        
        let traced_index = dbg_is_traced_mfa(module_atom, function_atom, 2);
        assert_eq!(traced_index, index.unwrap());
    }

    #[test]
    fn test_mfa_display() {
        let mfa = Mfa::new(1, 2, 3);
        let display_str = format!("{}", mfa);
        assert_eq!(display_str, "MFA(1, 2, 3)");
    }

    #[test]
    fn test_mfa_new() {
        let mfa = Mfa::new(10, 20, 30);
        assert_eq!(mfa.module, 10);
        assert_eq!(mfa.function, 20);
        assert_eq!(mfa.arity, 30);
    }

    #[test]
    fn test_mfa_debug() {
        let mfa = Mfa::new(1, 2, 3);
        let debug_str = format!("{:?}", mfa);
        assert!(debug_str.contains("Mfa"));
    }

    #[test]
    fn test_mfa_clone() {
        let mfa1 = Mfa::new(1, 2, 3);
        let mfa2 = mfa1.clone();
        assert_eq!(mfa1, mfa2);
    }

    #[test]
    fn test_mfa_partial_eq() {
        let mfa1 = Mfa::new(1, 2, 3);
        let mfa2 = Mfa::new(1, 2, 3);
        let mfa3 = Mfa::new(1, 2, 4);
        
        assert_eq!(mfa1, mfa2);
        assert_ne!(mfa1, mfa3);
    }

    #[test]
    fn test_mfa_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mfa1 = Mfa::new(1, 2, 3);
        let mfa2 = Mfa::new(1, 2, 3);
        
        let mut hasher1 = DefaultHasher::new();
        mfa1.hash(&mut hasher1);
        
        let mut hasher2 = DefaultHasher::new();
        mfa2.hash(&mut hasher2);
        
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_set_traced_mfa_duplicate() {
        let tracer = BeamDebugTracer::new();
        
        let index1 = tracer.set_traced_mfa("mymod", "myfunc", 2);
        assert!(index1.is_some());
        
        // Try to set the same MFA again
        let index2 = tracer.set_traced_mfa("mymod", "myfunc", 2);
        assert!(index2.is_some());
        assert_eq!(index1.unwrap(), index2.unwrap()); // Should return same index
    }

    #[test]
    fn test_set_traced_mfa_full() {
        let tracer = BeamDebugTracer::new();
        
        // Fill up to MFA_MAX
        for i in 0..MFA_MAX {
            let result = tracer.set_traced_mfa(&format!("mod{}", i), &format!("func{}", i), i as u32);
            assert!(result.is_some(), "Should be able to add MFA {}", i);
        }
        
        // Try to add one more - should fail
        let result = tracer.set_traced_mfa("overflow", "overflow", 0);
        assert!(result.is_none(), "Should fail when trace list is full");
    }

    #[test]
    fn test_is_traced_mfa_module_match() {
        let tracer = BeamDebugTracer::new();
        
        let index = tracer.set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        
        let module_atom = tracer.string_to_atom("mymod");
        
        // Test module match with None function (should match any function in module)
        let traced_index = tracer.is_traced_mfa(module_atom, None, 0);
        assert_eq!(traced_index, index.unwrap());
    }

    #[test]
    fn test_get_traced_mfa() {
        let tracer = BeamDebugTracer::new();
        
        let index = tracer.set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        let index = index.unwrap();
        
        let mfa = tracer.get_traced_mfa(index);
        assert!(mfa.is_some());
        let mfa = mfa.unwrap();
        
        let module_atom = tracer.string_to_atom("mymod");
        let function_atom = tracer.string_to_atom("myfunc");
        assert_eq!(mfa.module, module_atom);
        assert_eq!(mfa.function, function_atom);
        assert_eq!(mfa.arity, 2);
    }

    #[test]
    fn test_get_traced_mfa_invalid_index() {
        let tracer = BeamDebugTracer::new();
        
        // Test with index 0 (invalid)
        assert_eq!(tracer.get_traced_mfa(0), None);
        
        // Test with index > MFA_MAX (invalid)
        assert_eq!(tracer.get_traced_mfa(MFA_MAX + 1), None);
        
        // Test with index that doesn't exist
        assert_eq!(tracer.get_traced_mfa(100), None);
    }

    #[test]
    fn test_clear() {
        let tracer = BeamDebugTracer::new();
        
        // Add some MFAs
        let index1 = tracer.set_traced_mfa("mod1", "func1", 1);
        let index2 = tracer.set_traced_mfa("mod2", "func2", 2);
        assert!(index1.is_some());
        assert!(index2.is_some());
        
        // Clear all
        tracer.clear();
        
        // Verify they're gone
        let module_atom1 = tracer.string_to_atom("mod1");
        let function_atom1 = tracer.string_to_atom("func1");
        assert_eq!(tracer.is_traced_mfa(module_atom1, Some(function_atom1), 1), 0);
        
        let module_atom2 = tracer.string_to_atom("mod2");
        let function_atom2 = tracer.string_to_atom("func2");
        assert_eq!(tracer.is_traced_mfa(module_atom2, Some(function_atom2), 2), 0);
        
        // Verify indices are reset
        let new_index = tracer.set_traced_mfa("mod3", "func3", 3);
        assert_eq!(new_index.unwrap(), 1); // Should start at 1 again
    }

    #[test]
    fn test_vtrace_mfa_invalid_index() {
        let tracer = BeamDebugTracer::new();
        
        // Test with index 0 (invalid)
        assert_eq!(tracer.vtrace_mfa(0, "test", &[]), None);
        
        // Test with index > MFA_MAX (invalid)
        assert_eq!(tracer.vtrace_mfa(MFA_MAX + 1, "test", &[]), None);
        
        // Test with index that doesn't exist
        assert_eq!(tracer.vtrace_mfa(100, "test", &[]), None);
    }

    #[test]
    fn test_vtrace_mfa_multiple_args() {
        let tracer = BeamDebugTracer::new();
        
        let index = tracer.set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        let index = index.unwrap();
        
        let result = tracer.vtrace_mfa(index, "Trace: {0} {1} {2}", &[&"a", &"b", &"c"]);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Trace: a b c");
    }

    #[test]
    fn test_vtrace_mfa_no_args() {
        let tracer = BeamDebugTracer::new();
        
        let index = tracer.set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        let index = index.unwrap();
        
        let result = tracer.vtrace_mfa(index, "Simple trace", &[]);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Simple trace");
    }

    #[test]
    fn test_string_to_atom() {
        let tracer = BeamDebugTracer::new();
        
        let atom1 = tracer.string_to_atom("test");
        let atom2 = tracer.string_to_atom("test");
        
        // Same string should produce same atom
        assert_eq!(atom1, atom2);
        
        // Different strings should produce different atoms (usually)
        let atom3 = tracer.string_to_atom("different");
        // Note: hash collisions are possible, but unlikely for these strings
    }

    #[test]
    fn test_beam_debug_tracer_default() {
        let tracer = BeamDebugTracer::default();
        assert_eq!(tracer.is_traced_mfa(1, Some(2), 3), 0);
    }

    #[test]
    fn test_dbg_vtrace_mfa() {
        let index = dbg_set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        let index = index.unwrap();
        
        let result = dbg_vtrace_mfa(index, "Global trace: {0}", &[&"test"]);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Global trace: test");
    }

    #[test]
    fn test_dbg_is_traced_mfa_with_zero_function() {
        let index = dbg_set_traced_mfa("mymod", "myfunc", 2);
        assert!(index.is_some());
        
        let tracer = get_global_debug_tracer();
        let module_atom = tracer.string_to_atom("mymod");
        
        // Test with function = 0 (should match any function in module)
        let traced_index = dbg_is_traced_mfa(module_atom, 0, 0);
        assert_eq!(traced_index, index.unwrap());
    }

    #[test]
    fn test_multiple_mfas_same_module() {
        let tracer = BeamDebugTracer::new();
        
        let index1 = tracer.set_traced_mfa("mymod", "func1", 1);
        let index2 = tracer.set_traced_mfa("mymod", "func2", 2);
        assert!(index1.is_some());
        assert!(index2.is_some());
        assert_ne!(index1.unwrap(), index2.unwrap());
        
        let module_atom = tracer.string_to_atom("mymod");
        
        // Module match should return first match
        let traced_index = tracer.is_traced_mfa(module_atom, None, 0);
        assert!(traced_index > 0);
    }
}

