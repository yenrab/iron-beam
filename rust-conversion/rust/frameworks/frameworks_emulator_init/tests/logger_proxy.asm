L56:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# i_flush_stubs
# i_func_label_L
label_1:
# func_line_I
# i_func_info_IaaI
# logger_proxy:log/1
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0xDD, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
log/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L59
    bl L61
L59:
# i_test_yield
    adr x2, log/1
    subs w22, w22, 1
    b.le L63
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L64
    mov x3, 1
    bl L66
L64:
    sub x20, x20, 16
# store_two_values_sdsd
    mov x8, 59
    stp x8, x25, [x20]
# i_move_sd
    mov x26, 907
# i_move_sd
    mov x25, 219019
# line_I
# call_light_bif_be
L67:
    ldr x3, [L68]
    ldr x7, [L69]
    adr x2, L67
# BIF: persistent_term:get/2
    bl L71
# i_move_sd
    str x25, [x20]
# is_ne_exact_fss
    cmp x25, 907
    b.eq @label_3-0
# line_I
# i_call_ext_e
    ldr x0, [L73]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# self_d
    ldr x26, [x21]
# is_eq_exact_fss
    cmp x25, x26
    b.eq L74
    orr x14, x25, x26
    tbnz x14, 0, @label_4-1
    mov x0, x25
    mov x1, x26
    stp x15, x16, [x19, 96]
    bl L77
    ldp x15, x16, [x19, 96]
    cbz w0, @label_4-1
L74:
# label_L
@label_3-0:
label_3:
# i_move_sd
    mov x26, 450443
# move_trim_sdt
    ldr x25, [x20, 8]
    add x20, x20, 16
# line_I
# i_call_f
    bl @handle_load/2-2
# i_move_sd
    mov x25, 32139
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_4-1:
label_4:
# load_two_xregs_dxdx
    ldp x25, x26, [x20]
# line_I
# i_call_ext_last_et
    add x20, x20, 16
    ldr x0, [L81]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_5:
# func_line_I
# i_func_info_IaaI
# logger_proxy:start_link/0
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x6D, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
start_link/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L82
    bl L61
L82:
# i_test_yield
    adr x2, start_link/0
    subs w22, w22, 1
    b.le L63
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L83
    mov x3, xzr
    bl L66
L83:
# line_I
# i_call_ext_e
    ldr x0, [L84]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x27, 59
# i_move_sd
    mov x26, 219019
# i_move_sd
    mov x28, x25
# i_move_sd
    mov x25, 219019
# i_call_ext_last_et
    ldr x0, [L85]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_7:
# func_line_I
# i_func_info_IaaI
# logger_proxy:restart/0
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x93, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
restart/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L86
    bl L61
L86:
# i_test_yield
    adr x2, restart/0
    subs w22, w22, 1
    b.le L63
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L87
    mov x3, xzr
    bl L66
L87:
# line_I
# i_call_f
    bl @child_spec/0-3
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 219147
# i_call_ext_e
    ldr x0, [L89]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_select_tuple_arity_SfI
    tbnz x25, 0, @label_11-4
    ldur x8, [x25, -2]
    tst x8, 63
    b.ne @label_11-4
# Linear search in [0..1], 2 elements
    cmp x8, 128
    b.eq @label_10-5
    cmp x8, 192
    b.eq @label_9-6
    b @label_11-4
# label_L
@label_9-6:
label_9:
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 8]
# is_eq_exact_fss
    mov x14, 32139
    cmp x26, x14
    b.ne @label_11-4
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L93
    mov x3, 1
    bl L66
L93:
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x25, [x0, 24]
# put_tuple2_SA
    mov x9, 128
    mov x10, 32139
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_10-5:
label_10:
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 8]
# is_eq_exact_fss
    cmp x26, 779
    b.ne @label_11-4
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# i_is_tuple_of_arity_fsA
    tbnz x26, 0, @label_11-4
    and x0, x26, -8
    ldr x8, [x0]
    cmp x8, 128
    b.ne @label_11-4
# i_get_tuple_element_sPS
    ldr x27, [x0, 16]
# i_is_tuple_fs
    tbnz x27, 0, @label_11-4
    and x0, x27, -8
    ldr x8, [x0]
    tst x8, 63
    b.ne @label_11-4
# bif_element_jssd
# simplified element/2 because arguments are known types
    ldur x9, [x27, -2]
    cmp x9, 64
    b.lo @label_11-4
L94:
    ldur x27, [x27, 6]
# is_eq_exact_fss
    mov x14, 262731
    cmp x27, x14
    b.ne @label_11-4
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L95
    mov x3, 2
    bl L66
L95:
# load_tuple_ptr_s
    and x0, x26, -8
# i_get_tuple_element_sPS
    ldr x25, [x0, 8]
# put_tuple2_SA
    mov x9, 128
    mov x10, 779
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_11-4:
label_11:
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_12:
# func_line_I
# i_func_info_IaaI
# logger_proxy:child_spec/0
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0xDF, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@child_spec/0-3:
child_spec/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L96
    bl L61
L96:
# i_test_yield
    adr x2, child_spec/0
    subs w22, w22, 1
    b.le L63
# i_move_sd
    ldr x25, [L97]
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_14:
# func_line_I
# i_func_info_IaaI
# logger_proxy:get_default_config/0
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x5A, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
get_default_config/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L98
    bl L61
L98:
# i_test_yield
    adr x2, get_default_config/0
    subs w22, w22, 1
    b.le L63
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L99
    mov x3, xzr
    bl L66
L99:
# line_I
# i_call_ext_e
    ldr x0, [L100]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# is_map_fs
    tbnz x25, 0, @label_16-7
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_16-7
# update_map_assoc_sdtI
    ldr x4, [L104]
.section .rodata {#1}
L102:
.byte 0x4B, 0xE0, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0xE0, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8F, 0x3E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0xCB, 0xE0, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8F, 0x38, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0xE1, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4F, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.section .text {#0}
L103:
    mov x26, x25
    mov x2, 1
    mov x3, 8
    bl L106
    mov x25, x0
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
    align 8
L104:
.xword L102
# label_L
@label_16-7:
label_16:
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L107
    mov x3, 1
    bl L66
L107:
# put_tuple2_SA
    mov x9, 128
    mov x10, 5387
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# line_I
# call_light_bif_be
L108:
    ldr x3, [L109]
    ldr x7, [L110]
    adr x2, L108
# BIF: erlang:error/1
    bl L71
# mark_unreachable
# i_flush_stubs
# i_func_label_L
label_17:
# func_line_I
# i_func_info_IaaI
# logger_proxy:init/1
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
init/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L111
    bl L61
L111:
# i_test_yield
    adr x2, init/1
    subs w22, w22, 1
    b.le L63
# is_nil_fS
    cmp x25, 59
    b.ne label_17
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L112
    mov x3, xzr
    bl L66
L112:
# i_move_sd
    mov x26, 75
# i_move_sd
    mov x25, 45515
# line_I
# call_light_bif_be
L113:
    ldr x3, [L114]
    ldr x7, [L115]
    adr x2, L113
# BIF: erlang:process_flag/2
    bl L71
# self_d
    ldr x26, [x21]
# i_move_sd
    mov x25, 450891
# line_I
# call_light_bif_be
L116:
    ldr x3, [L117]
    ldr x7, [L118]
    adr x2, L116
# BIF: erlang:system_flag/2
    bl L71
# line_I
# i_call_ext_e
    ldr x0, [L119]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 219019
# call_light_bif_be
L120:
    ldr x3, [L121]
    ldr x7, [L122]
    adr x2, L120
# BIF: persistent_term:put/2
    bl L71
# i_move_sd
    ldr x25, [L123]
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_19:
# func_line_I
# i_func_info_IaaI
# logger_proxy:handle_load/2
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0xE1, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@handle_load/2-2:
handle_load/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L124
    bl L61
L124:
# i_test_yield
    adr x2, handle_load/2
    subs w22, w22, 1
    b.le L63
# i_select_tuple_arity_SfI
    tbnz x25, 0, label_19
    ldur x8, [x25, -2]
    tst x8, 63
    b.ne label_19
# Linear search in [0..2], 3 elements
    cmp x8, 192
    b.eq @label_23-8
    cmp x8, 256
    b.eq @label_22-9
    cmp x8, 320
    b.eq @label_21-10
    b label_19
# label_L
@label_21-10:
label_21:
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x27, [x0, 8]
# is_eq_exact_fss
    mov x14, 56779
    cmp x27, x14
    b.ne label_19
# allocate_heap_tIt
    add x2, x23, 104
    cmp x2, x20
    b.ls L128
    mov x3, 2
    bl L66
L128:
    sub x20, x20, 8
# i_move_sd
    str x26, [x20]
# load_tuple_ptr_s
    and x0, x25, -8
# get_two_tuple_elements_sPSS
    ldp x26, x27, [x0, 16]
# get_two_tuple_elements_sPSS
    ldp x28, x25, [x0, 32]
# put_list_ssd
    mov x9, 59
    stp x25, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    stp x28, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    stp x27, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    stp x26, x25, [x23], 16
    sub x25, x23, 15
# line_I
# i_call_f
    bl @try_log/1-11
# move_deallocate_return
    ldp x25, x30, [x20], 16
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_22-9:
label_22:
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x27, [x0, 8]
# is_eq_exact_fss
    mov x14, 56779
    cmp x27, x14
    b.ne label_19
# allocate_heap_tIt
    add x2, x23, 88
    cmp x2, x20
    b.ls L130
    mov x3, 2
    bl L66
L130:
    sub x20, x20, 8
# i_move_sd
    str x26, [x20]
# load_tuple_ptr_s
    and x0, x25, -8
# get_two_tuple_elements_sPSS
    ldp x26, x27, [x0, 16]
# i_get_tuple_element_sPS
    ldr x25, [x0, 32]
# put_list_ssd
    mov x9, 59
    stp x25, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    stp x27, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    stp x26, x25, [x23], 16
    sub x25, x23, 15
# line_I
# i_call_f
    bl @try_log/1-11
# move_deallocate_return
    ldp x25, x30, [x20], 16
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_23-8:
label_23:
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x27, [x0, 8]
# is_eq_exact_fss
    mov x14, 171083
    cmp x27, x14
    b.ne label_19
# allocate_heap_tIt
    add x2, x23, 64
    cmp x2, x20
    b.ls L131
    mov x3, 2
    bl L66
L131:
    sub x20, x20, 8
# i_move_sd
    str x26, [x20]
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# put_tuple2_SA
    mov x9, 128
    mov x10, 219019
    stp x9, x10, [x23], 16
    str x26, [x23], 8
    sub x26, x23, 22
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x25, [x0, 24]
# i_move_sd
    ldr x27, [L132]
# swap_dd
    mov x8, x26
    mov x26, x25
    mov x25, x8
# line_I
# call_light_bif_be
L133:
    ldr x3, [L134]
    ldr x7, [L135]
    adr x2, L133
# BIF: erlang:send/3
    bl L71
# move_deallocate_return
    ldp x25, x30, [x20], 16
    subs w22, w22, 1
    b.mi L80
    ret x30
# i_flush_stubs
# i_func_label_L
label_24:
# func_line_I
# i_func_info_IaaI
# logger_proxy:handle_info/2
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x8F, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
handle_info/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L136
    bl L61
L136:
# i_test_yield
    adr x2, handle_info/2
    subs w22, w22, 1
    b.le L63
# i_is_tuple_fs
    tbnz x25, 0, @label_26-12
    and x0, x25, -8
    ldr x8, [x0]
    tst x8, 63
    b.ne @label_26-12
# bif_element_jssd
# simplified element/2 because arguments are known types
    ldur x9, [x25, -2]
    cmp x9, 64
    b.lo @label_26-12
L138:
    ldur x25, [x25, 6]
# is_eq_exact_fss
    mov x14, 56779
    cmp x25, x14
    b.ne @label_26-12
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L139
    mov x3, 2
    bl L66
L139:
# put_tuple2_SA
    mov x9, 128
    mov x10, 226251
    stp x9, x10, [x23], 16
    str x26, [x23], 8
    sub x25, x23, 22
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_26-12:
label_26:
# i_move_sd
    mov x25, x26
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# i_flush_stubs
# i_func_label_L
label_27:
# func_line_I
# i_func_info_IaaI
# logger_proxy:terminate/2
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x54, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
terminate/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L140
    bl L61
L140:
# i_test_yield
    adr x2, terminate/2
    subs w22, w22, 1
    b.le L63
# is_eq_exact_fss
    mov x14, 451083
    cmp x25, x14
    b.ne @label_29-13
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L142
    mov x3, xzr
    bl L66
L142:
# i_move_sd
    mov x26, 907
# i_move_sd
    mov x25, 450891
# line_I
# call_light_bif_be
L143:
    ldr x3, [L117]
    ldr x7, [L118]
    adr x2, L143
# BIF: erlang:system_flag/2
    bl L71
# i_move_sd
    ldr x25, [L144]
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_29-13:
label_29:
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L145
    mov x3, xzr
    bl L66
L145:
# i_move_sd
    mov x25, 25163
# line_I
# call_light_bif_be
L146:
    ldr x3, [L147]
    ldr x7, [L148]
    adr x2, L146
# BIF: erlang:whereis/1
    bl L71
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 450891
# call_light_bif_be
L149:
    ldr x3, [L117]
    ldr x7, [L118]
    adr x2, L149
# BIF: erlang:system_flag/2
    bl L71
# i_move_sd
    mov x25, 32139
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_30:
# func_line_I
# i_func_info_IaaI
# logger_proxy:notify/2
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
notify/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L150
    bl L61
L150:
# i_test_yield
    adr x2, notify/2
    subs w22, w22, 1
    b.le L63
# i_is_tuple_fs
    tbnz x25, 0, @label_40-14
    and x0, x25, -8
    ldr x8, [x0]
    tst x8, 63
    b.ne @label_40-14
# i_select_tuple_arity_SfI
# skipped box test since argument is always boxed
    ldur x8, [x25, -2]
# simplified tuple test since the source is always a tuple when boxed
# Linear search in [0..1], 2 elements
    cmp x8, 128
    b.eq @label_37-16
    cmp x8, 192
    b.eq @label_32-17
    b @label_43-15
# label_L
@label_32-17:
label_32:
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x27, [x0, 8]
# is_eq_exact_fss
    mov x14, 451147
    cmp x27, x14
    b.ne @label_43-15
# allocate_tt
    add x2, x23, 56
    cmp x2, x20
    b.ls L155
    mov x3, 2
    bl L66
L155:
    sub x20, x20, 24
# i_move_sd
    str x26, [x20, 16]
# load_tuple_ptr_s
    and x0, x25, -8
# get_two_tuple_elements_sPSS
    ldp x8, x9, [x0, 16]
    stp x9, x8, [x20]
# is_eq_exact_fss
# simplified fetching of BEAM register
    mov x0, x9
    mov x14, 451211
    cmp x0, x14
    b.ne @label_33-18
# i_move_sd
    mov x26, 907
# i_move_sd
    mov x25, 450891
# line_I
# call_light_bif_be
L157:
    ldr x3, [L117]
    ldr x7, [L118]
    adr x2, L157
# BIF: erlang:system_flag/2
    bl L71
# jump_f
    b @label_34-19
# label_L
@label_33-18:
label_33:
# is_eq_exact_fss
    ldr x0, [x20, 8]
    mov x14, 451211
    cmp x0, x14
    b.ne @label_34-20
# self_d
    ldr x26, [x21]
# i_move_sd
    mov x25, 450891
# line_I
# call_light_bif_be
L160:
    ldr x3, [L117]
    ldr x7, [L118]
    adr x2, L160
# BIF: erlang:system_flag/2
    bl L71
# label_L
@label_34-19:
@label_34-20:
label_34:
# i_move_sd
    mov x26, 219019
# i_move_sd
    mov x25, 225547
# line_I
# i_call_ext_e
    ldr x0, [L161]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_select_val_lins_sfI
    cmp x25, 11
    b.eq @label_36-21
    cmp x25, 75
    b.eq @label_35-22
    b L164
# label_L
@label_35-22:
label_35:
# test_heap_It
    add x2, x23, 112
    cmp x2, x20
    b.ls L165
    mov x3, xzr
    bl L66
L165:
# put_list2_sssd
    ldp x8, x9, [x20]
    mov x10, 59
    stp x8, x10, [x23], 16
    sub x25, x23, 15
    stp x9, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 219019
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x9, 59
    stp x25, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    ldr x8, [L166]
    stp x8, x25, [x23], 16
    sub x28, x23, 15
# i_move_sd
    ldr x26, [L167]
# i_move_sd
    ldr x27, [L168]
# init_yregs_I
    mov x8, 59
    stp x8, x8, [x20]
# i_move_sd
    mov x25, 225547
# i_call_ext_e
    ldr x0, [L169]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# label_L
@label_36-21:
label_36:
# i_move_sd
    ldr x25, [x20, 16]
# deallocate_t
    add x20, x20, 24
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_37-16:
label_37:
# load_tuple_ptr_s
    and x0, x25, -8
# get_two_tuple_elements_sPSS
    ldp x27, x25, [x0, 8]
# is_eq_exact_fss
    mov x14, 451275
    cmp x27, x14
    b.ne @label_43-15
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L170
    mov x3, 2
    bl L66
L170:
    sub x20, x20, 16
# store_two_values_sdsd
    stp x25, x26, [x20]
# i_move_sd
    mov x26, 219019
# i_move_sd
    mov x25, 225547
# line_I
# i_call_ext_e
    ldr x0, [L161]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_select_val_lins_sfI
    cmp x25, 11
    b.eq @label_39-23
    cmp x25, 75
    b.eq @label_38-24
    b L173
# label_L
@label_38-24:
label_38:
# test_heap_It
    add x2, x23, 96
    cmp x2, x20
    b.ls L174
    mov x3, xzr
    bl L66
L174:
# put_list_ssd
    ldr x8, [x20]
    mov x9, 59
    stp x8, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 219019
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x9, 59
    stp x25, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    ldr x8, [L175]
    stp x8, x25, [x23], 16
    sub x28, x23, 15
# i_move_sd
    ldr x26, [L176]
# i_move_sd
    ldr x27, [L168]
# i_move_sd
    mov x14, 59
    str x14, [x20]
# i_move_sd
    mov x25, 225547
# i_call_ext_e
    ldr x0, [L169]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# label_L
@label_39-23:
label_39:
# i_move_sd
    ldr x25, [x20, 8]
# deallocate_t
    add x20, x20, 16
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_40-14:
label_40:
# is_eq_exact_fss
    mov x14, 37835
    cmp x25, x14
    b.ne @label_43-15
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L177
    mov x3, 2
    bl L66
L177:
    sub x20, x20, 8
# i_move_sd
    str x26, [x20]
# i_move_sd
    mov x26, 219019
# i_move_sd
    mov x25, 225547
# line_I
# i_call_ext_e
    ldr x0, [L161]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_select_val_lins_sfI
    cmp x25, 11
    b.eq @label_42-25
    cmp x25, 75
    b.eq @label_41-26
    b L180
# label_L
@label_41-26:
label_41:
# i_move_sd
    ldr x27, [L168]
# i_move_sd
    ldr x26, [L181]
# i_move_sd
    ldr x28, [L182]
# i_move_sd
    mov x25, 225547
# i_call_ext_e
    ldr x0, [L169]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# label_L
@label_42-25:
label_42:
# move_deallocate_return
    ldp x25, x30, [x20], 16
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_43-15:
label_43:
# i_move_sd
    mov x25, x26
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
L164:
label_44:
# line_I
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L184
# label_L
L173:
label_45:
# line_I
    nop
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L184
# label_L
L180:
label_46:
# line_I
    nop
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L184
# i_flush_stubs
# i_func_label_L
    nop
label_47:
# func_line_I
# i_func_info_IaaI
# logger_proxy:try_log/1
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xE3, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@try_log/1-11:
try_log/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L185
    bl L61
L185:
# i_test_yield
    adr x2, try_log/1
    subs w22, w22, 1
    b.le L63
# allocate_tt
    add x2, x23, 64
    cmp x2, x20
    b.ls L186
    mov x3, 1
    bl L66
L186:
    sub x20, x20, 32
# init_yregs_I
    mov x8, 59
    stp x8, x8, [x20]
# i_move_sd
    str x25, [x20, 16]
# catch_yH
    ldr x8, [x21, 248]
    add x8, x8, 1
    str x8, [x21, 248]
    ldr x14, [L187]
    str x14, [x20, 24]
# i_move_sd
    mov x26, 56779
# i_move_sd
    mov x27, x25
# i_move_sd
    mov x25, 25163
# line_I
# i_apply
L189:
    stp x23, x20, [x21, 80]
    str w22, [x21, 112]
    stp x25, x26, [x19, 64]
    str x27, [x19, 80]
    mov x0, x21
    add x1, x19, 64
    mov x2, xzr
    mov x3, xzr
# apply()
    bl L191
    ldp x23, x20, [x21, 80]
    ldr w22, [x21, 112]
    ldp x25, x26, [x19, 64]
    ldp x27, x28, [x19, 80]
    ldp x15, x16, [x19, 96]
    cbnz x0, L188
    adr x1, L189
    ldr x3, [L192]
    b L194
L188:
    ldr x8, [x0, x24 lsl 3]
    blr x8
# try_end_deallocate_t
    ldr x8, [x21, 248]
    sub x8, x8, 1
    str x8, [x21, 248]
    add x20, x20, 32
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
label_49:
# try_case_y
    ldr x8, [x21, 248]
    mov x25, x28
    sub x8, x8, 1
    str x8, [x21, 248]
# store_two_values_sdsd
    stp x25, x26, [x20]
# i_move_sd
    str x27, [x20, 24]
# i_move_sd
    mov x26, 219019
# i_move_sd
    mov x25, 81867
# line_I
# i_call_ext_e
    ldr x0, [L161]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_select_val_lins_sfI
    cmp x25, 11
    b.eq @label_51-27
    cmp x25, 75
    b.eq @label_50-28
    b L197
# label_L
@label_50-28:
label_50:
# i_move_sd
    ldr x25, [x20, 24]
# build_stacktrace
    mov x1, x25
    stp x23, x20, [x21, 80]
    mov x0, x21
    bl L199
    ldp x23, x20, [x21, 80]
    mov x25, x0
# test_heap_It
    add x2, x23, 176
    cmp x2, x20
    b.ls L200
    mov x3, 1
    bl L66
L200:
# put_tuple2_SA
    mov x9, 192
    ldr x10, [x20]
    stp x9, x10, [x23], 16
    ldr x9, [x20, 8]
    stp x9, x25, [x23], 16
    sub x25, x23, 30
# put_tuple2_SA
    mov x9, 128
    mov x10, 36875
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# put_list_ssd
    mov x9, 59
    stp x25, x9, [x23], 16
    sub x25, x23, 15
# put_tuple2_SA
    mov x9, 128
    mov x10, 56779
    stp x9, x10, [x23], 16
    ldr x14, [x20, 16]
    str x14, [x23], 8
    sub x26, x23, 22
# put_list_ssd
    stp x26, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    ldr x8, [L201]
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x9, 59
    stp x25, x9, [x23], 16
    sub x28, x23, 15
# i_move_sd
    ldr x26, [L202]
# move_trim_sdt
    ldr x27, [L168]
    add x20, x20, 32
# i_move_sd
    mov x25, 81867
# line_I
# i_call_ext_e
    ldr x0, [L169]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x25, 32139
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
@label_51-27:
label_51:
# i_move_sd
    mov x25, 32139
# deallocate_t
    add x20, x20, 32
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# label_L
L197:
label_52:
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L184
# i_flush_stubs
# i_func_label_L
    nop
label_53:
# func_line_I
# i_func_info_IaaI
# logger_proxy:module_info/0
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L203
    bl L61
L203:
# i_test_yield
    adr x2, module_info/0
    subs w22, w22, 1
    b.le L63
# i_move_sd
    mov x25, 219019
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L204
    mov x3, 1
    bl L66
L204:
# call_light_bif_be
L205:
    ldr x3, [L206]
    ldr x7, [L207]
    adr x2, L205
# BIF: erlang:get_module_info/1
    bl L71
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_55:
# func_line_I
# i_func_info_IaaI
# logger_proxy:module_info/1
    bl L58
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x57, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L208
    bl L61
L208:
# i_test_yield
    adr x2, module_info/1
    subs w22, w22, 1
    b.le L63
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 219019
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L209
    mov x3, 2
    bl L66
L209:
# call_light_bif_be
L210:
    ldr x3, [L211]
    ldr x7, [L212]
    adr x2, L210
# BIF: erlang:get_module_info/2
    bl L71
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L80
    ret x30
# int_code_end
L213:
    mov x0, 4369093202
    bl L215
# Begin stub section
L68:
.xword 0x7FFFFFFFFFFFFFFF
L69:
.xword 0x0000000104430718
L73:
.xword 0x7FFFFFFFFFFFFFFF
L81:
.xword 0x7FFFFFFFFFFFFFFF
# End stub section
L216:
L215:
L214:
    mov x14, 4365818364
    br x14
L194:
L193:
    mov x14, 4481916936
    br x14
L199:
L198:
    mov x14, 4366179236
    br x14
L106:
L105:
    mov x14, 4481917344
    br x14
L80:
L79:
    mov x14, 4481911760
    br x14
L77:
L76:
    mov x14, 4366560408
    br x14
L58:
L57:
    mov x14, 4481913584
    br x14
L71:
L70:
    mov x14, 4481910672
    br x14
L66:
L65:
    mov x14, 4481912640
    br x14
L184:
L183:
    mov x14, 4481916920
    br x14
L63:
L62:
    mov x14, 4481914968
    br x14
L191:
L190:
    mov x14, 4366180552
    br x14
L61:
L60:
    mov x14, 4481913368
    br x14
# Begin stub section
L84:
.xword 0x7FFFFFFFFFFFFFFF
L85:
.xword 0x7FFFFFFFFFFFFFFF
L89:
.xword 0x7FFFFFFFFFFFFFFF
L97:
.xword 0x7FFFFFFFFFFFFFFF
L100:
.xword 0x7FFFFFFFFFFFFFFF
L109:
.xword 0x7FFFFFFFFFFFFFFF
L110:
.xword 0x000000010444DA38
L114:
.xword 0x7FFFFFFFFFFFFFFF
L115:
.xword 0x000000010444E650
L117:
.xword 0x7FFFFFFFFFFFFFFF
L118:
.xword 0x0000000104453FD0
L119:
.xword 0x7FFFFFFFFFFFFFFF
L121:
.xword 0x7FFFFFFFFFFFFFFF
L122:
.xword 0x000000010442F304
L123:
.xword 0x7FFFFFFFFFFFFFFF
L132:
.xword 0x7FFFFFFFFFFFFFFF
L134:
.xword 0x7FFFFFFFFFFFFFFF
L135:
.xword 0x000000010444F41C
L144:
.xword 0x7FFFFFFFFFFFFFFF
L147:
.xword 0x7FFFFFFFFFFFFFFF
L148:
.xword 0x000000010444F0FC
L161:
.xword 0x7FFFFFFFFFFFFFFF
L166:
.xword 0x7FFFFFFFFFFFFFFF
L167:
.xword 0x7FFFFFFFFFFFFFFF
L168:
.xword 0x7FFFFFFFFFFFFFFF
L169:
.xword 0x7FFFFFFFFFFFFFFF
L175:
.xword 0x7FFFFFFFFFFFFFFF
L176:
.xword 0x7FFFFFFFFFFFFFFF
L181:
.xword 0x7FFFFFFFFFFFFFFF
L182:
.xword 0x7FFFFFFFFFFFFFFF
L187:
.xword 0x000000007FFFFFFF
L192:
.xword 0x000000010476C578
L201:
.xword 0x7FFFFFFFFFFFFFFF
L202:
.xword 0x7FFFFFFFFFFFFFFF
L206:
.xword 0x7FFFFFFFFFFFFFFF
L207:
.xword 0x000000010442AAD0
L211:
.xword 0x7FFFFFFFFFFFFFFF
L212:
.xword 0x000000010442AD84
# End stub section
L217:
.section .rodata {#1}
line:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.section .text {#0}
.section .rodata {#1}
attr:
.byte 0x83, 0x6C, 0x00, 0x00, 0x00, 0x01, 0x68, 0x02, 0x77, 0x03, 0x76, 0x73, 0x6E, 0x6C, 0x00, 0x00, 0x00, 0x01, 0x6E, 0x10, 0x00, 0xA4, 0x67, 0xFB, 0xF1, 0x82, 0x5B, 0x36, 0xDC, 0x8C, 0x0D, 0x58, 0xB6, 0x66, 0xC7, 0xDD, 0xC5, 0x6A, 0x6A
.section .text {#0}
.section .rodata {#1}
compile:
.byte 0x83, 0x6C, 0x00, 0x00, 0x00, 0x03, 0x68, 0x02, 0x77, 0x07, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x6B, 0x00, 0x05, 0x38, 0x2E, 0x36, 0x2E, 0x31, 0x68, 0x02, 0x77, 0x07, 0x6F, 0x70, 0x74, 0x69, 0x6F, 0x6E, 0x73, 0x6C, 0x00, 0x00, 0x00, 0x06, 0x77, 0x0A, 0x64, 0x65, 0x62, 0x75, 0x67, 0x5F, 0x69, 0x6E, 0x66, 0x6F, 0x68, 0x02, 0x77, 0x01, 0x69, 0x6B, 0x00, 0x28, 0x2F, 0x62, 0x75, 0x69, 0x6C, 0x64, 0x72, 0x6F, 0x6F, 0x74, 0x2F, 0x6F, 0x74, 0x70, 0x2F, 0x6C, 0x69, 0x62, 0x2F, 0x6B, 0x65, 0x72, 0x6E, 0x65, 0x6C, 0x2F, 0x73, 0x72, 0x63, 0x2F, 0x2E, 0x2E, 0x2F, 0x69, 0x6E, 0x63, 0x6C, 0x75, 0x64, 0x65, 0x77, 0x19, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x64, 0x6F, 0x63, 0x5F, 0x66, 0x75, 0x6E, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x77, 0x19, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x64, 0x6F, 0x63, 0x5F, 0x63, 0x61, 0x6C, 0x6C, 0x62, 0x61, 0x63, 0x6B, 0x77, 0x1C, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x73, 0x70, 0x65, 0x63, 0x5F, 0x64, 0x6F, 0x63, 0x75, 0x6D, 0x65, 0x6E, 0x74, 0x65, 0x64, 0x77, 0x15, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x64, 0x65, 0x70, 0x72, 0x65, 0x63, 0x61, 0x74, 0x65, 0x64, 0x5F, 0x63, 0x61, 0x74, 0x63, 0x68, 0x6A, 0x68, 0x02, 0x77, 0x06, 0x73, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x6B, 0x00, 0x2E, 0x2F, 0x62, 0x75, 0x69, 0x6C, 0x64, 0x72, 0x6F, 0x6F, 0x74, 0x2F, 0x6F, 0x74, 0x70, 0x2F, 0x6C, 0x69, 0x62, 0x2F, 0x6B, 0x65, 0x72, 0x6E, 0x65, 0x6C, 0x2F, 0x73, 0x72, 0x63, 0x2F, 0x6C, 0x6F, 0x67, 0x67, 0x65, 0x72, 0x5F, 0x70, 0x72, 0x6F, 0x78, 0x79, 0x2E, 0x65, 0x72, 0x6C, 0x6A
.section .text {#0}
.section .rodata {#1}
md5:
.byte 0xC5, 0xDD, 0xC7, 0x66, 0xB6, 0x58, 0x0D, 0x8C, 0xDC, 0x36, 0x5B, 0x82, 0xF1, 0xFB, 0x67, 0xA4
.section .text {#0}
