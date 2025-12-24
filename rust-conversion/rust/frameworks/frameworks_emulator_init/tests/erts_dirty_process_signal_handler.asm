L33:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# i_flush_stubs
# i_func_label_L
label_1:
# func_line_I
# i_func_info_IaaI
# erts_dirty_process_signal_handler:start/0
    bl L35
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x7D, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0xA7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
start/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L36
    bl L38
L36:
# i_test_yield
    adr x2, start/0
    subs w22, w22, 1
    b.le L40
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L41
    mov x3, xzr
    bl L43
L41:
# i_move_sd
    mov x26, 75
# i_move_sd
    mov x25, 45515
# line_I
# call_light_bif_be
L44:
    ldr x3, [L45]
    ldr x7, [L46]
    adr x2, L44
# BIF: erlang:process_flag/2
    bl L48
# i_call_last_ft
    ldr x30, [x20], 8
    b @label_4-0
# i_flush_stubs
# i_func_label_L
    align 8
label_3:
# func_line_I
# i_func_info_IaaI
# erts_dirty_process_signal_handler:msg_loop/0
    bl L35
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x7D, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x7A, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@label_4-0:
label_4:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L50
    bl L38
L50:
# i_test_yield
    adr x2, label_4
    subs w22, w22, 1
    b.le L40
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L51
    mov x3, xzr
    bl L43
L51:
    sub x20, x20, 8
# i_move_sd
    mov x14, 59
    str x14, [x20]
# aligned_label_Lt
label_5:
# i_loop_rec_f
L52:
    adr x0, L52
    ldr x1, [L53]
    bl L55
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L57
    mov w22, w0
    ldp x15, x16, [x19, 96]
# catch_yH
    ldr x8, [x21, 248]
    add x8, x8, 1
    str x8, [x21, 248]
    ldr x14, [L58]
    str x14, [x20]
# line_I
# i_call_f
    bl @label_10-1
# try_end_y
    ldr x8, [x21, 248]
    sub x8, x8, 1
    str x8, [x21, 248]
    mov x8, 59
    str x8, [x20]
# jump_f
    b @label_7-2
# label_L
label_6:
# try_case_y
    ldr x8, [x21, 248]
    mov x25, x28
    sub x8, x8, 1
    str x8, [x21, 248]
# label_L
@label_7-2:
label_7:
# i_call_last_ft
    add x20, x20, 8
    ldr x30, [x20], 8
    b label_4
# aligned_label_Lt
label_8:
# wait_locked_f
    mov x0, x21
    ldr x1, [L61]
    bl L63
    b L65
# i_flush_stubs
# i_func_label_L
    align 8
label_9:
# func_line_I
# i_func_info_IaaI
# erts_dirty_process_signal_handler:handle_request/1
    bl L35
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x7D, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x37, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@label_10-1:
label_10:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L66
    bl L38
L66:
# i_test_yield
    adr x2, label_10
    subs w22, w22, 1
    b.le L40
# is_pid_fs
    and x9, x25, 15
    cmp x9, 3
    b.eq L67
    tbnz x9, 0, @label_11-3
    ldur x9, [x25, -2]
    and x9, x9, 63
    cmp x9, 48
    b.ne @label_11-3
L67:
# i_move_sd
    mov x26, 15
# i_call_only_f
    ldr x30, [x20], 8
    b @label_21-4
# label_L
@label_11-3:
label_11:
# i_is_tuple_of_arity_fsA
    tbnz x25, 0, @label_17-5
    and x0, x25, -8
    ldr x8, [x0]
    cmp x8, 256
    b.ne @label_17-5
# i_get_tuple_element_sPS
    ldr x26, [x0, 32]
# i_is_tuple_of_arity_fsA
    tbnz x26, 0, @label_17-5
    and x0, x26, -8
    ldr x8, [x0]
    cmp x8, 192
    b.ne @label_17-5
# allocate_tt
    add x2, x23, 64
    cmp x2, x20
    b.ls L71
    mov x3, 2
    bl L43
L71:
    sub x20, x20, 32
# store_two_values_sdsd
    stp x26, x25, [x20, 16]
# load_tuple_ptr_s
    and x0, x26, -8
# get_two_tuple_elements_sPSS
    ldp x27, x28, [x0, 8]
# i_get_tuple_element_sPS
    ldr x15, [x0, 24]
# load_tuple_ptr_s
    and x0, x25, -8
# get_two_tuple_elements_sPSS
    ldp x8, x9, [x0, 8]
    stp x9, x8, [x20]
# i_move_sd
# simplified fetching of BEAM register
    mov x26, x9
# i_move_sd
    mov x16, 15
# i_move_sd
# simplified fetching of BEAM register
    mov x25, x8
# line_I
# i_call_f
    bl @label_25-6
# i_select_val_lins_sfI
    cmp x25, 523
    b.eq @label_12-7
    mov x14, 7627
    cmp x25, x14
    b.eq @label_16-8
    mov x14, 163147
    cmp x25, x14
    b.eq @label_15-9
    b L76
# label_L
@label_12-7:
label_12:
# load_tuple_ptr_s
    ldr x8, [x20, 24]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x27, [x0, 24]
# i_move_sd
    ldr x26, [x20]
# load_two_xregs_dxdx
    ldp x25, x28, [x20, 8]
# init_yregs_I
    mov x8, 59
    stp x8, x8, [x20]
    str x8, [x20, 16]
# line_I
# call_light_bif_be
L77:
    ldr x3, [L78]
    ldr x7, [L79]
    adr x2, L77
# BIF: erts_internal:request_system_task/4
    bl L48
# i_select_val_lins_sfI
    mov x14, 12427
    cmp x25, x14
    b.eq @label_14-10
    mov x14, 32139
    cmp x25, x14
    b.eq @label_13-11
    b L82
# label_L
@label_13-11:
label_13:
# deallocate_t
    add x20, x20, 32
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# label_L
@label_14-10:
label_14:
# move_call_last_ydft
    ldp x25, x30, [x20, 24]
    add x20, x20, 40
    b label_10
# label_L
@label_15-9:
label_15:
# i_move_sd
    mov x25, 32139
# deallocate_t
    add x20, x20, 32
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# label_L
@label_16-8:
label_16:
# self_d
    ldr x25, [x21]
# i_move_sd
    ldr x26, [x20, 24]
# line_I
# send
L85:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L85
    bl L48
# deallocate_t
    add x20, x20, 32
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# label_L
@label_17-5:
label_17:
# i_move_sd
    mov x25, 21579
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# label_L
L82:
label_18:
# line_I
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L89
# label_L
L76:
label_19:
# line_I
    nop
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L89
# i_flush_stubs
# i_func_label_L
    nop
label_20:
# func_line_I
# i_func_info_IaaI
# erts_dirty_process_signal_handler:handle_incoming_signals/2
    bl L35
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x7D, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x7D, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@label_21-4:
label_21:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L90
    bl L38
L90:
# i_test_yield
    adr x2, label_21
    subs w22, w22, 1
    b.le L40
# is_eq_exact_fss
    cmp x26, 95
    b.ne @label_22-12
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L92
    mov x3, 1
    bl L43
L92:
# i_move_sd
    mov x26, x25
# self_d
    ldr x25, [x21]
# line_I
# send
L93:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L93
    bl L48
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# label_L
@label_22-12:
label_22:
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L94
    mov x3, 2
    bl L43
L94:
    sub x20, x20, 16
# store_two_values_sdsd
    stp x26, x25, [x20]
# line_I
# call_light_bif_be
L95:
    ldr x3, [L96]
    ldr x7, [L97]
    adr x2, L95
# BIF: erts_internal:dirty_process_handle_signals/1
    bl L48
# is_eq_exact_fss
    mov x14, 28299
    cmp x25, x14
    b.ne @label_23-13
# line_I
# i_plus_jIssd
    ldr x1, [x20]
    mov x2, 31
    adds x0, x1, 16
    and x8, x1, 15
# test for not overflow and small operands
    ccmp x8, 15, 0, 9
    b.eq L99
    bl L101
L99:
    mov x26, x0
# move_call_last_ydft
    ldp x25, x30, [x20, 8]
    add x20, x20, 24
    b label_21
# label_L
@label_23-13:
label_23:
# i_move_sd
    mov x25, 32139
# deallocate_t
    add x20, x20, 16
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# i_flush_stubs
# i_func_label_L
label_24:
# func_line_I
# i_func_info_IaaI
# erts_dirty_process_signal_handler:handle_sys_task/6
    bl L35
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x7D, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x7D, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@label_25-6:
label_25:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L102
    bl L38
L102:
# i_test_yield
    adr x2, label_25
    subs w22, w22, 1
    b.le L40
# is_eq_exact_fss
    mov x14, 53963
    cmp x27, x14
    b.ne label_24
# allocate_tt
    add x2, x23, 72
    cmp x2, x20
    b.ls L103
    mov x3, 6
    bl L43
L103:
    sub x20, x20, 40
# store_two_values_sdsd
    stp x16, x15, [x20]
# store_two_values_sdsd
    stp x28, x26, [x20, 16]
# i_move_sd
    str x25, [x20, 32]
# i_move_sd
    mov x25, x26
# i_move_sd
    mov x26, x15
# line_I
# call_light_bif_be
L104:
    ldr x3, [L105]
    ldr x7, [L106]
    adr x2, L104
# BIF: erts_internal:check_dirty_process_code/2
    bl L48
# i_select_val_lins_sfI
# (Src == 0xb || Src == 0x4b) <=> (Src | 0x40) == 0x4b
    orr x13, x25, 64
    cmp x13, 75
    b.eq @label_26-14
    mov x14, 7627
    cmp x25, x14
    b.eq @label_27-15
    b L109
# label_L
@label_26-14:
label_26:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L110
    mov x3, 1
    bl L43
L110:
# put_tuple2_SA
    mov x9, 192
    mov x10, 53963
    stp x9, x10, [x23], 16
    ldr x9, [x20, 16]
    stp x9, x25, [x23], 16
    sub x26, x23, 30
# move_trim_sdt
    ldr x25, [x20, 32]
    add x20, x20, 40
# line_I
# send
L111:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L111
    bl L48
# i_move_sd
    mov x25, 163147
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# label_L
@label_27-15:
label_27:
# is_ge_fss
    ldr x0, [x20]
    mov x1, 111
# simplified test because it always succeeds when LHS is a bignum
    tbz x0, 0, L112
    cmp x0, x1
    b.lt @label_28-16
L112:
# i_move_sd
    mov x25, 7627
# deallocate_t
    add x20, x20, 40
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# label_L
@label_28-16:
label_28:
# line_I
# i_plus_jIssd
    ldr x8, [x20]
# add small constant without overflow check
    add x16, x8, 16
# i_move_sd
    mov x27, 53963
# load_two_xregs_dxdx
    ldp x15, x28, [x20, 8]
# load_two_xregs_dxdx
    ldp x26, x25, [x20, 24]
# i_call_last_ft
    add x20, x20, 40
    ldr x30, [x20], 8
    b label_25
# label_L
L109:
label_29:
# deallocate_t
    add x20, x20, 40
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# i_flush_stubs
# i_func_label_L
label_30:
# func_line_I
# i_func_info_IaaI
# erts_dirty_process_signal_handler:module_info/0
    bl L35
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x7D, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L114
    bl L38
L114:
# i_test_yield
    adr x2, module_info/0
    subs w22, w22, 1
    b.le L40
# i_move_sd
    mov x25, 163083
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L115
    mov x3, 1
    bl L43
L115:
# call_light_bif_be
L116:
    ldr x3, [L117]
    ldr x7, [L118]
    adr x2, L116
# BIF: erlang:get_module_info/1
    bl L48
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_32:
# func_line_I
# i_func_info_IaaI
# erts_dirty_process_signal_handler:module_info/1
    bl L35
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x7D, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L119
    bl L38
L119:
# i_test_yield
    adr x2, module_info/1
    subs w22, w22, 1
    b.le L40
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 163083
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L120
    mov x3, 2
    bl L43
L120:
# call_light_bif_be
L121:
    ldr x3, [L122]
    ldr x7, [L123]
    adr x2, L121
# BIF: erlang:get_module_info/2
    bl L48
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L84
    ret x30
# int_code_end
L124:
    mov x0, 4369093202
    bl L126
L126:
L125:
    mov x14, 4365818364
    br x14
L101:
L100:
    mov x14, 4481916304
    br x14
L84:
L83:
    mov x14, 4481911760
    br x14
L57:
L56:
    mov x14, 4365840208
    br x14
L65:
L64:
    mov x14, 4481916892
    br x14
L55:
L54:
    mov x14, 4481914736
    br x14
L63:
L62:
    mov x14, 4365841468
    br x14
L35:
L34:
    mov x14, 4481913584
    br x14
L48:
L47:
    mov x14, 4481910672
    br x14
L43:
L42:
    mov x14, 4481912640
    br x14
L89:
L88:
    mov x14, 4481916920
    br x14
L40:
L39:
    mov x14, 4481914968
    br x14
L38:
L37:
    mov x14, 4481913368
    br x14
# Begin stub section
L45:
.xword 0x7FFFFFFFFFFFFFFF
L46:
.xword 0x000000010444E650
L53:
.xword label_8
L58:
.xword 0x000000007FFFFFFF
L61:
.xword label_5
L78:
.xword 0x7FFFFFFFFFFFFFFF
L79:
.xword 0x0000000104378FB8
L86:
.xword 0x0000000104787C18
L87:
.xword 0x000000010444FFB0
L96:
.xword 0x7FFFFFFFFFFFFFFF
L97:
.xword 0x0000000104477E14
L105:
.xword 0x7FFFFFFFFFFFFFFF
L106:
.xword 0x00000001043ECF44
L117:
.xword 0x7FFFFFFFFFFFFFFF
L118:
.xword 0x000000010442AAD0
L122:
.xword 0x7FFFFFFFFFFFFFFF
L123:
.xword 0x000000010442AD84
# End stub section
L127:
.section .rodata {#1}
md5:
.byte 0x54, 0x09, 0xC4, 0x55, 0x9F, 0xE7, 0x0E, 0x00, 0x09, 0x68, 0x6E, 0x60, 0xEA, 0x3D, 0x0B, 0x31
.section .text {#0}
