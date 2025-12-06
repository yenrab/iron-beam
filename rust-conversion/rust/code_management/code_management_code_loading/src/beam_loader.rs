//! BEAM File Loader
//!
//! Provides BEAM file parsing and module loading functionality.
//! Based on beam_load.c and beam_file.c - BEAM file reading and parsing.

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

use crate::module_management::ModuleTableManager;
use crate::code_index::get_global_code_ix;

/// BEAM file read result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamFileReadResult {
    /// Successfully read BEAM file
    Success,
    /// Corrupt file header
    CorruptFileHeader,
    /// Missing atom table
    MissingAtomTable,
    /// Obsolete atom table (OTP 20 or earlier)
    ObsoleteAtomTable,
    /// Corrupt atom table
    CorruptAtomTable,
    /// Missing code chunk
    MissingCodeChunk,
    /// Corrupt code chunk
    CorruptCodeChunk,
    /// Missing export table
    MissingExportTable,
    /// Corrupt export table
    CorruptExportTable,
    /// Missing import table
    MissingImportTable,
    /// Corrupt import table
    CorruptImportTable,
    /// Corrupt lambda table
    CorruptLambdaTable,
    /// Corrupt line table
    CorruptLineTable,
    /// Corrupt literal table
    CorruptLiteralTable,
    /// Corrupt locals table
    CorruptLocalsTable,
    /// Corrupt type table
    CorruptTypeTable,
    /// Corrupt debug table
    CorruptDebugTable,
}

/// BEAM file structure (simplified)
#[derive(Debug, Clone, PartialEq)]
pub struct BeamFile {
    /// Module name (atom index)
    pub module: u32,
    /// Code chunk data
    pub code_data: Vec<u8>,
    /// Code size
    pub code_size: u32,
    /// Export table (simplified - list of {function, arity, label})
    pub exports: Vec<(u32, u32, i32)>, // (function_atom, arity, label)
    /// Import table (simplified)
    pub imports: Vec<(u32, u32, u32)>, // (module_atom, function_atom, arity)
    /// Atom table (simplified)
    pub atoms: Vec<String>,
    /// Whether module has on_load function
    pub has_on_load: bool,
    /// Attributes chunk data (raw bytes - will be decoded to ErlangTerm when term decoding supports tuples/lists)
    pub attributes_data: Option<Vec<u8>>,
    /// Compile info chunk data (raw bytes - will be decoded to ErlangTerm when term decoding supports tuples/lists)
    pub compile_info_data: Option<Vec<u8>>,
}

/// BEAM file loader
pub struct BeamLoader;

impl BeamLoader {
    /// Read and parse a BEAM file
    ///
    /// # Arguments
    /// * `data` - BEAM file bytes
    ///
    /// # Returns
    /// Parsed BEAM file or error
    pub fn read_beam_file(data: &[u8]) -> Result<BeamFile, BeamFileReadResult> {
        // Check minimum size (IFF header is at least 8 bytes)
        if data.len() < 8 {
            return Err(BeamFileReadResult::CorruptFileHeader);
        }

        // Check IFF form ID (first 4 bytes should be "FOR1" or "FORX")
        let form_id = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if form_id != 0x464F5231 && form_id != 0x464F5258 {
            // "FOR1" or "FORX"
            return Err(BeamFileReadResult::CorruptFileHeader);
        }

        // Check BEAM form type (bytes 8-11 should be "BEAM")
        if data.len() < 12 {
            return Err(BeamFileReadResult::CorruptFileHeader);
        }
        
        let beam_id = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        if beam_id != 0x4245414D {
            // "BEAM"
            return Err(BeamFileReadResult::CorruptFileHeader);
        }

        // Parse IFF chunks
        let mut beam_file = BeamFile {
            module: 0, // Will be set from atom table
            code_data: vec![],
            code_size: 0,
            exports: vec![],
            imports: vec![],
            atoms: vec![],
            has_on_load: false,
            attributes_data: None,
            compile_info_data: None,
        };
        
        // Parse IFF chunks starting after the BEAM form type (byte 12)
        let mut pos = 12;
        
        // IFF chunk format:
        // - 4 bytes: chunk ID
        // - 4 bytes: chunk size (big-endian)
        // - chunk data (aligned to 4-byte boundary)
        
        while pos + 8 <= data.len() {
            // Read chunk ID (4 bytes)
            let chunk_id = u32::from_be_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
            ]);
            pos += 4;
            
            // Read chunk size (4 bytes, big-endian)
            if pos + 4 > data.len() {
                break; // Incomplete chunk size
            }
            let chunk_size = u32::from_be_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
            ]) as usize;
            pos += 4;
            
            // Check if we have enough data for the chunk
            if pos + chunk_size > data.len() {
                break; // Incomplete chunk data
            }
            
            // Extract chunk data
            let chunk_data = data[pos..pos + chunk_size].to_vec();
            
            // Process chunk based on ID
            match chunk_id {
                0x41747472 => { // "Attr" - Attributes chunk
                    beam_file.attributes_data = Some(chunk_data);
                }
                0x43496E66 => { // "CInf" - Compile info chunk
                    beam_file.compile_info_data = Some(chunk_data);
                }
                0x436F6465 => { // "Code" - Code chunk
                    beam_file.code_data = chunk_data.clone();
                    beam_file.code_size = chunk_size as u32;
                }
                0x45787054 => { // "ExpT" - Export table chunk
                    // Parse export table
                    // Export table format: 4-byte count (big-endian), then entries of:
                    // - 4-byte function atom index (big-endian)
                    // - 4-byte arity (big-endian)
                    // - 4-byte label (big-endian, signed)
                    if chunk_size >= 4 {
                        let count = u32::from_be_bytes([
                            chunk_data[0],
                            chunk_data[1],
                            chunk_data[2],
                            chunk_data[3],
                        ]);
                        
                        let mut pos = 4;
                        let mut exports = Vec::new();
                        
                        // Each entry is 12 bytes (3 * 4 bytes)
                        for _ in 0..count {
                            if pos + 12 <= chunk_size {
                                let function_atom = u32::from_be_bytes([
                                    chunk_data[pos],
                                    chunk_data[pos + 1],
                                    chunk_data[pos + 2],
                                    chunk_data[pos + 3],
                                ]);
                                pos += 4;
                                
                                let arity = u32::from_be_bytes([
                                    chunk_data[pos],
                                    chunk_data[pos + 1],
                                    chunk_data[pos + 2],
                                    chunk_data[pos + 3],
                                ]);
                                pos += 4;
                                
                                let label = i32::from_be_bytes([
                                    chunk_data[pos],
                                    chunk_data[pos + 1],
                                    chunk_data[pos + 2],
                                    chunk_data[pos + 3],
                                ]);
                                pos += 4;
                                
                                exports.push((function_atom, arity, label));
                            } else {
                                break; // Incomplete entry
                            }
                        }
                        
                        beam_file.exports = exports;
                    }
                }
                _ => {
                    // Other chunks - ignore for now
                }
            }
            
            // Move to next chunk (aligned to 4-byte boundary)
            let aligned_size = (chunk_size + 3) & !3;
            pos += aligned_size;
        }
        
        Ok(beam_file)
    }

    /// Prepare loading a module from BEAM file
    ///
    /// This parses the BEAM file and prepares it for loading.
    /// Based on erts_prepare_loading().
    ///
    /// # Arguments
    /// * `code` - BEAM file bytes
    /// * `module_atom` - Expected module name (atom index), or None to not check
    ///
    /// # Returns
    /// Parsed BEAM file or error
    pub fn prepare_loading(
        code: &[u8],
        module_atom: Option<u32>,
    ) -> Result<BeamFile, BeamFileReadResult> {
        let beam = Self::read_beam_file(code)?;

        // Verify module name if provided
        if let Some(expected_module) = module_atom {
            if beam.module != expected_module {
                return Err(BeamFileReadResult::CorruptFileHeader);
            }
        }

        // Check code chunk exists
        if beam.code_data.is_empty() {
            return Err(BeamFileReadResult::MissingCodeChunk);
        }

        // Check export table exists
        // In a full implementation, we'd verify the export table was parsed correctly
        // For now, we just check that we got this far

        Ok(beam)
    }

    /// Finish loading a module
    ///
    /// This completes the module loading process by:
    /// 1. Making current code old (if module exists)
    /// 2. Inserting new code as current
    /// 3. Updating module table
    ///
    /// Based on erts_finish_loading().
    ///
    /// # Arguments
    /// * `beam` - Parsed BEAM file
    /// * `module_atom` - Module atom index
    /// * `module_manager` - Module table manager
    ///
    /// # Returns
    /// Ok(()) if successful, error otherwise
    pub fn finish_loading(
        beam: &BeamFile,
        module_atom: u32,
        module_manager: &ModuleTableManager,
    ) -> Result<(), BeamLoadError> {
        let code_ix = get_global_code_ix();
        let staging_ix = code_ix.staging_code_ix() as usize;
        let table = module_manager.get_table(staging_ix);

        // Put module in table (creates if doesn't exist)
        let module = table.put_module(module_atom);

        // Make current code old (if module already existed)
        Self::make_current_old(module_manager, module_atom)?;

        // Finalize code into module instance
        Self::finalize_code(beam, &module, staging_ix)?;

        Ok(())
    }

    /// Make current code old (for code updates)
    ///
    /// Equivalent to beam_make_current_old(). Moves the current module instance
    /// to the old instance, making room for new code.
    ///
    /// # Arguments
    /// * `module_manager` - Module table manager
    /// * `module_atom` - Module atom index
    ///
    /// # Returns
    /// Ok(()) if successful, error if old code already exists
    pub fn make_current_old(
        module_manager: &ModuleTableManager,
        module_atom: u32,
    ) -> Result<(), BeamLoadError> {
        let code_ix = get_global_code_ix();
        let staging_ix = code_ix.staging_code_ix() as usize;
        let table = module_manager.get_table(staging_ix);

        if table.get_module(module_atom).is_some() {
            // Check if old code already exists (would need to check old instance)
            // For now, simplified: just copy curr to old
            // In full implementation, would check if old is already populated
        }

        Ok(())
    }

    /// Finalize code loading into module instance
    ///
    /// Equivalent to beam_load_finalize_code(). Sets up the module instance with
    /// the loaded code.
    ///
    /// # Arguments
    /// * `beam` - Parsed BEAM file
    /// * `_module` - Module to update
    /// * `_code_ix` - Code index
    ///
    /// # Returns
    /// Ok(()) if successful, error otherwise
    pub fn finalize_code(
        beam: &BeamFile,
        _module: &crate::module_management::Module,
        _code_ix: usize,
    ) -> Result<(), BeamLoadError> {
        // In a full implementation, this would:
        // 1. Allocate code memory
        // 2. Copy code data from beam
        // 3. Set up code header
        // 4. Update module.curr with code information
        // 5. Set executable_region and writable_region
        
        // For now, simplified: just verify beam has code
        if beam.code_data.is_empty() {
            return Err(BeamLoadError::InvalidModule);
        }

        Ok(())
    }

    /// Initialize loading subsystem
    ///
    /// Equivalent to init_load(). Called at system startup.
    pub fn init_load() {
        // In a full implementation, this would:
        // 1. Initialize beam_catches
        // 2. Initialize ranges
        // 3. Set up other loading infrastructure
    }

    /// Check if module has on_load function
    ///
    /// Equivalent to erts_has_code_on_load(). Returns true if the module
    /// has an on_load function, false otherwise.
    ///
    /// # Arguments
    /// * `beam` - Parsed BEAM file
    ///
    /// # Returns
    /// true if module has on_load, false otherwise
    pub fn has_code_on_load(beam: &BeamFile) -> bool {
        beam.has_on_load
    }

    /// Report loading error
    ///
    /// Equivalent to beam_load_report_error(). Formats and reports an error
    /// that occurred during module loading.
    ///
    /// # Arguments
    /// * `line` - Line number where error occurred
    /// * `module` - Module atom index
    /// * `function` - Function atom index (if applicable)
    /// * `arity` - Function arity (if applicable)
    /// * `format` - Error message format string
    /// * `args` - Format arguments
    pub fn report_error(
        line: u32,
        module: u32,
        function: Option<u32>,
        arity: Option<u32>,
        format: &str,
        args: &[&dyn std::fmt::Display],
    ) {
        eprint!("beam_load.rs({}): Error loading ", line);
        
        if let (Some(func), Some(arity_val)) = (function, arity) {
            eprint!("function {}:{}:{}", module, func, arity_val);
        } else {
            eprint!("module {}", module);
        }
        
        eprint!(":\n  ");
        
        // Simple format string handling
        let mut format_chars = format.chars().peekable();
        let mut arg_idx = 0;
        while let Some(ch) = format_chars.next() {
            if ch == '{' && format_chars.peek() == Some(&'}') {
                format_chars.next(); // consume '}'
                if arg_idx < args.len() {
                    eprint!("{}", args[arg_idx]);
                    arg_idx += 1;
                }
            } else {
                eprint!("{}", ch);
            }
        }
        eprintln!();
    }

    /// Prepare emit (begin code emission)
    ///
    /// Equivalent to beam_load_prepare_emit(). Prepares the loader state
    /// for emitting code.
    ///
    /// # Arguments
    /// * `beam` - Parsed BEAM file
    ///
    /// # Returns
    /// Ok(()) if successful, error otherwise
    pub fn prepare_emit(beam: &BeamFile) -> Result<(), BeamLoadError> {
        // In a full implementation, this would:
        // 1. Allocate code memory
        // 2. Set up code header
        // 3. Initialize emit state
        
        if beam.code_data.is_empty() {
            return Err(BeamLoadError::InvalidModule);
        }
        
        Ok(())
    }

    /// Emit an operation
    ///
    /// Equivalent to beam_load_emit_op(). Emits a single operation to the code.
    ///
    /// # Arguments
    /// * `beam` - Parsed BEAM file
    /// * `op_code` - Operation code
    /// * `args` - Operation arguments
    ///
    /// # Returns
    /// Ok(()) if successful, error otherwise
    pub fn emit_op(
        _beam: &BeamFile,
        _op_code: u32,
        _args: &[u32],
    ) -> Result<(), BeamLoadError> {
        // In a full implementation, this would:
        // 1. Encode operation
        // 2. Write to code buffer
        // 3. Update code position
        
        Ok(())
    }

    /// Finish emit (complete code emission)
    ///
    /// Equivalent to beam_load_finish_emit(). Completes code emission and
    /// finalizes the code.
    ///
    /// # Arguments
    /// * `beam` - Parsed BEAM file
    ///
    /// # Returns
    /// Ok(()) if successful, error otherwise
    pub fn finish_emit(beam: &BeamFile) -> Result<(), BeamLoadError> {
        // In a full implementation, this would:
        // 1. Finalize code header
        // 2. Set up code pointers
        // 3. Validate code
        
        if beam.code_data.is_empty() {
            return Err(BeamLoadError::InvalidModule);
        }
        
        Ok(())
    }

    /// Purge auxiliary code
    ///
    /// Equivalent to beam_load_purge_aux(). Purges old code from a module.
    ///
    /// # Arguments
    /// * `code_hdr` - Code header pointer (simplified as usize)
    pub fn purge_aux(_code_hdr: usize) {
        // In a full implementation, this would:
        // 1. Free code memory
        // 2. Release resources
        // 3. Update module state
    }

    /// Create new generic operation
    ///
    /// Equivalent to beam_load_new_genop(). Creates a new generic operation
    /// in the loader state.
    ///
    /// # Returns
    /// Operation code (simplified)
    pub fn new_genop() -> u32 {
        // In a full implementation, this would:
        // 1. Allocate operation structure
        // 2. Initialize operation
        // 3. Return operation pointer
        
        0 // Simplified
    }

    /// Create new label
    ///
    /// Equivalent to beam_load_new_label(). Creates a new label in the loader state.
    ///
    /// # Returns
    /// Label index
    pub fn new_label() -> i32 {
        // In a full implementation, this would:
        // 1. Allocate label
        // 2. Return label index
        
        0 // Simplified
    }
}

/// BEAM load error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamLoadError {
    /// Module not found
    ModuleNotFound,
    /// Old code exists (needs purging)
    OldCodeExists,
    /// Invalid module
    InvalidModule,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beam_file_read_invalid_header() {
        let invalid_data = b"INVALID";
        let result = BeamLoader::read_beam_file(invalid_data);
        assert_eq!(result, Err(BeamFileReadResult::CorruptFileHeader));
    }

    #[test]
    fn test_beam_file_read_too_small() {
        let small_data = b"FOR";
        let result = BeamLoader::read_beam_file(small_data);
        assert_eq!(result, Err(BeamFileReadResult::CorruptFileHeader));
    }

    #[test]
    fn test_beam_file_read_valid_header() {
        // Create minimal valid BEAM header
        let mut data = vec![0u8; 16];
        // FOR1 form ID
        data[0..4].copy_from_slice(b"FOR1");
        // Form size (4 bytes, little-endian)
        data[4..8].copy_from_slice(&8u32.to_le_bytes());
        // BEAM form type
        data[8..12].copy_from_slice(b"BEAM");
        
        let result = BeamLoader::read_beam_file(&data);
        // Should succeed (even though chunks aren't parsed yet)
        assert!(result.is_ok());
    }

    #[test]
    fn test_prepare_loading() {
        // Create minimal valid BEAM file with Code chunk
        // FOR1 header (4) + form size (4) + BEAM (4) + Code chunk (8 + 4 = 12) = 24 bytes
        // Form size = 4 (BEAM) + 12 (Code chunk) = 16 bytes
        let mut data = vec![0u8; 28]; // Extra space for alignment
        let mut pos = 0;
        
        // FOR1 form ID
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        
        // Form size (16 bytes: BEAM + Code chunk)
        data[pos..pos+4].copy_from_slice(&16u32.to_le_bytes());
        pos += 4;
        
        // BEAM form type
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // Code chunk ID (0x436F6465 = "Code")
        data[pos..pos+4].copy_from_slice(b"Code");
        pos += 4;
        
        // Code chunk size (4 bytes of data)
        data[pos..pos+4].copy_from_slice(&4u32.to_be_bytes());
        pos += 4;
        
        // Code chunk data (minimal - 4 bytes)
        data[pos..pos+4].copy_from_slice(&[0u8; 4]);
        pos += 4;
        
        // Resize to actual size
        data.truncate(pos);
        
        let result = BeamLoader::prepare_loading(&data, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_finish_loading() {
        // Create a BEAM file with Code chunk
        let mut data = vec![0u8; 28];
        let mut pos = 0;
        
        // FOR1 form ID
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        
        // Form size (16 bytes: BEAM + Code chunk)
        data[pos..pos+4].copy_from_slice(&16u32.to_le_bytes());
        pos += 4;
        
        // BEAM form type
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // Code chunk ID
        data[pos..pos+4].copy_from_slice(b"Code");
        pos += 4;
        
        // Code chunk size (4 bytes of data)
        data[pos..pos+4].copy_from_slice(&4u32.to_be_bytes());
        pos += 4;
        
        // Code chunk data (minimal - 4 bytes)
        data[pos..pos+4].copy_from_slice(&[0u8; 4]);
        pos += 4;
        
        // Resize to actual size
        data.truncate(pos);
        
        let beam = BeamLoader::read_beam_file(&data).unwrap();
        let module_manager = ModuleTableManager::new();
        let result = BeamLoader::finish_loading(&beam, 1, &module_manager);
        // Should succeed (module will be created)
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_code_on_load() {
        let mut data = vec![0u8; 16];
        data[0..4].copy_from_slice(b"FOR1");
        data[4..8].copy_from_slice(&8u32.to_le_bytes());
        data[8..12].copy_from_slice(b"BEAM");
        
        let mut beam = BeamLoader::read_beam_file(&data).unwrap();
        assert!(!BeamLoader::has_code_on_load(&beam));
        
        beam.has_on_load = true;
        assert!(BeamLoader::has_code_on_load(&beam));
    }

    #[test]
    fn test_report_error() {
        // Test error reporting (should not panic)
        BeamLoader::report_error(100, 1, Some(2), Some(3), "test error {}", &[&"message"]);
    }

    #[test]
    fn test_emit_functions() {
        // Create a BEAM file with Code chunk
        let mut data = vec![0u8; 28];
        let mut pos = 0;
        
        // FOR1 form ID
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        
        // Form size (16 bytes: BEAM + Code chunk)
        data[pos..pos+4].copy_from_slice(&16u32.to_le_bytes());
        pos += 4;
        
        // BEAM form type
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // Code chunk ID
        data[pos..pos+4].copy_from_slice(b"Code");
        pos += 4;
        
        // Code chunk size (4 bytes of data)
        data[pos..pos+4].copy_from_slice(&4u32.to_be_bytes());
        pos += 4;
        
        // Code chunk data (minimal - 4 bytes)
        data[pos..pos+4].copy_from_slice(&[0u8; 4]);
        pos += 4;
        
        // Resize to actual size
        data.truncate(pos);
        
        let beam = BeamLoader::read_beam_file(&data).unwrap();
        
        // Test emit functions
        assert!(BeamLoader::prepare_emit(&beam).is_ok());
        assert!(BeamLoader::emit_op(&beam, 0, &[]).is_ok());
        assert!(BeamLoader::finish_emit(&beam).is_ok());
    }

    #[test]
    fn test_purge_and_helpers() {
        // Test purge (should not panic)
        BeamLoader::purge_aux(0x1000);
        
        // Test new_genop and new_label
        let _genop = BeamLoader::new_genop();
        let _label = BeamLoader::new_label();
    }
    
    #[test]
    fn test_beam_file_read_forx_header() {
        // Test FORX form ID (alternative to FOR1)
        let mut data = vec![0u8; 16];
        data[0..4].copy_from_slice(b"FORX");
        data[4..8].copy_from_slice(&8u32.to_le_bytes());
        data[8..12].copy_from_slice(b"BEAM");
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_beam_file_read_missing_beam_type() {
        // FOR1 header but missing BEAM type
        let mut data = vec![0u8; 8];
        data[0..4].copy_from_slice(b"FOR1");
        data[4..8].copy_from_slice(&8u32.to_le_bytes());
        
        let result = BeamLoader::read_beam_file(&data);
        assert_eq!(result, Err(BeamFileReadResult::CorruptFileHeader));
    }
    
    #[test]
    fn test_beam_file_read_invalid_beam_type() {
        // FOR1 header with invalid BEAM type
        let mut data = vec![0u8; 12];
        data[0..4].copy_from_slice(b"FOR1");
        data[4..8].copy_from_slice(&8u32.to_le_bytes());
        data[8..12].copy_from_slice(b"INVA");
        
        let result = BeamLoader::read_beam_file(&data);
        assert_eq!(result, Err(BeamFileReadResult::CorruptFileHeader));
    }
    
    #[test]
    fn test_beam_file_read_attr_chunk() {
        let mut data = vec![0u8; 32];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&20u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // Attr chunk
        data[pos..pos+4].copy_from_slice(b"Attr");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&4u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&[1, 2, 3, 4]);
        pos += 4;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
        let beam = result.unwrap();
        assert!(beam.attributes_data.is_some());
        assert_eq!(beam.attributes_data.unwrap(), vec![1, 2, 3, 4]);
    }
    
    #[test]
    fn test_beam_file_read_cinf_chunk() {
        let mut data = vec![0u8; 32];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&20u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // CInf chunk
        data[pos..pos+4].copy_from_slice(b"CInf");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&4u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&[5, 6, 7, 8]);
        pos += 4;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
        let beam = result.unwrap();
        assert!(beam.compile_info_data.is_some());
        assert_eq!(beam.compile_info_data.unwrap(), vec![5, 6, 7, 8]);
    }
    
    #[test]
    fn test_beam_file_read_export_table_empty() {
        let mut data = vec![0u8; 32];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&20u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // ExpT chunk with 0 exports
        data[pos..pos+4].copy_from_slice(b"ExpT");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&4u32.to_be_bytes()); // Size = 4 (just count)
        pos += 4;
        data[pos..pos+4].copy_from_slice(&0u32.to_be_bytes()); // Count = 0
        pos += 4;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
        let beam = result.unwrap();
        assert_eq!(beam.exports.len(), 0);
    }
    
    #[test]
    fn test_beam_file_read_export_table_single() {
        let mut data = vec![0u8; 48];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&36u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // ExpT chunk with 1 export
        data[pos..pos+4].copy_from_slice(b"ExpT");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&16u32.to_be_bytes()); // Size = 16 (4 + 12)
        pos += 4;
        data[pos..pos+4].copy_from_slice(&1u32.to_be_bytes()); // Count = 1
        pos += 4;
        // Export entry: function=1, arity=2, label=3
        data[pos..pos+4].copy_from_slice(&1u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&2u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&3i32.to_be_bytes());
        pos += 4;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
        let beam = result.unwrap();
        assert_eq!(beam.exports.len(), 1);
        assert_eq!(beam.exports[0], (1, 2, 3));
    }
    
    #[test]
    fn test_beam_file_read_export_table_multiple() {
        let mut data = vec![0u8; 80];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&68u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // ExpT chunk with 2 exports
        data[pos..pos+4].copy_from_slice(b"ExpT");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&28u32.to_be_bytes()); // Size = 28 (4 + 12*2)
        pos += 4;
        data[pos..pos+4].copy_from_slice(&2u32.to_be_bytes()); // Count = 2
        pos += 4;
        // First export: function=10, arity=20, label=30
        data[pos..pos+4].copy_from_slice(&10u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&20u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&30i32.to_be_bytes());
        pos += 4;
        // Second export: function=11, arity=21, label=31
        data[pos..pos+4].copy_from_slice(&11u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&21u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&31i32.to_be_bytes());
        pos += 4;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
        let beam = result.unwrap();
        assert_eq!(beam.exports.len(), 2);
        assert_eq!(beam.exports[0], (10, 20, 30));
        assert_eq!(beam.exports[1], (11, 21, 31));
    }
    
    #[test]
    fn test_beam_file_read_export_table_incomplete_entry() {
        let mut data = vec![0u8; 32];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&20u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // ExpT chunk with incomplete entry
        data[pos..pos+4].copy_from_slice(b"ExpT");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&16u32.to_be_bytes()); // Size = 16
        pos += 4;
        data[pos..pos+4].copy_from_slice(&1u32.to_be_bytes()); // Count = 1
        pos += 4;
        // Incomplete entry (only 8 bytes instead of 12)
        data[pos..pos+8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        pos += 8;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
        let beam = result.unwrap();
        // Should have 0 exports because entry was incomplete
        assert_eq!(beam.exports.len(), 0);
    }
    
    #[test]
    fn test_beam_file_read_export_table_negative_label() {
        let mut data = vec![0u8; 48];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&36u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // ExpT chunk with negative label
        data[pos..pos+4].copy_from_slice(b"ExpT");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&16u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&1u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&1u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&2u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&(-1i32).to_be_bytes());
        pos += 4;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
        let beam = result.unwrap();
        assert_eq!(beam.exports.len(), 1);
        assert_eq!(beam.exports[0], (1, 2, -1));
    }
    
    #[test]
    fn test_beam_file_read_chunk_alignment() {
        // Test chunk alignment to 4-byte boundary
        let mut data = vec![0u8; 40];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&28u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // Code chunk with size 5 (should align to 8)
        data[pos..pos+4].copy_from_slice(b"Code");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&5u32.to_be_bytes());
        pos += 4;
        data[pos..pos+5].copy_from_slice(&[1, 2, 3, 4, 5]);
        pos += 5;
        // Padding to align (3 bytes)
        pos += 3;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
        let beam = result.unwrap();
        assert_eq!(beam.code_data.len(), 5);
    }
    
    #[test]
    fn test_beam_file_read_unknown_chunk() {
        let mut data = vec![0u8; 32];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&20u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // Unknown chunk
        data[pos..pos+4].copy_from_slice(b"UNKN");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&4u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&[1, 2, 3, 4]);
        pos += 4;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        // Should succeed (unknown chunks are ignored)
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_beam_file_read_incomplete_chunk_size() {
        let mut data = vec![0u8; 20];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&8u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // Chunk ID but incomplete size
        data[pos..pos+4].copy_from_slice(b"Code");
        pos += 4;
        // Only 2 bytes of size (incomplete)
        if pos + 2 <= data.len() {
            data[pos..pos+2].copy_from_slice(&[0, 0]);
            pos += 2;
        }
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        // Should succeed (incomplete chunks are skipped)
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_beam_file_read_incomplete_chunk_data() {
        let mut data = vec![0u8; 24];
        let mut pos = 0;
        
        // FOR1 header
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&12u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        
        // Chunk with size larger than available data
        data[pos..pos+4].copy_from_slice(b"Code");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&100u32.to_be_bytes()); // Size = 100
        pos += 4;
        // Only 2 bytes available (incomplete)
        if pos + 2 <= data.len() {
            data[pos..pos+2].copy_from_slice(&[1, 2]);
            pos += 2;
        }
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        // Should succeed (incomplete chunks are skipped)
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_prepare_loading_module_mismatch() {
        let mut data = vec![0u8; 28];
        let mut pos = 0;
        
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&16u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"Code");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&4u32.to_be_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(&[0u8; 4]);
        pos += 4;
        data.truncate(pos);
        
        // Module name mismatch (beam.module is 0, but we expect 1)
        let result = BeamLoader::prepare_loading(&data, Some(1));
        assert_eq!(result, Err(BeamFileReadResult::CorruptFileHeader));
    }
    
    #[test]
    fn test_prepare_loading_missing_code_chunk() {
        let mut data = vec![0u8; 16];
        data[0..4].copy_from_slice(b"FOR1");
        data[4..8].copy_from_slice(&8u32.to_le_bytes());
        data[8..12].copy_from_slice(b"BEAM");
        
        let result = BeamLoader::prepare_loading(&data, None);
        assert_eq!(result, Err(BeamFileReadResult::MissingCodeChunk));
    }
    
    #[test]
    fn test_finalize_code_empty() {
        let beam = BeamFile {
            module: 0,
            code_data: vec![],
            code_size: 0,
            exports: vec![],
            imports: vec![],
            atoms: vec![],
            has_on_load: false,
            attributes_data: None,
            compile_info_data: None,
        };
        
        let module_manager = ModuleTableManager::new();
        let code_ix = get_global_code_ix();
        let staging_ix = code_ix.staging_code_ix() as usize;
        let table = module_manager.get_table(staging_ix);
        let module = table.put_module(1);
        
        let result = BeamLoader::finalize_code(&beam, &module, staging_ix);
        assert_eq!(result, Err(BeamLoadError::InvalidModule));
    }
    
    #[test]
    fn test_prepare_emit_empty() {
        let beam = BeamFile {
            module: 0,
            code_data: vec![],
            code_size: 0,
            exports: vec![],
            imports: vec![],
            atoms: vec![],
            has_on_load: false,
            attributes_data: None,
            compile_info_data: None,
        };
        
        let result = BeamLoader::prepare_emit(&beam);
        assert_eq!(result, Err(BeamLoadError::InvalidModule));
    }
    
    #[test]
    fn test_finish_emit_empty() {
        let beam = BeamFile {
            module: 0,
            code_data: vec![],
            code_size: 0,
            exports: vec![],
            imports: vec![],
            atoms: vec![],
            has_on_load: false,
            attributes_data: None,
            compile_info_data: None,
        };
        
        let result = BeamLoader::finish_emit(&beam);
        assert_eq!(result, Err(BeamLoadError::InvalidModule));
    }
    
    #[test]
    fn test_report_error_without_function() {
        // Test error reporting without function/arity
        BeamLoader::report_error(200, 5, None, None, "module error {}", &[&"test"]);
    }
    
    #[test]
    fn test_report_error_multiple_args() {
        // Test error reporting with multiple format arguments
        BeamLoader::report_error(300, 6, Some(7), Some(8), "error {} and {}", &[&"arg1", &"arg2"]);
    }
    
    #[test]
    fn test_report_error_no_args() {
        // Test error reporting with no format arguments
        BeamLoader::report_error(400, 9, Some(10), Some(11), "simple error", &[]);
    }
    
    #[test]
    fn test_beam_file_read_result_variants() {
        let variants = vec![
            BeamFileReadResult::Success,
            BeamFileReadResult::CorruptFileHeader,
            BeamFileReadResult::MissingAtomTable,
            BeamFileReadResult::ObsoleteAtomTable,
            BeamFileReadResult::CorruptAtomTable,
            BeamFileReadResult::MissingCodeChunk,
            BeamFileReadResult::CorruptCodeChunk,
            BeamFileReadResult::MissingExportTable,
            BeamFileReadResult::CorruptExportTable,
            BeamFileReadResult::MissingImportTable,
            BeamFileReadResult::CorruptImportTable,
            BeamFileReadResult::CorruptLambdaTable,
            BeamFileReadResult::CorruptLineTable,
            BeamFileReadResult::CorruptLiteralTable,
            BeamFileReadResult::CorruptLocalsTable,
            BeamFileReadResult::CorruptTypeTable,
            BeamFileReadResult::CorruptDebugTable,
        ];
        
        for variant in variants {
            let debug_str = format!("{:?}", variant);
            assert!(!debug_str.is_empty());
        }
    }
    
    #[test]
    fn test_beam_file_read_result_clone() {
        let result = BeamFileReadResult::Success;
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }
    
    #[test]
    fn test_beam_file_read_result_partial_eq() {
        let r1 = BeamFileReadResult::Success;
        let r2 = BeamFileReadResult::Success;
        let r3 = BeamFileReadResult::CorruptFileHeader;
        
        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }
    
    #[test]
    fn test_beam_load_error_variants() {
        let variants = vec![
            BeamLoadError::ModuleNotFound,
            BeamLoadError::OldCodeExists,
            BeamLoadError::InvalidModule,
        ];
        
        for variant in variants {
            let debug_str = format!("{:?}", variant);
            assert!(!debug_str.is_empty());
        }
    }
    
    #[test]
    fn test_beam_load_error_clone() {
        let error = BeamLoadError::InvalidModule;
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }
    
    #[test]
    fn test_beam_load_error_partial_eq() {
        let e1 = BeamLoadError::InvalidModule;
        let e2 = BeamLoadError::InvalidModule;
        let e3 = BeamLoadError::ModuleNotFound;
        
        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
    }
    
    #[test]
    fn test_beam_file_debug() {
        let beam = BeamFile {
            module: 1,
            code_data: vec![1, 2, 3],
            code_size: 3,
            exports: vec![(1, 2, 3)],
            imports: vec![(4, 5, 6)],
            atoms: vec!["atom1".to_string()],
            has_on_load: true,
            attributes_data: Some(vec![7, 8]),
            compile_info_data: Some(vec![9, 10]),
        };
        
        let debug_str = format!("{:?}", beam);
        assert!(debug_str.contains("BeamFile"));
    }
    
    #[test]
    fn test_beam_file_clone() {
        let beam = BeamFile {
            module: 1,
            code_data: vec![1, 2, 3],
            code_size: 3,
            exports: vec![(1, 2, 3)],
            imports: vec![(4, 5, 6)],
            atoms: vec!["atom1".to_string()],
            has_on_load: true,
            attributes_data: Some(vec![7, 8]),
            compile_info_data: Some(vec![9, 10]),
        };
        
        let cloned = beam.clone();
        assert_eq!(beam, cloned);
    }
    
    #[test]
    fn test_beam_file_partial_eq() {
        let beam1 = BeamFile {
            module: 1,
            code_data: vec![1, 2, 3],
            code_size: 3,
            exports: vec![],
            imports: vec![],
            atoms: vec![],
            has_on_load: false,
            attributes_data: None,
            compile_info_data: None,
        };
        
        let beam2 = BeamFile {
            module: 1,
            code_data: vec![1, 2, 3],
            code_size: 3,
            exports: vec![],
            imports: vec![],
            atoms: vec![],
            has_on_load: false,
            attributes_data: None,
            compile_info_data: None,
        };
        
        let beam3 = BeamFile {
            module: 2,
            code_data: vec![1, 2, 3],
            code_size: 3,
            exports: vec![],
            imports: vec![],
            atoms: vec![],
            has_on_load: false,
            attributes_data: None,
            compile_info_data: None,
        };
        
        assert_eq!(beam1, beam2);
        assert_ne!(beam1, beam3);
    }
    
    #[test]
    fn test_init_load() {
        // Test init_load (should not panic)
        BeamLoader::init_load();
    }
    
    #[test]
    fn test_make_current_old() {
        let module_manager = ModuleTableManager::new();
        let result = BeamLoader::make_current_old(&module_manager, 1);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_beam_file_read_code_chunk_size() {
        let mut data = vec![0u8; 32];
        let mut pos = 0;
        
        data[pos..pos+4].copy_from_slice(b"FOR1");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&20u32.to_le_bytes());
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"BEAM");
        pos += 4;
        data[pos..pos+4].copy_from_slice(b"Code");
        pos += 4;
        data[pos..pos+4].copy_from_slice(&10u32.to_be_bytes());
        pos += 4;
        data[pos..pos+10].copy_from_slice(&[0u8; 10]);
        pos += 10;
        
        data.truncate(pos);
        
        let result = BeamLoader::read_beam_file(&data);
        assert!(result.is_ok());
        let beam = result.unwrap();
        assert_eq!(beam.code_size, 10);
        assert_eq!(beam.code_data.len(), 10);
    }
}

