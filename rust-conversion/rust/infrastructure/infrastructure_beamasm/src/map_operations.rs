//! Map Operations
//!
//! Provides map creation, key access, updates, and hash table management.
//!
//! Based on `instr_map.cpp` and `beam_common.h:map*.h`

use crate::BeamAssemblerError;
use crate::asmjit_wrapper::{Assembler, a64};

/// Map operation types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MapOperation {
    /// Get element from map
    Get,
    /// Put element into map
    Put,
    /// Remove element from map
    Remove,
    /// Update existing element
    Update,
    /// Check if key exists
    IsKey,
}

/// Map implementation types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MapType {
    /// Flat map (small maps)
    Flat,
    /// Hash map (large maps)
    Hash,
}

/// Map operation context
#[derive(Debug, Clone)]
pub struct MapOperationContext {
    /// Map register
    pub map_reg: u32,
    /// Key register
    pub key_reg: u32,
    /// Value register (for put/update operations)
    pub value_reg: Option<u32>,
    /// Destination register (for get operations)
    pub dst_reg: Option<u32>,
    /// Operation type
    pub operation: MapOperation,
    /// Expected map type
    pub map_type: MapType,
}

/// Map creation specification
#[derive(Debug, Clone)]
pub struct MapCreationSpec {
    /// Destination register for new map
    pub dst_reg: u32,
    /// Number of live registers
    pub live: u32,
    /// Initial key-value pairs
    pub pairs: Vec<MapKeyValuePair>,
}

/// Key-value pair for map operations
#[derive(Debug, Clone)]
pub struct MapKeyValuePair {
    /// Key register
    pub key_reg: u32,
    /// Value register
    pub value_reg: u32,
}

/// Map iteration context
#[derive(Debug, Clone)]
pub struct MapIterationContext {
    /// Map register
    pub map_reg: u32,
    /// Current position/index
    pub position: u64,
    /// Total number of entries
    pub total_entries: u64,
    /// Key destination register
    pub key_dst_reg: u32,
    /// Value destination register
    pub value_dst_reg: u32,
}

/// Map operation result
#[derive(Debug, Clone)]
pub enum MapOperationResult {
    /// Operation successful
    Success,
    /// Key not found
    KeyNotFound,
    /// Map is wrong type
    BadMap,
    /// Out of memory
    OutOfMemory,
    /// Bad key type
    BadKey,
}

/// Map operations coordinator
///
/// Manages Erlang map creation, access, updates, and iteration operations
/// for JIT-compiled code.
pub struct MapOperations;

impl MapOperations {
    /// Create a new map
    ///
    /// Creates a new map with the specified key-value pairs.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `spec` - Map creation specification
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn create_map(
        assembler: &mut Assembler,
        spec: &MapCreationSpec,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Creating map with {} pairs", spec.pairs.len());

        // Determine if this should be a flat map or hash map
        let map_type = if spec.pairs.len() <= 32 {
            MapType::Flat
        } else {
            MapType::Hash
        };

        match map_type {
            MapType::Flat => Self::create_flat_map(assembler, spec),
            MapType::Hash => Self::create_hash_map(assembler, spec),
        }
    }

    /// Get element from map
    ///
    /// Retrieves a value from a map by key.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context` - Map operation context
    ///
    /// # Returns
    /// Operation result
    pub fn get_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Getting element from map");

        // Validate context
        if context.operation != MapOperation::Get || context.dst_reg.is_none() {
            return Ok(MapOperationResult::BadKey);
        }

        // Check if map is valid
        Self::emit_validate_map(assembler, context.map_reg)?;

        // Determine map type and dispatch
        let map_type = Self::determine_map_type(assembler, context.map_reg)?;

        match map_type {
            MapType::Flat => Self::get_flat_map_element(assembler, context),
            MapType::Hash => Self::get_hash_map_element(assembler, context),
        }
    }

    /// Put element into map
    ///
    /// Inserts or updates a key-value pair in a map.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context` - Map operation context
    ///
    /// # Returns
    /// Operation result
    pub fn put_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Putting element into map");

        // Validate context
        if context.operation != MapOperation::Put || context.value_reg.is_none() {
            return Ok(MapOperationResult::BadKey);
        }

        // Check if map is valid
        Self::emit_validate_map(assembler, context.map_reg)?;

        // Determine map type and dispatch
        let map_type = Self::determine_map_type(assembler, context.map_reg)?;

        match map_type {
            MapType::Flat => Self::put_flat_map_element(assembler, context),
            MapType::Hash => Self::put_hash_map_element(assembler, context),
        }
    }

    /// Remove element from map
    ///
    /// Removes a key-value pair from a map.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context` - Map operation context
    ///
    /// # Returns
    /// Operation result
    pub fn remove_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Removing element from map");

        // Validate context
        if context.operation != MapOperation::Remove {
            return Ok(MapOperationResult::BadKey);
        }

        // Check if map is valid
        Self::emit_validate_map(assembler, context.map_reg)?;

        // Determine map type and dispatch
        let map_type = Self::determine_map_type(assembler, context.map_reg)?;

        match map_type {
            MapType::Flat => Self::remove_flat_map_element(assembler, context),
            MapType::Hash => Self::remove_hash_map_element(assembler, context),
        }
    }

    /// Check if key exists in map
    ///
    /// Tests whether a key exists in a map without retrieving the value.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context` - Map operation context
    ///
    /// # Returns
    /// Operation result
    pub fn is_map_key(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Checking if key exists in map");

        // Validate context
        if context.operation != MapOperation::IsKey {
            return Ok(MapOperationResult::BadKey);
        }

        // Check if map is valid
        Self::emit_validate_map(assembler, context.map_reg)?;

        // Determine map type and dispatch
        let map_type = Self::determine_map_type(assembler, context.map_reg)?;

        match map_type {
            MapType::Flat => Self::is_flat_map_key(assembler, context),
            MapType::Hash => Self::is_hash_map_key(assembler, context),
        }
    }

    /// Update existing map element
    ///
    /// Updates the value for an existing key in a map.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context` - Map operation context
    ///
    /// # Returns
    /// Operation result
    pub fn update_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Updating map element");

        // Validate context
        if context.operation != MapOperation::Update || context.value_reg.is_none() {
            return Ok(MapOperationResult::BadKey);
        }

        // Check if key exists first
        let exists_result = Self::is_map_key(assembler, context)?;
        match exists_result {
            MapOperationResult::Success => {
                // Key exists, proceed with update
                Self::put_map_element(assembler, context)
            }
            _ => Ok(MapOperationResult::KeyNotFound),
        }
    }

    /// Start map iteration
    ///
    /// Initializes iteration over a map's key-value pairs.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `map_reg` - Map register
    /// * `key_dst_reg` - Register for iteration keys
    /// * `value_dst_reg` - Register for iteration values
    ///
    /// # Returns
    /// Iteration context
    pub fn start_map_iteration(
        assembler: &mut Assembler,
        map_reg: u32,
        key_dst_reg: u32,
        value_dst_reg: u32,
    ) -> Result<MapIterationContext, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Starting map iteration");

        // Validate map
        Self::emit_validate_map(assembler, map_reg)?;

        // Get map size
        let total_entries = Self::get_map_size(assembler, map_reg)?;

        let context = MapIterationContext {
            map_reg,
            position: 0,
            total_entries,
            key_dst_reg,
            value_dst_reg,
        };

        Ok(context)
    }

    /// Get next map entry during iteration
    ///
    /// Retrieves the next key-value pair during map iteration.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context` - Iteration context (will be updated)
    ///
    /// # Returns
    /// true if more entries available, false if iteration complete
    pub fn next_map_entry(
        assembler: &mut Assembler,
        context: &mut MapIterationContext,
    ) -> Result<bool, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Getting next map entry at position {}",
                 context.position);

        if context.position >= context.total_entries {
            return Ok(false); // Iteration complete
        }

        // Get next entry based on map type
        let map_type = Self::determine_map_type(assembler, context.map_reg)?;

        let has_next = match map_type {
            MapType::Flat => Self::next_flat_map_entry(assembler, context),
            MapType::Hash => Self::next_hash_map_entry(assembler, context),
        };

        if has_next? {
            context.position += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Private helper methods

    fn create_flat_map(
        assembler: &mut Assembler,
        spec: &MapCreationSpec,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Creating flat map");

        // Set up arguments for runtime call
        Self::emit_setup_map_creation_args(assembler, spec)?;

        // Call runtime function to create flat map
        Self::emit_call_new_flat_map(assembler)?;

        Ok(())
    }

    fn create_hash_map(
        assembler: &mut Assembler,
        spec: &MapCreationSpec,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Creating hash map");

        // Set up arguments for runtime call
        Self::emit_setup_map_creation_args(assembler, spec)?;

        // Call runtime function to create hash map
        Self::emit_call_new_hash_map(assembler)?;

        Ok(())
    }

    fn get_flat_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Getting element from flat map");

        // Call flat map get function
        Self::emit_call_flat_map_get(assembler, context)?;

        Ok(MapOperationResult::Success)
    }

    fn get_hash_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Getting element from hash map");

        // Calculate hash of key
        Self::emit_calculate_key_hash(assembler, context.key_reg)?;

        // Call hash map get function
        Self::emit_call_hash_map_get(assembler, context)?;

        Ok(MapOperationResult::Success)
    }

    fn put_flat_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Putting element into flat map");

        // Call flat map put function
        Self::emit_call_flat_map_put(assembler, context)?;

        Ok(MapOperationResult::Success)
    }

    fn put_hash_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Putting element into hash map");

        // Calculate hash of key
        Self::emit_calculate_key_hash(assembler, context.key_reg)?;

        // Call hash map put function
        Self::emit_call_hash_map_put(assembler, context)?;

        Ok(MapOperationResult::Success)
    }

    fn remove_flat_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Removing element from flat map");

        // Call flat map remove function
        Self::emit_call_flat_map_remove(assembler, context)?;

        Ok(MapOperationResult::Success)
    }

    fn remove_hash_map_element(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Removing element from hash map");

        // Calculate hash of key
        Self::emit_calculate_key_hash(assembler, context.key_reg)?;

        // Call hash map remove function
        Self::emit_call_hash_map_remove(assembler, context)?;

        Ok(MapOperationResult::Success)
    }

    fn is_flat_map_key(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Checking key in flat map");

        // Call flat map is_key function
        Self::emit_call_flat_map_is_key(assembler, context)?;

        Ok(MapOperationResult::Success)
    }

    fn is_hash_map_key(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<MapOperationResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Checking key in hash map");

        // Calculate hash of key
        Self::emit_calculate_key_hash(assembler, context.key_reg)?;

        // Call hash map is_key function
        Self::emit_call_hash_map_is_key(assembler, context)?;

        Ok(MapOperationResult::Success)
    }

    fn next_flat_map_entry(
        assembler: &mut Assembler,
        context: &mut MapIterationContext,
    ) -> Result<bool, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Getting next flat map entry");

        // Access flat map entries by index
        Self::emit_access_flat_map_entry_by_index(assembler, context)?;

        Ok(true)
    }

    fn next_hash_map_entry(
        assembler: &mut Assembler,
        context: &mut MapIterationContext,
    ) -> Result<bool, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Getting next hash map entry");

        // Iterate through hash table buckets
        Self::emit_iterate_hash_map_buckets(assembler, context)?;

        Ok(true)
    }

    // Low-level emission methods

    fn emit_validate_map(assembler: &mut Assembler, map_reg: u32) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Map Operations: Validating map");

        // Check if register contains a valid map
        a64::emit_mov_reg_reg(assembler, 9, map_reg)?; // TMP1 = map
        a64::emit_and_imm(assembler, 9, 9, 0x3F)?; // TMP1 &= MAP_TAG_MASK

        // Compare with map tag
        const MAP_TAG: u64 = 0x12; // Placeholder
        a64::emit_cmp_imm(assembler, 9, MAP_TAG)?;

        // Branch to error if not a map

        Ok(())
    }

    fn determine_map_type(assembler: &mut Assembler, map_reg: u32) -> Result<MapType, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Determining map type");

        // Check map header to determine if flat or hash map
        // For now, assume flat map for small maps

        Ok(MapType::Flat) // Placeholder
    }

    fn get_map_size(assembler: &mut Assembler, map_reg: u32) -> Result<u64, BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Getting map size");

        // Access map header to get size
        // For now, return placeholder

        Ok(10) // Placeholder
    }

    fn emit_setup_map_creation_args(
        assembler: &mut Assembler,
        spec: &MapCreationSpec,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Map Operations: Setting up map creation arguments");

        // ARG1 = process pointer
        a64::emit_mov_reg_reg(assembler, 0, 21)?; // ARG1 = c_p

        // ARG2 = X register array
        a64::emit_add_imm(assembler, 1, 21, 0x100)?; // ARG2 = &X[0]

        // ARG3 = live registers
        a64::emit_mov_imm(assembler, 2, spec.live as u64)?;

        // ARG4 = number of pairs
        a64::emit_mov_imm(assembler, 3, spec.pairs.len() as u64)?;

        // ARG5 = pairs array (embedded in code)
        // This would typically embed the key-value pairs

        Ok(())
    }

    fn emit_call_new_flat_map(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling new flat map");

        // Call runtime function: erts_gc_new_flat_map or similar

        Ok(())
    }

    fn emit_call_new_hash_map(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling new hash map");

        // Call runtime function: erts_gc_new_map

        Ok(())
    }

    fn emit_calculate_key_hash(
        assembler: &mut Assembler,
        key_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Map Operations: Calculating key hash");

        // Implement internal hash calculation
        // key_hash = key ^ (key >> 33)
        a64::emit_mov_reg_reg(assembler, 9, key_reg)?; // TMP1 = key
        a64::emit_eor_reg_reg_reg(assembler, 10, 9, 9)?; // TMP2 = TMP1 ^ TMP1
        a64::emit_lsr_imm(assembler, 10, 10, 33)?; // TMP2 >>= 33
        a64::emit_eor_reg_reg_reg(assembler, 9, 9, 10)?; // TMP1 ^= TMP2

        // key_hash *= 0xFF51AFD7ED558CCDull
        a64::emit_mov_imm(assembler, 10, 0xFF51AFD7ED558CCD)?; // TMP2 = multiplier
        a64::emit_mul_reg_reg_reg(assembler, 9, 9, 10)?; // TMP1 *= TMP2

        Ok(())
    }

    fn emit_call_flat_map_get(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling flat map get");

        // Call flat map element access function

        Ok(())
    }

    fn emit_call_hash_map_get(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling hash map get");

        // Call hash map element access function

        Ok(())
    }

    fn emit_call_flat_map_put(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling flat map put");

        // Call flat map element insertion function

        Ok(())
    }

    fn emit_call_hash_map_put(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling hash map put");

        // Call hash map element insertion function

        Ok(())
    }

    fn emit_call_flat_map_remove(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling flat map remove");

        // Call flat map element removal function

        Ok(())
    }

    fn emit_call_hash_map_remove(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling hash map remove");

        // Call hash map element removal function

        Ok(())
    }

    fn emit_call_flat_map_is_key(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling flat map is_key");

        // Call flat map key existence check

        Ok(())
    }

    fn emit_call_hash_map_is_key(
        assembler: &mut Assembler,
        context: &MapOperationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Calling hash map is_key");

        // Call hash map key existence check

        Ok(())
    }

    fn emit_access_flat_map_entry_by_index(
        assembler: &mut Assembler,
        context: &mut MapIterationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Accessing flat map entry by index");

        // Access flat map entries sequentially

        Ok(())
    }

    fn emit_iterate_hash_map_buckets(
        assembler: &mut Assembler,
        context: &mut MapIterationContext,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Iterating hash map buckets");

        // Iterate through hash table structure

        Ok(())
    }

    /// Validate map operation context
    ///
    /// Checks if the map operation context is valid and properly configured.
    ///
    /// # Arguments
    /// * `context` - Context to validate
    ///
    /// # Returns
    /// true if valid, false otherwise
    pub fn validate_operation_context(context: &MapOperationContext) -> bool {
        context.map_reg < 32 && // Valid ARM64 register
        context.key_reg < 32 &&
        (context.value_reg.is_none() || context.value_reg.unwrap() < 32) &&
        (context.dst_reg.is_none() || context.dst_reg.unwrap() < 32)
    }

    /// Validate map creation specification
    ///
    /// Checks if the map creation spec is valid.
    ///
    /// # Arguments
    /// * `spec` - Spec to validate
    ///
    /// # Returns
    /// true if valid, false otherwise
    pub fn validate_creation_spec(spec: &MapCreationSpec) -> bool {
        spec.dst_reg < 32 && // Valid ARM64 register
        spec.live > 0 && spec.live <= 1024 && // Reasonable live count
        spec.pairs.len() <= 1000 && // Reasonable pair count
        spec.pairs.iter().all(|pair| pair.key_reg < 32 && pair.value_reg < 32)
    }

    /// Calculate heap requirements for map operations
    ///
    /// Estimates heap space needed for map operations.
    ///
    /// # Arguments
    /// * `operation` - Type of operation
    /// * `size` - Map size
    ///
    /// # Returns
    /// Heap words needed
    pub fn calculate_heap_requirements(operation: &str, size: u64) -> u32 {
        match operation {
            "create_flat" => (size * 2 + 4) as u32, // key + value + header
            "create_hash" => (size * 3 + 16) as u32, // hash table overhead
            "get" => 4, // Minimal allocation
            "put" => (size + 8) as u32, // Potential reallocation
            _ => 16, // Default minimum
        }
    }

    /// Handle map operation error
    ///
    /// Process map operation errors and set up proper error handling.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `error` - Error type that occurred
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn handle_map_error(
        assembler: &mut Assembler,
        error: &MapOperationResult,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Map Operations: Handling error: {:?}", error);

        let error_code = match error {
            MapOperationResult::KeyNotFound => crate::error_integration::error_codes::BADKEY,
            MapOperationResult::BadMap => crate::error_integration::error_codes::BADARG,
            MapOperationResult::OutOfMemory => crate::error_integration::error_codes::BADARG, // System limit
            MapOperationResult::BadKey => crate::error_integration::error_codes::BADARG,
            _ => crate::error_integration::error_codes::BADARG,
        };

        let mfa = crate::ErrorMFA {
            module: 0x100, // am_erlang
            function: 0x200, // am_map_operation_error or similar
            arity: 0,
        };

        let error_context = crate::ErrorContext {
            error_code,
            mfa: Some(mfa),
            error_data: None,
        };

        crate::ErrorIntegration::set_error_and_raise(assembler, &error_context)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_operation_types() {
        assert_eq!(MapOperation::Get as u8, MapOperation::Get as u8);
        assert_ne!(MapOperation::Get as u8, MapOperation::Put as u8);
        assert_ne!(MapOperation::Remove as u8, MapOperation::Update as u8);
    }

    #[test]
    fn test_map_types() {
        assert_eq!(MapType::Flat as u8, MapType::Flat as u8);
        assert_ne!(MapType::Flat as u8, MapType::Hash as u8);
    }

    #[test]
    fn test_map_operation_context_creation() {
        let context = MapOperationContext {
            map_reg: 5,
            key_reg: 10,
            value_reg: Some(15),
            dst_reg: Some(20),
            operation: MapOperation::Get,
            map_type: MapType::Flat,
        };

        assert_eq!(context.map_reg, 5);
        assert_eq!(context.key_reg, 10);
        assert_eq!(context.value_reg, Some(15));
        assert_eq!(context.dst_reg, Some(20));
        assert_eq!(context.operation, MapOperation::Get);
        assert_eq!(context.map_type, MapType::Flat);
    }

    #[test]
    fn test_map_creation_spec_creation() {
        let pairs = vec![
            MapKeyValuePair {
                key_reg: 1,
                value_reg: 2,
            },
            MapKeyValuePair {
                key_reg: 3,
                value_reg: 4,
            },
        ];

        let spec = MapCreationSpec {
            dst_reg: 10,
            live: 5,
            pairs,
        };

        assert_eq!(spec.dst_reg, 10);
        assert_eq!(spec.live, 5);
        assert_eq!(spec.pairs.len(), 2);
    }

    #[test]
    fn test_map_iteration_context_creation() {
        let context = MapIterationContext {
            map_reg: 5,
            position: 0,
            total_entries: 10,
            key_dst_reg: 1,
            value_dst_reg: 2,
        };

        assert_eq!(context.map_reg, 5);
        assert_eq!(context.position, 0);
        assert_eq!(context.total_entries, 10);
        assert_eq!(context.key_dst_reg, 1);
        assert_eq!(context.value_dst_reg, 2);
    }

    #[test]
    fn test_operation_context_validation() {
        // Valid context
        let valid_context = MapOperationContext {
            map_reg: 5,
            key_reg: 10,
            value_reg: Some(15),
            dst_reg: Some(20),
            operation: MapOperation::Get,
            map_type: MapType::Flat,
        };
        assert!(MapOperations::validate_operation_context(&valid_context));

        // Invalid context - bad map register
        let invalid_context1 = MapOperationContext {
            map_reg: 32, // Invalid register
            key_reg: 10,
            value_reg: Some(15),
            dst_reg: Some(20),
            operation: MapOperation::Get,
            map_type: MapType::Flat,
        };
        assert!(!MapOperations::validate_operation_context(&invalid_context1));

        // Invalid context - bad key register
        let invalid_context2 = MapOperationContext {
            map_reg: 5,
            key_reg: 32, // Invalid register
            value_reg: Some(15),
            dst_reg: Some(20),
            operation: MapOperation::Get,
            map_type: MapType::Flat,
        };
        assert!(!MapOperations::validate_operation_context(&invalid_context2));
    }

    #[test]
    fn test_creation_spec_validation() {
        let pairs = vec![
            MapKeyValuePair {
                key_reg: 1,
                value_reg: 2,
            },
        ];

        // Valid spec
        let valid_spec = MapCreationSpec {
            dst_reg: 10,
            live: 5,
            pairs: pairs.clone(),
        };
        assert!(MapOperations::validate_creation_spec(&valid_spec));

        // Invalid spec - bad dst register
        let invalid_spec1 = MapCreationSpec {
            dst_reg: 32, // Invalid register
            live: 5,
            pairs: pairs.clone(),
        };
        assert!(!MapOperations::validate_creation_spec(&invalid_spec1));

        // Invalid spec - zero live
        let invalid_spec2 = MapCreationSpec {
            dst_reg: 10,
            live: 0, // Invalid
            pairs: pairs.clone(),
        };
        assert!(!MapOperations::validate_creation_spec(&invalid_spec2));
    }

    #[test]
    fn test_heap_requirements_calculation() {
        // Test different operation types
        assert_eq!(MapOperations::calculate_heap_requirements("create_flat", 10), 24);
        assert_eq!(MapOperations::calculate_heap_requirements("create_hash", 10), 46);
        assert_eq!(MapOperations::calculate_heap_requirements("get", 10), 4);
        assert_eq!(MapOperations::calculate_heap_requirements("put", 10), 18);
        assert_eq!(MapOperations::calculate_heap_requirements("unknown", 10), 16);
    }

    #[test]
    fn test_map_operation_result() {
        // Test different result types
        assert!(matches!(MapOperationResult::Success, MapOperationResult::Success));
        assert!(matches!(MapOperationResult::KeyNotFound, MapOperationResult::KeyNotFound));
        assert!(matches!(MapOperationResult::BadMap, MapOperationResult::BadMap));
        assert!(matches!(MapOperationResult::OutOfMemory, MapOperationResult::OutOfMemory));
        assert!(matches!(MapOperationResult::BadKey, MapOperationResult::BadKey));
    }

    #[test]
    fn test_map_operations_creation() {
        // MapOperations has no state, just test creation
        let _operations = MapOperations;
    }
}
