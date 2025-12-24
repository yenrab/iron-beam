//! BEAM Opcodes
//!
//! Definitions of BEAM instruction opcodes and their numeric values.
//! Based on the Erlang/OTP BEAM instruction set.

/// BEAM instruction opcodes
/// These match the opcodes defined in Erlang/OTP beam_opcodes.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BeamOpcode {
    // Control flow
    Label = 1,
    FuncInfo = 2,
    IntCodeEnd = 3,
    Line = 153,
    FuncLine = 89,
    EmptyFuncLine = 88,

    // Function calls
    Call = 4,
    CallLast = 5,
    CallOnly = 6,
    CallExt = 7,
    CallExtLast = 8,
    CallExtOnly = 78,
    CallBif = 93,  // Similar to CallExtOnly
    Bif0 = 9,
    Bif1 = 10,
    Bif2 = 11,
    GcBif2 = 125,

    // Function returns
    Return = 19,
    Send = 20,

    // Register operations
    Move = 64,
    GetList = 65,
    GetTupleElement = 66,
    SetTupleElement = 67,
    PutList = 69,
    PutTuple = 70,

    // Arithmetic
    Add = 27,  // genop_m_plus_4
    Subtract = 28,  // genop_m_minus_4
    Multiply = 29,  // genop_m_times_4
    Divide = 30,  // genop_m_div_4
    Negate = 348,  // genop_i_unary_minus_4

    // Comparisons
    IsLt = 39,
    IsGe = 40,
    IsEq = 41,
    IsNe = 42,
    IsEqExact = 43,
    IsNeExact = 44,

    // Type tests
    IsInteger = 45,
    IsFloat = 46,
    IsNumber = 47,
    IsAtom = 48,
    IsPid = 49,
    IsReference = 50,
    IsPort = 51,
    IsNil = 52,
    IsBinary = 53,
    IsList = 55,
    IsNonemptyList = 56,
    IsTuple = 57,
    IsBitstring = 249,  // genop_is_bitstring_2
    IsBoolean = 114,  // genop_is_boolean_2
    IsFunction2 = 174,

    // Control flow
    Jump = 61,
    Badmatch = 72,  // genop_badmatch_1
    IfEnd = 73,  // genop_if_end_0
    CaseEnd = 74,  // genop_case_end_1
    Try = 104,  // genop_try_2
    TryEnd = 105,  // genop_try_end_1
    TryCase = 106,  // genop_try_case_1
    TryCaseEnd = 107,  // genop_try_case_end_1
    Raise = 108,  // genop_raise_2
    Catch = 62,  // genop_catch_2
    CatchEnd = 63,  // genop_catch_end_1

    // Stack operations
    Allocate = 12,  // genop_allocate_2
    AllocateHeap = 13,  // genop_allocate_heap_3
    Deallocate = 18,  // genop_deallocate_1
    Trim = 136,  // genop_trim_2
    TestHeap = 16,  // genop_test_heap_2
    InitYregs = 17,  // genop_init_1

    // List operations
    PutList2 = 54,
    GetHd = 162,  // genop_get_hd_2
    GetTl = 163,  // genop_get_tl_2

    // Tuple operations
    PutTuple2 = 164,  // genop_put_tuple2_2
    GetTupleElement2 = 58,

    // More arithmetic
    Plus = 351,  // Placeholder - specific opcode
    Minus = 352,  // Placeholder - specific opcode
    Div = 353,  // Placeholder - specific opcode
    Rem = 354,  // Placeholder - specific opcode
    Bsl = 36,  // genop_int_bsl_4
    Bsr = 37,  // genop_int_bsr_4
    Band = 33,  // genop_int_band_4
    Bor = 34,  // genop_int_bor_4
    Bxor = 35,  // genop_int_bxor_4
    Bnot = 38,  // genop_int_bnot_3

    // Bit syntax operations
    BsGetInteger2 = 988,
    BsGetBinary2 = 991,
    BsGetFloat2 = 1009,
    BsSkipBits2 = 1018,
    BsTestTail2 = 1023,
    BsStartMatch3 = 1038,
    BsGetPosition = 1064,
    BsSetPosition = 1065,  // Similar to BsGetPosition
    BsMatchString = 983,

    // Miscellaneous operations
    BuildStacktrace = 1367,
    RawRaise = 1368,
    OnLoad = 1360,
    RecvMarkerReserve = 1374,
    RecvMarkerBind = 1375,
    RecvMarkerClear = 1376,
    RecvMarkerUse = 1377,
}

impl BeamOpcode {
    /// Convert the current opcode enum value to the correct C implementation opcode number
    /// This allows us to keep the existing Rust enum names while using correct C numbers
    pub fn to_c_opcode(self) -> u32 {
        match self {
            // Generic opcodes - map to C numbers where they differ
            BeamOpcode::Label => 188,  // op_label_L
            BeamOpcode::Line => 91,    // op_line_I
            BeamOpcode::FuncLine => 89,  // op_func_line_I
            BeamOpcode::EmptyFuncLine => 88,  // op_empty_func_line
            BeamOpcode::CallOnly => 99,  // op_i_call_only_f
            BeamOpcode::CallExtOnly => 92,  // op_call_ext_only_uF
            BeamOpcode::CallBif => 92,  // Same as CallExtOnly
            BeamOpcode::Return => 217,  // op_return
            BeamOpcode::Send => 220,  // op_send
            BeamOpcode::Move => 138,  // op_i_move_sd
            BeamOpcode::Allocate => 104,  // op_allocate_tI
            BeamOpcode::AllocateHeap => 105,  // op_allocate_heap_tII
            BeamOpcode::Deallocate => 107,  // op_deallocate_t
            BeamOpcode::Trim => 109,  // op_trim_tI
            BeamOpcode::TestHeap => 111,  // op_test_heap_I t
            BeamOpcode::InitYregs => 117,  // op_init_yregs_sz
            BeamOpcode::GetList => 51,  // op_get_list_Sdd
            BeamOpcode::SetTupleElement => 221,  // op_set_tuple_element_sSP
            BeamOpcode::PutList => 206,  // op_put_list_ssd
            BeamOpcode::PutTuple => 19,  // op_put_tuple (generic)
            BeamOpcode::IsLt => 177,  // op_is_lt_fss
            BeamOpcode::IsGe => 169,  // op_is_ge_fss
            BeamOpcode::IsEq => 164,  // op_is_eq_fss
            BeamOpcode::IsNe => 179,  // op_is_ne_fss
            BeamOpcode::IsEqExact => 165,  // op_is_eq_exact_fss
            BeamOpcode::IsNeExact => 180,  // op_is_ne_exact_fss
            BeamOpcode::IsInteger => 175,  // op_is_integer_fs
            BeamOpcode::IsFloat => 166,  // op_is_float_fs
            BeamOpcode::IsNumber => 183,  // op_is_number_fs
            BeamOpcode::IsAtom => 160,  // op_is_atom_fs
            BeamOpcode::IsPid => 184,  // op_is_pid_fs
            BeamOpcode::IsReference => 186,  // op_is_reference_fs
            BeamOpcode::IsPort => 185,  // op_is_port_fs
            BeamOpcode::IsNil => 181,  // op_is_nil_fS
            BeamOpcode::IsBinary => 161,  // op_is_binary_fs
            BeamOpcode::IsList => 176,  // op_is_list_fs
            BeamOpcode::IsNonemptyList => 182,  // op_is_nonempty_list_fS
            BeamOpcode::IsTuple => 123,  // op_i_is_tuple_fs
            BeamOpcode::IsBitstring => 249,  // genop_is_bitstring_2
            BeamOpcode::IsBoolean => 163,  // op_is_boolean_fs
            BeamOpcode::IsFunction2 => 174,  // op_is_function2_fs
            BeamOpcode::Jump => 187,  // op_jump_f
            BeamOpcode::IfEnd => 157,  // op_if_end
            BeamOpcode::CaseEnd => 38,  // op_case_end_s
            BeamOpcode::PutList2 => 207,  // op_put_list2_sssd
            BeamOpcode::GetHd => 50,  // op_get_hd_Sd
            BeamOpcode::GetTl => 52,  // op_get_tl_Sd
            BeamOpcode::PutTuple2 => 209,  // op_put_tuple2_SA
            BeamOpcode::GetTupleElement2 => 58,  // op_i_get_tuple_element_sPS
            BeamOpcode::Plus => 143,  // op_i_plus_jIssd
            BeamOpcode::Minus => 137,  // op_i_minus_jIssd
            BeamOpcode::Div => 23,    // op_/ is generic
            BeamOpcode::Rem => 144,   // op_i_rem_jIssd
            BeamOpcode::Bsl => 84,    // op_i_bsl_jIssd
            BeamOpcode::Bsr => 85,    // op_i_bsr_jIssd
            BeamOpcode::Band => 61,   // op_i_band_jIssd
            BeamOpcode::Bor => 66,    // op_i_bor_jIssd
            BeamOpcode::Bxor => 86,   // op_i_bxor_jIssd
            BeamOpcode::Bnot => 65,   // op_i_bnot_jIsd
            BeamOpcode::BsGetInteger2 => 988,  // op_bs_get_integer2_fS t s t t d
            BeamOpcode::BsGetBinary2 => 991,   // op_bs_get_binary2_fS t s t _F _U d
            BeamOpcode::BsGetFloat2 => 1009,   // op_bs_get_float2_fS t s t _F d
            BeamOpcode::BsSkipBits2 => 1018,   // op_bs_skip_bits2_fS s t
            BeamOpcode::BsTestTail2 => 1023,   // op_bs_test_tail2_fS W
            BeamOpcode::BsStartMatch3 => 1038, // op_bs_start_match3_f _L S d
            BeamOpcode::BsGetPosition => 1064, // op_bs_get_position_S d _L
            BeamOpcode::BsSetPosition => 1064, // Same as BsGetPosition
            BeamOpcode::BsMatchString => 983,  // op_bs_match_string_f S W M
            BeamOpcode::BuildStacktrace => 1367, // op_build_stacktrace
            BeamOpcode::RawRaise => 1368, // op_raw_raise
            BeamOpcode::OnLoad => 1360, // op_on_load

            // Keep existing numbers for opcodes that match C implementation
            other => other as u32,
        }
    }

    /// Convert a u32 back to a BeamOpcode
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(BeamOpcode::Label),
            2 => Some(BeamOpcode::FuncInfo),
            3 => Some(BeamOpcode::IntCodeEnd),
            4 => Some(BeamOpcode::Call),
            5 => Some(BeamOpcode::CallLast),
            6 => Some(BeamOpcode::CallOnly),
            7 => Some(BeamOpcode::CallExt),
            8 => Some(BeamOpcode::CallExtLast),
            78 => Some(BeamOpcode::CallExtOnly),
            9 => Some(BeamOpcode::Bif0),
            10 => Some(BeamOpcode::Bif1),
            11 => Some(BeamOpcode::Bif2),
            12 => Some(BeamOpcode::Allocate),
            13 => Some(BeamOpcode::AllocateHeap),
            18 => Some(BeamOpcode::Deallocate),
            125 => Some(BeamOpcode::GcBif2),
            19 => Some(BeamOpcode::Return),
            20 => Some(BeamOpcode::Send),
            27 => Some(BeamOpcode::Add),
            28 => Some(BeamOpcode::Subtract),
            29 => Some(BeamOpcode::Multiply),
            30 => Some(BeamOpcode::Divide),
            33 => Some(BeamOpcode::Band),
            34 => Some(BeamOpcode::Bor),
            35 => Some(BeamOpcode::Bxor),
            36 => Some(BeamOpcode::Bsl),
            37 => Some(BeamOpcode::Bsr),
            38 => Some(BeamOpcode::Bnot),
            348 => Some(BeamOpcode::Negate),
            62 => Some(BeamOpcode::Catch),
            63 => Some(BeamOpcode::CatchEnd),
            64 => Some(BeamOpcode::Move),
            65 => Some(BeamOpcode::GetList),
            66 => Some(BeamOpcode::GetTupleElement),
            67 => Some(BeamOpcode::SetTupleElement),
            69 => Some(BeamOpcode::PutList),
            70 => Some(BeamOpcode::PutTuple),
            39 => Some(BeamOpcode::IsLt),
            40 => Some(BeamOpcode::IsGe),
            41 => Some(BeamOpcode::IsEq),
            42 => Some(BeamOpcode::IsNe),
            43 => Some(BeamOpcode::IsEqExact),
            44 => Some(BeamOpcode::IsNeExact),
            45 => Some(BeamOpcode::IsInteger),
            46 => Some(BeamOpcode::IsFloat),
            47 => Some(BeamOpcode::IsNumber),
            48 => Some(BeamOpcode::IsAtom),
            49 => Some(BeamOpcode::IsPid),
            50 => Some(BeamOpcode::IsReference),
            51 => Some(BeamOpcode::IsPort),
            52 => Some(BeamOpcode::IsNil),
            53 => Some(BeamOpcode::IsBinary),
            55 => Some(BeamOpcode::IsList),
            56 => Some(BeamOpcode::IsNonemptyList),
            57 => Some(BeamOpcode::IsTuple),
            61 => Some(BeamOpcode::Jump),
            72 => Some(BeamOpcode::Badmatch),
            73 => Some(BeamOpcode::IfEnd),
            74 => Some(BeamOpcode::CaseEnd),
            104 => Some(BeamOpcode::Try),
            105 => Some(BeamOpcode::TryEnd),
            106 => Some(BeamOpcode::TryCase),
            107 => Some(BeamOpcode::TryCaseEnd),
            108 => Some(BeamOpcode::Raise),
            54 => Some(BeamOpcode::PutList2),
            58 => Some(BeamOpcode::GetTupleElement2),
            136 => Some(BeamOpcode::Trim),
            16 => Some(BeamOpcode::TestHeap),
            17 => Some(BeamOpcode::InitYregs),
            // Additional opcodes
            88 => Some(BeamOpcode::EmptyFuncLine),
            89 => Some(BeamOpcode::FuncLine),
            91 => Some(BeamOpcode::Line),
            99 => Some(BeamOpcode::CallOnly),  // Updated mapping
            104 => Some(BeamOpcode::Allocate),
            105 => Some(BeamOpcode::AllocateHeap),
            107 => Some(BeamOpcode::Deallocate),
            109 => Some(BeamOpcode::Trim),
            111 => Some(BeamOpcode::TestHeap),
            117 => Some(BeamOpcode::InitYregs),
            123 => Some(BeamOpcode::IsTuple),  // Updated mapping
            138 => Some(BeamOpcode::Move),     // Updated mapping
            143 => Some(BeamOpcode::Plus),     // Updated mapping
            144 => Some(BeamOpcode::Rem),      // Updated mapping
            157 => Some(BeamOpcode::IfEnd),    // Updated mapping
            160 => Some(BeamOpcode::IsAtom),   // Updated mapping
            161 => Some(BeamOpcode::IsBinary), // Updated mapping
            249 => Some(BeamOpcode::IsBitstring),
            114 => Some(BeamOpcode::IsBoolean),
            164 => Some(BeamOpcode::IsEq),     // Updated mapping
            165 => Some(BeamOpcode::IsEqExact), // Updated mapping
            166 => Some(BeamOpcode::IsFloat),   // Updated mapping
            169 => Some(BeamOpcode::IsGe),      // Updated mapping
            174 => Some(BeamOpcode::IsFunction2),
            175 => Some(BeamOpcode::IsInteger), // Updated mapping
            176 => Some(BeamOpcode::IsList),    // Updated mapping
            177 => Some(BeamOpcode::IsLt),      // Updated mapping
            179 => Some(BeamOpcode::IsNe),      // Updated mapping
            180 => Some(BeamOpcode::IsNeExact), // Updated mapping
            181 => Some(BeamOpcode::IsNil),     // Updated mapping
            182 => Some(BeamOpcode::IsNonemptyList), // Updated mapping
            183 => Some(BeamOpcode::IsNumber),   // Updated mapping
            184 => Some(BeamOpcode::IsPid),      // Updated mapping
            185 => Some(BeamOpcode::IsPort),     // Updated mapping
            186 => Some(BeamOpcode::IsReference), // Updated mapping
            187 => Some(BeamOpcode::Jump),       // Updated mapping
            188 => Some(BeamOpcode::Label),      // Updated mapping
            206 => Some(BeamOpcode::PutList),    // Updated mapping
            207 => Some(BeamOpcode::PutList2),   // Updated mapping
            209 => Some(BeamOpcode::PutTuple2),  // Updated mapping
            217 => Some(BeamOpcode::Return),     // Updated mapping
            220 => Some(BeamOpcode::Send),       // Updated mapping
            221 => Some(BeamOpcode::SetTupleElement), // Updated mapping
            983 => Some(BeamOpcode::BsMatchString),
            988 => Some(BeamOpcode::BsGetInteger2),
            991 => Some(BeamOpcode::BsGetBinary2),
            1009 => Some(BeamOpcode::BsGetFloat2),
            1018 => Some(BeamOpcode::BsSkipBits2),
            1023 => Some(BeamOpcode::BsTestTail2),
            1038 => Some(BeamOpcode::BsStartMatch3),
            1064 => Some(BeamOpcode::BsGetPosition),
            1360 => Some(BeamOpcode::OnLoad),
            162 => Some(BeamOpcode::GetHd),
            163 => Some(BeamOpcode::GetTl),
            164 => Some(BeamOpcode::PutTuple2),
            348 => Some(BeamOpcode::Negate),
            1367 => Some(BeamOpcode::BuildStacktrace),
            1368 => Some(BeamOpcode::RawRaise),
            1374 => Some(BeamOpcode::RecvMarkerReserve),
            1375 => Some(BeamOpcode::RecvMarkerBind),
            1376 => Some(BeamOpcode::RecvMarkerClear),
            1377 => Some(BeamOpcode::RecvMarkerUse),
            _ => None,
        }
    }
}
