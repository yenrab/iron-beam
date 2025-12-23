//! BEAM Code Parser
//!
//! Parses BEAM bytecode from code chunks into structured instruction representations.

use super::types::*;
// BeamOpcode not used in parser.rs
use std::io::{Cursor, Read};

/// Errors that can occur during BEAM parsing
#[derive(Debug, thiserror::Error)]
pub enum BeamParseError {
    #[error("Invalid code header")]
    InvalidHeader,
    #[error("Unexpected end of code")]
    UnexpectedEnd,
    #[error("Unknown opcode: {0}")]
    UnknownOpcode(u32),
    #[error("Invalid argument encoding")]
    InvalidArgument,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// BEAM code parser
pub struct BeamParser;

impl BeamParser {
    /// Get the number of arguments for a given opcode
    pub fn get_opcode_arity(opcode: u32) -> usize {
        match opcode {
            1 => 1,   // label
            2 => 3,   // func_info
            3 => 0,   // int_code_end
            4 => 2,   // call
            5 => 3,   // call_last
            6 => 2,   // call_only
            7 => 2,   // call_ext
            8 => 3,   // call_ext_last
            9 => 2,   // bif0
            10 => 4,  // bif1
            11 => 5,  // bif2
            12 => 2,  // allocate
            13 => 3,  // allocate_heap
            14 => 2,  // allocate_zero
            15 => 3,  // allocate_heap_zero
            16 => 2,  // test_heap
            17 => 1,  // init
            18 => 1,  // deallocate
            19 => 0,  // return
            20 => 0,  // send
            21 => 0,  // remove_message
            22 => 0,  // timeout
            23 => 2,  // loop_rec
            24 => 1,  // loop_rec_end
            25 => 1,  // wait
            26 => 2,  // wait_timeout
            27 => 4,  // m_plus
            28 => 4,  // m_minus
            29 => 4,  // m_times
            30 => 4,  // m_div
            45 => 2,  // is_integer
            46 => 2,  // is_float
            47 => 2,  // is_number
            48 => 2,  // is_atom
            49 => 2,  // is_pid
            50 => 2,  // is_reference
            51 => 2,  // is_port
            52 => 2,  // is_nil
            39 => 3,  // is_lt
            40 => 3,  // is_ge
            41 => 3,  // is_eq
            42 => 3,  // is_ne
            43 => 3,  // is_eq_exact
            44 => 3,  // is_ne_exact
            49 => 2,  // is_pid
            50 => 2,  // is_reference
            51 => 2,  // is_port
            52 => 2,  // is_nil
            53 => 2,  // is_binary
            54 => 2,  // is_constant
            55 => 2,  // is_list
            56 => 2,  // is_nonempty_list
            57 => 2,  // is_tuple
            58 => 3,  // test_arity
            59 => 3,  // select_val
            60 => 3,  // select_tuple_arity
            61 => 1,  // jump
            62 => 2,  // catch
            63 => 1,  // catch_end
            64 => 2,  // move (also used for BIF calls in some contexts)
            65 => 3,  // get_list
            66 => 3,  // get_tuple_element
            67 => 3,  // set_tuple_element
            68 => 3,  // put_string
            69 => 3,  // put_list
            70 => 2,  // put_tuple
            71 => 1,  // put
            72 => 1,  // badmatch
            73 => 0,  // if_end
            74 => 1,  // case_end
            75 => 1,  // call_fun
            76 => 3,  // make_fun
            77 => 2,  // is_function
            78 => 2,  // call_ext_only
            79 => 2,  // bs_start_match
            80 => 5,  // bs_get_integer
            81 => 5,  // bs_get_float
            82 => 5,  // bs_get_binary
            83 => 4,  // bs_skip_bits
            84 => 2,  // bs_test_tail
            85 => 1,  // bs_save
            86 => 1,  // bs_restore
            87 => 2,  // bs_init
            88 => 2,  // bs_final
            89 => 5,  // bs_put_integer
            90 => 5,  // bs_put_float
            91 => 5,  // bs_put_binary
            92 => 2,  // bs_put_string
            93 => 1,  // bs_need_buf
            94 => 0,  // fclearerror
            95 => 1,  // fcheckerror
            96 => 2,  // fmove
            97 => 2,  // fconv
            98 => 4,  // fadd
            99 => 4,  // fsub
            100 => 4,  // fmul
            101 => 4,  // fdiv
            102 => 3,  // fnegate
            103 => 1,  // make_fun2
            104 => 2,  // try
            105 => 1,  // try_end
            106 => 1,  // try_case
            107 => 1,  // try_case_end
            108 => 2,  // raise
            109 => 6,  // bs_init2
            110 => 3,  // bs_bits_to_bytes
            111 => 5,  // bs_add
            112 => 1,  // apply
            113 => 2,  // apply_last
            114 => 2,  // is_boolean
            115 => 3,  // is_function2
            116 => 5,  // bs_start_match2
            117 => 7,  // bs_get_integer2
            118 => 7,  // bs_get_float2
            119 => 7,  // bs_get_binary2
            120 => 5,  // bs_skip_bits2
            121 => 3,  // bs_test_tail2
            122 => 2,  // bs_save2
            123 => 2,  // bs_restore2
            124 => 5,  // gc_bif1
            125 => 6,  // gc_bif2
            126 => 2,  // bs_final2
            127 => 2,  // bs_bits_to_bytes2
            128 => 2,  // put_literal
            129 => 2,  // is_bitstr
            130 => 1,  // bs_context_to_binary
            131 => 3,  // bs_test_unit
            132 => 4,  // bs_match_string
            133 => 0,  // bs_init_writable
            134 => 8,  // bs_append
            135 => 6,  // bs_private_append
            136 => 2,  // trim
            137 => 6,  // bs_init_bits
            138 => 5,  // bs_get_utf8
            139 => 4,  // bs_skip_utf8
            140 => 5,  // bs_get_utf16
            141 => 4,  // bs_skip_utf16
            142 => 5,  // bs_get_utf32
            143 => 4,  // bs_skip_utf32
            144 => 3,  // bs_utf8_size
            145 => 3,  // bs_put_utf8
            146 => 3,  // bs_utf16_size
            147 => 3,  // bs_put_utf16
            148 => 3,  // bs_put_utf32
            149 => 0,  // on_load
            150 => 1,  // recv_mark
            151 => 1,  // recv_set
            152 => 7,  // gc_bif3
            153 => 1,  // line
            154 => 5,  // put_map_assoc
            155 => 5,  // put_map_exact
            156 => 2,  // is_map
            157 => 3,  // has_map_fields
            158 => 3,  // get_map_elements
            159 => 4,  // is_tagged_tuple
            160 => 0,  // build_stacktrace
            161 => 0,  // raw_raise
            162 => 2,  // get_hd
            163 => 2,  // get_tl
            164 => 2,  // put_tuple2
            165 => 3,  // bs_get_tail
            166 => 4,  // bs_start_match3
            167 => 3,  // bs_get_position
            168 => 2,  // bs_set_position
            169 => 2,  // swap
            170 => 4,  // bs_start_match4
            171 => 3,  // make_fun3
            172 => 1,  // init_yregs
            173 => 2,  // recv_marker_bind
            174 => 1,  // recv_marker_clear
            175 => 1,  // recv_marker_reserve
            176 => 1,  // recv_marker_use
            177 => 6,  // bs_create_bin
            178 => 3,  // call_fun2
            179 => 0,  // nif_start
            180 => 1,  // badrecord
            181 => 5,  // update_record
            182 => 3,  // bs_match
            183 => 2,  // executable_line
            184 => 4,  // debug_line
            185 => 6,  // bif3
            186 => 4,  // i_func_info
            187 => 0,  // i_generic_breakpoint
            188 => 0,  // i_debug_breakpoint
            189 => 0,  // i_call_trace_return
            190 => 0,  // i_return_to_trace
            191 => 1,  // i_disabled_line_breakpoint
            192 => 1,  // i_enabled_line_breakpoint
            193 => 0,  // i_line_breakpoint_cleanup
            194 => 0,  // i_yield
            195 => 1,  // trace_jump
            196 => 5,  // int_func_start
            197 => 2,  // int_func_end
            198 => 0,  // i_nif_padding
            199 => 0,  // padding
            200 => 2,  // i_debug_line
            201 => 2,  // i_allocate_zero
            202 => 3,  // i_allocate_heap_zero
            203 => 1,  // i_init
            204 => 3,  // move_trim
            205 => 1,  // i_trim
            206 => 1,  // i_init_seq3
            207 => 1,  // i_init_seq4
            208 => 1,  // i_init_seq5
            209 => 2,  // i_init2
            210 => 3,  // i_init3
            211 => 3,  // i_select_val_bins
            212 => 3,  // i_select_val_lins
            213 => 4,  // i_select_val2
            214 => 3,  // i_select_tuple_arity
            215 => 4,  // i_select_tuple_arity2
            216 => 3,  // i_jump_on_val_zero
            217 => 4,  // i_jump_on_val
            218 => 3,  // i_get_tuple_element
            219 => 3,  // i_get_tuple_element2
            220 => 4,  // i_get_tuple_element2_dst
            221 => 3,  // i_get_tuple_element3
            222 => 0,  // i_raise
            223 => 0,  // delete_me
            224 => 1,  // system_limit
            225 => 0,  // system_limit_body
            226 => 3,  // move_jump
            227 => 3,  // move_window2
            228 => 4,  // move_window3
            229 => 5,  // move_window4
            230 => 6,  // move_window5
            231 => 4,  // move_src_window
            232 => 5,  // move_src_window2
            233 => 4,  // move_src_window3
            234 => 5,  // move_src_window4
            235 => 3,  // swap2
            236 => 3,  // move_shift
            237 => 4,  // move2_par
            238 => 6,  // move3
            239 => 0,  // timeout_locked
            240 => 1,  // i_loop_rec
            241 => 1,  // wait_locked
            242 => 1,  // wait_unlocked
            243 => 2,  // wait_timeout_unlocked_int
            244 => 2,  // wait_timeout_unlocked
            245 => 2,  // wait_timeout_locked_int
            246 => 2,  // wait_timeout_locked
            247 => 0,  // i_wait_error
            248 => 0,  // i_wait_error_locked
            249 => 3,  // i_is_eq_exact_immed
            250 => 3,  // i_is_ne_exact_immed
            251 => 3,  // i_is_ne_exact_literal
            252 => 3,  // is_lt_literal
            253 => 3,  // is_ge_literal
            254 => 2,  // update_list
            255 => 0,  // normal_exit
            256 => 0,  // continue_exit
            257 => 1,  // call_bif
            258 => 3,  // call_nif
            259 => 0,  // call_nif_early
            260 => 0,  // call_error_handler
            261 => 0,  // error_action_code
            262 => 0,  // return_trace
            263 => 0,  // move_return
            264 => 1,  // move_deallocate_return
            265 => 0,  // deallocate_return0
            266 => 0,  // deallocate_return1
            267 => 0,  // deallocate_return2
            268 => 0,  // deallocate_return3
            269 => 0,  // deallocate_return4
            270 => 1,  // deallocate_return
            271 => 2,  // test_heap1_put_list
            272 => 3,  // is_tuple_of_arity
            273 => 5,  // test_arity_get_tuple_element
            274 => 5,  // is_tagged_tuple_ff
            275 => 4,  // is_integer_allocate
            276 => 4,  // is_nonempty_list_allocate
            277 => 4,  // is_nonempty_list_get_list
            278 => 3,  // is_nonempty_list_get_hd
            279 => 3,  // is_nonempty_list_get_tl
            280 => 2,  // is_bitstring
            281 => 3,  // cold_is_function2
            282 => 3,  // hot_is_function2
            283 => 3,  // allocate_init
            284 => 1,  // call_light_bif
            285 => 1,  // call_light_bif_only
            286 => 2,  // call_light_bif_last
            287 => 0,  // i_load_nif
            288 => 0,  // i_apply
            289 => 1,  // i_apply_last
            290 => 0,  // i_apply_only
            291 => 0,  // i_apply_fun
            292 => 1,  // i_apply_fun_last
            293 => 0,  // i_apply_fun_only
            294 => 2,  // call_light_bif2
            295 => 2,  // call_light_bif_only2
            296 => 0,  // i_hibernate
            297 => 0,  // i_perf_counter
            298 => 3,  // i_get_hash
            299 => 2,  // i_get
            300 => 1,  // self
            301 => 1,  // node
            302 => 4,  // i_fast_element
            303 => 4,  // i_element
            304 => 4,  // i_bif1
            305 => 3,  // i_bif1_body
            306 => 5,  // i_bif2
            307 => 4,  // i_bif2_body
            308 => 6,  // i_bif3
            309 => 5,  // i_bif3_body
            310 => 2,  // move_call
            311 => 3,  // move_call_last
            312 => 2,  // move_call_only
            313 => 1,  // i_call
            314 => 2,  // i_call_last
            315 => 1,  // i_call_only
            316 => 1,  // i_call_ext
            317 => 2,  // i_call_ext_last
            318 => 1,  // i_call_ext_only
            319 => 2,  // i_move_call_ext
            320 => 3,  // i_move_call_ext_last
            321 => 2,  // i_move_call_ext_only
            322 => 1,  // i_call_fun
            323 => 2,  // i_call_fun_last
            324 => 4,  // i_make_fun3
            325 => 1,  // i_lambda_error
            326 => 3,  // i_bs_ensure_bits
            327 => 4,  // i_bs_ensure_bits_unit
            328 => 2,  // i_bs_read_bits
            329 => 3,  // i_bs_eq
            330 => 2,  // i_bs_extract_integer
            331 => 2,  // i_bs_read_integer8
            332 => 4,  // i_bs_get_fixed_integer
            333 => 3,  // i_bs_get_fixed_binary
            334 => 2,  // i_bs_get_tail
            335 => 2,  // i_bs_skip
            336 => 1,  // i_bs_drop
            337 => 3,  // i_bs_ensure_bits_read
            338 => 1,  // bad_bs_match
            339 => 4,  // i_bs_match_string
            340 => 5,  // i_bs_get_integer_small_imm
            341 => 6,  // i_bs_get_integer_imm
            342 => 6,  // i_bs_get_integer
            343 => 3,  // i_bs_get_integer8
            344 => 3,  // i_bs_get_integer16
            345 => 3,  // i_bs_get_integer32
            346 => 5,  // i_bs_get_binary_imm2
            347 => 6,  // i_bs_get_binary2
            348 => 5,  // i_bs_get_binary_all2
            349 => 6,  // i_bs_get_float2
            350 => 4,  // i_bs_skip_bits2
            351 => 2,  // bs_test_zero_tail2
            352 => 3,  // bs_test_tail_imm2
            353 => 2,  // bs_test_unit8
            354 => 5,  // i_bs_start_match3_gp
            355 => 4,  // i_bs_start_match3
            356 => 2,  // i_bs_get_position
            357 => 3,  // i_bs_get_utf8
            358 => 4,  // i_bs_get_utf16
            359 => 3,  // i_bs_validate_unicode_retract
            360 => 5,  // i_bs_create_bin
            361 => 2,  // fstore
            362 => 2,  // fload
            363 => 3,  // ifadd
            364 => 3,  // ifsub
            365 => 3,  // ifmul
            366 => 3,  // ifdiv
            367 => 2,  // ifnegate
            368 => 4,  // i_put_map_assoc
            369 => 4,  // sorted_put_map_assoc
            370 => 5,  // sorted_put_map_exact
            371 => 3,  // new_map
            372 => 3,  // i_new_small_map_lit
            373 => 4,  // update_map_assoc
            374 => 5,  // update_map_exact
            375 => 3,  // i_get_map_elements
            376 => 5,  // i_get_map_element_hash
            377 => 4,  // i_get_map_element
            378 => 5,  // gen_plus
            379 => 5,  // gen_minus
            380 => 3,  // i_increment
            381 => 4,  // i_plus
            382 => 3,  // i_unary_minus
            383 => 4,  // i_minus
            384 => 4,  // i_times
            385 => 4,  // im_div
            386 => 4,  // i_int_div
            387 => 4,  // i_rem
            388 => 4,  // i_bsl
            389 => 4,  // i_bsr
            390 => 4,  // i_band
            391 => 4,  // i_bor
            392 => 4,  // i_bxor
            393 => 3,  // i_int_bnot
            394 => 2,  // i_length_setup
            395 => 3,  // i_length
            396 => 3,  // unsupported_guard_bif
            397 => 1,  // move_x1
            398 => 1,  // move_x2
            399 => 5,  // i_update_record_copy
            400 => 5,  // i_update_record_in_place
            401 => 2,  // i_update_record_continue
            402 => 0,  // i_update_record_in_place_done
            403 => 2,  // i_update_record_in_place_done2
            404 => 2,  // i_update_record_in_place_done2
            _ => 0, // Unknown opcode, assume 0 arguments
        }
    }

    /// Parse a BEAM code chunk into structured instructions
    ///
    /// # Arguments
    /// * `code_data` - Raw code chunk data from BEAM file
    ///
    /// # Returns
    /// Parsed BEAM code or error
    pub fn parse_code(code_data: &[u8]) -> Result<BeamCode, BeamParseError> {
        let mut cursor = Cursor::new(code_data);

        // Parse header
        let header = Self::parse_header(&mut cursor)?;

        // Parse instructions
        let mut instructions = Vec::new();
        let mut functions = Vec::new();
        let mut current_function = None;

        while cursor.position() < code_data.len() as u64 {
            let opcode = Self::read_u8(&mut cursor)? as u32;
            let instruction = Self::parse_instruction(&mut cursor, opcode)?;
            instructions.push(instruction.clone());

            // Check for function boundaries
            if let Some(op) = instruction.opcode_enum() {
                match op {
                    super::BeamOpcode::FuncInfo => {
                        // Start of new function
                        if let Some(func) = current_function.take() {
                            functions.push(func);
                        }

                        // Parse function info: module, function, arity
                        if instruction.args.len() >= 3 {
                            if let (BeamArg::Literal(module), BeamArg::Literal(function), BeamArg::Literal(arity)) =
                                (&instruction.args[0], &instruction.args[1], &instruction.args[2]) {
                                current_function = Some(BeamFunction {
                                    module: *module as u32,
                                    function: *function as u32,
                                    arity: *arity as u32,
                                    entry_label: 0, // Will be set by label instruction
                                    instructions: Vec::new(),
                                });
                            }
                        }
                    }
                    super::BeamOpcode::Label => {
                        // Update entry label for current function
                        if let Some(ref mut func) = current_function {
                            if let Some(BeamArg::Label(label)) = instruction.args.first() {
                                func.entry_label = *label;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Add instruction to current function
            if let Some(ref mut func) = current_function {
                func.instructions.push(instruction);
            }
        }

        // Add final function
        if let Some(func) = current_function {
            functions.push(func);
        }

        Ok(BeamCode {
            header,
            functions,
            raw_code: code_data.to_vec(),
        })
    }

    /// Parse BEAM code header
    fn parse_header(cursor: &mut Cursor<&[u8]>) -> Result<BeamCodeHeader, BeamParseError> {
        let sub_size = Self::read_u32_be(cursor)?;
        let instruction_set = Self::read_u32_be(cursor)?;
        let max_opcode = Self::read_u32_be(cursor)?;
        let label_count = Self::read_u32_be(cursor)?;
        let function_count = Self::read_u32_be(cursor)?;

        Ok(BeamCodeHeader {
            sub_size,
            instruction_set,
            max_opcode,
            label_count,
            function_count,
        })
    }

    /// Parse a single BEAM instruction
    fn parse_instruction(cursor: &mut Cursor<&[u8]>, opcode: u32) -> Result<BeamInstruction, BeamParseError> {
        let mut args = Vec::new();

        // Get the number of arguments for this opcode
        let arity = Self::get_opcode_arity(opcode);

        // Read the correct number of arguments
        for _ in 0..arity {
            let arg = Self::read_arg(cursor)?;
            args.push(arg);
        }

        Ok(BeamInstruction::new(opcode, args))
    }

    /// Read a generic argument
    fn read_arg(cursor: &mut Cursor<&[u8]>) -> Result<BeamArg, BeamParseError> {
        // Read tag byte to determine argument type
        let tag = Self::read_u8(cursor)?;

        match tag {
            0..=127 => {
                // Small integer or atom
                Ok(BeamArg::Literal(tag as u64))
            }
            128..=255 => {
                // Extended encoding
                match tag {
                    0x80..=0xBF => {
                        // Literal value
                        let value = Self::read_u32_be(cursor)?;
                        Ok(BeamArg::Literal(value as u64))
                    }
                    0xC0..=0xDF => {
                        // Register X
                        let index = tag & 0x1F;
                        Ok(BeamArg::Register { index: index as u32, is_y: false })
                    }
                    0xE0..=0xFF => {
                        // Register Y
                        let index = tag & 0x1F;
                        Ok(BeamArg::Register { index: index as u32, is_y: true })
                    }
                    _ => Err(BeamParseError::InvalidArgument),
                }
            }
            _ => Err(BeamParseError::InvalidArgument),
        }
    }

    /// Read a u32 in big-endian format
    fn read_u32_be(cursor: &mut Cursor<&[u8]>) -> Result<u32, BeamParseError> {
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Read a u8
    fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, BeamParseError> {
        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Read a u32 argument
    fn read_u32_arg(cursor: &mut Cursor<&[u8]>) -> Result<u32, BeamParseError> {
        Self::read_u32_be(cursor)
    }
}
