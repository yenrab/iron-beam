L38:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# i_flush_stubs
# i_func_label_L
label_1:
# func_line_I
# i_func_info_IaaI
# atomics:new/2
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
new/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L41
    bl L43
L41:
# i_test_yield
    adr x2, new/2
    subs w22, w22, 1
    b.le L45
# allocate_tt
    add x2, x23, 56
    cmp x2, x20
    b.ls L46
    mov x3, 2
    bl L48
L46:
    sub x20, x20, 24
# store_two_values_sdsd
    stp x26, x25, [x20]
# catch_yH
    ldr x8, [x21, 248]
    add x8, x8, 1
    str x8, [x21, 248]
    ldr x14, [L49]
    str x14, [x20, 16]
# i_move_sd
    mov x25, x26
# i_move_sd
    mov x26, 31
# line_I
# i_call_f
    bl @label_8-0
# i_move_sd
    mov x26, x25
# i_move_sd
    ldr x25, [x20, 8]
# line_I
# call_light_bif_be
L51:
    ldr x3, [L52]
    ldr x7, [L53]
    adr x2, L51
# BIF: erts_internal:atomics_new/2
    bl L55
# try_end_deallocate_t
    ldr x8, [x21, 248]
    sub x8, x8, 1
    str x8, [x21, 248]
    add x20, x20, 24
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L57
    ret x30
# label_L
label_3:
# try_case_y
    ldr x8, [x21, 248]
    mov x25, x28
    sub x8, x8, 1
    str x8, [x21, 248]
# i_select_val_lins_sfI
    cmp x25, 715
    b.eq @label_4-1
    cmp x25, 779
    b.eq @label_5-2
    b L60
# label_L
@label_4-1:
label_4:
# is_eq_exact_fss
    mov x14, 5643
    cmp x26, x14
    b.ne @label_6-3
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L62
    mov x3, xzr
    bl L48
L62:
# put_list_ssd
    ldr x8, [x20]
    mov x9, 59
    stp x8, x9, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    ldr x8, [x20, 8]
    stp x8, x25, [x23], 16
    sub x26, x23, 15
# i_move_sd
    ldr x27, [L63]
# i_move_sd
    mov x25, 5003
# line_I
# call_light_bif_be
L64:
    ldr x3, [L65]
    ldr x7, [L66]
    adr x2, L64
# BIF: erlang:error/3
    bl L55
# mark_unreachable
# label_L
@label_5-2:
label_5:
# test_heap_It
    add x2, x23, 64
    cmp x2, x20
    b.ls L67
    mov x3, 2
    bl L48
L67:
# put_list2_sssd
    ldp x8, x9, [x20]
    mov x10, 59
    stp x8, x10, [x23], 16
    sub x25, x23, 15
    stp x9, x25, [x23], 16
    sub x25, x23, 15
# i_move_sd
    ldr x27, [L68]
# swap_dd
    mov x8, x26
    mov x26, x25
    mov x25, x8
# line_I
# call_light_bif_be
L69:
    ldr x3, [L65]
    ldr x7, [L66]
    adr x2, L69
# BIF: erlang:error/3
    bl L55
# mark_unreachable
# label_L
L60:
@label_6-3:
label_6:
# raise_ss
    mov x0, x26
    mov x1, x27
    bl L71
# i_flush_stubs
# i_func_label_L
    nop
label_7:
# func_line_I
# i_func_info_IaaI
# atomics:encode_opts/2
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0xAD, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@label_8-0:
label_8:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L72
    bl L43
L72:
# i_test_yield
    adr x2, label_8
    subs w22, w22, 1
    b.le L45
# is_nonempty_list_fS
    tbnz x25, 1, @label_11-4
# get_list_Sdd
    and x8, x25, -8
    ldp x27, x25, [x8]
# i_is_tagged_tuple_fsAa
    tbnz x27, 0, @label_12-5
    and x0, x27, -8
    ldp x8, x9, [x0]
    mov x14, 41035
    cmp x9, x14
    mov x10, 128
    ccmp x8, x10, 0, 2
    b.ne @label_12-5
# i_get_tuple_element_sPS
    ldr x27, [x0, 16]
# i_select_val_lins_sfI
    cmp x27, 11
    b.eq @label_10-6
    cmp x27, 75
    b.eq @label_9-7
    b @label_12-5
# label_L
@label_9-7:
label_9:
# i_move_sd
    mov x26, 31
# i_call_only_f
    ldr x30, [x20], 8
    b label_8
# label_L
@label_10-6:
label_10:
# line_I
# i_band_jIssd
# skipped test for small operands since they are always small
    and x26, x26, -17
# i_call_only_f
    ldr x30, [x20], 8
    b label_8
# label_L
@label_11-4:
label_11:
# is_nil_fS
    cmp x25, 59
    b.ne @label_12-5
# i_move_sd
    mov x25, x26
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L57
    ret x30
# label_L
@label_12-5:
label_12:
# i_move_sd
    mov x25, 5643
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L77
    mov x3, 1
    bl L48
L77:
# call_light_bif_be
L78:
    ldr x3, [L79]
    ldr x7, [L80]
    adr x2, L78
# BIF: erlang:throw/1
    bl L55
# mark_unreachable
# i_flush_stubs
# i_func_label_L
label_13:
# func_line_I
# i_func_info_IaaI
# atomics:put/3
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0xCA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
put/3:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L81
    bl L43
L81:
# call_bif_mfa_aaI
    adr x2, put/3
    sub x1, x2, 24
# HBIF: atomics:put/3
    mov x3, 4366474016
    b L83
# i_move_sd
    mov x25, 46027
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L84
    mov x3, 1
    bl L48
L84:
# call_light_bif_be
L85:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L85
# BIF: erlang:nif_error/1
    bl L55
# mark_unreachable
# i_flush_stubs
# i_func_label_L
    align 8
label_15:
# func_line_I
# i_func_info_IaaI
# atomics:get/2
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0xC1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
get/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L88
    bl L43
L88:
# call_bif_mfa_aaI
    adr x2, get/2
    sub x1, x2, 24
# HBIF: atomics:get/2
    mov x3, 4366474284
    b L83
# i_move_sd
    mov x25, 46027
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L89
    mov x3, 1
    bl L48
L89:
# call_light_bif_be
L90:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L90
# BIF: erlang:nif_error/1
    bl L55
# mark_unreachable
# i_flush_stubs
# i_func_label_L
    align 8
label_17:
# func_line_I
# i_func_info_IaaI
# atomics:add/3
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
add/3:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L91
    bl L43
L91:
# call_bif_mfa_aaI
    adr x2, add/3
    sub x1, x2, 24
# HBIF: atomics:add/3
    mov x3, 4366474712
    b L83
# i_move_sd
    mov x25, 46027
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L92
    mov x3, 1
    bl L48
L92:
# call_light_bif_be
L93:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L93
# BIF: erlang:nif_error/1
    bl L55
# mark_unreachable
# i_flush_stubs
# i_func_label_L
    align 8
label_19:
# func_line_I
# i_func_info_IaaI
# atomics:add_get/3
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
add_get/3:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L94
    bl L43
L94:
# call_bif_mfa_aaI
    adr x2, add_get/3
    sub x1, x2, 24
# HBIF: atomics:add_get/3
    mov x3, 4366474980
    b L83
# i_move_sd
    mov x25, 46027
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L95
    mov x3, 1
    bl L48
L95:
# call_light_bif_be
L96:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L96
# BIF: erlang:nif_error/1
    bl L55
# mark_unreachable
# i_flush_stubs
# i_func_label_L
    align 8
label_21:
# func_line_I
# i_func_info_IaaI
# atomics:sub/3
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x7E, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
sub/3:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L97
    bl L43
L97:
# i_test_yield
    adr x2, sub/3
    subs w22, w22, 1
    b.le L45
# allocate_tt
    add x2, x23, 64
    cmp x2, x20
    b.ls L98
    mov x3, 3
    bl L48
L98:
    sub x20, x20, 32
# store_two_values_sdsd
    stp x27, x26, [x20]
# i_move_sd
    str x25, [x20, 16]
# catch_yH
    ldr x8, [x21, 248]
    add x8, x8, 1
    str x8, [x21, 248]
    ldr x14, [L99]
    str x14, [x20, 24]
# line_I
# i_unary_minus_jIsd
    mov x8, 15
    and x9, x27, -16
    subs x0, x8, x9
    ccmp x9, 15, 0, 9
    b.eq L100
    mov x1, x27
    bl L102
L100:
    mov x27, x0
# call_light_bif_be
L103:
    ldr x3, [L104]
    ldr x7, [L105]
    adr x2, L103
# BIF: atomics:add/3
    bl L55
# try_end_deallocate_t
    ldr x8, [x21, 248]
    sub x8, x8, 1
    str x8, [x21, 248]
    add x20, x20, 32
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L57
    ret x30
# label_L
label_23:
# try_case_y
    ldr x8, [x21, 248]
    mov x25, x28
    sub x8, x8, 1
    str x8, [x21, 248]
# is_eq_exact_fss
    cmp x25, 779
    b.ne @label_24-8
# test_heap_It
    add x2, x23, 80
    cmp x2, x20
    b.ls L107
    mov x3, 2
    bl L48
L107:
# put_list2_sssd
    ldp x8, x9, [x20]
    mov x10, 59
    stp x8, x10, [x23], 16
    sub x25, x23, 15
    stp x9, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    ldr x8, [x20, 16]
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# i_move_sd
    ldr x27, [L68]
# swap_dd
    mov x8, x26
    mov x26, x25
    mov x25, x8
# line_I
# call_light_bif_be
L108:
    ldr x3, [L65]
    ldr x7, [L66]
    adr x2, L108
# BIF: erlang:error/3
    bl L55
# mark_unreachable
# label_L
@label_24-8:
label_24:
# raise_ss
    mov x0, x26
    mov x1, x27
    bl L71
# i_flush_stubs
# i_func_label_L
    nop
    align 8
label_25:
# func_line_I
# i_func_info_IaaI
# atomics:sub_get/3
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x7E, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
sub_get/3:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L109
    bl L43
L109:
# i_test_yield
    adr x2, sub_get/3
    subs w22, w22, 1
    b.le L45
# allocate_tt
    add x2, x23, 64
    cmp x2, x20
    b.ls L110
    mov x3, 3
    bl L48
L110:
    sub x20, x20, 32
# store_two_values_sdsd
    stp x27, x26, [x20]
# i_move_sd
    str x25, [x20, 16]
# catch_yH
    ldr x8, [x21, 248]
    add x8, x8, 1
    str x8, [x21, 248]
    ldr x14, [L111]
    str x14, [x20, 24]
# line_I
# i_unary_minus_jIsd
    mov x8, 15
    and x9, x27, -16
    subs x0, x8, x9
    ccmp x9, 15, 0, 9
    b.eq L112
    mov x1, x27
    bl L102
L112:
    mov x27, x0
# call_light_bif_be
L113:
    ldr x3, [L114]
    ldr x7, [L115]
    adr x2, L113
# BIF: atomics:add_get/3
    bl L55
# try_end_deallocate_t
    ldr x8, [x21, 248]
    sub x8, x8, 1
    str x8, [x21, 248]
    add x20, x20, 32
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L57
    ret x30
# label_L
label_27:
# try_case_y
    ldr x8, [x21, 248]
    mov x25, x28
    sub x8, x8, 1
    str x8, [x21, 248]
# is_eq_exact_fss
    cmp x25, 779
    b.ne @label_28-9
# test_heap_It
    add x2, x23, 80
    cmp x2, x20
    b.ls L117
    mov x3, 2
    bl L48
L117:
# put_list2_sssd
    ldp x8, x9, [x20]
    mov x10, 59
    stp x8, x10, [x23], 16
    sub x25, x23, 15
    stp x9, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    ldr x8, [x20, 16]
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# i_move_sd
    ldr x27, [L68]
# swap_dd
    mov x8, x26
    mov x26, x25
    mov x25, x8
# line_I
# call_light_bif_be
L118:
    ldr x3, [L65]
    ldr x7, [L66]
    adr x2, L118
# BIF: erlang:error/3
    bl L55
# mark_unreachable
# label_L
@label_28-9:
label_28:
# raise_ss
    mov x0, x26
    mov x1, x27
    bl L71
# i_flush_stubs
# i_func_label_L
    nop
    align 8
label_29:
# func_line_I
# i_func_info_IaaI
# atomics:exchange/3
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
exchange/3:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L119
    bl L43
L119:
# call_bif_mfa_aaI
    adr x2, exchange/3
    sub x1, x2, 24
# HBIF: atomics:exchange/3
    mov x3, 4366475456
    b L83
# i_move_sd
    mov x25, 46027
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L120
    mov x3, 1
    bl L48
L120:
# call_light_bif_be
L121:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L121
# BIF: erlang:nif_error/1
    bl L55
# mark_unreachable
# i_flush_stubs
# i_func_label_L
    align 8
label_31:
# func_line_I
# i_func_info_IaaI
# atomics:compare_exchange/4
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
compare_exchange/4:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L122
    bl L43
L122:
# call_bif_mfa_aaI
    adr x2, compare_exchange/4
    sub x1, x2, 24
# HBIF: atomics:compare_exchange/4
    mov x3, 4366475944
    b L83
# i_move_sd
    mov x25, 46027
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L123
    mov x3, 1
    bl L48
L123:
# call_light_bif_be
L124:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L124
# BIF: erlang:nif_error/1
    bl L55
# mark_unreachable
# i_flush_stubs
# i_func_label_L
    align 8
label_33:
# func_line_I
# i_func_info_IaaI
# atomics:info/1
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
info/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L125
    bl L43
L125:
# call_bif_mfa_aaI
    adr x2, info/1
    sub x1, x2, 24
# HBIF: atomics:info/1
    mov x3, 4366476476
    b L83
# i_move_sd
    mov x25, 46027
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L126
    mov x3, 1
    bl L48
L126:
# call_light_bif_be
L127:
    ldr x3, [L86]
    ldr x7, [L87]
    adr x2, L127
# BIF: erlang:nif_error/1
    bl L55
# mark_unreachable
# i_flush_stubs
# i_func_label_L
    align 8
label_35:
# func_line_I
# i_func_info_IaaI
# atomics:module_info/0
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L128
    bl L43
L128:
# i_test_yield
    adr x2, module_info/0
    subs w22, w22, 1
    b.le L45
# i_move_sd
    mov x25, 69515
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L129
    mov x3, 1
    bl L48
L129:
# call_light_bif_be
L130:
    ldr x3, [L131]
    ldr x7, [L132]
    adr x2, L130
# BIF: erlang:get_module_info/1
    bl L55
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L57
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_37:
# func_line_I
# i_func_info_IaaI
# atomics:module_info/1
    bl L40
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x8B, 0x0F, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L133
    bl L43
L133:
# i_test_yield
    adr x2, module_info/1
    subs w22, w22, 1
    b.le L45
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 69515
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L134
    mov x3, 2
    bl L48
L134:
# call_light_bif_be
L135:
    ldr x3, [L136]
    ldr x7, [L137]
    adr x2, L135
# BIF: erlang:get_module_info/2
    bl L55
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L57
    ret x30
# int_code_end
L138:
    mov x0, 4369093202
    bl L140
L102:
L101:
    mov x14, 4481917272
    br x14
L71:
L70:
    mov x14, 4481917016
    br x14
L57:
L56:
    mov x14, 4481911760
    br x14
L140:
L139:
    mov x14, 4365818364
    br x14
L55:
L54:
    mov x14, 4481910672
    br x14
L48:
L47:
    mov x14, 4481912640
    br x14
L45:
L44:
    mov x14, 4481914968
    br x14
L83:
L82:
    mov x14, 4481910448
    br x14
L43:
L42:
    mov x14, 4481913368
    br x14
L40:
L39:
    mov x14, 4481913584
    br x14
# Begin stub section
L49:
.xword 0x000000007FFFFFFF
L52:
.xword 0x7FFFFFFFFFFFFFFF
L53:
.xword 0x0000000104431944
L63:
.xword 0x7FFFFFFFFFFFFFFF
L65:
.xword 0x7FFFFFFFFFFFFFFF
L66:
.xword 0x000000010444DADC
L68:
.xword 0x7FFFFFFFFFFFFFFF
L79:
.xword 0x7FFFFFFFFFFFFFFF
L80:
.xword 0x00000001044524F4
L86:
.xword 0x7FFFFFFFFFFFFFFF
L87:
.xword 0x000000010444DC44
L99:
.xword 0x000000007FFFFFFF
L104:
.xword 0x7FFFFFFFFFFFFFFF
L105:
.xword 0x0000000104431DD8
L111:
.xword 0x000000007FFFFFFF
L114:
.xword 0x7FFFFFFFFFFFFFFF
L115:
.xword 0x0000000104431EE4
L131:
.xword 0x7FFFFFFFFFFFFFFF
L132:
.xword 0x000000010442AAD0
L136:
.xword 0x7FFFFFFFFFFFFFFF
L137:
.xword 0x000000010442AD84
# End stub section
L141:
.section .rodata {#1}
md5:
.byte 0x3E, 0x7B, 0xA1, 0x37, 0x84, 0x87, 0x3F, 0xB4, 0x8A, 0x64, 0xEE, 0x95, 0x20, 0xD2, 0xBF, 0x98
.section .text {#0}
