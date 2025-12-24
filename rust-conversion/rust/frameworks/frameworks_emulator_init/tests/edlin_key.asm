L89:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# i_flush_stubs
# i_func_label_L
label_1:
# func_line_I
# i_func_info_IaaI
# edlin_key:get_key_map/0
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x04, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
get_key_map/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L92
    bl L94
L92:
# i_test_yield
    adr x2, get_key_map/0
    subs w22, w22, 1
    b.le L96
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L97
    mov x3, xzr
    bl L99
L97:
# i_move_sd
    mov x26, 595211
# i_move_sd
    mov x27, 1291
# i_move_sd
    mov x25, 226315
# line_I
# i_call_ext_e
    ldr x0, [L100]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# is_eq_exact_fss
    cmp x25, 1291
    b.ne @label_3-0
# i_call_last_ft
    ldr x30, [x20], 8
    b @key_map/0-1
# label_L
@label_3-0:
label_3:
# i_call_last_ft
    ldr x30, [x20], 8
    b @merge/1-2
# i_flush_stubs
# i_func_label_L
    align 8
label_4:
# func_line_I
# i_func_info_IaaI
# edlin_key:get_valid_escape_key/2
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x05, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
get_valid_escape_key/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L104
    bl L94
L104:
# i_test_yield
    adr x2, get_valid_escape_key/2
    subs w22, w22, 1
    b.le L96
# is_nonempty_list_fS
    tbnz x25, 1, @label_42-3
# get_list_Sdd
    and x8, x25, -8
    ldp x27, x28, [x8]
# i_is_tuple_of_arity_ff_ffsA
    tbnz x26, 0, @label_17-4
    and x0, x26, -8
    ldr x8, [x0]
    tst x8, 63
    b.ne @label_17-4
    cmp x8, 128
    b.ne @label_57-5
# get_two_tuple_elements_sPSS
    ldp x15, x16, [x0, 8]
# i_select_val_lins_sfI
    mov x14, 125963
    cmp x15, x14
    b.eq @label_6-6
    mov x14, 595275
    cmp x15, x14
    b.eq @label_8-7
    b @label_57-5
# label_L
@label_6-6:
label_6:
# is_eq_exact_fss
    cmp x27, 2031
    b.ne @label_7-8
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L111
    mov x3, 6
    bl L99
L111:
    sub x20, x20, 8
# i_move_sd
    str x28, [x20]
# i_move_sd
    ldr x26, [L112]
# i_move_sd
    mov x25, x16
# line_I
# call_light_bif_be
L113:
    ldr x3, [L114]
    ldr x7, [L115]
    adr x2, L113
# BIF: erlang:'++'/2
    bl L117
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L118
    mov x3, 1
    bl L99
L118:
# put_tuple2_SA
    mov x9, 192
    mov x10, 125963
    stp x9, x10, [x23], 16
    ldr x10, [x20]
    stp x25, x10, [x23], 16
    sub x26, x23, 30
# i_move_sd
    mov x25, 59
# i_call_last_ft
    add x20, x20, 8
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_7-8:
label_7:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L119
    mov x3, 6
    bl L99
L119:
# put_tuple2_SA
    mov x9, 192
    mov x10, 125963
    stp x9, x10, [x23], 16
    stp x16, x25, [x23], 16
    sub x26, x23, 30
# i_move_sd
    mov x25, 59
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_8-7:
label_8:
# is_nonempty_list_fS
    tbnz x16, 1, @label_10-9
# get_hd_Sd
    ldur x25, [x16, -1]
# is_eq_exact_fss
    cmp x25, 959
    b.ne @label_10-9
# is_ge_lt_ffScc
    mov x1, 783
    mov x2, 927
    and x8, x27, 15
    cmp x8, 15
    b.ne L121
    cmp x27, x1
    b.lt @label_9-10
    cmp x2, x27
    b.ge @label_14-11
    b L122
L121:
    mov x0, x27
    bl L126
    b.lt @label_9-10
    b.gt @label_14-11
L122:
# label_L
@label_9-10:
label_9:
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L127
    mov x3, 6
    bl L99
L127:
    sub x20, x20, 16
# store_two_values_sdsd
    stp x27, x28, [x20]
# i_move_sd
    mov x25, x16
# line_I
# i_call_ext_e
    ldr x0, [L128]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L129
    mov x3, 1
    bl L99
L129:
# put_list_ssd
    ldr x8, [x20]
    mov x9, 59
    stp x8, x9, [x23], 16
    sub x26, x23, 15
# trim_tt
    add x20, x20, 8
# call_light_bif_be
L130:
    ldr x3, [L114]
    ldr x7, [L115]
    adr x2, L130
# BIF: erlang:'++'/2
    bl L117
# test_heap_It
    add x2, x23, 88
    cmp x2, x20
    b.ls L131
    mov x3, 1
    bl L99
L131:
# put_list_ssd
    mov x8, 1471
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 22859
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b get_valid_escape_key/2
# label_L
@label_10-9:
label_10:
# i_select_val_lins_sfI
    cmp x27, 959
    b.eq @label_12-12
    cmp x27, 2031
    b.eq @label_11-13
    b L134
# label_L
@label_11-13:
label_11:
# allocate_heap_tIt
    add x2, x23, 56
    cmp x2, x20
    b.ls L135
    mov x3, 6
    bl L99
L135:
    sub x20, x20, 8
# i_move_sd
    str x28, [x20]
# put_list_ssd
    mov x8, 2031
    stp x8, x16, [x23], 16
    sub x25, x23, 15
# line_I
# i_call_ext_e
    ldr x0, [L128]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# test_heap_It
    add x2, x23, 88
    cmp x2, x20
    b.ls L136
    mov x3, 1
    bl L99
L136:
# put_list_ssd
    mov x8, 1471
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 125963
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b get_valid_escape_key/2
# label_L
@label_12-12:
label_12:
# test_heap_It
    add x2, x23, 72
    cmp x2, x20
    b.ls L137
    mov x3, 6
    bl L99
L137:
# put_list_ssd
    mov x8, 959
    stp x8, x16, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 595275
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
L134:
label_13:
# is_in_range_ffScc
    and x8, x27, 15
    cmp x8, 15
    b.ne L139
    cmp x27, 783
    b.lt @label_16-14
    cmp x27, 927
    b.gt @label_15-15
    b L138
L139:
    mov x0, x27
    mov x1, 783
    mov x2, 927
    bl L143
    b.lt @label_16-14
    b.gt @label_15-15
L138:
# label_L
@label_14-11:
label_14:
# test_heap_It
    add x2, x23, 72
    cmp x2, x20
    b.ls L144
    mov x3, 6
    bl L99
L144:
# put_list_ssd
    stp x27, x16, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 595275
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_15-15:
label_15:
# is_eq_exact_fss
    cmp x27, 1759
    b.ne @label_16-14
# allocate_heap_tIt
    add x2, x23, 56
    cmp x2, x20
    b.ls L145
    mov x3, 6
    bl L99
L145:
    sub x20, x20, 8
# i_move_sd
    str x28, [x20]
# put_list_ssd
    mov x8, 1759
    stp x8, x16, [x23], 16
    sub x25, x23, 15
# line_I
# i_call_ext_e
    ldr x0, [L128]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# test_heap_It
    add x2, x23, 112
    cmp x2, x20
    b.ls L146
    mov x3, 1
    bl L99
L146:
# put_list_ssd
    mov x8, 1471
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 1759
    ldr x9, [x20]
    stp x8, x9, [x23], 16
    sub x26, x23, 15
# put_tuple2_SA
    mov x9, 192
    mov x10, 22859
    stp x9, x10, [x23], 16
    stp x25, x26, [x23], 16
    sub x25, x23, 30
# deallocate_t
    add x20, x20, 8
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_16-14:
label_16:
# is_in_range_ffScc
# simplified small & range tests since failure labels are equal
    sub x8, x27, 543
    tst x8, 15
    b.ne L150
    cmp x8, 1488
    b.hi @label_59-16
    b L149
L150:
    mov x0, x27
    mov x1, 543
    mov x2, 2031
    bl L143
    b.ne @label_59-16
L149:
# allocate_heap_tIt
    add x2, x23, 56
    cmp x2, x20
    b.ls L152
    mov x3, 6
    bl L99
L152:
    sub x20, x20, 8
# i_move_sd
    str x28, [x20]
# put_list_ssd
    stp x27, x16, [x23], 16
    sub x25, x23, 15
# line_I
# i_call_ext_e
    ldr x0, [L128]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# test_heap_It
    add x2, x23, 88
    cmp x2, x20
    b.ls L153
    mov x3, 1
    bl L99
L153:
# put_list_ssd
    mov x8, 1471
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 125963
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b get_valid_escape_key/2
# label_L
@label_17-4:
label_17:
# i_select_val_lins_sfI
    cmp x26, 1291
    b.eq @label_18-17
    mov x14, 26891
    cmp x26, x14
    b.eq @label_33-18
    mov x14, 595339
    cmp x26, x14
    b.eq @label_31-19
    mov x14, 595403
    cmp x26, x14
    b.eq @label_27-20
    mov x14, 595467
    cmp x26, x14
    b.eq @label_24-21
    mov x14, 595531
    cmp x26, x14
    b.eq @label_22-22
    b @label_57-5
# label_L
@label_18-17:
label_18:
# is_eq_exact_fss
    cmp x27, 447
    b.ne @label_19-23
# i_move_sd
    mov x26, 26891
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_19-23:
label_19:
# is_ge_lt_ffScc
    mov x1, 15
    mov x2, 511
    and x8, x27, 15
    cmp x8, 15
    b.ne L161
    cmp x27, x1
    b.lt @label_21-24
    cmp x2, x27
    b.ge @label_20-25
    b L162
L161:
    mov x0, x27
    bl L126
    b.lt @label_21-24
    b.gt @label_20-25
L162:
# is_eq_exact_fss
    cmp x27, 2047
    b.ne @label_21-24
# label_L
@label_20-25:
label_20:
# test_heap_It
    add x2, x23, 80
    cmp x2, x20
    b.ls L165
    mov x3, 4
    bl L99
L165:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 192
    mov x10, 591371
    stp x9, x10, [x23], 16
    stp x25, x28, [x23], 16
    sub x25, x23, 30
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_21-24:
label_21:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L166
    mov x3, 4
    bl L99
L166:
# put_tuple2_SA
    mov x9, 192
    mov x10, 60107
    stp x9, x10, [x23], 16
    stp x27, x28, [x23], 16
    sub x25, x23, 30
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_22-22:
label_22:
# is_in_range_ffScc
# simplified small & range tests since failure labels are equal
    sub x8, x27, 543
    tst x8, 15
    b.ne L168
    cmp x8, 1488
    b.hi @label_23-26
    b L167
L168:
    mov x0, x27
    mov x1, 543
    mov x2, 2031
    bl L143
    b.ne @label_23-26
L167:
# test_heap_It
    add x2, x23, 104
    cmp x2, x20
    b.ls L170
    mov x3, 4
    bl L99
L170:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 1279
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 125963
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_23-26:
label_23:
# test_heap_It
    add x2, x23, 104
    cmp x2, x20
    b.ls L171
    mov x3, 4
    bl L99
L171:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 1279
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 22859
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_24-21:
label_24:
# is_eq_exact_fss
    cmp x27, 1471
    b.ne @label_25-27
# i_move_sd
    mov x26, 595339
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_25-27:
label_25:
# is_in_range_ffScc
# simplified small & range tests since failure labels are equal
    sub x8, x27, 543
    tst x8, 15
    b.ne L174
    cmp x8, 1488
    b.hi @label_26-28
    b L173
L174:
    mov x0, x27
    mov x1, 543
    mov x2, 2031
    bl L143
    b.ne @label_26-28
L173:
# test_heap_It
    add x2, x23, 104
    cmp x2, x20
    b.ls L176
    mov x3, 4
    bl L99
L176:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 125963
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_26-28:
label_26:
# test_heap_It
    add x2, x23, 104
    cmp x2, x20
    b.ls L177
    mov x3, 4
    bl L99
L177:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 22859
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_27-20:
label_27:
# is_in_range_ffScc
    and x8, x27, 15
    cmp x8, 15
    b.ne L179
    cmp x27, 783
    b.lt @label_32-29
    cmp x27, 927
    b.gt @label_28-30
    b L178
L179:
    mov x0, x27
    mov x1, 783
    mov x2, 927
    bl L143
    b.lt @label_32-29
    b.gt @label_28-30
L178:
# test_heap_It
    add x2, x23, 72
    cmp x2, x20
    b.ls L182
    mov x3, 4
    bl L99
L182:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 595275
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_28-30:
label_28:
# is_in_range_ffScc
    and x8, x27, 15
    cmp x8, 15
    b.ne L184
    cmp x27, 1567
    b.lt @label_29-31
    cmp x27, 1967
    b.gt @label_32-29
    b L183
L184:
    mov x0, x27
    mov x1, 1567
    mov x2, 1967
    bl L143
    b.lt @label_29-31
    b.gt @label_32-29
L183:
# jump_f
    b @label_30-32
# label_L
@label_29-31:
label_29:
# is_in_range_ffScc
# simplified small test since all other types are boxed
    tbz x27, 0, L188
# simplified range test since failure labels are equal
    sub x8, x27, 1055
    cmp x8, 400
    b.hi @label_32-29
    b L187
L188:
    mov x0, x27
    mov x1, 1055
    mov x2, 1455
    bl L143
    b.ne @label_32-29
L187:
# label_L
@label_30-32:
label_30:
# test_heap_It
    add x2, x23, 104
    cmp x2, x20
    b.ls L189
    mov x3, 4
    bl L99
L189:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 1471
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 125963
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_31-19:
label_31:
# is_in_range_ffScc
# simplified small & range tests since failure labels are equal
    sub x8, x27, 543
    tst x8, 15
    b.ne L191
    cmp x8, 1488
    b.hi @label_32-29
    b L190
L191:
    mov x0, x27
    mov x1, 543
    mov x2, 2031
    bl L143
    b.ne @label_32-29
L190:
# test_heap_It
    add x2, x23, 120
    cmp x2, x20
    b.ls L192
    mov x3, 4
    bl L99
L192:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 1471
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 125963
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_32-29:
label_32:
# test_heap_It
    add x2, x23, 104
    cmp x2, x20
    b.ls L193
    mov x3, 4
    bl L99
L193:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 1471
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 22859
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_33-18:
label_33:
# i_select_val_lins_sfI
    cmp x27, 447
    b.eq @label_36-33
    cmp x27, 1279
    b.eq @label_35-34
    cmp x27, 1471
    b.eq @label_34-35
    b L197
# label_L
@label_34-35:
label_34:
# i_move_sd
    mov x26, 595403
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_35-34:
label_35:
# i_move_sd
    mov x26, 595531
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_36-33:
label_36:
# i_move_sd
    mov x26, 595467
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
L197:
label_37:
# is_in_range_ffScc
    and x8, x27, 15
    cmp x8, 15
    b.ne L199
    cmp x27, 543
    b.lt @label_38-36
    cmp x27, 2031
    b.gt @label_39-37
    b L198
L199:
    mov x0, x27
    mov x1, 543
    mov x2, 2031
    bl L143
    b.lt @label_38-36
    b.gt @label_39-37
L198:
# jump_f
    b @label_40-38
# label_L
@label_38-36:
label_38:
# is_ge_lt_ffScc
    mov x1, 15
    mov x2, 511
    and x8, x27, 15
    cmp x8, 15
    b.ne L203
    cmp x27, x1
    b.lt @label_41-39
    cmp x2, x27
    b.ge @label_40-40
    b L204
L203:
    mov x0, x27
    bl L126
    b.lt @label_41-39
    b.gt @label_40-40
L204:
# label_L
@label_39-37:
label_39:
# is_eq_exact_fss
    cmp x27, 2047
    b.ne @label_41-39
# label_L
@label_40-38:
@label_40-40:
label_40:
# test_heap_It
    add x2, x23, 88
    cmp x2, x20
    b.ls L207
    mov x3, 4
    bl L99
L207:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 125963
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_41-39:
label_41:
# test_heap_It
    add x2, x23, 88
    cmp x2, x20
    b.ls L208
    mov x3, 4
    bl L99
L208:
# put_list_ssd
    mov x9, 59
    stp x27, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 22859
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, x28
# i_call_only_f
    ldr x30, [x20], 8
    b get_valid_escape_key/2
# label_L
@label_42-3:
label_42:
# is_nil_fS
    cmp x25, 59
    b.ne @label_57-5
# i_is_tagged_tuple_fsAa
    tbnz x26, 0, @label_43-41
    and x0, x26, -8
    ldp x8, x9, [x0]
    mov x14, 595275
    cmp x9, x14
    mov x10, 128
    ccmp x8, x10, 0, 2
    b.ne @label_43-41
# i_get_tuple_element_sPS
    ldr x25, [x0, 16]
# is_nonempty_list_fS
    tbnz x25, 1, @label_43-41
# get_tl_Sd
    ldur x27, [x25, 7]
# is_nil_fS
    cmp x27, 59
    b.ne @label_43-41
# test_heap_It
    add x2, x23, 96
    cmp x2, x20
    b.ls L210
    mov x3, 1
    bl L99
L210:
# put_list_ssd
    mov x8, 1471
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 447
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 192
    mov x10, 591371
    stp x9, x10, [x23], 16
    mov x10, 59
    stp x25, x10, [x23], 16
    sub x25, x23, 30
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_43-41:
label_43:
# i_is_tuple_fs
    tbnz x26, 0, @label_51-42
    and x0, x26, -8
    ldr x8, [x0]
    tst x8, 63
    b.ne @label_51-42
# i_select_tuple_arity_SfI
# skipped box test since argument is always boxed
    ldur x8, [x26, -2]
# simplified tuple test since the source is always a tuple when boxed
# Linear search in [0..1], 2 elements
    cmp x8, 128
    b.eq @label_47-44
    cmp x8, 192
    b.eq @label_44-45
    b @label_62-43
# label_L
@label_44-45:
label_44:
# load_tuple_ptr_s
    and x0, x26, -8
# get_two_tuple_elements_sPSS
    ldp x25, x27, [x0, 8]
# i_get_tuple_element_sPS
    ldr x26, [x0, 24]
# i_select_val_lins_sfI
    mov x14, 22859
    cmp x25, x14
    b.eq @label_45-46
    mov x14, 125963
    cmp x25, x14
    b.eq @label_46-47
    b L217
# label_L
@label_45-46:
label_45:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L218
    mov x3, 3
    bl L99
L218:
# put_tuple2_SA
    mov x9, 192
    mov x10, 22859
    stp x9, x10, [x23], 16
    stp x27, x26, [x23], 16
    sub x25, x23, 30
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_46-47:
label_46:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L219
    mov x3, 3
    bl L99
L219:
# put_tuple2_SA
    mov x9, 192
    mov x10, 591371
    stp x9, x10, [x23], 16
    stp x27, x26, [x23], 16
    sub x25, x23, 30
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_47-44:
label_47:
# load_tuple_ptr_s
    and x0, x26, -8
# get_two_tuple_elements_sPSS
    ldp x25, x26, [x0, 8]
# i_select_val_lins_sfI
    mov x14, 22859
    cmp x25, x14
    b.eq @label_48-48
    mov x14, 125963
    cmp x25, x14
    b.eq @label_49-49
    mov x14, 595275
    cmp x25, x14
    b.eq @label_50-50
    b L223
# label_L
@label_48-48:
label_48:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L224
    mov x3, 2
    bl L99
L224:
# put_tuple2_SA
    mov x9, 192
    mov x10, 22859
    stp x9, x10, [x23], 16
    mov x10, 59
    stp x26, x10, [x23], 16
    sub x25, x23, 30
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_49-49:
label_49:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L225
    mov x3, 2
    bl L99
L225:
# put_tuple2_SA
    mov x9, 192
    mov x10, 591371
    stp x9, x10, [x23], 16
    mov x10, 59
    stp x26, x10, [x23], 16
    sub x25, x23, 30
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_50-50:
label_50:
# test_heap_It
    add x2, x23, 80
    cmp x2, x20
    b.ls L226
    mov x3, 2
    bl L99
L226:
# put_tuple2_SA
    mov x9, 128
    mov x10, 595275
    stp x9, x10, [x23], 16
    str x26, [x23], 8
    sub x25, x23, 22
# put_tuple2_SA
    mov x9, 128
    mov x10, 83275
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_51-42:
label_51:
# i_select_val_lins_sfI
    mov x14, 26891
    cmp x26, x14
    b.eq @label_56-51
    mov x14, 595339
    cmp x26, x14
    b.eq @label_55-52
    mov x14, 595403
    cmp x26, x14
    b.eq @label_54-53
    mov x14, 595467
    cmp x26, x14
    b.eq @label_53-54
    mov x14, 595531
    cmp x26, x14
    b.eq @label_52-55
    b @label_62-43
# label_L
@label_52-55:
label_52:
# i_move_sd
    ldr x25, [L232]
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_53-54:
label_53:
# i_move_sd
    ldr x25, [L233]
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_54-53:
label_54:
# i_move_sd
    ldr x25, [L234]
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_55-52:
label_55:
# i_move_sd
    ldr x25, [L235]
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_56-51:
label_56:
# i_move_sd
    ldr x25, [L236]
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_57-5:
label_57:
# i_is_tagged_tuple_fsAa
    tbnz x26, 0, @label_58-56
    and x0, x26, -8
    ldp x8, x9, [x0]
    mov x14, 22859
    cmp x9, x14
    mov x10, 128
    ccmp x8, x10, 0, 2
    b.ne @label_58-56
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L238
    mov x3, 2
    bl L99
L238:
# load_tuple_ptr_s
    and x0, x26, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# put_tuple2_SA
    mov x9, 192
    mov x10, 22859
    stp x9, x10, [x23], 16
    stp x26, x25, [x23], 16
    sub x25, x23, 30
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_58-56:
label_58:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L239
    mov x3, 2
    bl L99
L239:
# put_tuple2_SA
    mov x9, 192
    mov x10, 22859
    stp x9, x10, [x23], 16
    stp x26, x25, [x23], 16
    sub x25, x23, 30
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_59-16:
label_59:
# line_I
# case_end_s
    mov x8, 7248
    stp x8, x27, [x21, 96]
    bl L241
# label_L
L217:
label_60:
# line_I
    nop
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L241
# label_L
L223:
label_61:
# line_I
    nop
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L241
# label_L
@label_62-43:
label_62:
# line_I
    nop
# case_end_s
    mov x8, 7248
    stp x8, x26, [x21, 96]
    bl L241
# i_flush_stubs
# i_func_label_L
    nop
    align 8
label_63:
# func_line_I
# i_func_info_IaaI
# edlin_key:merge/1
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@merge/1-2:
merge/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L242
    bl L94
L242:
# i_test_yield
    adr x2, merge/1
    subs w22, w22, 1
    b.le L96
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L243
    mov x3, 1
    bl L99
L243:
    sub x20, x20, 8
# i_move_sd
    str x25, [x20]
# line_I
# i_call_f
    bl @key_map/0-1
# i_move_sd
    ldr x26, [L244]
# i_move_sd
    mov x27, x25
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b @merge/3-57
# i_flush_stubs
# i_func_label_L
label_65:
# func_line_I
# i_func_info_IaaI
# edlin_key:merge/3
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@merge/3-57:
merge/3:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L246
    bl L94
L246:
# i_test_yield
    adr x2, merge/3
    subs w22, w22, 1
    b.le L96
# is_nonempty_list_fS
    tbnz x26, 1, @label_70-58
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L248
    mov x3, 3
    bl L99
L248:
# get_list_Sdd
    and x8, x26, -8
    ldp x28, x26, [x8]
# i_move_sd
    ldr x15, [L249]
# is_map_fs
    tbnz x25, 0, @label_69-59
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_69-59
# allocate_tt
    add x2, x23, 64
    cmp x2, x20
    b.ls L251
    mov x3, 5
    bl L99
L251:
    sub x20, x20, 32
# store_two_values_sdsd
    stp x28, x26, [x20]
# store_two_values_sdsd
    stp x27, x25, [x20, 16]
# i_get_map_element_fSSS
    mov x0, x25
    mov x1, x28
    bl L253
    b.ne @label_67-60
    mov x26, x0
# jump_f
    b @label_68-61
# label_L
@label_67-60:
label_67:
# i_move_sd
    ldr x26, [L256]
# label_L
@label_68-61:
label_68:
# i_move_sd
    mov x25, x15
# line_I
# i_call_ext_e
    ldr x0, [L257]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# line_I
# i_move_sd
    mov x26, x25
# bif_map_get_jssd
    ldr x0, [x20, 16]
    ldr x1, [x20]
# skipped test for map for known map argument
    bl L253
    b.eq L258
    ldr x0, [x20, 16]
    ldr x1, [x20]
    bl L260
L258:
    mov x25, x0
# call_light_bif_be
L261:
    ldr x3, [L262]
    ldr x7, [L263]
    adr x2, L261
# BIF: maps:merge/2
    bl L117
# update_map_assoc_sdtI
    ldr x1, [x20]
    ldr x3, [x20, 16]
    mov x2, x25
    bl L265
    mov x27, x0
# i_move_sd
    ldr x26, [x20, 8]
# move_call_last_ydft
    ldp x25, x30, [x20, 24]
    add x20, x20, 40
    b merge/3
# label_L
@label_69-59:
label_69:
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L266
    mov x3, 1
    bl L99
L266:
# put_tuple2_SA
    mov x9, 128
    mov x10, 5387
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L267
    mov x3, 1
    bl L99
L267:
# call_light_bif_be
L268:
    ldr x3, [L269]
    ldr x7, [L270]
    adr x2, L268
# BIF: erlang:error/1
    bl L117
# mark_unreachable
# label_L
@label_70-58:
label_70:
# i_move_sd
    mov x25, x27
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_71:
# func_line_I
# i_func_info_IaaI
# edlin_key:key_map/0
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x05, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@key_map/0-1:
key_map/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L271
    bl L94
L271:
# i_test_yield
    adr x2, key_map/0
    subs w22, w22, 1
    b.le L96
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L272
    mov x3, xzr
    bl L99
L272:
# line_I
# i_call_f
    bl @normal_map/0-62
# i_new_small_map_lit_dtqI
    add x2, x23, 96
    cmp x2, x20
    b.ls L274
    mov x3, 1
    bl L99
L274:
    add x8, x23, 2
    mov x9, 300
    mov x10, 4
    stp x9, x10, [x23], 16
    ldr x9, [L275]
    stp x9, x25, [x23], 16
    ldr x9, [L276]
    ldr x10, [L277]
    stp x9, x10, [x23], 16
    ldr x14, [L278]
    str x14, [x23], 8
    mov x25, x8
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_73:
# func_line_I
# i_func_info_IaaI
# edlin_key:normal_map/0
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x16, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@normal_map/0-62:
normal_map/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L279
    bl L94
L279:
# i_test_yield
    adr x2, normal_map/0
    subs w22, w22, 1
    b.le L96
# i_move_sd
    ldr x25, [L280]
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_75:
# func_line_I
# i_func_info_IaaI
# edlin_key:valid_functions/0
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x16, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
valid_functions/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L281
    bl L94
L281:
# i_test_yield
    adr x2, valid_functions/0
    subs w22, w22, 1
    b.le L96
# i_move_sd
    ldr x25, [L282]
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_77:
# func_line_I
# i_func_info_IaaI
# edlin_key:module_info/0
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L283
    bl L94
L283:
# i_test_yield
    adr x2, module_info/0
    subs w22, w22, 1
    b.le L96
# i_move_sd
    mov x25, 221259
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L284
    mov x3, 1
    bl L99
L284:
# call_light_bif_be
L285:
    ldr x3, [L286]
    ldr x7, [L287]
    adr x2, L285
# BIF: erlang:get_module_info/1
    bl L117
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_79:
# func_line_I
# i_func_info_IaaI
# edlin_key:module_info/1
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L288
    bl L94
L288:
# i_test_yield
    adr x2, module_info/1
    subs w22, w22, 1
    b.le L96
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 221259
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L289
    mov x3, 2
    bl L99
L289:
# call_light_bif_be
L290:
    ldr x3, [L291]
    ldr x7, [L292]
    adr x2, L290
# BIF: erlang:get_module_info/2
    bl L117
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# i_flush_stubs
# i_func_label_L
label_81:
# func_line_I
# i_func_info_IaaI
# edlin_key:'-merge/3-fun-0-'/2
    bl L91
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x60, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x17, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
'-merge/3-fun-0-'/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L293
    bl L94
L293:
# i_test_yield
    adr x2, '-merge/3-fun-0-'/2
    subs w22, w22, 1
    b.le L96
# is_list_fs
    tst x25, 2
    mov x14, 59
    ccmp x25, x14, 4, 3
    b.ne @label_86-63
# is_atom_fs
    and x8, x26, 63
    cmp x8, 11
    b.ne @label_86-63
# allocate_tt
    add x2, x23, 56
    cmp x2, x20
    b.ls L295
    mov x3, 2
    bl L99
L295:
    sub x20, x20, 24
# store_two_values_sdsd
    stp x25, x26, [x20]
# catch_yH
    ldr x8, [x21, 248]
    add x8, x8, 1
    str x8, [x21, 248]
    ldr x14, [L296]
    str x14, [x20, 16]
# i_move_sd
    mov x26, 1291
# line_I
# i_call_f
    bl get_valid_escape_key/2
# i_is_tagged_tuple_fsAa
# skipped box test since argument is always boxed
    and x0, x25, -8
    ldp x8, x9, [x0]
    mov x14, 591371
    cmp x9, x14
    mov x10, 192
    ccmp x8, x10, 0, 2
    b.ne @label_89-64
# i_get_tuple_element_sPS
    ldr x26, [x0, 24]
# is_nil_fS
    cmp x26, 59
    b.ne @label_89-64
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# is_eq_exact_fss
    ldr x1, [x20]
    cmp x26, x1
    b.eq L298
    orr x14, x26, x1
    tbnz x14, 1, @label_89-64
    mov x0, x26
    stp x15, x16, [x19, 96]
    bl L300
    ldp x15, x16, [x19, 96]
    cbz w0, @label_89-64
L298:
# line_I
# i_call_f
    bl valid_functions/0
# i_move_sd
    mov x26, x25
# i_move_sd
    ldr x25, [x20, 8]
# call_light_bif_be
L301:
    ldr x3, [L302]
    ldr x7, [L303]
    adr x2, L301
# BIF: lists:member/2
    bl L117
# is_eq_exact_fss
    cmp x25, 75
    b.ne @label_83-65
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L305
    mov x3, xzr
    bl L99
L305:
# put_tuple2_SA
    mov x9, 128
    mov x10, 75
    stp x9, x10, [x23], 16
    ldr x14, [x20, 8]
    str x14, [x23], 8
    sub x25, x23, 22
# jump_f
    b @label_84-66
# label_L
@label_83-65:
label_83:
# test_heap_It
    add x2, x23, 80
    cmp x2, x20
    b.ls L307
    mov x3, xzr
    bl L99
L307:
# put_list_ssd
    ldr x8, [x20, 8]
    mov x9, 59
    stp x8, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    ldr x8, [x20]
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    ldr x8, [x20, 8]
    stp x8, x25, [x23], 16
    sub x27, x23, 15
# i_move_sd
    ldr x26, [L308]
# i_move_sd
    mov x25, 95307
# line_I
# i_call_ext_e
    ldr x0, [L309]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x25, 11
# label_L
@label_84-66:
label_84:
# try_end_deallocate_t
    ldr x8, [x21, 248]
    sub x8, x8, 1
    str x8, [x21, 248]
    add x20, x20, 24
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
label_85:
# try_case_y
    ldr x8, [x21, 248]
    mov x25, x28
    sub x8, x8, 1
    str x8, [x21, 248]
# test_heap_It
    add x2, x23, 80
    cmp x2, x20
    b.ls L310
    mov x3, xzr
    bl L99
L310:
# put_list_ssd
    ldr x8, [x20, 8]
    mov x9, 59
    stp x8, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    ldr x8, [x20]
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
# skipped fetching of BEAM register
    stp x8, x25, [x23], 16
    sub x27, x23, 15
# move_trim_sdt
    ldr x26, [L311]
    add x20, x20, 24
# i_move_sd
    mov x25, 95307
# line_I
# i_call_ext_e
    ldr x0, [L309]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x25, 11
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_86-63:
label_86:
# is_eq_exact_fss
    mov x14, 11723
    cmp x25, x14
    b.ne @label_88-67
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L313
    mov x3, 2
    bl L99
L313:
    sub x20, x20, 8
# i_move_sd
    str x26, [x20]
# line_I
# i_call_f
    bl valid_functions/0
# i_move_sd
    mov x26, x25
# i_move_sd
    ldr x25, [x20]
# call_light_bif_be
L314:
    ldr x3, [L302]
    ldr x7, [L303]
    adr x2, L314
# BIF: lists:member/2
    bl L117
# is_eq_exact_fss
    cmp x25, 75
    b.ne @label_87-68
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L316
    mov x3, xzr
    bl L99
L316:
# put_tuple2_SA
    mov x9, 128
    mov x10, 75
    stp x9, x10, [x23], 16
    ldr x14, [x20]
    str x14, [x23], 8
    sub x25, x23, 22
# deallocate_t
    add x20, x20, 8
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_87-68:
label_87:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L317
    mov x3, xzr
    bl L99
L317:
# put_list_ssd
    ldr x8, [x20]
    mov x9, 59
    stp x8, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
# skipped fetching of BEAM register
    stp x8, x25, [x23], 16
    sub x27, x23, 15
# move_trim_sdt
    ldr x26, [L318]
    add x20, x20, 8
# i_move_sd
    mov x25, 95307
# line_I
# i_call_ext_e
    ldr x0, [L309]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x25, 11
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_88-67:
label_88:
# allocate_heap_tIt
    add x2, x23, 64
    cmp x2, x20
    b.ls L319
    mov x3, 2
    bl L99
L319:
# put_list_ssd
    mov x9, 59
    stp x26, x9, [x23], 16
    sub x26, x23, 15
# put_list_ssd
    stp x25, x26, [x23], 16
    sub x27, x23, 15
# i_move_sd
    ldr x26, [L320]
# i_move_sd
    mov x25, 95307
# line_I
# i_call_ext_e
    ldr x0, [L309]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x25, 11
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L148
    ret x30
# label_L
@label_89-64:
label_89:
# line_I
# badmatch_s
    mov x8, 5200
    stp x8, x25, [x21, 96]
    bl L241
# int_code_end
L321:
    mov x0, 4369093202
    bl L323
# Begin stub section
    align 8
L100:
.xword 0x7FFFFFFFFFFFFFFF
L112:
.xword 0x7FFFFFFFFFFFFFFF
L114:
.xword 0x7FFFFFFFFFFFFFFF
L115:
.xword 0x000000010442CDE4
L128:
.xword 0x7FFFFFFFFFFFFFFF
# End stub section
L324:
L323:
L322:
    mov x14, 4365818364
    br x14
L265:
L264:
    mov x14, 4481917432
    br x14
L94:
L93:
    mov x14, 4481913368
    br x14
L260:
L259:
    mov x14, 4481912456
    br x14
L143:
L142:
    mov x14, 4481915600
    br x14
L253:
L252:
    mov x14, 4481913616
    br x14
L300:
L299:
    mov x14, 4366560408
    br x14
L148:
L147:
    mov x14, 4481911760
    br x14
L126:
L125:
    mov x14, 4481915776
    br x14
L91:
L90:
    mov x14, 4481913584
    br x14
L117:
L116:
    mov x14, 4481910672
    br x14
L99:
L98:
    mov x14, 4481912640
    br x14
L241:
L240:
    mov x14, 4481916920
    br x14
L96:
L95:
    mov x14, 4481914968
    br x14
# Begin stub section
L232:
.xword 0x7FFFFFFFFFFFFFFF
L233:
.xword 0x7FFFFFFFFFFFFFFF
L234:
.xword 0x7FFFFFFFFFFFFFFF
L235:
.xword 0x7FFFFFFFFFFFFFFF
L236:
.xword 0x7FFFFFFFFFFFFFFF
L244:
.xword 0x7FFFFFFFFFFFFFFF
L249:
.xword 0x7FFFFFFFFFFFFFFF
L256:
.xword 0x7FFFFFFFFFFFFFFF
L257:
.xword 0x7FFFFFFFFFFFFFFF
L262:
.xword 0x7FFFFFFFFFFFFFFF
L263:
.xword 0x000000010454F074
L269:
.xword 0x7FFFFFFFFFFFFFFF
L270:
.xword 0x000000010444DA38
L275:
.xword 0x7FFFFFFFFFFFFFFF
L276:
.xword 0x7FFFFFFFFFFFFFFF
L277:
.xword 0x7FFFFFFFFFFFFFFF
L278:
.xword 0x7FFFFFFFFFFFFFFF
L280:
.xword 0x7FFFFFFFFFFFFFFF
L282:
.xword 0x7FFFFFFFFFFFFFFF
L286:
.xword 0x7FFFFFFFFFFFFFFF
L287:
.xword 0x000000010442AAD0
L291:
.xword 0x7FFFFFFFFFFFFFFF
L292:
.xword 0x000000010442AD84
L296:
.xword 0x000000007FFFFFFF
L302:
.xword 0x7FFFFFFFFFFFFFFF
L303:
.xword 0x000000010442D528
L308:
.xword 0x7FFFFFFFFFFFFFFF
L309:
.xword 0x7FFFFFFFFFFFFFFF
L311:
.xword 0x7FFFFFFFFFFFFFFF
L318:
.xword 0x7FFFFFFFFFFFFFFF
L320:
.xword 0x7FFFFFFFFFFFFFFF
# End stub section
L325:
.section .rodata {#1}
line:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.section .text {#0}
.section .rodata {#1}
attr:
.byte 0x83, 0x6C, 0x00, 0x00, 0x00, 0x01, 0x68, 0x02, 0x77, 0x03, 0x76, 0x73, 0x6E, 0x6C, 0x00, 0x00, 0x00, 0x01, 0x6E, 0x10, 0x00, 0x59, 0x14, 0xC9, 0x10, 0xDB, 0xED, 0x46, 0x42, 0x50, 0x9D, 0x55, 0x84, 0x58, 0xFC, 0x19, 0xA4, 0x6A, 0x6A
.section .text {#0}
.section .rodata {#1}
compile:
.byte 0x83, 0x6C, 0x00, 0x00, 0x00, 0x03, 0x68, 0x02, 0x77, 0x07, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x6B, 0x00, 0x05, 0x38, 0x2E, 0x36, 0x2E, 0x31, 0x68, 0x02, 0x77, 0x07, 0x6F, 0x70, 0x74, 0x69, 0x6F, 0x6E, 0x73, 0x6C, 0x00, 0x00, 0x00, 0x07, 0x77, 0x0A, 0x64, 0x65, 0x62, 0x75, 0x67, 0x5F, 0x69, 0x6E, 0x66, 0x6F, 0x68, 0x02, 0x77, 0x01, 0x69, 0x6B, 0x00, 0x28, 0x2F, 0x62, 0x75, 0x69, 0x6C, 0x64, 0x72, 0x6F, 0x6F, 0x74, 0x2F, 0x6F, 0x74, 0x70, 0x2F, 0x6C, 0x69, 0x62, 0x2F, 0x73, 0x74, 0x64, 0x6C, 0x69, 0x62, 0x2F, 0x73, 0x72, 0x63, 0x2F, 0x2E, 0x2E, 0x2F, 0x69, 0x6E, 0x63, 0x6C, 0x75, 0x64, 0x65, 0x68, 0x02, 0x77, 0x01, 0x69, 0x6B, 0x00, 0x32, 0x2F, 0x62, 0x75, 0x69, 0x6C, 0x64, 0x72, 0x6F, 0x6F, 0x74, 0x2F, 0x6F, 0x74, 0x70, 0x2F, 0x6C, 0x69, 0x62, 0x2F, 0x73, 0x74, 0x64, 0x6C, 0x69, 0x62, 0x2F, 0x73, 0x72, 0x63, 0x2F, 0x2E, 0x2E, 0x2F, 0x2E, 0x2E, 0x2F, 0x6B, 0x65, 0x72, 0x6E, 0x65, 0x6C, 0x2F, 0x69, 0x6E, 0x63, 0x6C, 0x75, 0x64, 0x65, 0x77, 0x19, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x64, 0x6F, 0x63, 0x5F, 0x66, 0x75, 0x6E, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x77, 0x19, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x64, 0x6F, 0x63, 0x5F, 0x63, 0x61, 0x6C, 0x6C, 0x62, 0x61, 0x63, 0x6B, 0x77, 0x1C, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x73, 0x70, 0x65, 0x63, 0x5F, 0x64, 0x6F, 0x63, 0x75, 0x6D, 0x65, 0x6E, 0x74, 0x65, 0x64, 0x77, 0x15, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x64, 0x65, 0x70, 0x72, 0x65, 0x63, 0x61, 0x74, 0x65, 0x64, 0x5F, 0x63, 0x61, 0x74, 0x63, 0x68, 0x6A, 0x68, 0x02, 0x77, 0x06, 0x73, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x6B, 0x00, 0x2B, 0x2F, 0x62, 0x75, 0x69, 0x6C, 0x64, 0x72, 0x6F, 0x6F, 0x74, 0x2F, 0x6F, 0x74, 0x70, 0x2F, 0x6C, 0x69, 0x62, 0x2F, 0x73, 0x74, 0x64, 0x6C, 0x69, 0x62, 0x2F, 0x73, 0x72, 0x63, 0x2F, 0x65, 0x64, 0x6C, 0x69, 0x6E, 0x5F, 0x6B, 0x65, 0x79, 0x2E, 0x65, 0x72, 0x6C, 0x6A
.section .text {#0}
.section .rodata {#1}
md5:
.byte 0xA4, 0x19, 0xFC, 0x58, 0x84, 0x55, 0x9D, 0x50, 0x42, 0x46, 0xED, 0xDB, 0x10, 0xC9, 0x14, 0x59
.section .text {#0}
