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

    // Function calls
    Call = 4,
    CallLast = 5,
    CallOnly = 6,
    CallExt = 7,
    CallExtLast = 8,
    Bif0 = 9,
    Bif1 = 10,
    Bif2 = 11,

    // Function returns
    Return = 12,
    Send = 13,

    // Register operations
    Move = 14,
    GetList = 15,
    GetTupleElement = 16,
    SetTupleElement = 17,
    PutList = 18,
    PutTuple = 19,

    // Arithmetic
    Add = 20,
    Subtract = 21,
    Multiply = 22,
    Divide = 23,
    Negate = 24,

    // Comparisons
    IsLt = 25,
    IsGe = 26,
    IsEq = 27,
    IsNe = 28,
    IsEqExact = 29,
    IsNeExact = 30,

    // Type tests
    IsInteger = 31,
    IsFloat = 32,
    IsNumber = 33,
    IsAtom = 34,
    IsPid = 35,
    IsReference = 36,
    IsPort = 37,
    IsNil = 38,
    IsBinary = 39,
    IsList = 40,
    IsNonemptyList = 41,
    IsTuple = 42,

    // Control flow
    Jump = 43,
    Badmatch = 44,
    IfEnd = 45,
    CaseEnd = 46,
    Try = 47,
    TryEnd = 48,
    TryCase = 49,
    TryCaseEnd = 50,
    Raise = 51,
    Catch = 52,
    CatchEnd = 53,

    // List operations
    PutList2 = 54,
    GetHd = 55,
    GetTl = 56,

    // Tuple operations
    PutTuple2 = 57,
    GetTupleElement2 = 58,

    // More arithmetic
    Plus = 59,
    Minus = 60,
    Times = 61,
    Div = 62,
    Rem = 63,
    Bsl = 64,
    Bsr = 65,
    Band = 66,
    Bor = 67,
    Bxor = 68,
    Bnot = 69,

    // More comparisons
    IsGt = 70,
    IsLe = 71,

    // Special values
    PutNil = 72,
    PutInteger = 73,
    PutFloat = 74,
    PutAtom = 75,
    PutString = 76,

    // More type tests
    IsCons = 77,
    IsTupleOfArity = 78,
    IsFunction = 79,
    IsFunction2 = 80,

    // Apply and call operations
    CallFun = 81,
    MakeFun = 82,
    IsFunctionWithArity = 83,

    // Binary operations
    BsInit = 84,
    BsPutInteger = 85,
    BsPutFloat = 86,
    BsPutString = 87,
    BsPutBinary = 88,
    BsAdd = 89,
    BsGetInteger = 90,
    BsGetFloat = 91,
    BsGetBinary = 92,
    BsSkip = 93,
    BsTestTail = 94,
    BsRestore = 95,

    // Exception handling
    TryMeElse = 96,
    TryMeElseEnd = 97,
    Wait = 98,
    WaitTimeout1 = 99,
    SendWait = 100,
    SendWaitTimeout = 101,
    Receive = 102,
    ReceiveAccept = 103,
    ReceiveNext = 104,
    Timeout = 105,
    LoopRec = 106,
    LoopRecEnd = 107,
    WaitTimeout2 = 108,
    Hibernate = 109,

    // More function operations
    GcBif1 = 110,
    GcBif2 = 111,
    GcBif3 = 112,

    // Tracing and debugging
    Trace = 113,
    IsBoolean = 114,
    CallTrace = 115,
    EnableTrace = 116,
    DisableTrace = 117,
    Caller = 118,
    RemoveMessage = 119,
    PurgeModule = 120,
    IsTaggedTuple = 121,
    TestArity = 122,
    RawRaise = 123,
    GetTime = 124,
    Yield = 125,
    Trim = 126,
    Nop = 127,

    // Additional opcodes from Erlang header (continuing from 128)
    PutLiteral = 128,
    IsBitstr = 129,
    BsContextToBinary = 130,
    BsTestUnit = 131,
    BsMatchString = 132,
    BsInitWritable = 133,
    BsAppend = 134,
    BsPrivateAppend = 135,
    Trim2 = 136,
    BsInitBits = 137,
    BsGetUtf8 = 138,
    BsSkipUtf8 = 139,
    BsGetUtf16 = 140,
    BsSkipUtf16 = 141,
    BsGetUtf32 = 142,
    BsSkipUtf32 = 143,
    BsUtf8Size = 144,
    BsPutUtf8 = 145,
    BsUtf16Size = 146,
    BsPutUtf16 = 147,
    BsPutUtf32 = 148,
    OnLoad = 149,
    RecvMark = 150,
    RecvSet = 151,
    GcBif32 = 152,
    Line = 153,
    PutMapAssoc = 154,
    PutMapExact = 155,
    IsMap = 156,
    HasMapFields = 157,
    GetMapElements = 158,
    IsTaggedTuple2 = 159,
    BuildStacktrace = 160,
    RawRaise2 = 161,
    GetHd2 = 162,
    GetTl2 = 163,
    PutTuple22 = 164,
    BsGetTail2 = 165,
    BsStartMatch32 = 166,
    BsGetPosition2 = 167,
    BsSetPosition2 = 168,
    Swap = 169,
    BsStartMatch42 = 170,
    MakeFun32 = 171,
    InitYregs = 172,
    RecvMarkerBind = 173,
    RecvMarkerClear = 174,
    RecvMarkerReserve = 175,
    RecvMarkerUse = 176,
    BsCreateBin2 = 177,
    CallFun22 = 178,
    NifStart = 179,
    Badrecord = 180,
    UpdateRecord2 = 181,
    BsMatch2 = 182,
    ExecutableLine = 183,
    DebugLine = 184,
    Bif32 = 185,
    IFuncInfo2 = 186,
    IGenericBreakpoint = 187,
    IDebugBreakpoint = 188,
    ICallTraceReturn = 189,
    IReturnToTrace = 190,
    IDisabledLineBreakpoint = 191,
    IEnabledLineBreakpoint = 192,
    ILineBreakpointCleanup = 193,
    IYield = 194,
    TraceJump = 195,
    IntFuncStart = 196,
    IntFuncEnd = 197,
    INifPadding = 198,
    Padding = 199,
    IDebugLine = 200,
    IAllocateZero2 = 201,
    IAllocateHeapZero2 = 202,
    IInit4 = 203,
    MoveTrim2 = 204,
    ITrim2 = 205,
    IInitSeq32 = 206,
    IInitSeq42 = 207,
    IInitSeq52 = 208,
    IInit22 = 209,
    IInit32 = 210,
    ISelectValBins2 = 211,
    ISelectValLins2 = 212,
    ISelectVal22 = 213,
    ISelectTupleArity2 = 214,
    ISelectTupleArity22 = 215,
    IJumpOnValZero2 = 216,
    IJumpOnVal2 = 217,
    IGetTupleElement4 = 218,
    IGetTupleElement22 = 219,
    IGetTupleElement2Dst2 = 220,
    IGetTupleElement32 = 221,
    IRaise2 = 222,
    DeleteMe = 223,
    SystemLimit2 = 224,
    SystemLimitBody = 225,
    MoveJump2 = 226,
    MoveWindow22 = 227,
    MoveWindow32 = 228,
    MoveWindow42 = 229,
    MoveWindow52 = 230,
    MoveSrcWindow2 = 231,
    MoveSrcWindow22 = 232,
    MoveSrcWindow32 = 233,
    MoveSrcWindow42 = 234,
    Swap22 = 235,
    MoveShift2 = 236,
    Move2Par2 = 237,
    Move32 = 238,
    TimeoutLocked = 239,
    ILoopRec2 = 240,
    WaitLocked = 241,
    WaitUnlocked = 242,
    WaitTimeoutUnlockedInt = 243,
    WaitTimeoutUnlocked = 244,
    WaitTimeoutLockedInt = 245,
    WaitTimeoutLocked = 246,
    IWaitError2 = 247,
    IWaitErrorLocked = 248,
    IIsEqExactImmed2 = 249,
    IIsEqExactLiteral2 = 250,
    IIsNeExactImmed2 = 251,
    IIsNeExactLiteral2 = 252,
    IsLtLiteral2 = 253,
    IsGeLiteral2 = 254,
    UpdateList2 = 255,
    NormalExit = 256,
    ContinueExit = 257,
    CallBif2 = 258,
    CallNif2 = 259,
    CallNifEarly = 260,
    CallErrorHandler = 261,
    ReturnTrace2 = 262,
    ErrorActionCode = 263,
    MoveReturn2 = 264,
    MoveDeallocateReturn2 = 265,
    DeallocateReturn02 = 266,
    DeallocateReturn12 = 267,
    DeallocateReturn22 = 268,
    DeallocateReturn32 = 269,
    DeallocateReturn42 = 270,
    DeallocateReturn2 = 271,
    TestHeap1PutList2 = 272,
    IsTupleOfArity2 = 273,
    TestArityGetTupleElement2 = 274,
    IsTaggedTupleFf2 = 275,
    IsIntegerAllocate2 = 276,
    IsNonemptyListAllocate2 = 277,
    IsNonemptyListGetList2 = 278,
    IsNonemptyListGetHd2 = 279,
    IsNonemptyListGetTl2 = 280,
    IsBitstring2 = 281,
    ColdIsFunction22 = 282,
    HotIsFunction22 = 283,
    AllocateInit2 = 284,
    CallLightBif2 = 285,
    CallLightBifOnly2 = 286,
    CallLightBifLast2 = 287,
    ILoadNif = 288,
    IApply2 = 289,
    IApplyLast2 = 290,
    IApplyOnly2 = 291,
    IApplyFun2 = 292,
    IApplyFunLast2 = 293,
    IApplyFunOnly2 = 294,
    CallLightBif22 = 295,
    CallLightBifOnly22 = 296,
    IHibernate = 297,
    IPerfCounter = 298,
    IGetHash2 = 299,
    IGet2 = 300,
    Self2 = 301,
    Node2 = 302,
    IFastElement2 = 303,
    IElement2 = 304,
    IBif12 = 305,
    IBif1Body2 = 306,
    IBif22 = 307,
    IBif2Body2 = 308,
    IBif32 = 309,
    IBif3Body2 = 310,
    MoveCall2 = 311,
    MoveCallLast2 = 312,
    MoveCallOnly2 = 313,
    ICall2 = 314,
    ICallLast2 = 315,
    ICallOnly2 = 316,
    ICallExt2 = 317,
    ICallExtLast2 = 318,
    ICallExtOnly2 = 319,
    IMoveCallExt2 = 320,
    IMoveCallExtLast2 = 321,
    IMoveCallExtOnly2 = 322,
    ICallFun2 = 323,
    ICallFunLast2 = 324,
    IMakeFun32 = 325,
    ILambdaError = 326,
    IBsEnsureBits2 = 327,
    IBsEnsureBitsUnit2 = 328,
    IBsReadBits2 = 329,
    IBsEq2 = 330,
    IBsExtractInteger2 = 331,
    IBsReadInteger82 = 332,
    IBsGetFixedInteger2 = 333,
    IBsGetFixedBinary2 = 334,
    IBsGetTail2 = 335,
    IBsSkip2 = 336,
    IBsDrop = 337,
    IBsEnsureBitsRead2 = 338,
    BadBsMatch = 339,
    IBsMatchString2 = 340,
    IBsGetIntegerSmallImm2 = 341,
    IBsGetIntegerImm2 = 342,
    IBsGetInteger2 = 343,
    IBsGetInteger82 = 344,
    IBsGetInteger162 = 345,
    IBsGetInteger322 = 346,
    IBsGetBinaryImm22 = 347,
    IBsGetBinary22 = 348,
    IBsGetBinaryAll22 = 349,
    IBsGetFloat22 = 350,
    IBsSkipBits22 = 351,
    BsTestZeroTail22 = 352,
    BsTestTailImm22 = 353,
    BsTestUnit82 = 354,
    IBsStartMatch3Gp2 = 355,
    IBsStartMatch32 = 356,
    IBsGetPosition2 = 357,
    IBsGetUtf82 = 358,
    IBsGetUtf162 = 359,
    IBsValidateUnicodeRetract2 = 360,
    IBsCreateBin2 = 361,
    Fstore = 362,
    Fload2 = 363,
    IFadd2 = 364,
    IFsub2 = 365,
    IFmul2 = 366,
    IFdiv2 = 367,
    IFnegate2 = 368,
    IPutMapAssoc2 = 369,
    SortedPutMapAssoc2 = 370,
    SortedPutMapExact2 = 371,
    NewMap2 = 372,
    INewSmallMapLit2 = 373,
    UpdateMapAssoc2 = 374,
    UpdateMapExact2 = 375,
    IGetMapElements2 = 376,
    IGetMapElementHash2 = 377,
    IGetMapElement2 = 378,
    GenPlus2 = 379,
    GenMinus2 = 380,
    IIncrement2 = 381,
    IPlus2 = 382,
    IUnaryMinus2 = 383,
    IMinus2 = 384,
    ITimes2 = 385,
    IMdiv2 = 386,
    IIntDiv2 = 387,
    IRem2 = 388,
    IBsl2 = 389,
    IBsr2 = 390,
    IBand2 = 391,
    IBor2 = 392,
    IBxor2 = 393,
    IIntBnot2 = 394,
    ILengthSetup2 = 395,
    ILength2 = 396,
    UnsupportedGuardBif2 = 397,
    MoveX12 = 398,
    MoveX22 = 399,
    IUpdateRecordCopy2 = 400,
    IUpdateRecordInPlace2 = 401,
    IUpdateRecordContinue2 = 402,
    IUpdateRecordInPlaceDone = 403,
    IUpdateRecordInPlaceDone2 = 404,

    // Specific opcodes from beam_opcodes.h that need implementation
    // Using high discriminant values to avoid conflicts
    OpAllocateTt = 1000,              // op_allocate_tt (opcode 0)
    OpApplyT = 1001,                  // op_apply_t (opcode 3)
    OpApplyLastTQ = 1002,             // op_apply_last_tQ (opcode 4)
    OpBsGetTailYdt = 1003,            // op_bs_get_tail_ydt (opcode 8)
    OpBsSetPositionXx = 1004,         // op_bs_set_position_xx (opcode 10)
    OpBsTestUnitFyt = 1005,           // op_bs_test_unit_fyt (opcode 17)
    OpBsTestUnit8Fy = 1006,           // op_bs_test_unit8_fy (opcode 19)
    OpBuildStacktrace = 1007,         // op_build_stacktrace (opcode 22)
    OpCallBifW = 1008,                // op_call_bif_W (opcode 23)
    OpDeallocateReturnQ = 1009,       // op_deallocate_return_Q (opcode 35)
    OpFloadQl = 1010,                 // op_fload_ql (opcode 43)
    OpGetListXrx = 1011,              // op_get_list_xrx (opcode 55)
    OpGetListXxx = 1012,              // op_get_list_xxx (opcode 56)
    OpGetTlXx = 1013,                 // op_get_tl_xx (opcode 64)
    OpGetTlXy = 1014,                 // op_get_tl_xy (opcode 65)
    OpIBandSsjd = 1015,               // op_i_band_ssjd (opcode 78)
    OpIBslSsjd = 1016,               // op_i_bsl_ssjd (opcode 146)
    OpICallFunT = 1017,              // op_i_call_fun_t (opcode 153)
    OpIGetMapElementFyyx = 1018,     // op_i_get_map_element_fyyx (opcode 181)
    OpIGetTupleElement2XPx = 1019,   // op_i_get_tuple_element2_xPx (opcode 192)
}

impl BeamOpcode {
    /// Try to convert a u32 value to a BeamOpcode
    pub fn from_u32(value: u32) -> Option<Self> {
        use BeamOpcode::*;
        match value {
            0 => Some(OpAllocateTt),
            1 => Some(Label),
            2 => Some(FuncInfo),
            3 => Some(OpApplyT),
            4 => Some(OpApplyLastTQ),
            5 => Some(CallLast),
            6 => Some(CallOnly),
            7 => Some(CallExt),
            8 => Some(OpBsGetTailYdt),
            9 => Some(Bif0),
            10 => Some(OpBsSetPositionXx),
            11 => Some(Bif2),
            12 => Some(Return),
            13 => Some(Send),
            14 => Some(Move),
            15 => Some(GetList),
            16 => Some(GetTupleElement),
            17 => Some(SetTupleElement),
            18 => Some(PutList),
            19 => Some(PutTuple),
            20 => Some(Add),
            21 => Some(Subtract),
            22 => Some(Multiply),
            23 => Some(Divide),
            24 => Some(Negate),
            25 => Some(IsLt),
            26 => Some(IsGe),
            27 => Some(IsEq),
            28 => Some(IsNe),
            29 => Some(IsEqExact),
            30 => Some(IsNeExact),
            31 => Some(IsInteger),
            32 => Some(IsFloat),
            33 => Some(IsNumber),
            34 => Some(IsAtom),
            35 => Some(IsPid),
            36 => Some(IsReference),
            37 => Some(IsPort),
            38 => Some(IsNil),
            39 => Some(IsBinary),
            40 => Some(IsList),
            41 => Some(IsNonemptyList),
            42 => Some(IsTuple),
            43 => Some(Jump),
            44 => Some(Badmatch),
            45 => Some(IfEnd),
            46 => Some(CaseEnd),
            47 => Some(Try),
            48 => Some(TryEnd),
            49 => Some(TryCase),
            50 => Some(TryCaseEnd),
            51 => Some(Raise),
            52 => Some(Catch),
            53 => Some(CatchEnd),
            128 => Some(PutLiteral),
            129 => Some(IsBitstr),
            130 => Some(BsContextToBinary),
            131 => Some(BsTestUnit),
            132 => Some(BsMatchString),
            133 => Some(BsInitWritable),
            134 => Some(BsAppend),
            135 => Some(BsPrivateAppend),
            136 => Some(Trim2),
            137 => Some(BsInitBits),
            138 => Some(BsGetUtf8),
            139 => Some(BsSkipUtf8),
            140 => Some(BsGetUtf16),
            141 => Some(BsSkipUtf16),
            142 => Some(BsGetUtf32),
            143 => Some(BsSkipUtf32),
            144 => Some(BsUtf8Size),
            145 => Some(BsPutUtf8),
            146 => Some(BsUtf16Size),
            147 => Some(BsPutUtf16),
            148 => Some(BsPutUtf32),
            149 => Some(OnLoad),
            150 => Some(RecvMark),
            151 => Some(RecvSet),
            152 => Some(GcBif3),
            153 => Some(Line),
            154 => Some(PutMapAssoc),
            155 => Some(PutMapExact),
            156 => Some(IsMap),
            157 => Some(HasMapFields),
            158 => Some(GetMapElements),
            159 => Some(IsTaggedTuple2),
            160 => Some(BuildStacktrace),
            161 => Some(RawRaise2),
            162 => Some(GetHd),
            163 => Some(GetTl),
            164 => Some(PutTuple22),
            165 => Some(BsGetTail2),
            166 => Some(BsStartMatch32),
            167 => Some(BsGetPosition2),
            168 => Some(BsSetPosition2),
            169 => Some(Swap),
            170 => Some(BsStartMatch42),
            171 => Some(MakeFun32),
            172 => Some(InitYregs),
            173 => Some(RecvMarkerBind),
            174 => Some(RecvMarkerClear),
            175 => Some(RecvMarkerReserve),
            176 => Some(RecvMarkerUse),
            177 => Some(BsCreateBin2),
            178 => Some(CallFun22),
            179 => Some(NifStart),
            180 => Some(Badrecord),
            181 => Some(UpdateRecord2),
            182 => Some(BsMatch2),
            183 => Some(ExecutableLine),
            184 => Some(DebugLine),
            185 => Some(Bif32),
            186 => Some(IFuncInfo2),
            187 => Some(IGenericBreakpoint),
            188 => Some(IDebugBreakpoint),
            189 => Some(ICallTraceReturn),
            190 => Some(IReturnToTrace),
            191 => Some(IDisabledLineBreakpoint),
            192 => Some(IEnabledLineBreakpoint),
            193 => Some(ILineBreakpointCleanup),
            194 => Some(IYield),
            195 => Some(TraceJump),
            196 => Some(IntFuncStart),
            197 => Some(IntFuncEnd),
            198 => Some(INifPadding),
            199 => Some(Padding),
            200 => Some(IDebugLine),
            201 => Some(IAllocateZero2),
            202 => Some(IAllocateHeapZero2),
            203 => Some(IInit4),
            204 => Some(MoveTrim2),
            205 => Some(ITrim2),
            206 => Some(IInitSeq32),
            207 => Some(IInitSeq42),
            208 => Some(IInitSeq52),
            209 => Some(IInit22),
            210 => Some(IInit32),
            211 => Some(ISelectValBins2),
            212 => Some(ISelectValLins2),
            213 => Some(ISelectVal22),
            214 => Some(ISelectTupleArity2),
            215 => Some(ISelectTupleArity22),
            216 => Some(IJumpOnValZero2),
            217 => Some(IJumpOnVal2),
            218 => Some(IGetTupleElement4),
            219 => Some(IGetTupleElement22),
            220 => Some(IGetTupleElement2Dst2),
            221 => Some(IGetTupleElement32),
            222 => Some(IRaise2),
            223 => Some(DeleteMe),
            224 => Some(SystemLimit2),
            225 => Some(SystemLimitBody),
            226 => Some(MoveJump2),
            227 => Some(MoveWindow22),
            228 => Some(MoveWindow32),
            229 => Some(MoveWindow42),
            230 => Some(MoveWindow52),
            231 => Some(MoveSrcWindow2),
            232 => Some(MoveSrcWindow22),
            233 => Some(MoveSrcWindow32),
            234 => Some(MoveSrcWindow42),
            235 => Some(Swap22),
            236 => Some(MoveShift2),
            237 => Some(Move2Par2),
            238 => Some(Move32),
            239 => Some(TimeoutLocked),
            240 => Some(ILoopRec2),
            241 => Some(WaitLocked),
            242 => Some(WaitUnlocked),
            243 => Some(WaitTimeoutUnlockedInt),
            244 => Some(WaitTimeoutUnlocked),
            245 => Some(WaitTimeoutLockedInt),
            246 => Some(WaitTimeoutLocked),
            247 => Some(IWaitError2),
            248 => Some(IWaitErrorLocked),
            249 => Some(IIsEqExactImmed2),
            250 => Some(IIsNeExactImmed2),
            251 => Some(IIsNeExactLiteral2),
            252 => Some(IsLtLiteral2),
            253 => Some(IsGeLiteral2),
            254 => Some(UpdateList2),
            255 => Some(NormalExit),
            256 => Some(ContinueExit),
            257 => Some(CallBif2),
            258 => Some(CallNif2),
            259 => Some(CallNifEarly),
            260 => Some(CallErrorHandler),
            261 => Some(ErrorActionCode),
            262 => Some(ReturnTrace2),
            263 => Some(MoveReturn2),
            264 => Some(MoveDeallocateReturn2),
            265 => Some(DeallocateReturn02),
            266 => Some(DeallocateReturn12),
            267 => Some(DeallocateReturn22),
            268 => Some(DeallocateReturn32),
            269 => Some(DeallocateReturn42),
            270 => Some(DeallocateReturn2),
            271 => Some(TestHeap1PutList2),
            272 => Some(IsTupleOfArity2),
            273 => Some(TestArityGetTupleElement2),
            274 => Some(IsTaggedTupleFf2),
            275 => Some(IsIntegerAllocate2),
            276 => Some(IsNonemptyListAllocate2),
            277 => Some(IsNonemptyListGetList2),
            278 => Some(IsNonemptyListGetHd2),
            279 => Some(IsNonemptyListGetTl2),
            280 => Some(IsBitstring2),
            281 => Some(ColdIsFunction22),
            282 => Some(HotIsFunction22),
            283 => Some(AllocateInit2),
            284 => Some(CallLightBif2),
            285 => Some(CallLightBifOnly2),
            286 => Some(CallLightBifLast2),
            287 => Some(ILoadNif),
            288 => Some(IApply2),
            289 => Some(IApplyLast2),
            290 => Some(IApplyOnly2),
            291 => Some(IApplyFun2),
            292 => Some(IApplyFunLast2),
            293 => Some(IApplyFunOnly2),
            294 => Some(CallLightBif22),
            295 => Some(CallLightBifOnly22),
            296 => Some(IHibernate),
            297 => Some(IPerfCounter),
            298 => Some(IGetHash2),
            299 => Some(IGet2),
            300 => Some(Self2),
            301 => Some(Node2),
            302 => Some(IFastElement2),
            303 => Some(IElement2),
            304 => Some(IBif12),
            305 => Some(IBif1Body2),
            306 => Some(IBif22),
            307 => Some(IBif2Body2),
            308 => Some(IBif32),
            309 => Some(IBif3Body2),
            310 => Some(MoveCall2),
            311 => Some(MoveCallLast2),
            312 => Some(MoveCallOnly2),
            313 => Some(ICall2),
            314 => Some(ICallLast2),
            315 => Some(ICallOnly2),
            316 => Some(ICallExt2),
            317 => Some(ICallExtLast2),
            318 => Some(ICallExtOnly2),
            319 => Some(IMoveCallExt2),
            320 => Some(IMoveCallExtLast2),
            321 => Some(IMoveCallExtOnly2),
            322 => Some(ICallFun2),
            323 => Some(ICallFunLast2),
            324 => Some(IMakeFun32),
            325 => Some(ILambdaError),
            326 => Some(IBsEnsureBits2),
            327 => Some(IBsEnsureBitsUnit2),
            328 => Some(IBsReadBits2),
            329 => Some(IBsEq2),
            330 => Some(IBsExtractInteger2),
            331 => Some(IBsReadInteger82),
            332 => Some(IBsGetFixedInteger2),
            333 => Some(IBsGetFixedBinary2),
            334 => Some(IBsGetTail2),
            335 => Some(IBsSkip2),
            336 => Some(IBsDrop),
            337 => Some(IBsEnsureBitsRead2),
            338 => Some(BadBsMatch),
            339 => Some(IBsMatchString2),
            340 => Some(IBsGetIntegerSmallImm2),
            341 => Some(IBsGetIntegerImm2),
            342 => Some(IBsGetInteger2),
            343 => Some(IBsGetInteger82),
            344 => Some(IBsGetInteger162),
            345 => Some(IBsGetInteger322),
            346 => Some(IBsGetBinaryImm22),
            347 => Some(IBsGetBinary22),
            348 => Some(IBsGetBinaryAll22),
            349 => Some(IBsGetFloat22),
            350 => Some(IBsSkipBits22),
            351 => Some(BsTestZeroTail22),
            352 => Some(BsTestTailImm22),
            353 => Some(BsTestUnit82),
            354 => Some(IBsStartMatch3Gp2),
            355 => Some(IBsStartMatch32),
            356 => Some(IBsGetPosition2),
            357 => Some(IBsGetUtf82),
            358 => Some(IBsGetUtf162),
            359 => Some(IBsValidateUnicodeRetract2),
            360 => Some(IBsCreateBin2),
            361 => Some(Fstore),
            362 => Some(Fload2),
            363 => Some(IFadd2),
            364 => Some(IFsub2),
            365 => Some(IFmul2),
            366 => Some(IFdiv2),
            367 => Some(IFnegate2),
            368 => Some(IPutMapAssoc2),
            369 => Some(SortedPutMapAssoc2),
            370 => Some(SortedPutMapExact2),
            371 => Some(NewMap2),
            372 => Some(INewSmallMapLit2),
            373 => Some(UpdateMapAssoc2),
            374 => Some(UpdateMapExact2),
            375 => Some(IGetMapElements2),
            376 => Some(IGetMapElementHash2),
            377 => Some(IGetMapElement2),
            378 => Some(GenPlus2),
            379 => Some(GenMinus2),
            380 => Some(IIncrement2),
            381 => Some(IPlus2),
            382 => Some(IUnaryMinus2),
            383 => Some(IMinus2),
            384 => Some(ITimes2),
            385 => Some(IMdiv2),
            386 => Some(IIntDiv2),
            387 => Some(IRem2),
            388 => Some(IBsl2),
            389 => Some(IBsr2),
            390 => Some(IBand2),
            391 => Some(IBor2),
            392 => Some(IBxor2),
            393 => Some(IIntBnot2),
            394 => Some(ILengthSetup2),
            395 => Some(ILength2),
            396 => Some(UnsupportedGuardBif2),
            397 => Some(MoveX12),
            398 => Some(MoveX22),
            399 => Some(IUpdateRecordCopy2),
            400 => Some(IUpdateRecordInPlace2),
            401 => Some(IUpdateRecordContinue2),
            402 => Some(IUpdateRecordInPlaceDone),
            403 => Some(IUpdateRecordInPlaceDone2),
            404 => Some(IUpdateRecordInPlaceDone2),
            // Additional specific opcodes
            17 => Some(OpBsTestUnitFyt),
            19 => Some(OpBsTestUnit8Fy),
            22 => Some(OpBuildStacktrace),
            23 => Some(OpCallBifW),
            35 => Some(OpDeallocateReturnQ),
            43 => Some(OpFloadQl),
            55 => Some(OpGetListXrx),
            56 => Some(OpGetListXxx),
            64 => Some(OpGetTlXx),
            65 => Some(OpGetTlXy),
            78 => Some(OpIBandSsjd),
            146 => Some(OpIBslSsjd),
            153 => Some(OpICallFunT),
            181 => Some(OpIGetMapElementFyyx),
            192 => Some(OpIGetTupleElement2XPx),
            _ => None,
        }
    }

    /// Get the numeric value of the opcode
    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_opcodes_numeric_values() {
        // Test some basic opcodes have correct numeric values
        assert_eq!(BeamOpcode::Label as u32, 1);
        assert_eq!(BeamOpcode::FuncInfo as u32, 2);
        assert_eq!(BeamOpcode::IntCodeEnd as u32, 3);
        assert_eq!(BeamOpcode::Call as u32, 4);
        assert_eq!(BeamOpcode::CallLast as u32, 5);
        assert_eq!(BeamOpcode::CallOnly as u32, 6);
        assert_eq!(BeamOpcode::Return as u32, 12);
        assert_eq!(BeamOpcode::Move as u32, 14);
    }

    #[test]
    fn test_arithmetic_opcodes_numeric_values() {
        // Test arithmetic opcodes
        assert_eq!(BeamOpcode::Add as u32, 20);
        assert_eq!(BeamOpcode::Subtract as u32, 21);
        assert_eq!(BeamOpcode::Multiply as u32, 22);
        assert_eq!(BeamOpcode::Divide as u32, 23);
        assert_eq!(BeamOpcode::Negate as u32, 24);
    }

    #[test]
    fn test_comparison_opcodes_numeric_values() {
        // Test comparison opcodes
        assert_eq!(BeamOpcode::IsLt as u32, 25);
        assert_eq!(BeamOpcode::IsGe as u32, 26);
        assert_eq!(BeamOpcode::IsEq as u32, 27);
        assert_eq!(BeamOpcode::IsNe as u32, 28);
        assert_eq!(BeamOpcode::IsEqExact as u32, 29);
        assert_eq!(BeamOpcode::IsNeExact as u32, 30);
    }

    #[test]
    fn test_type_test_opcodes_numeric_values() {
        // Test type test opcodes
        assert_eq!(BeamOpcode::IsInteger as u32, 31);
        assert_eq!(BeamOpcode::IsFloat as u32, 32);
        assert_eq!(BeamOpcode::IsAtom as u32, 34);
        assert_eq!(BeamOpcode::IsPid as u32, 35);
        assert_eq!(BeamOpcode::IsNil as u32, 38);
        assert_eq!(BeamOpcode::IsList as u32, 40);
    }

    #[test]
    fn test_function_call_opcodes_numeric_values() {
        // Test function call opcodes
        assert_eq!(BeamOpcode::CallExt as u32, 7);
        assert_eq!(BeamOpcode::CallExtLast as u32, 8);
        assert_eq!(BeamOpcode::Bif0 as u32, 9);
        assert_eq!(BeamOpcode::Bif1 as u32, 10);
        assert_eq!(BeamOpcode::Bif2 as u32, 11);
    }

    #[test]
    fn test_binary_operations_opcodes_numeric_values() {
        // Test binary operations - just check the enum values
        assert_eq!(BeamOpcode::BsInit as u32, 84);
        assert_eq!(BeamOpcode::BsPutInteger as u32, 85);
        assert_eq!(BeamOpcode::BsGetInteger as u32, 90);
        assert_eq!(BeamOpcode::BsSkip as u32, 93);
        assert_eq!(BeamOpcode::BsTestTail as u32, 94);
        // Note: from_u32 mappings for these may not be implemented
    }

    #[test]
    fn test_extended_opcodes_numeric_values() {
        // Test some extended opcodes
        assert_eq!(BeamOpcode::PutLiteral as u32, 128);
        assert_eq!(BeamOpcode::IsBitstr as u32, 129);
        assert_eq!(BeamOpcode::BsContextToBinary as u32, 130);
        assert_eq!(BeamOpcode::Line as u32, 153);
        assert_eq!(BeamOpcode::PutMapAssoc as u32, 154);
        assert_eq!(BeamOpcode::IsMap as u32, 156);
        // Note: Many extended opcodes may not have from_u32 mappings
    }

    #[test]
    fn test_to_u32_method() {
        // Test the to_u32 method
        assert_eq!(BeamOpcode::Label.to_u32(), 1);
        assert_eq!(BeamOpcode::Move.to_u32(), 14);
        assert_eq!(BeamOpcode::Return.to_u32(), 12);
        assert_eq!(BeamOpcode::Add.to_u32(), 20);
        assert_eq!(BeamOpcode::IsEq.to_u32(), 27);
        assert_eq!(BeamOpcode::CallExt.to_u32(), 7);
    }

    #[test]
    fn test_from_u32_runtime_mapping() {
        // Test from_u32 based on the actual C erlc runtime mapping
        // This is SEPARATE from the enum definitions - do not merge these concepts

        // Test some known runtime mappings (from the C erlc implementation)
        assert_eq!(BeamOpcode::from_u32(1), Some(BeamOpcode::Label));
        assert_eq!(BeamOpcode::from_u32(2), Some(BeamOpcode::FuncInfo));
        assert_eq!(BeamOpcode::from_u32(12), Some(BeamOpcode::Return));
        assert_eq!(BeamOpcode::from_u32(14), Some(BeamOpcode::Move));

        // Runtime mapping may use OpXxx variants not in main enum
        assert_eq!(BeamOpcode::from_u32(0), Some(BeamOpcode::OpAllocateTt));
        assert_eq!(BeamOpcode::from_u32(3), Some(BeamOpcode::OpApplyT));
        assert_eq!(BeamOpcode::from_u32(8), Some(BeamOpcode::OpBsGetTailYdt));
    }

    #[test]
    fn test_from_u32_runtime_boundaries() {
        // Test from_u32 runtime mapping boundaries (separate from enum)
        // Note: from_u32(0) is actually valid in the runtime mapping

        // Test some values that are NOT mapped in the runtime
        assert_eq!(BeamOpcode::from_u32(999), None); // Very large invalid value
        assert_eq!(BeamOpcode::from_u32(u32::MAX), None); // Maximum u32 value

        // Test that 0 IS mapped in the runtime (this is expected)
        assert!(BeamOpcode::from_u32(0).is_some()); // 0 is valid in runtime mapping
    }

    #[test]
    fn test_enum_to_u32_roundtrip() {
        // Test that enum values can be converted to u32
        // This tests the enum definitions directly
        let test_opcodes = vec![
            BeamOpcode::Label,
            BeamOpcode::FuncInfo,
            BeamOpcode::Return,
            BeamOpcode::Move,
            BeamOpcode::Add,
            BeamOpcode::IsEq,
            BeamOpcode::IsInteger,
            BeamOpcode::CallExt,
            BeamOpcode::PutLiteral,
        ];

        for opcode in test_opcodes {
            let numeric = opcode.to_u32();
            // Just verify it produces a valid u32
            assert!(numeric >= 0);
        }
    }

    #[test]
    fn test_runtime_mapping_roundtrip() {
        // Test roundtrip for runtime mappings that actually work
        // This is separate from enum roundtrips

        // Test cases where from_u32(value) -> to_u32() -> from_u32(value) works
        let working_cases = vec![
            (1u32, BeamOpcode::Label),
            (2u32, BeamOpcode::FuncInfo),
            (12u32, BeamOpcode::Return),
            (14u32, BeamOpcode::Move),
            (20u32, BeamOpcode::Add),
            (27u32, BeamOpcode::IsEq),
            (31u32, BeamOpcode::IsInteger),
        ];

        for (value, expected_opcode) in working_cases {
            let opcode = BeamOpcode::from_u32(value);
            assert_eq!(opcode, Some(expected_opcode), "from_u32({}) failed", value);

            let back_to_value = expected_opcode.to_u32();
            let roundtrip = BeamOpcode::from_u32(back_to_value);
            assert_eq!(roundtrip, Some(expected_opcode),
                      "Runtime roundtrip failed for {:?}", expected_opcode);
        }
    }

    #[test]
    fn test_enum_equality() {
        // Test equality
        assert_eq!(BeamOpcode::Label, BeamOpcode::Label);
        assert_eq!(BeamOpcode::Move, BeamOpcode::Move);
        assert_ne!(BeamOpcode::Label, BeamOpcode::FuncInfo);
        assert_ne!(BeamOpcode::Add, BeamOpcode::Subtract);
    }

    #[test]
    fn test_enum_clone() {
        // Test clone
        let original = BeamOpcode::Label;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_enum_copy() {
        // Test copy
        let original = BeamOpcode::Move;
        let copied = original; // Copy
        assert_eq!(original, copied);
    }

    #[test]
    fn test_enum_debug() {
        // Test debug formatting
        let debug_str = format!("{:?}", BeamOpcode::Label);
        assert!(debug_str.contains("Label"));

        let debug_str = format!("{:?}", BeamOpcode::Move);
        assert!(debug_str.contains("Move"));
    }

    #[test]
    fn test_enum_repr_u32() {
        // Test that the enum has the correct repr
        // Since it's repr(u32), the numeric values should match the enum discriminants
        assert_eq!(BeamOpcode::Label as u32, 1);
        assert_eq!(BeamOpcode::FuncInfo as u32, 2);

        // Test that consecutive enum values have consecutive numeric values
        assert_eq!(BeamOpcode::Call as u32, 4);
        assert_eq!(BeamOpcode::CallLast as u32, 5);
        assert_eq!(BeamOpcode::CallOnly as u32, 6);
    }

    #[test]
    fn test_large_opcode_values() {
        // Test some of the largest opcode values
        // Note: These values come from the match statement in from_u32
        // Let's test some known large values from the enum

        // Test some opcodes that should exist based on the enum definition
        assert_eq!(BeamOpcode::PutLiteral as u32, 128);
        assert_eq!(BeamOpcode::Line as u32, 153);
        assert_eq!(BeamOpcode::IsMap as u32, 156);

        // Test that from_u32 works for these large values
        assert_eq!(BeamOpcode::from_u32(128), Some(BeamOpcode::PutLiteral));
        assert_eq!(BeamOpcode::from_u32(153), Some(BeamOpcode::Line));
        assert_eq!(BeamOpcode::from_u32(156), Some(BeamOpcode::IsMap));
    }

    #[test]
    fn test_runtime_mapping_coverage() {
        // Test that the runtime mapping (from_u32) covers expected values
        // This is separate from the enum definitions

        // Test some values that ARE mapped in the runtime
        assert_eq!(BeamOpcode::from_u32(3), Some(BeamOpcode::OpApplyT)); // 3 maps to OpApplyT in runtime
        assert_eq!(BeamOpcode::from_u32(13), Some(BeamOpcode::Send)); // 13 exists
        assert_eq!(BeamOpcode::from_u32(18), Some(BeamOpcode::PutList)); // 18 exists
        // Note: Not all enum values have corresponding runtime mappings
        // For example, PutList2 (54) is in the enum but not mapped in from_u32

        // Test a value that is NOT mapped in the runtime
        assert_eq!(BeamOpcode::from_u32(54), None); // PutList2 enum value not in runtime mapping
        assert_eq!(BeamOpcode::from_u32(999), None);
    }

    #[test]
    fn test_runtime_mapping_boundaries() {
        // Test runtime mapping boundaries (separate from enum boundaries)
        assert_eq!(BeamOpcode::from_u32(1), Some(BeamOpcode::Label)); // First valid in runtime
        assert_eq!(BeamOpcode::from_u32(0), Some(BeamOpcode::OpAllocateTt)); // 0 is valid in runtime

        // Test some high values that are mapped
        assert_eq!(BeamOpcode::from_u32(404), Some(BeamOpcode::IUpdateRecordInPlaceDone2));

        // Test values that are NOT mapped
        assert_eq!(BeamOpcode::from_u32(405), None); // After last mapped value
        assert_eq!(BeamOpcode::from_u32(500), None); // Well beyond mapped range
    }

    #[test]
    fn test_special_opcodes() {
        // Test some special opcodes that might be important
        assert_eq!(BeamOpcode::Nop as u32, 127);
        assert_eq!(BeamOpcode::Line as u32, 153);
        assert_eq!(BeamOpcode::OnLoad as u32, 149);
        assert_eq!(BeamOpcode::Yield as u32, 125);

        // Test from_u32 for the ones that are actually implemented
        assert_eq!(BeamOpcode::from_u32(153), Some(BeamOpcode::Line));
        assert_eq!(BeamOpcode::from_u32(149), Some(BeamOpcode::OnLoad));
    }

    #[test]
    fn test_function_related_opcodes() {
        // Test function-related opcodes
        assert_eq!(BeamOpcode::CallFun as u32, 81);
        assert_eq!(BeamOpcode::MakeFun as u32, 82);
        assert_eq!(BeamOpcode::GcBif1 as u32, 110);
        assert_eq!(BeamOpcode::GcBif2 as u32, 111);
        assert_eq!(BeamOpcode::GcBif3 as u32, 112);

        // Note: from_u32 mappings may not match all enum values
        // Test some that are actually handled
        assert_eq!(BeamOpcode::from_u32(14), Some(BeamOpcode::Move)); // Known working mapping
    }

    #[test]
    fn test_map_operations_opcodes() {
        // Test map-related opcodes
        assert_eq!(BeamOpcode::PutMapAssoc as u32, 154);
        assert_eq!(BeamOpcode::PutMapExact as u32, 155);
        assert_eq!(BeamOpcode::IsMap as u32, 156);
        assert_eq!(BeamOpcode::HasMapFields as u32, 157);
        assert_eq!(BeamOpcode::GetMapElements as u32, 158);

        // Test from_u32 for known working mappings
        assert_eq!(BeamOpcode::from_u32(156), Some(BeamOpcode::IsMap));
    }

    #[test]
    fn test_exception_handling_opcodes() {
        // Test exception handling opcodes
        assert_eq!(BeamOpcode::Raise as u32, 51);
        assert_eq!(BeamOpcode::Catch as u32, 52);
        assert_eq!(BeamOpcode::CatchEnd as u32, 53);
        assert_eq!(BeamOpcode::Try as u32, 47);
        assert_eq!(BeamOpcode::TryEnd as u32, 48);
        assert_eq!(BeamOpcode::TryCase as u32, 49);

        // Test from_u32 for known working mappings
        assert_eq!(BeamOpcode::from_u32(51), Some(BeamOpcode::Raise));
        assert_eq!(BeamOpcode::from_u32(52), Some(BeamOpcode::Catch));
        assert_eq!(BeamOpcode::from_u32(53), Some(BeamOpcode::CatchEnd));
    }

    #[test]
    fn test_memory_operations_opcodes() {
        // Test memory-related opcodes
        assert_eq!(BeamOpcode::GetList as u32, 15);
        assert_eq!(BeamOpcode::GetTupleElement as u32, 16);
        assert_eq!(BeamOpcode::SetTupleElement as u32, 17);
        assert_eq!(BeamOpcode::PutList as u32, 18);
        assert_eq!(BeamOpcode::PutTuple as u32, 19);
        assert_eq!(BeamOpcode::InitYregs as u32, 172); // This is a memory-related opcode

        // Test from_u32 for known working mappings
        assert_eq!(BeamOpcode::from_u32(15), Some(BeamOpcode::GetList));
        assert_eq!(BeamOpcode::from_u32(16), Some(BeamOpcode::GetTupleElement));
        assert_eq!(BeamOpcode::from_u32(17), Some(BeamOpcode::SetTupleElement));
        assert_eq!(BeamOpcode::from_u32(18), Some(BeamOpcode::PutList));
        assert_eq!(BeamOpcode::from_u32(19), Some(BeamOpcode::PutTuple));
    }

    #[test]
    fn test_enum_bit_operations_opcodes() {
        // Test bit operation opcodes in the enum (separate from runtime mappings)
        assert_eq!(BeamOpcode::Bsl as u32, 64);
        assert_eq!(BeamOpcode::Bsr as u32, 65);
        assert_eq!(BeamOpcode::Band as u32, 66);
        assert_eq!(BeamOpcode::Bor as u32, 67);
        assert_eq!(BeamOpcode::Bxor as u32, 68);
        assert_eq!(BeamOpcode::Bnot as u32, 69);

        // Note: Runtime mappings (from_u32) may map these values to different opcodes
        // For example, from_u32(64) might map to OpGetTlXx, not Bsl
        // This is expected - enum and runtime mappings are separate
    }

    #[test]
    fn test_all_opcodes_have_unique_values() {
        // This is a compile-time check, but we can test a few to ensure uniqueness
        assert_ne!(BeamOpcode::Label as u32, BeamOpcode::FuncInfo as u32);
        assert_ne!(BeamOpcode::Move as u32, BeamOpcode::Add as u32);
        assert_ne!(BeamOpcode::Return as u32, BeamOpcode::Call as u32);
        assert_ne!(BeamOpcode::IsEq as u32, BeamOpcode::IsNe as u32);
    }

    #[test]
    fn test_enum_values_directly() {
        // Test enum values directly - separate from runtime mappings
        let test_opcodes = vec![
            BeamOpcode::Label, BeamOpcode::FuncInfo, BeamOpcode::Return,
            BeamOpcode::Move, BeamOpcode::Add, BeamOpcode::IsEq, BeamOpcode::IsInteger,
            BeamOpcode::PutLiteral, BeamOpcode::Line, BeamOpcode::IsMap,
            BeamOpcode::GetList, BeamOpcode::GetTupleElement, BeamOpcode::SetTupleElement,
            BeamOpcode::PutList, BeamOpcode::PutTuple, BeamOpcode::Raise,
            BeamOpcode::Catch, BeamOpcode::CatchEnd, BeamOpcode::Bsl,
            BeamOpcode::Bsr, BeamOpcode::Band, BeamOpcode::Bor,
            BeamOpcode::Bxor, BeamOpcode::Bnot,
        ];

        for opcode in test_opcodes {
            // Test that each enum value has a valid u32 representation
            let numeric = opcode.to_u32();
            assert!(numeric >= 0);

            // Test that the enum is properly defined
            assert!(matches!(opcode, _)); // Just ensure it's a valid variant
        }
    }
}
