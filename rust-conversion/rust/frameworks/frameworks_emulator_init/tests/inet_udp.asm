L63:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# i_flush_stubs
# i_func_label_L
label_1:
# func_line_I
# i_func_info_IaaI
# inet_udp:getserv/1
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0xDB, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
getserv/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L66
    bl L68
L66:
# i_test_yield
    adr x2, getserv/1
    subs w22, w22, 1
    b.le L70
# is_integer_fs
    and x9, x25, 15
    cmp x9, 15
    b.eq L71
    tbnz x9, 0, @label_3-0
    ldur x8, [x25, -2]
    and x8, x8, 56
    cmp x8, 8
    b.ne @label_3-0
L71:
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L73
    mov x3, 1
    bl L75
L73:
# put_tuple2_SA
    mov x9, 128
    mov x10, 32139
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L77
    ret x30
# label_L
@label_3-0:
label_3:
# is_atom_fs
    and x8, x25, 63
    cmp x8, 11
    b.ne label_1
# i_move_sd
    mov x26, 73419
# line_I
# i_call_ext_only_e
    ldr x0, [L78]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
    align 8
label_4:
# func_line_I
# i_func_info_IaaI
# inet_udp:getaddr/1
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xDC, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
getaddr/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L79
    bl L68
L79:
# i_test_yield
    adr x2, getaddr/1
    subs w22, w22, 1
    b.le L70
# i_move_sd
    mov x26, 73227
# i_call_ext_only_e
    ldr x0, [L80]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_6:
# func_line_I
# i_func_info_IaaI
# inet_udp:getaddr/2
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xDC, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
getaddr/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L81
    bl L68
L81:
# i_test_yield
    adr x2, getaddr/2
    subs w22, w22, 1
    b.le L70
# i_move_sd
    mov x27, x26
# i_move_sd
    mov x26, 73227
# i_call_ext_only_e
    ldr x0, [L82]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
    align 8
label_8:
# func_line_I
# i_func_info_IaaI
# inet_udp:translate_ip/1
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0xDC, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
translate_ip/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L83
    bl L68
L83:
# i_test_yield
    adr x2, translate_ip/1
    subs w22, w22, 1
    b.le L70
# i_move_sd
    mov x26, 73227
# i_call_ext_only_e
    ldr x0, [L84]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_10:
# func_line_I
# i_func_info_IaaI
# inet_udp:open/1
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
open/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L85
    bl L68
L85:
# i_test_yield
    adr x2, open/1
    subs w22, w22, 1
    b.le L70
# i_move_sd
    mov x26, 59
# i_call_only_f
    ldr x30, [x20], 8
    b @open/2-1
# i_flush_stubs
# i_func_label_L
    align 8
label_12:
# func_line_I
# i_func_info_IaaI
# inet_udp:open/2
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@open/2-1:
open/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L87
    bl L68
L87:
# i_test_yield
    adr x2, open/2
    subs w22, w22, 1
    b.le L70
# allocate_heap_tIt
    add x2, x23, 72
    cmp x2, x20
    b.ls L88
    mov x3, 2
    bl L75
L88:
# put_tuple2_SA
    mov x9, 128
    mov x10, 34379
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# put_list_ssd
    stp x25, x26, [x23], 16
    sub x25, x23, 15
# i_move_sd
    mov x26, 218379
# line_I
# i_call_ext_e
    ldr x0, [L89]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_is_tuple_of_arity_fsA
    tbnz x25, 0, @label_25-2
    and x0, x25, -8
    ldr x8, [x0]
    cmp x8, 128
    b.ne @label_25-2
# get_two_tuple_elements_sPSS
    ldp x26, x27, [x0, 8]
# i_select_val_lins_sfI
    cmp x26, 779
    b.eq @label_24-3
    mov x14, 32139
    cmp x26, x14
    b.eq @label_14-4
    b @label_25-2
# label_L
@label_14-4:
label_14:
# i_is_tagged_tuple_fsAa
    tbnz x27, 0, @label_23-5
    and x0, x27, -8
    ldp x8, x9, [x0]
    mov x14, 515211
    cmp x9, x14
    mov x10, 320
    ccmp x8, x10, 0, 2
    b.ne @label_23-5
# get_two_tuple_elements_sPSS
    ldp x25, x26, [x0, 16]
# is_map_fs
    tbnz x25, 0, @label_15-6
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_15-6
# jump_f
    b @label_22-7
# label_L
@label_15-6:
label_15:
# i_band_jIssd
    mov x2, -1048561
    and x0, x26, x2
    and x8, x0, 15
    cmp x8, 15
    b.eq L96
    mov x1, x26
    mov x0, x21
    bl L98
    cbz x0, @label_18-8
L96:
    mov x28, x0
# bif_is_eq_exact_Ssd
    cmp x28, 15
    mov x10, 75
    mov x11, 11
    csel x28, x10, x11, 2
# i_is_tuple_of_arity_ff_ffsA
    tbnz x25, 0, @label_18-8
    and x0, x25, -8
    ldr x8, [x0]
    tst x8, 63
    b.ne @label_18-8
    cmp x8, 256
    b.ne @label_16-9
# get_two_tuple_elements_sPSS
    ldp x15, x16, [x0, 8]
# i_bor_jIssd
    orr x0, x15, x16
    and x8, x15, x16
    and x8, x8, 15
    cmp x8, 15
    b.eq L101
    mov x1, x15
    mov x2, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, @label_18-8
L101:
    mov x15, x0
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x16, [x0, 24]
# i_bor_jIssd
    orr x0, x15, x16
    and x8, x15, x16
    and x8, x8, 15
    cmp x8, 15
    b.eq L104
    mov x1, x15
    mov x2, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, @label_18-8
L104:
    mov x15, x0
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x16, [x0, 32]
# i_bor_jIssd
    orr x0, x15, x16
    and x8, x15, x16
    and x8, x8, 15
    cmp x8, 15
    b.eq L105
    mov x1, x15
    mov x2, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, @label_18-8
L105:
    mov x15, x0
# line_I
# i_band_jIssd
    mov x2, -4081
    and x0, x15, x2
# simplified test for small operands since other types are boxed
    tbnz x0, 0, L106
    mov x1, x15
    str x15, [x19, 96]
    bl L108
    ldr x15, [x19, 96]
L106:
    mov x15, x0
# bif_is_eq_exact_Ssd
    cmp x15, 15
    mov x10, 75
    mov x11, 11
    csel x15, x10, x11, 2
# jump_f
    b @label_17-10
# label_L
@label_16-9:
label_16:
# i_move_sd
    mov x15, 11
# label_L
@label_17-10:
label_17:
# bif_and_jssd
# simplified type check because operands are atoms
    orr x10, x28, x15
    tst x10, -128
    b.eq L110
    mov x25, x28
    mov x26, x15
    bl L112
L110:
    and x28, x28, x15
# jump_f
    b @label_19-11
# label_L
@label_18-8:
label_18:
# i_move_sd
    mov x28, 11
# label_L
@label_19-11:
label_19:
# i_band_jIssd
    mov x2, -1048561
    and x0, x26, x2
    and x8, x0, 15
    cmp x8, 15
    b.eq L114
    mov x1, x26
    mov x0, x21
    bl L98
    cbz x0, @label_20-12
L114:
    mov x15, x0
# bif_is_eq_exact_Ssd
    cmp x15, 15
    mov x10, 75
    mov x11, 11
    csel x15, x10, x11, 2
# bif_is_eq_exact_Ssd
    cmp x25, 907
    mov x10, 75
    mov x11, 11
    csel x16, x10, x11, 2
# bif_and_jssd
# simplified type check because operands are atoms
    orr x10, x15, x16
    tst x10, -128
    b.eq L116
    mov x25, x15
    mov x26, x16
    bl L112
L116:
    and x15, x15, x16
# jump_f
    b @label_21-13
# label_L
@label_20-12:
label_20:
# i_move_sd
    mov x15, 11
# label_L
@label_21-13:
label_21:
# is_eq_exact_fss
    cmp x28, 11
    b.ne @label_22-14
# is_eq_exact_fss
    cmp x15, 75
    b.ne @label_23-5
# label_L
@label_22-7:
@label_22-14:
label_22:
# load_tuple_ptr_s
    and x0, x27, -8
# get_two_tuple_elements_sPSS
    ldp x28, x27, [x0, 32]
# i_move_sd
    mov x14, 97995
    str x14, [x19, 112]
# i_move_sd
    mov x16, 73227
# i_move_sd
    mov x14, 218379
    str x14, [x19, 120]
# i_move_sd
    mov x15, 73419
# swap3_dddd
    mov x12, x28
    mov x28, x27
    mov x27, x26
    mov x26, x25
    mov x25, x12
# line_I
# i_call_ext_last_et
    ldr x0, [L119]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# label_L
@label_23-5:
label_23:
# i_move_sd
    mov x25, 5003
# line_I
# call_light_bif_be
L120:
    ldr x3, [L121]
    ldr x7, [L122]
    adr x2, L120
# BIF: erlang:exit/1
    bl L124
# mark_unreachable
# label_L
@label_24-3:
label_24:
# i_move_sd
    mov x25, x27
# line_I
# call_light_bif_be
L125:
    ldr x3, [L121]
    ldr x7, [L122]
    adr x2, L125
# BIF: erlang:exit/1
    bl L124
# mark_unreachable
# label_L
@label_25-2:
label_25:
# line_I
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L127
# i_flush_stubs
# i_func_label_L
    nop
    align 8
label_26:
# func_line_I
# i_func_info_IaaI
# inet_udp:send/4
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x9B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
send/4:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L128
    bl L68
L128:
# i_test_yield
    adr x2, send/4
    subs w22, w22, 1
    b.le L70
# i_is_tuple_fs
    tbnz x26, 0, @label_32-15
    and x0, x26, -8
    ldr x8, [x0]
    tst x8, 63
    b.ne @label_32-15
# i_select_tuple_arity_SfI
# skipped box test since argument is always boxed
    ldur x8, [x26, -2]
# simplified tuple test since the source is always a tuple when boxed
# Linear search in [0..1], 2 elements
    cmp x8, 128
    b.eq @label_29-16
    cmp x8, 256
    b.eq @label_28-17
    b label_26
# label_L
@label_28-17:
label_28:
# load_tuple_ptr_s
    and x0, x26, -8
# get_two_tuple_elements_sPSS
    ldp x15, x16, [x0, 8]
# i_bor_jIssd
    orr x0, x15, x16
    and x8, x15, x16
    and x8, x8, 15
    cmp x8, 15
    b.eq L132
    mov x1, x15
    mov x2, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L132:
    mov x15, x0
# load_tuple_ptr_s
    and x0, x26, -8
# i_get_tuple_element_sPS
    ldr x16, [x0, 24]
# i_bor_jIssd
    orr x0, x15, x16
    and x8, x15, x16
    and x8, x8, 15
    cmp x8, 15
    b.eq L133
    mov x1, x15
    mov x2, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L133:
    mov x15, x0
# load_tuple_ptr_s
    and x0, x26, -8
# i_get_tuple_element_sPS
    ldr x16, [x0, 32]
# i_bor_jIssd
    orr x0, x15, x16
    and x8, x15, x16
    and x8, x8, 15
    cmp x8, 15
    b.eq L134
    mov x1, x15
    mov x2, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L134:
    mov x15, x0
# line_I
# i_band_jIssd
    mov x2, -4081
    and x0, x15, x2
# simplified test for small operands since other types are boxed
    tbnz x0, 0, L135
    mov x1, x15
    str x15, [x19, 96]
    bl L108
    ldr x15, [x19, 96]
L135:
    mov x15, x0
# is_eq_exact_fss
    cmp x15, 15
    b.ne label_26
# i_band_jIssd
    mov x2, -1048561
    and x0, x27, x2
    and x8, x0, 15
    cmp x8, 15
    b.eq L136
    mov x1, x27
    mov x0, x21
    bl L98
    cbz x0, label_26
L136:
    mov x15, x0
# is_eq_exact_fss
    cmp x15, 15
    b.ne label_26
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L137
    mov x3, 4
    bl L75
L137:
# put_tuple2_SA
    mov x9, 128
    stp x9, x26, [x23], 16
    str x27, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x27, 59
# line_I
# i_call_ext_only_e
    ldr x0, [L138]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# label_L
@label_29-16:
label_29:
# load_tuple_ptr_s
    and x0, x26, -8
# get_two_tuple_elements_sPSS
    ldp x15, x16, [x0, 8]
# i_is_tuple_of_arity_ff_ffsA
    tbnz x15, 0, @label_30-18
    and x0, x15, -8
    ldr x8, [x0]
    tst x8, 63
    b.ne @label_30-18
    cmp x8, 256
    b.ne label_26
# get_two_tuple_elements_sPSS
    ldp x8, x9, [x0, 8]
    stp x8, x9, [x19, 112]
# i_bor_jIssd
    ldp x1, x2, [x19, 112]
    orr x0, x1, x2
    and x8, x1, x2
    and x8, x8, 15
    cmp x8, 15
    b.eq L140
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L140:
    str x0, [x19, 112]
# load_tuple_ptr_s
    and x0, x15, -8
# i_get_tuple_element_sPS
    ldr x8, [x0, 24]
    str x8, [x19, 120]
# i_bor_jIssd
    ldp x1, x2, [x19, 112]
    orr x0, x1, x2
    and x8, x1, x2
    and x8, x8, 15
    cmp x8, 15
    b.eq L141
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L141:
    str x0, [x19, 112]
# load_tuple_ptr_s
    and x0, x15, -8
# i_get_tuple_element_sPS
    ldr x15, [x0, 32]
# i_bor_jIssd
    ldr x1, [x19, 112]
    orr x0, x1, x15
    and x8, x1, x15
    and x8, x8, 15
    cmp x8, 15
    b.eq L142
    mov x2, x15
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L142:
    mov x15, x0
# line_I
# i_band_jIssd
    mov x2, -4081
    and x0, x15, x2
# simplified test for small operands since other types are boxed
    tbnz x0, 0, L143
    mov x1, x15
    stp x15, x16, [x19, 96]
    bl L108
    ldp x15, x16, [x19, 96]
L143:
    mov x15, x0
# is_eq_exact_fss
    cmp x15, 15
    b.ne label_26
# i_band_jIssd
    mov x2, -1048561
    and x0, x16, x2
    and x8, x0, 15
    cmp x8, 15
    b.eq L144
    mov x1, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L98
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L144:
    mov x15, x0
# is_eq_exact_fss
    cmp x15, 15
    b.ne label_26
# is_list_fs
    tst x27, 2
    mov x14, 59
    ccmp x27, x14, 4, 3
    b.ne label_26
# line_I
# i_call_ext_only_e
    ldr x0, [L138]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# label_L
@label_30-18:
label_30:
# is_eq_exact_fss
    mov x14, 73227
    cmp x15, x14
    b.ne label_26
# i_is_tuple_of_arity_fsA
    tbnz x16, 0, label_26
    and x0, x16, -8
    ldr x8, [x0]
    cmp x8, 128
    b.ne label_26
# get_two_tuple_elements_sPSS
    ldp x15, x16, [x0, 8]
# i_is_tuple_of_arity_ff_ffsA
    tbnz x15, 0, @label_31-19
    and x0, x15, -8
    ldr x8, [x0]
    tst x8, 63
    b.ne @label_31-19
    cmp x8, 256
    b.ne label_26
# get_two_tuple_elements_sPSS
    ldp x8, x9, [x0, 8]
    stp x8, x9, [x19, 112]
# i_bor_jIssd
    ldp x1, x2, [x19, 112]
    orr x0, x1, x2
    and x8, x1, x2
    and x8, x8, 15
    cmp x8, 15
    b.eq L146
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L146:
    str x0, [x19, 112]
# load_tuple_ptr_s
    and x0, x15, -8
# i_get_tuple_element_sPS
    ldr x8, [x0, 24]
    str x8, [x19, 120]
# i_bor_jIssd
    ldp x1, x2, [x19, 112]
    orr x0, x1, x2
    and x8, x1, x2
    and x8, x8, 15
    cmp x8, 15
    b.eq L147
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L147:
    str x0, [x19, 112]
# load_tuple_ptr_s
    and x0, x15, -8
# i_get_tuple_element_sPS
    ldr x15, [x0, 32]
# i_bor_jIssd
    ldr x1, [x19, 112]
    orr x0, x1, x15
    and x8, x1, x15
    and x8, x8, 15
    cmp x8, 15
    b.eq L148
    mov x2, x15
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L148:
    mov x15, x0
# line_I
# i_band_jIssd
    mov x2, -4081
    and x0, x15, x2
# simplified test for small operands since other types are boxed
    tbnz x0, 0, L149
    mov x1, x15
    stp x15, x16, [x19, 96]
    bl L108
    ldp x15, x16, [x19, 96]
L149:
    mov x15, x0
# is_eq_exact_fss
    cmp x15, 15
    b.ne label_26
# i_band_jIssd
    mov x2, -1048561
    and x0, x16, x2
    and x8, x0, 15
    cmp x8, 15
    b.eq L150
    mov x1, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L98
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L150:
    mov x15, x0
# is_eq_exact_fss
    cmp x15, 15
    b.ne label_26
# is_list_fs
    tst x27, 2
    mov x14, 59
    ccmp x27, x14, 4, 3
    b.ne label_26
# line_I
# i_call_ext_only_e
    ldr x0, [L138]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# label_L
@label_31-19:
label_31:
# is_eq_exact_fss
    mov x14, 108683
    cmp x15, x14
    b.ne label_26
# i_band_jIssd
    mov x2, -1048561
    and x0, x16, x2
    and x8, x0, 15
    cmp x8, 15
    b.eq L151
    mov x1, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L98
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L151:
    mov x15, x0
# is_eq_exact_fss
    cmp x15, 15
    b.ne label_26
# is_list_fs
    tst x27, 2
    mov x14, 59
    ccmp x27, x14, 4, 3
    b.ne label_26
# line_I
# i_call_ext_only_e
    ldr x0, [L138]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# label_L
@label_32-15:
label_32:
# is_map_fs
    tbnz x26, 0, label_26
    ldur x10, [x26, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne label_26
# i_get_map_elements_fsI
    mov x0, x26
# simplified multi-element lookup
    and x8, x0, -8
    ldp x9, x10, [x8]
    and x9, x9, 252
    cmp x9, 44
    b.ne L152
    add x10, x10, 1
    ldr x9, [x8, 16]!
    and x9, x9, -8
L154:
    subs x10, x10, 1
    b.eq label_26
    ldr x11, [x9, x10 lsl 3]
    mov x14, 98827
    cmp x11, x14
    b.ne L154
    ldr x15, [x8, x10 lsl 3]
L155:
    subs x10, x10, 1
    b.eq label_26
    ldr x11, [x9, x10 lsl 3]
    mov x14, 34379
    cmp x11, x14
    b.ne L155
    ldr x16, [x8, x10 lsl 3]
    b L153
L152:
    adr x4, L156
    b L157
L156:
.byte 0x4B, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x53, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x7D, 0x1F, 0xB5, 0x47, 0x3D, 0x27, 0xFD, 0xFC
.byte 0x0B, 0x82, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x04, 0xA2, 0x18, 0x11, 0xFD, 0xCD, 0x56, 0x86
L157:
    mov x2, x20
    stp x25, x26, [x19, 64]
    stp x27, x28, [x19, 80]
    stp x15, x16, [x19, 96]
    mov x3, 2
    add x1, x19, 64
    bl L159
    ldp x25, x26, [x19, 64]
    ldp x27, x28, [x19, 80]
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L153:
# i_is_tuple_of_arity_fsA
    tbnz x15, 0, label_26
    and x0, x15, -8
    ldr x8, [x0]
    cmp x8, 256
    b.ne label_26
# get_two_tuple_elements_sPSS
    ldp x8, x9, [x0, 8]
    stp x8, x9, [x19, 112]
# i_bor_jIssd
    ldp x1, x2, [x19, 112]
    orr x0, x1, x2
    and x8, x1, x2
    and x8, x8, 15
    cmp x8, 15
    b.eq L160
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L160:
    str x0, [x19, 112]
# load_tuple_ptr_s
    and x0, x15, -8
# i_get_tuple_element_sPS
    ldr x8, [x0, 24]
    str x8, [x19, 120]
# i_bor_jIssd
    ldp x1, x2, [x19, 112]
    orr x0, x1, x2
    and x8, x1, x2
    and x8, x8, 15
    cmp x8, 15
    b.eq L161
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L161:
    str x0, [x19, 112]
# load_tuple_ptr_s
    and x0, x15, -8
# i_get_tuple_element_sPS
    ldr x15, [x0, 32]
# i_bor_jIssd
    ldr x1, [x19, 112]
    orr x0, x1, x15
    and x8, x1, x15
    and x8, x8, 15
    cmp x8, 15
    b.eq L162
    mov x2, x15
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L103
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L162:
    mov x15, x0
# line_I
# i_band_jIssd
    mov x2, -4081
    and x0, x15, x2
# simplified test for small operands since other types are boxed
    tbnz x0, 0, L163
    mov x1, x15
    stp x15, x16, [x19, 96]
    bl L108
    ldp x15, x16, [x19, 96]
L163:
    mov x15, x0
# is_eq_exact_fss
    cmp x15, 15
    b.ne label_26
# i_band_jIssd
    mov x2, -1048561
    and x0, x16, x2
    and x8, x0, 15
    cmp x8, 15
    b.eq L164
    mov x1, x16
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L98
    ldp x15, x16, [x19, 96]
    cbz x0, label_26
L164:
    mov x15, x0
# is_eq_exact_fss
    cmp x15, 15
    b.ne label_26
# is_list_fs
    tst x27, 2
    mov x14, 59
    ccmp x27, x14, 4, 3
    b.ne label_26
# line_I
# i_call_ext_only_e
    ldr x0, [L138]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_33:
# func_line_I
# i_func_info_IaaI
# inet_udp:send/2
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x9B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
send/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L165
    bl L68
L165:
# i_test_yield
    adr x2, send/2
    subs w22, w22, 1
    b.le L70
# i_move_sd
    mov x27, 59
# i_move_sd
    mov x28, x26
# i_move_sd
    ldr x26, [L166]
# line_I
# i_call_ext_only_e
    ldr x0, [L138]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
    align 8
label_35:
# func_line_I
# i_func_info_IaaI
# inet_udp:connect/2
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
connect/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L167
    bl L68
L167:
# i_test_yield
    adr x2, connect/2
    subs w22, w22, 1
    b.le L70
# is_map_fs
    tbnz x26, 0, label_35
    ldur x10, [x26, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne label_35
# i_get_map_element_hash_fScWS
    mov x0, x26
    mov x1, 108747
    ldr x2, [L168]
    bl L170
    b.ne label_35
    mov x27, x0
# is_eq_exact_fss
    mov x14, 73227
    cmp x27, x14
    b.ne label_35
# i_move_sd
    mov x27, 395
# line_I
# i_call_ext_only_e
    ldr x0, [L171]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
    align 8
label_37:
# func_line_I
# i_func_info_IaaI
# inet_udp:connect/3
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
connect/3:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L172
    bl L68
L172:
# i_test_yield
    adr x2, connect/3
    subs w22, w22, 1
    b.le L70
# i_is_tuple_of_arity_fsA
    tbnz x26, 0, label_37
    and x0, x26, -8
    ldr x8, [x0]
    cmp x8, 256
    b.ne label_37
# get_two_tuple_elements_sPSS
    ldp x28, x15, [x0, 8]
# i_bor_jIssd
    orr x0, x28, x15
    and x8, x28, x15
    and x8, x8, 15
    cmp x8, 15
    b.eq L173
    mov x1, x28
    mov x2, x15
    str x15, [x19, 96]
    mov x0, x21
    bl L103
    ldr x15, [x19, 96]
    cbz x0, label_37
L173:
    mov x28, x0
# load_tuple_ptr_s
    and x0, x26, -8
# i_get_tuple_element_sPS
    ldr x15, [x0, 24]
# i_bor_jIssd
    orr x0, x28, x15
    and x8, x28, x15
    and x8, x8, 15
    cmp x8, 15
    b.eq L174
    mov x1, x28
    mov x2, x15
    str x15, [x19, 96]
    mov x0, x21
    bl L103
    ldr x15, [x19, 96]
    cbz x0, label_37
L174:
    mov x28, x0
# load_tuple_ptr_s
    and x0, x26, -8
# i_get_tuple_element_sPS
    ldr x15, [x0, 32]
# i_bor_jIssd
    orr x0, x28, x15
    and x8, x28, x15
    and x8, x8, 15
    cmp x8, 15
    b.eq L175
    mov x1, x28
    mov x2, x15
    str x15, [x19, 96]
    mov x0, x21
    bl L103
    ldr x15, [x19, 96]
    cbz x0, label_37
L175:
    mov x28, x0
# line_I
# i_band_jIssd
    mov x2, -4081
    and x0, x28, x2
# simplified test for small operands since other types are boxed
    tbnz x0, 0, L176
    mov x1, x28
    bl L108
L176:
    mov x28, x0
# is_eq_exact_fss
    cmp x28, 15
    b.ne label_37
# i_band_jIssd
    mov x2, -1048561
    and x0, x27, x2
    and x8, x0, 15
    cmp x8, 15
    b.eq L177
    mov x1, x27
    mov x0, x21
    bl L98
    cbz x0, label_37
L177:
    mov x28, x0
# is_eq_exact_fss
    cmp x28, 15
    b.ne label_37
# line_I
# i_call_ext_only_e
    ldr x0, [L171]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
    align 8
label_39:
# func_line_I
# i_func_info_IaaI
# inet_udp:recv/2
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x88, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
recv/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L178
    bl L68
L178:
# line_I
# i_test_yield
    adr x2, recv/2
    subs w22, w22, 1
    b.le L70
# i_call_ext_only_e
    ldr x0, [L179]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_41:
# func_line_I
# i_func_info_IaaI
# inet_udp:recv/3
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x88, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
recv/3:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L180
    bl L68
L180:
# line_I
# i_test_yield
    adr x2, recv/3
    subs w22, w22, 1
    b.le L70
# i_call_ext_only_e
    ldr x0, [L181]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_43:
# func_line_I
# i_func_info_IaaI
# inet_udp:close/1
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
close/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L182
    bl L68
L182:
# line_I
# i_test_yield
    adr x2, close/1
    subs w22, w22, 1
    b.le L70
# i_call_ext_only_e
    ldr x0, [L183]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_45:
# func_line_I
# i_func_info_IaaI
# inet_udp:controlling_process/2
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x10, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
controlling_process/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L184
    bl L68
L184:
# line_I
# i_test_yield
    adr x2, controlling_process/2
    subs w22, w22, 1
    b.le L70
# i_call_ext_only_e
    ldr x0, [L185]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_47:
# func_line_I
# i_func_info_IaaI
# inet_udp:fdopen/2
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x7D, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
fdopen/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L186
    bl L68
L186:
# i_test_yield
    adr x2, fdopen/2
    subs w22, w22, 1
    b.le L70
# allocate_heap_tIt
    add x2, x23, 56
    cmp x2, x20
    b.ls L187
    mov x3, 2
    bl L75
L187:
    sub x20, x20, 8
# i_move_sd
    str x25, [x20]
# put_list_ssd
    ldr x8, [L188]
    stp x8, x26, [x23], 16
    sub x25, x23, 15
# line_I
# i_call_f
    bl @optuniquify/1-20
# i_move_sd
    mov x28, 73227
# i_move_sd
    mov x27, 73419
# i_move_sd
    mov x15, 97995
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x16, 218379
# move_call_ext_last_ydet
    ldr x0, [L190]
    ldp x25, x30, [x20], 16
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_49:
# func_line_I
# i_func_info_IaaI
# inet_udp:optuniquify/1
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0xDD, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@optuniquify/1-20:
optuniquify/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L191
    bl L68
L191:
# i_test_yield
    adr x2, optuniquify/1
    subs w22, w22, 1
    b.le L70
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L192
    mov x3, 1
    bl L75
L192:
# line_I
# i_call_ext_e
    ldr x0, [L193]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x26, 59
# i_call_last_ft
    ldr x30, [x20], 8
    b @optuniquify/2-21
# i_flush_stubs
# i_func_label_L
    align 8
label_51:
# func_line_I
# i_func_info_IaaI
# inet_udp:optuniquify/2
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0xDD, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@optuniquify/2-21:
optuniquify/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L195
    bl L68
L195:
# i_test_yield
    adr x2, optuniquify/2
    subs w22, w22, 1
    b.le L70
# is_nonempty_list_fS
    tbnz x25, 1, @label_53-22
# get_list_Sdd
    and x8, x25, -8
    ldp x27, x25, [x8]
# i_move_sd
    mov x28, x26
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, x27
# i_move_sd
    mov x27, 59
# i_call_only_f
    ldr x30, [x20], 8
    b @optuniquify/4-23
# label_L
@label_53-22:
label_53:
# i_move_sd
    mov x25, x26
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L77
    ret x30
# i_flush_stubs
# i_func_label_L
label_54:
# func_line_I
# i_func_info_IaaI
# inet_udp:optuniquify/4
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0xDD, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@optuniquify/4-23:
optuniquify/4:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L198
    bl L68
L198:
# i_test_yield
    adr x2, optuniquify/4
    subs w22, w22, 1
    b.le L70
# is_nonempty_list_fS
    tbnz x26, 1, @label_59-24
# get_list_Sdd
    and x8, x26, -8
    ldp x15, x26, [x8]
# bif_tuple_size_jSd
    mov x0, x25
    bl L201
    cbz x0, @label_56-25
    mov x16, x0
# bif_tuple_size_jSd
    mov x0, x15
    bl L201
    cbz x0, @label_56-25
    str x0, [x19, 112]
# is_eq_exact_fss
# simplified check since one argument is an immediate
# simplified fetching of BEAM register
    mov x14, x0
    cmp x16, x14
    b.ne @label_56-25
# bif_element_jssd
# simplified element/2 because arguments are known types
    ldur x9, [x25, -2]
    cmp x9, 64
    b.lo @label_56-25
L203:
    ldur x16, [x25, 6]
# bif_element_jssd
# simplified element/2 because arguments are known types
    ldur x9, [x15, -2]
    cmp x9, 64
    b.lo @label_56-25
L204:
    ldur x0, [x15, 6]
    str x0, [x19, 112]
# is_ne_exact_fss
# simplified fetching of BEAM register
    mov x1, x0
    cmp x16, x1
    b.eq @label_57-26
    orr x14, x16, x1
    and x14, x14, 3
    cmp x14, 3
    b.eq L205
    mov x0, x16
    stp x15, x16, [x19, 96]
    bl L208
    ldp x15, x16, [x19, 96]
    cbnz w0, @label_57-26
L205:
# label_L
@label_56-25:
label_56:
# is_eq_exact_fss
    cmp x15, x25
    b.eq L209
    orr x14, x15, x25
    and x14, x14, 3
    cmp x14, 3
    b.eq @label_58-27
    mov x0, x15
    mov x1, x25
    stp x15, x16, [x19, 96]
    bl L208
    ldp x15, x16, [x19, 96]
    cbz w0, @label_58-27
L209:
# label_L
@label_57-26:
label_57:
# i_call_only_f
    ldr x30, [x20], 8
    b optuniquify/4
# label_L
@label_58-27:
label_58:
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L211
    mov x3, 5
    bl L75
L211:
# put_list_ssd
    stp x15, x27, [x23], 16
    sub x27, x23, 15
# i_call_only_f
    ldr x30, [x20], 8
    b optuniquify/4
# label_L
@label_59-24:
label_59:
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L212
    mov x3, 4
    bl L75
L212:
    sub x20, x20, 16
# store_two_values_sdsd
    stp x28, x25, [x20]
# i_move_sd
    mov x25, x27
# line_I
# i_call_ext_e
    ldr x0, [L193]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L213
    mov x3, 1
    bl L75
L213:
# put_list_ssd
    ldp x9, x8, [x20]
    stp x8, x9, [x23], 16
    sub x26, x23, 15
# i_call_last_ft
    add x20, x20, 16
    ldr x30, [x20], 8
    b optuniquify/2
# i_flush_stubs
# i_func_label_L
label_60:
# func_line_I
# i_func_info_IaaI
# inet_udp:module_info/0
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L214
    bl L68
L214:
# i_test_yield
    adr x2, module_info/0
    subs w22, w22, 1
    b.le L70
# i_move_sd
    mov x25, 218379
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L215
    mov x3, 1
    bl L75
L215:
# call_light_bif_be
L216:
    ldr x3, [L217]
    ldr x7, [L218]
    adr x2, L216
# BIF: erlang:get_module_info/1
    bl L124
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L77
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_62:
# func_line_I
# i_func_info_IaaI
# inet_udp:module_info/1
    bl L65
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0B, 0x55, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L219
    bl L68
L219:
# i_test_yield
    adr x2, module_info/1
    subs w22, w22, 1
    b.le L70
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 218379
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L220
    mov x3, 2
    bl L75
L220:
# call_light_bif_be
L221:
    ldr x3, [L222]
    ldr x7, [L223]
    adr x2, L221
# BIF: erlang:get_module_info/2
    bl L124
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L77
    ret x30
# int_code_end
L224:
    mov x0, 4369093202
    bl L226
# Begin stub section
L78:
.xword 0x7FFFFFFFFFFFFFFF
L80:
.xword 0x7FFFFFFFFFFFFFFF
L82:
.xword 0x7FFFFFFFFFFFFFFF
L84:
.xword 0x7FFFFFFFFFFFFFFF
L89:
.xword 0x7FFFFFFFFFFFFFFF
L119:
.xword 0x7FFFFFFFFFFFFFFF
L121:
.xword 0x7FFFFFFFFFFFFFFF
L122:
.xword 0x000000010444DCE8
# End stub section
L227:
L226:
L225:
    mov x14, 4365818364
    br x14
L170:
L169:
    mov x14, 4481913944
    br x14
L159:
L158:
    mov x14, 4365837960
    br x14
L65:
L64:
    mov x14, 4481913584
    br x14
L124:
L123:
    mov x14, 4481910672
    br x14
L68:
L67:
    mov x14, 4481913368
    br x14
L103:
L102:
    mov x14, 4366798196
    br x14
L201:
L200:
    mov x14, 4481909816
    br x14
L112:
L111:
    mov x14, 4481912592
    br x14
L98:
L97:
    mov x14, 4366797356
    br x14
L108:
L107:
    mov x14, 4481912936
    br x14
L208:
L207:
    mov x14, 4366560408
    br x14
L77:
L76:
    mov x14, 4481911760
    br x14
L75:
L74:
    mov x14, 4481912640
    br x14
L127:
L126:
    mov x14, 4481916920
    br x14
L70:
L69:
    mov x14, 4481914968
    br x14
# Begin stub section
L138:
.xword 0x7FFFFFFFFFFFFFFF
L166:
.xword 0x7FFFFFFFFFFFFFFF
L168:
.xword 0x825F70EE1F36868F
L171:
.xword 0x7FFFFFFFFFFFFFFF
L179:
.xword 0x7FFFFFFFFFFFFFFF
L181:
.xword 0x7FFFFFFFFFFFFFFF
L183:
.xword 0x7FFFFFFFFFFFFFFF
L185:
.xword 0x7FFFFFFFFFFFFFFF
L188:
.xword 0x7FFFFFFFFFFFFFFF
L190:
.xword 0x7FFFFFFFFFFFFFFF
L193:
.xword 0x7FFFFFFFFFFFFFFF
L217:
.xword 0x7FFFFFFFFFFFFFFF
L218:
.xword 0x000000010442AAD0
L222:
.xword 0x7FFFFFFFFFFFFFFF
L223:
.xword 0x000000010442AD84
# End stub section
L228:
.section .rodata {#1}
line:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.section .text {#0}
.section .rodata {#1}
attr:
.byte 0x83, 0x6C, 0x00, 0x00, 0x00, 0x01, 0x68, 0x02, 0x77, 0x03, 0x76, 0x73, 0x6E, 0x6C, 0x00, 0x00, 0x00, 0x01, 0x6E, 0x10, 0x00, 0x0C, 0xFB, 0xA4, 0x3C, 0x6D, 0xD6, 0x9A, 0x95, 0x3E, 0xAF, 0xEE, 0xA0, 0xF6, 0xBF, 0x24, 0x77, 0x6A, 0x6A
.section .text {#0}
.section .rodata {#1}
compile:
.byte 0x83, 0x6C, 0x00, 0x00, 0x00, 0x03, 0x68, 0x02, 0x77, 0x07, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x6B, 0x00, 0x05, 0x38, 0x2E, 0x36, 0x2E, 0x31, 0x68, 0x02, 0x77, 0x07, 0x6F, 0x70, 0x74, 0x69, 0x6F, 0x6E, 0x73, 0x6C, 0x00, 0x00, 0x00, 0x06, 0x77, 0x0A, 0x64, 0x65, 0x62, 0x75, 0x67, 0x5F, 0x69, 0x6E, 0x66, 0x6F, 0x68, 0x02, 0x77, 0x01, 0x69, 0x6B, 0x00, 0x28, 0x2F, 0x62, 0x75, 0x69, 0x6C, 0x64, 0x72, 0x6F, 0x6F, 0x74, 0x2F, 0x6F, 0x74, 0x70, 0x2F, 0x6C, 0x69, 0x62, 0x2F, 0x6B, 0x65, 0x72, 0x6E, 0x65, 0x6C, 0x2F, 0x73, 0x72, 0x63, 0x2F, 0x2E, 0x2E, 0x2F, 0x69, 0x6E, 0x63, 0x6C, 0x75, 0x64, 0x65, 0x77, 0x19, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x64, 0x6F, 0x63, 0x5F, 0x66, 0x75, 0x6E, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x77, 0x19, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x64, 0x6F, 0x63, 0x5F, 0x63, 0x61, 0x6C, 0x6C, 0x62, 0x61, 0x63, 0x6B, 0x77, 0x1C, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x73, 0x70, 0x65, 0x63, 0x5F, 0x64, 0x6F, 0x63, 0x75, 0x6D, 0x65, 0x6E, 0x74, 0x65, 0x64, 0x77, 0x15, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x64, 0x65, 0x70, 0x72, 0x65, 0x63, 0x61, 0x74, 0x65, 0x64, 0x5F, 0x63, 0x61, 0x74, 0x63, 0x68, 0x6A, 0x68, 0x02, 0x77, 0x06, 0x73, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x6B, 0x00, 0x2A, 0x2F, 0x62, 0x75, 0x69, 0x6C, 0x64, 0x72, 0x6F, 0x6F, 0x74, 0x2F, 0x6F, 0x74, 0x70, 0x2F, 0x6C, 0x69, 0x62, 0x2F, 0x6B, 0x65, 0x72, 0x6E, 0x65, 0x6C, 0x2F, 0x73, 0x72, 0x63, 0x2F, 0x69, 0x6E, 0x65, 0x74, 0x5F, 0x75, 0x64, 0x70, 0x2E, 0x65, 0x72, 0x6C, 0x6A
.section .text {#0}
.section .rodata {#1}
md5:
.byte 0x77, 0x24, 0xBF, 0xF6, 0xA0, 0xEE, 0xAF, 0x3E, 0x95, 0x9A, 0xD6, 0x6D, 0x3C, 0xA4, 0xFB, 0x0C
.section .text {#0}
