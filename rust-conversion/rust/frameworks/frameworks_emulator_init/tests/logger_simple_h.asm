L107:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# i_flush_stubs
# i_func_label_L
label_1:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:adding_handler/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0xF6, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
adding_handler/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L110
    bl L112
L110:
# i_test_yield
    adr x2, adding_handler/1
    subs w22, w22, 1
    b.le L114
# is_map_fs
    tbnz x25, 0, label_1
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne label_1
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 21451
    ldr x2, [L115]
    bl L117
    b.ne label_1
    mov x26, x0
# is_eq_exact_fss
    mov x14, 408203
    cmp x26, x14
    b.ne label_1
# allocate_tt
    add x2, x23, 56
    cmp x2, x20
    b.ls L118
    mov x3, 1
    bl L120
L118:
    sub x20, x20, 24
# i_move_sd
    mov x14, 59
    str x14, [x20]
# i_move_sd
    str x25, [x20, 16]
# self_d
    ldr x14, [x21]
    str x14, [x20, 8]
# i_move_sd
    mov x25, 215627
# line_I
# call_light_bif_be
L121:
    ldr x3, [L122]
    ldr x7, [L123]
    adr x2, L121
# BIF: erlang:whereis/1
    bl L125
# is_eq_exact_fss
    cmp x25, 907
    b.ne @label_8-0
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L127
    mov x3, xzr
    bl L120
L127:
# i_make_fun3_FStt
    ldr x9, [L128]
# Create fun thing
    mov x8, 65556
    stp x8, x9, [x23]
# Move fun environment
    ldr x14, [x20, 8]
    str x14, [x23, 16]
# Create boxed ptr
    orr x25, x23, 2
    add x23, x23, 24
# i_move_sd
    ldr x26, [L129]
# i_move_sd
    mov x14, 59
    str x14, [x20, 8]
# line_I
# i_call_ext_e
    ldr x0, [L130]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_is_tuple_of_arity_fsA
    tbnz x25, 0, @label_9-1
    and x0, x25, -8
    ldr x8, [x0]
    cmp x8, 128
    b.ne @label_9-1
# get_two_tuple_elements_sPSS
    ldp x8, x9, [x0, 8]
    stp x9, x8, [x20]
# aligned_label_Lt
label_3:
# i_loop_rec_f
L132:
    adr x0, L132
    ldr x1, [L133]
    bl L135
# i_select_tuple_arity_SfI
    tbnz x25, 0, @label_6-2
    ldur x8, [x25, -2]
    tst x8, 63
    b.ne @label_6-2
# Linear search in [0..1], 2 elements
    cmp x8, 128
    b.eq @label_5-3
    cmp x8, 320
    b.eq @label_4-4
    b @label_6-2
# label_L
@label_4-4:
label_4:
# load_tuple_ptr_s
    and x0, x25, -8
# get_two_tuple_elements_sPSS
    ldp x26, x27, [x0, 8]
# get_two_tuple_elements_sPSS
    ldp x28, x15, [x0, 24]
# is_eq_exact_fss
    cmp x26, 1355
    b.ne @label_6-2
# is_eq_exact_fss
    mov x14, 35275
    cmp x28, x14
    b.ne @label_6-2
# is_eq_exact_fss
    ldr x1, [x20]
    cmp x27, x1
    b.eq L139
    orr x14, x27, x1
    and x14, x14, 3
    cmp x14, 3
    b.eq @label_6-2
    mov x0, x27
    stp x15, x16, [x19, 96]
    bl L141
    ldp x15, x16, [x19, 96]
    cbz w0, @label_6-2
L139:
# is_eq_exact_fss
    ldr x1, [x20, 8]
    cmp x15, x1
    b.eq L142
    orr x14, x15, x1
    and x14, x14, 3
    cmp x14, 3
    b.eq @label_6-2
    mov x0, x15
    stp x15, x16, [x19, 96]
    bl L141
    ldp x15, x16, [x19, 96]
    cbz w0, @label_6-2
L142:
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L143
    mov x3, 1
    bl L120
L143:
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x25, [x0, 40]
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# put_tuple2_SA
    mov x9, 128
    mov x10, 779
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# deallocate_t
    add x20, x20, 24
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_5-3:
label_5:
# load_tuple_ptr_s
    and x0, x25, -8
# get_two_tuple_elements_sPSS
    ldp x26, x25, [x0, 8]
# is_eq_exact_fss
    mov x14, 81163
    cmp x25, x14
    b.ne @label_6-2
# is_eq_exact_fss
    ldr x1, [x20, 8]
    cmp x26, x1
    b.eq L148
    orr x14, x26, x1
    and x14, x14, 3
    cmp x14, 3
    b.eq @label_6-2
    mov x0, x26
    stp x15, x16, [x19, 96]
    bl L141
    ldp x15, x16, [x19, 96]
    cbz w0, @label_6-2
L148:
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# move_trim_sdt
    ldr x25, [x20], 16
# line_I
# call_light_bif_be
L149:
    ldr x3, [L150]
    ldr x7, [L151]
    adr x2, L149
# BIF: erlang:demonitor/1
    bl L125
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L152
    mov x3, xzr
    bl L120
L152:
# put_tuple2_SA
    mov x9, 128
    mov x10, 32139
    stp x9, x10, [x23], 16
    ldr x14, [x20]
    str x14, [x23], 8
    sub x25, x23, 22
# deallocate_t
    add x20, x20, 8
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_6-2:
label_6:
# loop_rec_end_f
    mov x0, x21
    bl L154
    sub w22, w22, 1
    b label_3
# aligned_label_Lt
label_7:
# wait_locked_f
    mov x0, x21
    ldr x1, [L155]
    bl L157
    b L159
# label_L
@label_8-0:
label_8:
# i_move_sd
    ldr x25, [L160]
# deallocate_t
    add x20, x20, 24
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_9-1:
label_9:
# line_I
# badmatch_s
    mov x8, 5200
    stp x8, x25, [x21, 96]
    bl L162
# i_flush_stubs
# i_func_label_L
    nop
label_10:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:removing_handler/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xF7, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
removing_handler/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L163
    bl L112
L163:
# i_test_yield
    adr x2, removing_handler/1
    subs w22, w22, 1
    b.le L114
# is_map_fs
    tbnz x25, 0, label_10
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne label_10
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 21451
    ldr x2, [L115]
    bl L117
    b.ne label_10
    mov x26, x0
# is_eq_exact_fss
    mov x14, 408203
    cmp x26, x14
    b.ne label_10
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L164
    mov x3, xzr
    bl L120
L164:
    sub x20, x20, 16
# init_yregs_I
    mov x8, 59
    stp x8, x8, [x20]
# i_move_sd
    mov x25, 215627
# line_I
# call_light_bif_be
L165:
    ldr x3, [L122]
    ldr x7, [L123]
    adr x2, L165
# BIF: erlang:whereis/1
    bl L125
# i_move_sd
    str x25, [x20, 8]
# is_eq_exact_fss
    cmp x25, 907
    b.ne @label_12-5
# i_move_sd
    mov x25, 32139
# deallocate_t
    add x20, x20, 16
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_12-5:
label_12:
# recv_marker_reserve_S
    stp x23, x20, [x21, 80]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L168
    ldp x23, x20, [x21, 80]
    ldp x15, x16, [x19, 96]
    str x0, [x20]
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 35275
# line_I
# call_light_bif_be
L169:
    ldr x3, [L170]
    ldr x7, [L171]
    adr x2, L169
# BIF: erlang:monitor/2
    bl L125
# recv_marker_bind_SS
    ldr x1, [x20]
    mov x2, x25
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L173
    ldp x15, x16, [x19, 96]
# i_move_sd
    str x25, [x20]
# i_move_sd
    ldr x25, [x20, 8]
# i_move_sd
    mov x26, 43147
# line_I
# send
L174:
    ldr x3, [L175]
    ldr x7, [L176]
    adr x2, L174
    bl L125
# recv_marker_use_S
    ldr x1, [x20]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L178
    ldp x15, x16, [x19, 96]
# aligned_label_Lt
label_13:
# i_loop_rec_f
L179:
    adr x0, L179
    ldr x1, [L180]
    bl L135
# i_is_tagged_tuple_fsAa
    tbnz x25, 0, @label_14-6
    and x0, x25, -8
    ldp x8, x9, [x0]
    cmp x9, 1355
    mov x10, 320
    ccmp x8, x10, 0, 2
    b.ne @label_14-6
# get_two_tuple_elements_sPSS
    ldp x26, x27, [x0, 16]
# i_get_tuple_element_sPS
    ldr x25, [x0, 32]
# is_eq_exact_fss
    mov x14, 35275
    cmp x27, x14
    b.ne @label_14-6
# is_eq_exact_fss
    ldr x1, [x20]
    cmp x26, x1
    b.eq L182
    tbnz x26, 0, @label_14-6
    mov x0, x26
    stp x15, x16, [x19, 96]
    bl L141
    ldp x15, x16, [x19, 96]
    cbz w0, @label_14-6
L182:
# is_eq_exact_fss
    ldr x1, [x20, 8]
    cmp x25, x1
    b.eq L183
    orr x14, x25, x1
    and x14, x14, 3
    cmp x14, 3
    b.eq @label_14-6
    mov x0, x25
    stp x15, x16, [x19, 96]
    bl L141
    ldp x15, x16, [x19, 96]
    cbz w0, @label_14-6
L183:
# recv_marker_clear_S
    ldr x1, [x20]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L185
    ldp x15, x16, [x19, 96]
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# i_move_sd
    mov x25, 32139
# deallocate_t
    add x20, x20, 16
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_14-6:
label_14:
# loop_rec_end_f
    mov x0, x21
    bl L154
    sub w22, w22, 1
    b label_13
# aligned_label_Lt
label_15:
# wait_locked_f
    mov x0, x21
    ldr x1, [L186]
    bl L157
    b L159
# i_flush_stubs
# i_func_label_L
    align 8
label_16:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:log/2
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0xDD, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
log/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L187
    bl L112
L187:
# i_test_yield
    adr x2, log/2
    subs w22, w22, 1
    b.le L114
# is_map_fs
    tbnz x25, 0, @label_20-7
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_20-7
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 26891
    ldr x2, [L189]
    bl L117
    b.ne @label_20-7
    mov x26, x0
# is_map_fs
    tbnz x26, 0, @label_20-7
    ldur x10, [x26, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_20-7
# i_get_map_element_hash_fScWS
    mov x0, x26
    mov x1, 15691
    ldr x2, [L190]
    bl L117
    b.ne @label_20-7
    mov x26, x0
# is_map_fs
    tbnz x26, 0, @label_20-7
    ldur x10, [x26, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_20-7
# i_get_map_elements_fsI
    mov x0, x26
# simplified multi-element lookup
    and x8, x0, -8
    ldp x9, x10, [x8]
    and x9, x9, 252
    cmp x9, 44
    b.ne L191
    add x10, x10, 1
    ldr x9, [x8, 16]!
    and x9, x9, -8
L193:
    subs x10, x10, 1
    b.eq @label_20-7
    ldr x11, [x9, x10 lsl 3]
    mov x14, 45771
    cmp x11, x14
    b.ne L193
    ldr x26, [x8, x10 lsl 3]
L194:
    subs x10, x10, 1
    b.eq @label_20-7
    ldr x11, [x9, x10 lsl 3]
    mov x14, 44171
    cmp x11, x14
    b.ne L194
    ldr x27, [x8, x10 lsl 3]
    b L192
L191:
    adr x4, L195
    b L196
    align 8
L195:
.byte 0x8B, 0xAC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x03, 0xDE, 0x12, 0x8B, 0x42, 0xBB, 0xA9, 0xA0
.byte 0xCB, 0xB2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x54, 0xBF, 0xEE, 0x66, 0xC9, 0x7E, 0xA9, 0x96
L196:
    mov x2, x20
    stp x25, x26, [x19, 64]
    stp x27, x28, [x19, 80]
    stp x15, x16, [x19, 96]
    mov x3, 2
    add x1, x19, 64
    bl L198
    ldp x25, x26, [x19, 64]
    ldp x27, x28, [x19, 80]
    ldp x15, x16, [x19, 96]
    cbz x0, @label_20-7
L192:
# is_eq_exact_fss
    mov x14, 243147
    cmp x27, x14
    b.ne @label_20-7
# is_ne_exact_fss
    mov x14, 243275
    cmp x26, x14
    b.eq @label_20-7
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L199
    mov x3, 1
    bl L120
L199:
    sub x20, x20, 8
# i_move_sd
    str x25, [x20]
# i_move_sd
    mov x26, 215627
# i_move_sd
    mov x25, 81867
# line_I
# i_call_ext_e
    ldr x0, [L200]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_select_val_lins_sfI
    cmp x25, 11
    b.eq @label_19-8
    cmp x25, 75
    b.eq @label_18-9
    b L203
# label_L
@label_18-9:
label_18:
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b @do_log/1-10
# label_L
@label_19-8:
label_19:
# i_move_sd
    mov x25, 32139
# deallocate_t
    add x20, x20, 8
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_20-7:
label_20:
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L205
    mov x3, 1
    bl L120
L205:
# line_I
# i_call_f
    bl @do_log/1-10
# i_move_sd
    mov x25, 32139
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
L203:
label_21:
# line_I
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L162
# i_flush_stubs
# i_func_label_L
    nop
    align 8
label_22:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:do_log/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0xF7, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@do_log/1-10:
do_log/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L206
    bl L112
L206:
# i_test_yield
    adr x2, do_log/1
    subs w22, w22, 1
    b.le L114
# is_map_fs
    tbnz x25, 0, @label_27-11
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_27-11
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 26891
    ldr x2, [L189]
    bl L117
    b.ne @label_27-11
    mov x26, x0
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 133131
    ldr x2, [L208]
    bl L117
    b.ne @label_27-11
# is_map_fs
    tbnz x26, 0, @label_27-11
    ldur x10, [x26, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_27-11
# i_get_map_element_hash_fScWS
    mov x0, x26
    mov x1, 52491
    ldr x2, [L209]
    bl L117
    b.ne @label_27-11
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L210
    mov x3, 2
    bl L120
L210:
    sub x20, x20, 16
# store_two_values_sdsd
    stp x26, x25, [x20]
# i_move_sd
    mov x25, 215627
# line_I
# call_light_bif_be
L211:
    ldr x3, [L122]
    ldr x7, [L123]
    adr x2, L211
# BIF: erlang:whereis/1
    bl L125
# is_eq_exact_fss
    cmp x25, 907
    b.ne @label_26-12
# i_get_map_element_hash_fScWS
    ldr x0, [x20]
    mov x1, 414731
    ldr x2, [L213]
    bl L117
    b.ne @label_24-13
    mov x25, x0
# i_select_val_lins_sfI
    cmp x25, 11
    b.eq @label_24-13
    cmp x25, 75
    b.eq @label_25-14
    b L216
# label_L
@label_24-13:
label_24:
# i_move_sd
    mov x14, 59
    str x14, [x20]
# line_I
# i_call_ext_e
    ldr x0, [L217]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_new_small_map_lit_dtqI
    add x2, x23, 72
    cmp x2, x20
    b.ls L218
    mov x3, 1
    bl L120
L218:
    add x8, x23, 2
    mov x9, 300
    mov x10, 1
    stp x9, x10, [x23], 16
    ldr x9, [L219]
    stp x9, x25, [x23], 16
    mov x25, x8
# i_new_small_map_lit_dtqI
    add x2, x23, 88
    cmp x2, x20
    b.ls L220
    mov x3, 1
    bl L120
L220:
    add x26, x23, 2
    mov x9, 300
    mov x10, 3
    stp x9, x10, [x23], 16
    ldr x9, [L221]
    stp x9, x25, [x23], 16
    ldr x9, [L222]
    mov x10, 779
    stp x9, x10, [x23], 16
# i_move_sd
    mov x25, 408203
# line_I
# i_call_f
    bl @log_internal/2-15
# label_L
@label_25-14:
label_25:
# i_move_sd
    ldr x26, [x20, 8]
# i_move_sd
    mov x25, 408203
# i_call_last_ft
    add x20, x20, 16
    ldr x30, [x20], 8
    b @log_internal/2-15
# label_L
@label_26-12:
label_26:
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L224
    mov x3, xzr
    bl L120
L224:
# put_tuple2_SA
    mov x9, 128
    mov x10, 56779
    stp x9, x10, [x23], 16
    ldr x14, [x20, 8]
    str x14, [x23], 8
    sub x26, x23, 22
# i_move_sd
    mov x25, 215627
# line_I
# send
L225:
    ldr x3, [L175]
    ldr x7, [L176]
    adr x2, L225
    bl L125
# deallocate_t
    add x20, x20, 16
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_27-11:
label_27:
# i_move_sd
    mov x25, 32139
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
L216:
label_28:
# line_I
# case_end_s
    mov x8, 7248
    stp x8, x25, [x21, 96]
    bl L162
# i_flush_stubs
# i_func_label_L
    nop
    align 8
label_29:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:init/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
init/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L226
    bl L112
L226:
# i_test_yield
    adr x2, init/1
    subs w22, w22, 1
    b.le L114
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L227
    mov x3, 1
    bl L120
L227:
    sub x20, x20, 8
# i_move_sd
    str x25, [x20]
# self_d
    ldr x26, [x21]
# i_move_sd
    mov x25, 215627
# line_I
# call_light_bif_be
L228:
    ldr x3, [L229]
    ldr x7, [L230]
    adr x2, L228
# BIF: erlang:register/2
    bl L125
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L231
    mov x3, xzr
    bl L120
L231:
# self_d
    ldr x25, [x21]
# put_tuple2_SA
    mov x9, 128
    stp x9, x25, [x23], 16
    mov x14, 81163
    str x14, [x23], 8
    sub x26, x23, 22
# move_trim_sdt
    ldr x25, [x20], 8
# line_I
# send
L232:
    ldr x3, [L175]
    ldr x7, [L176]
    adr x2, L232
    bl L125
# i_move_sd
    ldr x26, [L233]
# i_move_sd
    mov x25, 414795
# i_call_last_ft
    ldr x30, [x20], 8
    b @loop/2-16
# i_flush_stubs
# i_func_label_L
label_31:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:loop/2
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x4C, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@loop/2-16:
loop/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L235
    bl L112
L235:
# i_test_yield
    adr x2, loop/2
    subs w22, w22, 1
    b.le L114
# allocate_tt
    add x2, x23, 56
    cmp x2, x20
    b.ls L236
    mov x3, 2
    bl L120
L236:
    sub x20, x20, 24
# store_two_values_sdsd
    mov x8, 59
    stp x8, x26, [x20]
# i_move_sd
    str x25, [x20, 16]
# aligned_label_Lt
label_33:
# i_loop_rec_f
L237:
    adr x0, L237
    ldr x1, [L238]
    bl L135
# i_is_tagged_tuple_ff_ffsAa
    tbnz x25, 0, @label_35-17
    and x0, x25, -8
    ldp x8, x9, [x0]
    cmp x8, 128
    b.eq L239
    tst x8, 63
    b.eq @label_37-18
    b @label_35-17
L239:
    mov x14, 56779
    cmp x9, x14
    b.ne @label_37-18
# i_get_tuple_element_sPS
    ldr x25, [x0, 16]
# is_map_fs
    tbnz x25, 0, @label_37-19
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_37-18
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 26891
    ldr x2, [L189]
    bl L117
    b.ne @label_37-18
    mov x26, x0
# is_map_fs
    tbnz x26, 0, @label_37-19
    ldur x10, [x26, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_37-18
# i_get_map_element_hash_fScWS
    mov x0, x26
    mov x1, 15691
    ldr x2, [L190]
    bl L117
    b.ne @label_34-20
    mov x27, x0
# is_map_fs
    tbnz x27, 0, @label_34-21
    ldur x10, [x27, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_34-20
# i_get_map_elements_fsI
    mov x0, x27
# simplified multi-element lookup
    and x8, x0, -8
    ldp x9, x10, [x8]
    and x9, x9, 252
    cmp x9, 44
    b.ne L245
    add x10, x10, 1
    ldr x9, [x8, 16]!
    and x9, x9, -8
L247:
    subs x10, x10, 1
    b.eq @label_34-20
    ldr x11, [x9, x10 lsl 3]
    mov x14, 45771
    cmp x11, x14
    b.ne L247
    ldr x27, [x8, x10 lsl 3]
L248:
    subs x10, x10, 1
    b.eq @label_34-20
    ldr x11, [x9, x10 lsl 3]
    mov x14, 44171
    cmp x11, x14
    b.ne L248
    ldr x28, [x8, x10 lsl 3]
    b L246
L245:
    adr x4, L249
    b L250
    align 8
L249:
.byte 0x8B, 0xAC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x03, 0xDE, 0x12, 0x8B, 0x42, 0xBB, 0xA9, 0xA0
.byte 0xCB, 0xB2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x54, 0xBF, 0xEE, 0x66, 0xC9, 0x7E, 0xA9, 0x96
L250:
    mov x2, x20
    stp x25, x26, [x19, 64]
    stp x27, x28, [x19, 80]
    stp x15, x16, [x19, 96]
    mov x3, 2
    add x1, x19, 64
    bl L198
    ldp x25, x26, [x19, 64]
    ldp x27, x28, [x19, 80]
    ldp x15, x16, [x19, 96]
    cbz x0, @label_34-20
L246:
# is_eq_exact_fss
    mov x14, 243147
    cmp x28, x14
    b.ne @label_34-20
# is_ne_exact_fss
    mov x14, 243275
    cmp x27, x14
    b.eq @label_34-20
# i_move_sd
    mov x26, x25
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# move_trim_sdt
    ldr x25, [x20, 8]
    add x20, x20, 16
# line_I
# i_call_f
    bl @update_buffer/2-22
# i_move_sd
    mov x26, x25
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b loop/2
# label_L
@label_34-20:
@label_34-21:
label_34:
# i_get_map_element_hash_fScWS
    mov x0, x26
    mov x1, 52491
    ldr x2, [L209]
    bl L117
    b.ne @label_37-18
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 133131
    ldr x2, [L208]
    bl L117
    b.ne @label_37-18
# i_move_sd
    str x25, [x20]
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# i_move_sd
    mov x26, x25
# i_move_sd
    ldr x25, [x20, 16]
# i_move_sd
    mov x14, 59
    str x14, [x20, 16]
# line_I
# i_call_f
    bl @log_internal/2-15
# i_move_sd
    ldr x27, [x20, 8]
# i_move_sd
    str x25, [x20, 16]
# move_trim_sdt
    ldr x26, [x20], 16
# i_move_sd
    mov x25, x27
# line_I
# i_call_f
    bl @update_buffer/2-22
# i_move_sd
    mov x26, x25
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b loop/2
# label_L
@label_35-17:
label_35:
# is_eq_exact_fss
    mov x14, 43147
    cmp x25, x14
    b.ne @label_37-18
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# i_move_sd
    mov x14, 59
    str x14, [x20, 16]
# i_move_sd
    mov x25, 11723
# line_I
# i_call_ext_e
    ldr x0, [L252]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_is_tagged_tuple_fsAa
    tbnz x25, 0, @label_36-23
    and x0, x25, -8
    ldp x8, x9, [x0]
    mov x14, 32139
    cmp x9, x14
    mov x10, 128
    ccmp x8, x10, 0, 2
    b.ne @label_36-23
# i_move_sd
    ldr x25, [x20, 8]
# i_move_sd
    mov x14, 59
    str x14, [x20, 8]
# line_I
# i_call_f
    bl @replay_buffer/1-24
# label_L
@label_36-23:
label_36:
# i_move_sd
    mov x25, 25163
# line_I
# call_light_bif_be
L255:
    ldr x3, [L122]
    ldr x7, [L123]
    adr x2, L255
# BIF: erlang:whereis/1
    bl L125
# call_light_bif_be
L256:
    ldr x3, [L257]
    ldr x7, [L258]
    adr x2, L256
# BIF: erlang:unlink/1
    bl L125
# i_move_sd
    mov x25, 32139
# deallocate_t
    add x20, x20, 24
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_37-18:
@label_37-19:
label_37:
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# load_two_xregs_dxdx
    ldp x26, x25, [x20, 8]
# i_call_last_ft
    add x20, x20, 24
    ldr x30, [x20], 8
    b loop/2
# aligned_label_Lt
label_38:
# wait_locked_f
    mov x0, x21
    ldr x1, [L259]
    bl L157
    b L159
# i_flush_stubs
# i_func_label_L
    align 8
label_39:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:update_buffer/2
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x54, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@update_buffer/2-22:
update_buffer/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L260
    bl L112
L260:
# i_test_yield
    adr x2, update_buffer/2
    subs w22, w22, 1
    b.le L114
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 414923
    ldr x2, [L261]
    bl L117
    b.ne label_39
    mov x27, x0
# is_eq_exact_fss
    cmp x27, 15
    b.ne @label_41-25
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 414987
    ldr x2, [L263]
    bl L117
    b.ne @label_41-25
    mov x28, x0
# line_I
# i_plus_jIssd
    mov x2, 31
    adds x0, x28, 16
    and x8, x28, 15
# test for not overflow and small operands
    ccmp x8, 15, 0, 9
    b.eq L264
    mov x1, x28
    bl L266
L264:
    mov x26, x0
# update_map_assoc_sdtI
    mov x1, 414987
    mov x2, x26
    mov x3, x25
    bl L268
    mov x25, x0
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_41-25:
label_41:
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 103755
    ldr x2, [L269]
    bl L117
    b.ne label_39
    mov x28, x0
# line_I
# i_minus_jIssd
    mov x2, 31
    subs x0, x27, 16
    and x8, x27, 15
# test for not overflow and small operands
    ccmp x8, 15, 0, 9
    b.eq L270
    mov x1, x27
    bl L272
L270:
    mov x27, x0
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L273
    mov x3, 4
    bl L120
L273:
# put_list_ssd
    stp x26, x28, [x23], 16
    sub x26, x23, 15
# update_map_assoc_sdtI
    adr x4, L274
    b L275
L274:
.byte 0x4B, 0x95, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0xCB, 0x54, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
L275:
    mov x28, x25
    mov x2, 3
    mov x3, 4
    bl L277
    mov x25, x0
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_42:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:replay_buffer/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x55, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@replay_buffer/1-24:
replay_buffer/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L278
    bl L112
L278:
# i_test_yield
    adr x2, replay_buffer/1
    subs w22, w22, 1
    b.le L114
# i_get_map_elements_fsI
    mov x0, x25
# simplified multi-element lookup
    and x8, x0, -8
    ldp x9, x10, [x8]
    and x9, x9, 252
    cmp x9, 44
    b.ne L279
    add x10, x10, 1
    ldr x9, [x8, 16]!
    and x9, x9, -8
L281:
    subs x10, x10, 1
    b.eq label_42
    ldr x11, [x9, x10 lsl 3]
    mov x14, 414987
    cmp x11, x14
    b.ne L281
    ldr x27, [x8, x10 lsl 3]
L282:
    subs x10, x10, 1
    b.eq label_42
    ldr x11, [x9, x10 lsl 3]
    mov x14, 103755
    cmp x11, x14
    b.ne L282
    ldr x26, [x8, x10 lsl 3]
    b L280
L279:
    adr x4, L283
    b L284
L283:
.byte 0x4B, 0x95, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0xC7, 0xD6, 0xE8, 0x66, 0xA2, 0x74, 0x19, 0xD6
.byte 0x0B, 0x55, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0xD2, 0x99, 0xCA, 0x50, 0x1B, 0xA6, 0xD2, 0x07
L284:
    mov x2, x20
    stp x25, x26, [x19, 64]
    stp x27, x28, [x19, 80]
    stp x15, x16, [x19, 96]
    mov x3, 2
    add x1, x19, 64
    bl L198
    ldp x25, x26, [x19, 64]
    ldp x27, x28, [x19, 80]
    ldp x15, x16, [x19, 96]
    cbz x0, label_42
L280:
# allocate_heap_tIt
    add x2, x23, 64
    cmp x2, x20
    b.ls L285
    mov x3, 3
    bl L120
L285:
    sub x20, x20, 16
# i_move_sd
    str x26, [x20, 8]
# i_move_sd
    ldr x14, [L286]
    str x14, [x20]
# i_move_sd
    mov x25, x27
# line_I
# i_call_f
    bl @drop_msg/1-26
# i_move_sd
    mov x26, x25
# move_two_trim_ydydt
    ldp x8, x25, [x20], 8
    str x8, [x20]
# call_light_bif_be
L288:
    ldr x3, [L289]
    ldr x7, [L290]
    adr x2, L288
# BIF: lists:reverse/2
    bl L125
# i_move_sd
    mov x26, x25
# move_call_ext_last_ydet
    ldr x0, [L291]
    ldp x25, x30, [x20], 16
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
label_44:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:drop_msg/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x55, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@drop_msg/1-26:
drop_msg/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L292
    bl L112
L292:
# i_test_yield
    adr x2, drop_msg/1
    subs w22, w22, 1
    b.le L114
# is_eq_exact_fss
    cmp x25, 15
    b.ne @label_46-27
# i_move_sd
    mov x25, 59
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_46-27:
label_46:
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L294
    mov x3, 1
    bl L120
L294:
    sub x20, x20, 8
# i_move_sd
    str x25, [x20]
# line_I
# i_call_ext_e
    ldr x0, [L217]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_new_small_map_lit_dtqI
    add x2, x23, 72
    cmp x2, x20
    b.ls L295
    mov x3, 1
    bl L120
L295:
    add x8, x23, 2
    mov x9, 300
    mov x10, 1
    stp x9, x10, [x23], 16
    ldr x9, [L219]
    stp x9, x25, [x23], 16
    mov x25, x8
# test_heap_It
    add x2, x23, 72
    cmp x2, x20
    b.ls L296
    mov x3, 1
    bl L120
L296:
# put_list_ssd
    ldr x8, [x20]
    mov x9, 59
    stp x8, x9, [x23], 16
    sub x26, x23, 15
# put_tuple2_SA
    mov x9, 128
    ldr x10, [L297]
    stp x9, x10, [x23], 16
    str x26, [x23], 8
    sub x26, x23, 22
# i_new_small_map_lit_dtqI
    add x2, x23, 88
    cmp x2, x20
    b.ls L298
    mov x3, 2
    bl L120
L298:
    add x8, x23, 2
    mov x9, 300
    mov x10, 3
    stp x9, x10, [x23], 16
    ldr x9, [L221]
    stp x9, x25, [x23], 16
    mov x10, 22091
    stp x26, x10, [x23], 16
    mov x25, x8
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L299
    mov x3, 1
    bl L120
L299:
# put_list_deallocate_ssdt
    mov x9, 59
    stp x25, x9, [x23], 16
    sub x25, x23, 15
    add x20, x20, 8
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_flush_stubs
# i_func_label_L
label_47:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:log_internal/2
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x55, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@log_internal/2-15:
log_internal/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L300
    bl L112
L300:
# i_test_yield
    adr x2, log_internal/2
    subs w22, w22, 1
    b.le L114
# is_eq_exact_fss
    mov x14, 408203
    cmp x25, x14
    b.ne @label_49-28
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L302
    mov x3, 2
    bl L120
L302:
# i_move_sd
    mov x25, x26
# line_I
# i_call_f
    bl @display_log/1-29
# i_move_sd
    mov x25, 408203
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_49-28:
label_49:
# allocate_heap_tIt
    add x2, x23, 96
    cmp x2, x20
    b.ls L304
    mov x3, 2
    bl L120
L304:
    sub x20, x20, 40
# init_yregs_I
    mov x8, 59
    stp x8, x8, [x20]
# store_two_values_sdsd
    stp x26, x25, [x20, 24]
# i_make_fun3_FStt
    ldr x9, [L305]
# Create fun thing
    mov x8, 65556
    stp x8, x9, [x23]
# Move fun environment
    str x26, [x23, 16]
# Create boxed ptr
    orr x25, x23, 2
    add x23, x23, 24
# recv_marker_reserve_S
    stp x23, x20, [x21, 80]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L168
    ldp x23, x20, [x21, 80]
    ldp x15, x16, [x19, 96]
    str x0, [x20, 16]
# line_I
# i_call_ext_e
    ldr x0, [L306]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x8, [x0, 16]
    str x8, [x20, 8]
# i_move_sd
    str x25, [x20]
# recv_marker_bind_SS
    ldp x2, x1, [x20, 8]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L173
    ldp x15, x16, [x19, 96]
# recv_marker_use_S
    ldr x1, [x20, 8]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L178
    ldp x15, x16, [x19, 96]
# aligned_label_Lt
label_50:
# i_loop_rec_f
L307:
    adr x0, L307
    ldr x1, [L308]
    bl L135
# i_is_tagged_tuple_fsAa
    tbnz x25, 0, @label_52-30
    and x0, x25, -8
    ldp x8, x9, [x0]
    cmp x9, 1355
    mov x10, 320
    ccmp x8, x10, 0, 2
    b.ne @label_52-30
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# i_get_tuple_element_sPS
    ldr x25, [x0, 40]
# is_eq_exact_fss
    cmp x25, 523
    b.ne @label_51-31
# is_eq_exact_fss
    ldr x1, [x20, 8]
    cmp x26, x1
    b.eq L311
    tbnz x26, 0, @label_52-30
    mov x0, x26
    stp x15, x16, [x19, 96]
    bl L141
    ldp x15, x16, [x19, 96]
    cbz w0, @label_52-30
L311:
# jump_f
    b @label_55-32
# label_L
@label_51-31:
label_51:
# is_eq_exact_fss
    ldr x1, [x20, 8]
    cmp x26, x1
    b.eq L313
    tbnz x26, 0, @label_52-30
    mov x0, x26
    stp x15, x16, [x19, 96]
    bl L141
    ldp x15, x16, [x19, 96]
    cbz w0, @label_52-30
L313:
# recv_marker_clear_S
    ldr x1, [x20, 8]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L185
    ldp x15, x16, [x19, 96]
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# move_trim_sdt
    ldr x25, [x20, 24]
    add x20, x20, 32
# line_I
# i_call_f
    bl @display_log/1-29
# move_deallocate_return
    ldp x25, x30, [x20], 16
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_52-30:
label_52:
# loop_rec_end_f
    mov x0, x21
    bl L154
    sub w22, w22, 1
    b label_50
# aligned_label_Lt
label_53:
# wait_timeout_locked_sf
    mov x1, 4815
    mov x0, x21
    adr x2, L315
    bl L317
    cmp x0, 1
    b.eq L314
    b.lt L315
    adr x1, label_53
    b L162
L314:
    mov x0, x21
    ldr x1, [L318]
    bl L157
    b L159
L315:
# timeout
    mov x0, x21
    bl L320
# load_tuple_ptr_s
    ldr x8, [x20]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x25, [x0, 8]
# i_move_sd
    mov x26, 23499
# init_yregs_I
    mov x8, 59
    str x8, [x20]
    str x8, [x20, 16]
# line_I
# call_light_bif_be
L321:
    ldr x3, [L322]
    ldr x7, [L323]
    adr x2, L321
# BIF: erlang:exit/2
    bl L125
# recv_marker_use_S
    ldr x1, [x20, 8]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L178
    ldp x15, x16, [x19, 96]
# aligned_label_Lt
label_54:
# i_loop_rec_f
L324:
    adr x0, L324
    ldr x1, [L325]
    bl L135
# i_is_tagged_tuple_fsAa
    tbnz x25, 0, @label_57-33
    and x0, x25, -8
    ldp x8, x9, [x0]
    cmp x9, 1355
    mov x10, 320
    ccmp x8, x10, 0, 2
    b.ne @label_57-33
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# i_get_tuple_element_sPS
    ldr x25, [x0, 40]
# is_eq_exact_fss
    cmp x25, 523
    b.ne @label_56-34
# is_eq_exact_fss
    ldr x1, [x20, 8]
    cmp x26, x1
    b.eq L328
    tbnz x26, 0, @label_57-33
    mov x0, x26
    stp x15, x16, [x19, 96]
    bl L141
    ldp x15, x16, [x19, 96]
    cbz w0, @label_57-33
L328:
# label_L
@label_55-32:
label_55:
# recv_marker_clear_S
    ldr x1, [x20, 8]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L185
    ldp x15, x16, [x19, 96]
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# i_move_sd
    ldr x25, [x20, 32]
# deallocate_t
    add x20, x20, 40
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_56-34:
label_56:
# is_eq_exact_fss
    ldr x1, [x20, 8]
    cmp x26, x1
    b.eq L329
    tbnz x26, 0, @label_57-33
    mov x0, x26
    stp x15, x16, [x19, 96]
    bl L141
    ldp x15, x16, [x19, 96]
    cbz w0, @label_57-33
L329:
# recv_marker_clear_S
    ldr x1, [x20, 8]
    stp x15, x16, [x19, 96]
    mov x0, x21
    bl L185
    ldp x15, x16, [x19, 96]
# remove_message
    mov x2, x23
    mov x3, x20
    stp x15, x16, [x19, 96]
    mov x0, x21
    mov w1, w22
    mov x4, x24
    bl L145
    mov w22, w0
    ldp x15, x16, [x19, 96]
# move_trim_sdt
    ldr x25, [x20, 24]
    add x20, x20, 40
# line_I
# i_call_f
    bl @display_log/1-29
# i_move_sd
    mov x25, 408203
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_57-33:
label_57:
# loop_rec_end_f
    mov x0, x21
    bl L154
    sub w22, w22, 1
    b label_54
# aligned_label_Lt
label_58:
# wait_locked_f
    mov x0, x21
    ldr x1, [L330]
    bl L157
    b L159
# i_flush_stubs
# i_func_label_L
    align 8
label_59:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:display_log/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x56, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@display_log/1-29:
display_log/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L331
    bl L112
L331:
# i_test_yield
    adr x2, display_log/1
    subs w22, w22, 1
    b.le L114
# i_get_map_elements_fsI
    mov x0, x25
# simplified multi-element lookup
    and x8, x0, -8
    ldp x9, x10, [x8]
    and x9, x9, 252
    cmp x9, 44
    b.ne L332
    add x10, x10, 1
    ldr x9, [x8, 16]!
    and x9, x9, -8
L334:
    subs x10, x10, 1
    b.eq label_59
    ldr x11, [x9, x10 lsl 3]
    mov x14, 133131
    cmp x11, x14
    b.ne L334
    ldr x27, [x8, x10 lsl 3]
L335:
    subs x10, x10, 1
    b.eq label_59
    ldr x11, [x9, x10 lsl 3]
    mov x14, 26891
    cmp x11, x14
    b.ne L335
    ldr x26, [x8, x10 lsl 3]
    b L333
L332:
    adr x4, L336
    b L337
    align 8
L336:
.byte 0x0B, 0x69, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0xDC, 0x7F, 0x95, 0xD4, 0xB9, 0xD8, 0xE8, 0x37
.byte 0x0B, 0x08, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x0F, 0xD6, 0xCB, 0x46, 0xF8, 0x05, 0x4F, 0xF1
L337:
    mov x2, x20
    stp x25, x26, [x19, 64]
    stp x27, x28, [x19, 80]
    stp x15, x16, [x19, 96]
    mov x3, 2
    add x1, x19, 64
    bl L198
    ldp x25, x26, [x19, 64]
    ldp x27, x28, [x19, 80]
    ldp x15, x16, [x19, 96]
    cbz x0, label_59
L333:
# is_map_fs
    tbnz x26, 0, label_59
    ldur x10, [x26, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne label_59
# i_get_map_element_hash_fScWS
    mov x0, x26
    mov x1, 52491
    ldr x2, [L209]
    bl L117
    b.ne label_59
    mov x28, x0
# i_get_map_element_hash_fScWS
    mov x0, x26
    mov x1, 15691
    ldr x2, [L190]
    bl L117
    b.ne @label_61-35
    mov x25, x0
# is_map_fs
    tbnz x25, 0, @label_61-36
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_61-35
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 45771
    ldr x2, [L340]
    bl L117
    b.ne @label_61-35
    mov x25, x0
# i_is_tagged_tuple_fsAa
    tbnz x27, 0, @label_61-36
    and x0, x27, -8
    ldp x8, x9, [x0]
    mov x14, 145931
    cmp x9, x14
    mov x10, 128
    ccmp x8, x10, 0, 2
    b.ne @label_61-35
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L341
    mov x3, 4
    bl L120
L341:
    sub x20, x20, 16
# store_two_values_sdsd
    stp x25, x27, [x20]
# i_move_sd
    mov x25, x28
# line_I
# i_call_f
    bl @display_date/1-37
# load_tuple_ptr_s
    ldr x8, [x20, 8]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# move_call_last_ydft
    ldr x25, [x20], 16
    ldr x30, [x20], 8
    b @display_report/2-38
# label_L
@label_61-35:
@label_61-36:
label_61:
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L344
    mov x3, 4
    bl L120
L344:
    sub x20, x20, 8
# i_move_sd
    str x27, [x20]
# i_move_sd
    mov x25, x28
# line_I
# i_call_f
    bl @display_date/1-37
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b @display/1-39
# i_flush_stubs
# i_func_label_L
label_62:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:display_date/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x56, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@display_date/1-37:
display_date/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L346
    bl L112
L346:
# i_test_yield
    adr x2, display_date/1
    subs w22, w22, 1
    b.le L114
# is_integer_fs
    and x9, x25, 15
    cmp x9, 15
    b.eq L347
    tbnz x9, 0, label_62
    ldur x8, [x25, -2]
    and x8, x8, 56
    cmp x8, 8
    b.ne label_62
L347:
# line_I
# i_rem_div_jIssdd
    mov x2, 16000015
    and x13, x25, 15
    cmp x13, 15
    b.ne L348
    asr x8, x25, 4
    mov x9, 1000000
    sdiv x0, x8, x9
    msub x1, x0, x9, x8
    orr x0, x13, x0, 4
    orr x1, x13, x1, 4
    b L349
L348:
    mov x1, x25
    mov x3, 4368875144
    bl L351
L349:
    mov x25, x0
    mov x26, x1
# allocate_tt
    add x2, x23, 80
    cmp x2, x20
    b.ls L352
    mov x3, 2
    bl L120
L352:
    sub x20, x20, 48
# init_yregs_I
    movi v0.2d, -1
    stp q0, q0, [x20]
    str d0, [x20, 32]
# i_move_sd
    str x26, [x20, 40]
# line_I
# call_light_bif_be
L353:
    ldr x3, [L354]
    ldr x7, [L355]
    adr x2, L353
# BIF: erlang:posixtime_to_universaltime/1
    bl L125
# line_I
# call_light_bif_be
L356:
    ldr x3, [L357]
    ldr x7, [L358]
    adr x2, L356
# BIF: erlang:universaltime_to_localtime/1
    bl L125
# i_is_tuple_of_arity_fsA
    tbnz x25, 0, @label_64-40
    and x0, x25, -8
    ldr x8, [x0]
    cmp x8, 128
    b.ne @label_64-40
# i_get_tuple_element_sPS
    ldr x8, [x0, 8]
    str x8, [x20, 32]
# i_is_tuple_of_arity_fsA
# simplified fetching of BEAM register
    mov x0, x8
    tbnz x0, 0, @label_64-40
    and x0, x0, -8
    ldr x8, [x0]
    cmp x8, 192
    b.ne @label_64-40
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x8, [x0, 16]
    str x8, [x20, 24]
# i_is_tuple_of_arity_fsA
# simplified fetching of BEAM register
    mov x0, x8
    tbnz x0, 0, @label_64-40
    and x0, x0, -8
    ldr x8, [x0]
    cmp x8, 192
    b.ne @label_64-40
# load_tuple_ptr_s
    ldr x8, [x20, 32]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x25, [x0, 8]
# line_I
# call_light_bif_be
L360:
    ldr x3, [L361]
    ldr x7, [L362]
    adr x2, L360
# BIF: erlang:integer_to_list/1
    bl L125
# load_tuple_ptr_s
    ldr x8, [x20, 32]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# i_move_sd
    str x25, [x20, 16]
# i_move_sd
    mov x25, x26
# i_move_sd
    mov x26, 47
# line_I
# i_call_f
    bl @pad/2-41
# load_tuple_ptr_s
    ldr x8, [x20, 32]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 24]
# i_move_sd
    str x25, [x20, 32]
# i_move_sd
    mov x25, x26
# i_move_sd
    mov x26, 47
# line_I
# i_call_f
    bl @pad/2-41
# load_tuple_ptr_s
    ldr x8, [x20, 24]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 8]
# i_move_sd
    str x25, [x20, 8]
# i_move_sd
    mov x25, x26
# i_move_sd
    mov x26, 47
# line_I
# i_call_f
    bl @pad/2-41
# load_tuple_ptr_s
    ldr x8, [x20, 24]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# i_move_sd
    str x25, [x20]
# i_move_sd
    mov x25, x26
# i_move_sd
    mov x26, 47
# line_I
# i_call_f
    bl @pad/2-41
# load_tuple_ptr_s
    ldr x8, [x20, 24]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 24]
# i_move_sd
    str x25, [x20, 24]
# i_move_sd
    mov x25, x26
# i_move_sd
    mov x26, 47
# line_I
# i_call_f
    bl @pad/2-41
# swap_dd
    ldr x8, [x20, 40]
    str x25, [x20, 40]
    mov x25, x8
# i_move_sd
    mov x26, 111
# line_I
# i_call_f
    bl @pad/2-41
# i_move_sd
    ldr x26, [L364]
# call_light_bif_be
L365:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L365
# BIF: erlang:'++'/2
    bl L125
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L368
    mov x3, 1
    bl L120
L368:
# put_list_ssd
    mov x8, 751
    stp x8, x25, [x23], 16
    sub x26, x23, 15
# i_move_sd
    ldr x25, [x20, 40]
# i_move_sd
    mov x14, 59
    str x14, [x20, 40]
# line_I
# call_light_bif_be
L369:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L369
# BIF: erlang:'++'/2
    bl L125
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L370
    mov x3, 1
    bl L120
L370:
# put_list_ssd
    mov x8, 943
    stp x8, x25, [x23], 16
    sub x26, x23, 15
# i_move_sd
    ldr x25, [x20, 24]
# i_move_sd
    mov x14, 59
    str x14, [x20, 24]
# line_I
# call_light_bif_be
L371:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L371
# BIF: erlang:'++'/2
    bl L125
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L372
    mov x3, 1
    bl L120
L372:
# put_list_ssd
    mov x8, 943
    stp x8, x25, [x23], 16
    sub x26, x23, 15
# move_trim_sdt
    ldr x25, [x20], 8
# line_I
# call_light_bif_be
L373:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L373
# BIF: erlang:'++'/2
    bl L125
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L374
    mov x3, 1
    bl L120
L374:
# put_list_ssd
    mov x8, 527
    stp x8, x25, [x23], 16
    sub x26, x23, 15
# move_trim_sdt
    ldr x25, [x20], 8
# line_I
# call_light_bif_be
L375:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L375
# BIF: erlang:'++'/2
    bl L125
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L376
    mov x3, 1
    bl L120
L376:
# put_list_ssd
    mov x8, 735
    stp x8, x25, [x23], 16
    sub x26, x23, 15
# i_move_sd
    ldr x25, [x20, 16]
# i_move_sd
    mov x14, 59
    str x14, [x20, 16]
# line_I
# call_light_bif_be
L377:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L377
# BIF: erlang:'++'/2
    bl L125
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L378
    mov x3, 1
    bl L120
L378:
# put_list_ssd
    mov x8, 735
    stp x8, x25, [x23], 16
    sub x26, x23, 15
# move_trim_sdt
    ldr x25, [x20], 32
# line_I
# call_light_bif_be
L379:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L379
# BIF: erlang:'++'/2
    bl L125
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 85579
# line_I
# call_light_bif_be
L380:
    ldr x3, [L381]
    ldr x7, [L382]
    adr x2, L380
# BIF: erlang:display_string/2
    bl L125
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_64-40:
label_64:
# line_I
# badmatch_s
    mov x8, 5200
    stp x8, x25, [x21, 96]
    bl L162
# i_flush_stubs
# i_func_label_L
    nop
    align 8
label_65:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:pad/2
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x56, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@pad/2-41:
pad/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L383
    bl L112
L383:
# i_test_yield
    adr x2, pad/2
    subs w22, w22, 1
    b.le L114
# is_integer_fs
    and x9, x25, 15
    cmp x9, 15
    b.eq L384
    tbnz x9, 0, @label_67-42
    ldur x8, [x25, -2]
    and x8, x8, 56
    cmp x8, 8
    b.ne @label_67-42
L384:
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L386
    mov x3, 2
    bl L120
L386:
    sub x20, x20, 8
# i_move_sd
    str x26, [x20]
# line_I
# call_light_bif_be
L387:
    ldr x3, [L361]
    ldr x7, [L362]
    adr x2, L387
# BIF: erlang:integer_to_list/1
    bl L125
# move_call_last_ydft
    ldp x26, x30, [x20], 16
    b pad/2
# label_L
@label_67-42:
label_67:
# i_length_setup_jts
    mov x28, 15
    mov x27, x25
# i_length_jtd
L388:
    mov x1, 2
    adr x2, L388
    bl L390
    cbz x0, @label_68-43
    mov x27, x0
# is_eq_exact_fss
# simplified check since one argument is an immediate
    cmp x27, x26
    b.ne @label_68-43
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_68-43:
label_68:
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L392
    mov x3, 2
    bl L120
L392:
# put_list_ssd
    mov x8, 783
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# i_call_only_f
    ldr x30, [x20], 8
    b pad/2
# i_flush_stubs
# i_func_label_L
label_69:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:display/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0xBF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@display/1-39:
display/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L393
    bl L112
L393:
# i_test_yield
    adr x2, display/1
    subs w22, w22, 1
    b.le L114
# i_is_tuple_of_arity_fsA
    tbnz x25, 0, label_69
    and x0, x25, -8
    ldr x8, [x0]
    cmp x8, 128
    b.ne label_69
# get_two_tuple_elements_sPSS
    ldp x26, x27, [x0, 8]
# i_select_val_lins_sfI
    mov x14, 63051
    cmp x26, x14
    b.eq @label_71-44
    mov x14, 145931
    cmp x26, x14
    b.eq @label_73-45
    b L396
# label_L
@label_71-44:
label_71:
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L397
    mov x3, 3
    bl L120
L397:
    sub x20, x20, 16
# i_move_sd
    str x27, [x20]
# catch_yH
    ldr x8, [x21, 248]
    add x8, x8, 1
    str x8, [x21, 248]
    ldr x14, [L398]
    str x14, [x20, 8]
# i_move_sd
    mov x25, x27
# line_I
# i_call_ext_e
    ldr x0, [L399]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# try_end_y
    ldr x8, [x21, 248]
    sub x8, x8, 1
    str x8, [x21, 248]
    mov x8, 59
    str x8, [x20, 8]
# move_trim_sdt
    mov x26, x25
    add x20, x20, 16
# i_move_sd
    mov x25, 85579
# line_I
# call_light_bif_be
L400:
    ldr x3, [L381]
    ldr x7, [L382]
    adr x2, L400
# BIF: erlang:display_string/2
    bl L125
# i_move_sd
    ldr x26, [L401]
# i_move_sd
    mov x25, 85579
# line_I
# call_light_bif_be
L402:
    ldr x3, [L381]
    ldr x7, [L382]
    adr x2, L402
# BIF: erlang:display_string/2
    bl L125
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
label_72:
# try_case_y
    ldr x8, [x21, 248]
    mov x25, x28
    sub x8, x8, 1
    str x8, [x21, 248]
# i_move_sd
    ldr x25, [x20]
# line_I
# call_light_bif_be
L403:
    ldr x3, [L404]
    ldr x7, [L405]
    adr x2, L403
# BIF: erlang:display/1
    bl L125
# deallocate_t
    add x20, x20, 16
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_73-45:
label_73:
# is_map_fs
    tbnz x27, 0, @label_74-46
    ldur x10, [x27, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_74-46
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L407
    mov x3, 3
    bl L120
L407:
# i_move_sd
    mov x25, x27
# line_I
# i_call_ext_e
    ldr x0, [L408]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_call_last_ft
    ldr x30, [x20], 8
    b @display_report/1-47
# label_L
@label_74-46:
label_74:
# i_move_sd
    mov x25, x27
# i_call_only_f
    ldr x30, [x20], 8
    b @display_report/1-47
# label_L
L396:
label_75:
# is_list_fs
    tst x26, 2
    mov x14, 59
    ccmp x26, x14, 4, 3
    b.ne label_69
# is_list_fs
    tst x27, 2
    mov x14, 59
    ccmp x27, x14, 4, 3
    b.ne label_69
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L410
    mov x3, 3
    bl L120
L410:
    sub x20, x20, 8
# i_move_sd
    str x27, [x20]
# i_move_sd
    mov x25, x26
# i_move_sd
    ldr x26, [L401]
# line_I
# call_light_bif_be
L411:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L411
# BIF: erlang:'++'/2
    bl L125
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 85579
# call_light_bif_be
L412:
    ldr x3, [L381]
    ldr x7, [L382]
    adr x2, L412
# BIF: erlang:display_string/2
    bl L125
# move_trim_sdt
    ldr x25, [x20], 8
# line_I
# i_call_f
    bl @'-display/1-lc$^0/1-0-'/1-48
# i_move_sd
    mov x25, 32139
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_flush_stubs
# i_func_label_L
label_76:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:display_report/2
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x56, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@display_report/2-38:
display_report/2:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L414
    bl L112
L414:
# i_test_yield
    adr x2, display_report/2
    subs w22, w22, 1
    b.le L114
# is_atom_fs
    and x8, x25, 63
    cmp x8, 11
    b.ne @label_78-49
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L416
    mov x3, 2
    bl L120
L416:
    sub x20, x20, 16
# store_two_values_sdsd
    mov x8, 59
    stp x8, x26, [x20]
# line_I
# call_light_bif_be
L417:
    ldr x3, [L418]
    ldr x7, [L419]
    adr x2, L417
# BIF: erlang:atom_to_list/1
    bl L125
# line_I
# i_length_setup_jts
    mov x27, 15
    mov x26, x25
    mov x28, x25
# i_length_jtd
L420:
    mov x1, 1
    adr x2, L420
    bl L422
    mov x26, x0
# line_I
# i_minus_jIssd
    mov x1, 335
# subtraction without overflow check
    and x8, x26, -16
    sub x26, x1, x8
# i_move_sd
    str x25, [x20]
# i_move_sd
    mov x25, x26
# i_move_sd
    mov x26, 527
# i_call_ext_e
    ldr x0, [L423]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x26, x25
# move_trim_sdt
    ldr x25, [x20], 8
# line_I
# call_light_bif_be
L424:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L424
# BIF: erlang:'++'/2
    bl L125
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 85579
# call_light_bif_be
L425:
    ldr x3, [L381]
    ldr x7, [L382]
    adr x2, L425
# BIF: erlang:display_string/2
    bl L125
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b @display_report/1-47
# label_L
@label_78-49:
label_78:
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L426
    mov x3, 2
    bl L120
L426:
# put_tuple2_SA
    mov x9, 128
    stp x9, x25, [x23], 16
    str x26, [x23], 8
    sub x25, x23, 22
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L427
    mov x3, 1
    bl L120
L427:
# call_light_bif_be
L428:
    ldr x3, [L404]
    ldr x7, [L405]
    adr x2, L428
# BIF: erlang:display/1
    bl L125
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_flush_stubs
# i_func_label_L
label_79:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:display_report/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x56, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@display_report/1-47:
display_report/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L429
    bl L112
L429:
# i_test_yield
    adr x2, display_report/1
    subs w22, w22, 1
    b.le L114
# is_nonempty_list_fS
    tbnz x25, 1, @label_83-50
# get_list_Sdd
    and x8, x25, -8
    ldp x26, x27, [x8]
# is_eq_exact_fss
# inlined equality test with [[]]
    tbnz x27, 1, @label_81-51
    sub x8, x27, 1
    ldp x9, x10, [x8]
    cmp x9, 59
    mov x11, 59
    ccmp x10, x11, 0, 2
    b.ne @label_81-51
# i_move_sd
    mov x25, x26
# i_call_only_f
    ldr x30, [x20], 8
    b display_report/1
# label_L
@label_81-51:
label_81:
# allocate_heap_tIt
    add x2, x23, 56
    cmp x2, x20
    b.ls L432
    mov x3, 1
    bl L120
L432:
    sub x20, x20, 8
# i_move_sd
    str x25, [x20]
# i_move_sd
    ldr x25, [L433]
# i_move_sd
    ldr x26, [x20]
# line_I
# i_call_ext_e
    ldr x0, [L434]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# is_eq_exact_fss
    cmp x25, 75
    b.ne @label_82-52
# i_move_sd
    ldr x26, [L401]
# i_move_sd
    mov x25, 85579
# line_I
# call_light_bif_be
L436:
    ldr x3, [L381]
    ldr x7, [L382]
    adr x2, L436
# BIF: erlang:display_string/2
    bl L125
# test_heap_It
    add x2, x23, 48
    cmp x2, x20
    b.ls L437
    mov x3, xzr
    bl L120
L437:
# i_move_sd
    ldr x25, [L438]
# i_move_sd
    ldr x26, [x20]
# line_I
# i_call_ext_last_et
    add x20, x20, 8
    ldr x0, [L291]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# label_L
@label_82-52:
label_82:
# i_move_sd
    ldr x25, [x20]
# line_I
# call_light_bif_be
L439:
    ldr x3, [L404]
    ldr x7, [L405]
    adr x2, L439
# BIF: erlang:display/1
    bl L125
# deallocate_t
    add x20, x20, 8
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_83-50:
label_83:
# is_map_fs
    tbnz x25, 0, @label_84-53
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne @label_84-53
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 145931
    ldr x2, [L441]
    bl L117
    b.ne @label_84-53
    mov x26, x0
# i_move_sd
    mov x25, x26
# i_call_only_f
    ldr x30, [x20], 8
    b display_report/1
# label_L
@label_84-53:
label_84:
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L442
    mov x3, 1
    bl L120
L442:
# call_light_bif_be
L443:
    ldr x3, [L404]
    ldr x7, [L405]
    adr x2, L443
# BIF: erlang:display/1
    bl L125
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_flush_stubs
# i_func_label_L
label_85:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:module_info/0
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/0:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L444
    bl L112
L444:
# i_test_yield
    adr x2, module_info/0
    subs w22, w22, 1
    b.le L114
# i_move_sd
    mov x25, 215627
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L445
    mov x3, 1
    bl L120
L445:
# call_light_bif_be
L446:
    ldr x3, [L447]
    ldr x7, [L448]
    adr x2, L446
# BIF: erlang:get_module_info/1
    bl L125
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_87:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:module_info/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
module_info/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L449
    bl L112
L449:
# i_test_yield
    adr x2, module_info/1
    subs w22, w22, 1
    b.le L114
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 215627
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L450
    mov x3, 2
    bl L120
L450:
# call_light_bif_be
L451:
    ldr x3, [L452]
    ldr x7, [L453]
    adr x2, L451
# BIF: erlang:get_module_info/2
    bl L125
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_flush_stubs
# i_func_label_L
label_89:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:'-display_report/1-fun-1-'/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x57, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
'-display_report/1-fun-1-'/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L454
    bl L112
L454:
# i_test_yield
    adr x2, '-display_report/1-fun-1-'/1
    subs w22, w22, 1
    b.le L114
# i_is_tuple_of_arity_fsA
    tbnz x25, 0, label_89
    and x0, x25, -8
    ldr x8, [x0]
    cmp x8, 128
    b.ne label_89
# allocate_tt
    add x2, x23, 40
    cmp x2, x20
    b.ls L455
    mov x3, 1
    bl L120
L455:
    sub x20, x20, 8
# i_move_sd
    str x25, [x20]
# load_tuple_ptr_s
    and x0, x25, -8
# i_get_tuple_element_sPS
    ldr x25, [x0, 8]
# line_I
# call_light_bif_be
L456:
    ldr x3, [L418]
    ldr x7, [L419]
    adr x2, L456
# BIF: erlang:atom_to_list/1
    bl L125
# i_move_sd
    ldr x26, [L457]
# call_light_bif_be
L458:
    ldr x3, [L366]
    ldr x7, [L367]
    adr x2, L458
# BIF: erlang:'++'/2
    bl L125
# test_heap_It
    add x2, x23, 96
    cmp x2, x20
    b.ls L459
    mov x3, 1
    bl L120
L459:
# put_list_ssd
    mov x8, 527
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 527
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 527
    stp x8, x25, [x23], 16
    sub x25, x23, 15
# put_list_ssd
    mov x8, 527
    stp x8, x25, [x23], 16
    sub x26, x23, 15
# i_move_sd
    mov x25, 85579
# line_I
# call_light_bif_be
L460:
    ldr x3, [L381]
    ldr x7, [L382]
    adr x2, L460
# BIF: erlang:display_string/2
    bl L125
# load_tuple_ptr_s
    ldr x8, [x20]
    and x0, x8, -8
# i_get_tuple_element_sPS
    ldr x25, [x0, 16]
# line_I
# call_light_bif_be
L461:
    ldr x3, [L404]
    ldr x7, [L405]
    adr x2, L461
# BIF: erlang:display/1
    bl L125
# deallocate_t
    add x20, x20, 8
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_flush_stubs
# i_func_label_L
label_91:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:'-display_report/1-fun-0-'/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x57, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
'-display_report/1-fun-0-'/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L462
    bl L112
L462:
# i_test_yield
    adr x2, '-display_report/1-fun-0-'/1
    subs w22, w22, 1
    b.le L114
# i_is_tuple_of_arity_fsA
    tbnz x25, 0, @label_93-54
    and x0, x25, -8
    ldr x8, [x0]
    cmp x8, 128
    b.ne @label_93-54
# i_get_tuple_element_sPS
    ldr x25, [x0, 8]
# nofail_bif1_sbd
    str x25, [x19, 64]
# UBIF: is_atom/1
    ldr x3, [L464]
    bl L466
    mov x25, x0
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_93-54:
label_93:
# i_move_sd
    mov x25, 11
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_flush_stubs
# i_func_label_L
    align 8
label_94:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:'-display/1-lc$^0/1-0-'/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x57, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
@'-display/1-lc$^0/1-0-'/1-48:
'-display/1-lc$^0/1-0-'/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L467
    bl L112
L467:
# i_test_yield
    adr x2, '-display/1-lc$^0/1-0-'/1
    subs w22, w22, 1
    b.le L114
# is_nonempty_list_fS
    tbnz x25, 1, @label_96-55
# allocate_tt
    add x2, x23, 48
    cmp x2, x20
    b.ls L469
    mov x3, 1
    bl L120
L469:
    sub x20, x20, 16
# get_list_Sdd
    and x8, x25, -8
    ldp x9, x10, [x8]
    stp x10, x9, [x20]
# i_move_sd
    ldr x26, [L470]
# i_move_sd
    mov x25, 85579
# line_I
# call_light_bif_be
L471:
    ldr x3, [L381]
    ldr x7, [L382]
    adr x2, L471
# BIF: erlang:display_string/2
    bl L125
# move_two_trim_ydydt
    ldp x8, x25, [x20], 8
    str x8, [x20]
# line_I
# call_light_bif_be
L472:
    ldr x3, [L404]
    ldr x7, [L405]
    adr x2, L472
# BIF: erlang:display/1
    bl L125
# move_call_last_ydft
    ldp x25, x30, [x20], 16
    b '-display/1-lc$^0/1-0-'/1
# label_L
@label_96-55:
label_96:
# is_nil_fS
    cmp x25, 59
    b.ne @label_97-56
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# label_L
@label_97-56:
label_97:
# test_heap_It
    add x2, x23, 56
    cmp x2, x20
    b.ls L474
    mov x3, 1
    bl L120
L474:
# put_tuple2_SA
    mov x9, 128
    mov x10, 94923
    stp x9, x10, [x23], 16
    str x25, [x23], 8
    sub x25, x23, 22
# line_I
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L475
    mov x3, 1
    bl L120
L475:
# call_light_bif_be
L476:
    ldr x3, [L477]
    ldr x7, [L478]
    adr x2, L476
# BIF: erlang:error/1
    bl L125
# mark_unreachable
# i_flush_stubs
# i_func_label_L
    align 8
label_98:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:'-log_internal/2-fun-0-'/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x57, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
'-log_internal/2-fun-0-'/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L479
    bl L112
L479:
# i_test_yield
    adr x2, '-log_internal/2-fun-0-'/1
    subs w22, w22, 1
    b.le L114
# allocate_tt
    add x2, x23, 32
    cmp x2, x20
    b.ls L480
    mov x3, 1
    bl L120
L480:
# i_move_sd
    ldr x26, [L481]
# line_I
# i_call_ext_e
    ldr x0, [L482]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# line_I
# i_call_ext_e
    ldr x0, [L399]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_call_ext_e
    ldr x0, [L483]
    ldr x8, [x0, x24 lsl 3]
    blr x8
# i_move_sd
    mov x26, x25
# i_move_sd
    mov x25, 85579
# call_light_bif_be
L484:
    ldr x3, [L381]
    ldr x7, [L382]
    adr x2, L484
# BIF: erlang:display_string/2
    bl L125
# deallocate_t
# return
    ldr x30, [x20], 8
    subs w22, w22, 1
    b.mi L147
    ret x30
# i_lambda_trampoline_FfWW
L106:
    ldur x25, [x3, 14]
    b '-log_internal/2-fun-0-'/1
# i_flush_stubs
# i_func_label_L
label_100:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:'-replay_buffer/1-F/1-0-'/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x58, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
'-replay_buffer/1-F/1-0-'/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L485
    bl L112
L485:
# i_test_yield
    adr x2, '-replay_buffer/1-F/1-0-'/1
    subs w22, w22, 1
    b.le L114
# is_map_fs
    tbnz x25, 0, label_100
    ldur x10, [x25, -2]
    and x10, x10, 63
    cmp x10, 44
    b.ne label_100
# i_get_map_element_hash_fScWS
    mov x0, x25
    mov x1, 133131
    ldr x2, [L208]
    bl L117
    b.ne label_100
    mov x26, x0
# i_is_tuple_of_arity_fsA
    tbnz x26, 0, @label_103-57
    and x0, x26, -8
    ldr x8, [x0]
    cmp x8, 128
    b.ne @label_103-57
# i_get_tuple_element_sPS
    ldr x27, [x0, 8]
# i_select_val_lins_sfI
    mov x14, 63051
    cmp x27, x14
    mov x13, 145931
    ccmp x27, x13, 4, 3
    b.eq @label_102-58
    b @label_103-57
# label_L
@label_102-58:
label_102:
# load_tuple_ptr_s
    and x0, x26, -8
# i_get_tuple_element_sPS
    ldr x26, [x0, 16]
# line_I
# update_map_exact_sjdtI
    mov x1, 133131
    mov x2, x26
    mov x3, x25
    bl L489
    mov x25, x0
# i_call_only_f
    ldr x30, [x20], 8
    b '-replay_buffer/1-F/1-0-'/1
# label_L
@label_103-57:
label_103:
# i_get_map_elements_fsI
    mov x0, x25
# simplified multi-element lookup
    and x8, x0, -8
    ldp x9, x10, [x8]
    and x9, x9, 252
    cmp x9, 44
    b.ne L490
    add x10, x10, 1
    ldr x9, [x8, 16]!
    and x9, x9, -8
L492:
    subs x10, x10, 1
    b.eq label_100
    ldr x11, [x9, x10 lsl 3]
    mov x14, 137547
    cmp x11, x14
    b.ne L492
    ldr x27, [x8, x10 lsl 3]
L493:
    subs x10, x10, 1
    b.eq label_100
    ldr x11, [x9, x10 lsl 3]
    mov x14, 26891
    cmp x11, x14
    b.ne L493
    ldr x28, [x8, x10 lsl 3]
    b L491
L490:
    adr x4, L494
    b L495
L494:
.byte 0x0B, 0x69, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0xDC, 0x7F, 0x95, 0xD4, 0xB9, 0xD8, 0xE8, 0x37
.byte 0x4B, 0x19, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0xAB, 0x2C, 0x1C, 0x17, 0x1A, 0x07, 0x00, 0x52
L495:
    mov x2, x20
    stp x25, x26, [x19, 64]
    stp x27, x28, [x19, 80]
    stp x15, x16, [x19, 96]
    mov x3, 2
    add x1, x19, 64
    bl L198
    ldp x25, x26, [x19, 64]
    ldp x27, x28, [x19, 80]
    ldp x15, x16, [x19, 96]
    cbz x0, label_100
L491:
# i_move_sd
    mov x25, x27
# i_move_sd
    mov x27, x28
# line_I
# i_call_ext_only_e
    ldr x0, [L496]
    ldr x30, [x20], 8
    ldr x14, [x0, x24 lsl 3]
    br x14
# i_flush_stubs
# i_func_label_L
    align 8
label_104:
# func_line_I
# i_func_info_IaaI
# logger_simple_h:'-adding_handler/1-fun-0-'/1
    bl L109
.word 0x00000000
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.byte 0x4B, 0x4A, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x58, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
# aligned_label_Lt
'-adding_handler/1-fun-0-'/1:
# i_breakpoint_trampoline
    str x30, [x20, -8]!
    b L497
    bl L112
L497:
# i_test_yield
    adr x2, '-adding_handler/1-fun-0-'/1
    subs w22, w22, 1
    b.le L114
# i_call_only_f
    ldr x30, [x20], 8
    b init/1
# i_lambda_trampoline_FfWW
L105:
    ldur x25, [x3, 14]
    b '-adding_handler/1-fun-0-'/1
# int_code_end
L498:
    mov x0, 4369093202
    bl L500
# Begin stub section
L115:
.xword 0xA5EC2C39A06643F2
L122:
.xword 0x7FFFFFFFFFFFFFFF
L123:
.xword 0x000000010444F0FC
L128:
.xword 0x7FFFFFFFFFFFFFFF
L129:
.xword 0x7FFFFFFFFFFFFFFF
L130:
.xword 0x7FFFFFFFFFFFFFFF
L133:
.xword label_7
L150:
.xword 0x7FFFFFFFFFFFFFFF
L151:
.xword 0x000000010444BCA0
L155:
.xword label_3
L160:
.xword 0x7FFFFFFFFFFFFFFF
L170:
.xword 0x7FFFFFFFFFFFFFFF
L171:
.xword 0x000000010444C4E4
L175:
.xword 0x0000000104787C18
L176:
.xword 0x000000010444FFB0
L180:
.xword label_15
L186:
.xword label_13
L189:
.xword 0x37E8D8B9D4957FDC
L190:
.xword 0x89BC8732BDB4B2A1
L200:
.xword 0x7FFFFFFFFFFFFFFF
L208:
.xword 0xF14F05F846CBD60F
L209:
.xword 0xCC640649C8FCC59B
L213:
.xword 0xD5281CBC0887615F
L217:
.xword 0x7FFFFFFFFFFFFFFF
L219:
.xword 0x7FFFFFFFFFFFFFFF
L221:
.xword 0x7FFFFFFFFFFFFFFF
L222:
.xword 0x7FFFFFFFFFFFFFFF
L229:
.xword 0x7FFFFFFFFFFFFFFF
L230:
.xword 0x000000010444F060
L233:
.xword 0x7FFFFFFFFFFFFFFF
L238:
.xword label_38
L252:
.xword 0x7FFFFFFFFFFFFFFF
L257:
.xword 0x7FFFFFFFFFFFFFFF
L258:
.xword 0x000000010444D260
L259:
.xword label_33
L261:
.xword 0xA17D100EFBF44F21
L263:
.xword 0x07D2A61B50CA99D2
L269:
.xword 0xD61974A266E8D6C7
L286:
.xword 0x7FFFFFFFFFFFFFFF
L289:
.xword 0x7FFFFFFFFFFFFFFF
L290:
.xword 0x000000010442D64C
L291:
.xword 0x7FFFFFFFFFFFFFFF
L297:
.xword 0x7FFFFFFFFFFFFFFF
L305:
.xword 0x7FFFFFFFFFFFFFFF
L306:
.xword 0x7FFFFFFFFFFFFFFF
L308:
.xword label_53
L318:
.xword label_50
L322:
.xword 0x7FFFFFFFFFFFFFFF
L323:
.xword 0x000000010444E064
L325:
.xword label_58
L330:
.xword label_54
L340:
.xword 0x96A97EC966EEBF54
L354:
.xword 0x7FFFFFFFFFFFFFFF
L355:
.xword 0x00000001044521E4
L357:
.xword 0x7FFFFFFFFFFFFFFF
L358:
.xword 0x0000000104451F88
L361:
.xword 0x7FFFFFFFFFFFFFFF
L362:
.xword 0x0000000104450A20
L364:
.xword 0x7FFFFFFFFFFFFFFF
L366:
.xword 0x7FFFFFFFFFFFFFFF
L367:
.xword 0x000000010442CDE4
L381:
.xword 0x7FFFFFFFFFFFFFFF
L382:
.xword 0x000000010443B9B8
L398:
.xword 0x000000007FFFFFFF
L399:
.xword 0x7FFFFFFFFFFFFFFF
L401:
.xword 0x7FFFFFFFFFFFFFFF
L404:
.xword 0x7FFFFFFFFFFFFFFF
L405:
.xword 0x000000010445250C
L408:
.xword 0x7FFFFFFFFFFFFFFF
# End stub section
L501:
L500:
L499:
    mov x14, 4365818364
    br x14
L489:
L488:
    mov x14, 4481917672
    br x14
L422:
L421:
    mov x14, 4481914328
    br x14
L390:
L389:
    mov x14, 4481914152
    br x14
L351:
L350:
    mov x14, 4481915144
    br x14
L320:
L319:
    mov x14, 4365842112
    br x14
L317:
L316:
    mov x14, 4365841688
    br x14
L277:
L276:
    mov x14, 4481917344
    br x14
L272:
L271:
    mov x14, 4481915888
    br x14
L268:
L267:
    mov x14, 4481917432
    br x14
L109:
L108:
    mov x14, 4481913584
    br x14
L157:
L156:
    mov x14, 4365841468
    br x14
L159:
L158:
    mov x14, 4481916892
    br x14
L266:
L265:
    mov x14, 4481916304
    br x14
L198:
L197:
    mov x14, 4365837960
    br x14
L178:
L177:
    mov x14, 4366078192
    br x14
L112:
L111:
    mov x14, 4481913368
    br x14
L173:
L172:
    mov x14, 4366077696
    br x14
L162:
L161:
    mov x14, 4481916920
    br x14
L154:
L153:
    mov x14, 4366078552
    br x14
L114:
L113:
    mov x14, 4481914968
    br x14
L141:
L140:
    mov x14, 4366560408
    br x14
L147:
L146:
    mov x14, 4481911760
    br x14
L145:
L144:
    mov x14, 4365840208
    br x14
L168:
L167:
    mov x14, 4366077348
    br x14
L135:
L134:
    mov x14, 4481914736
    br x14
L466:
L465:
    mov x14, 4481913312
    br x14
L125:
L124:
    mov x14, 4481910672
    br x14
L185:
L184:
    mov x14, 4366077948
    br x14
L120:
L119:
    mov x14, 4481912640
    br x14
L117:
L116:
    mov x14, 4481913944
    br x14
# Begin stub section
L418:
.xword 0x7FFFFFFFFFFFFFFF
L419:
.xword 0x0000000104450720
L423:
.xword 0x7FFFFFFFFFFFFFFF
L433:
.xword 0x7FFFFFFFFFFFFFFF
L434:
.xword 0x7FFFFFFFFFFFFFFF
L438:
.xword 0x7FFFFFFFFFFFFFFF
L441:
.xword 0x35A262A955BD6AA8
L447:
.xword 0x7FFFFFFFFFFFFFFF
L448:
.xword 0x000000010442AAD0
L452:
.xword 0x7FFFFFFFFFFFFFFF
L453:
.xword 0x000000010442AD84
L457:
.xword 0x7FFFFFFFFFFFFFFF
L464:
.xword 0x000000010442C55C
L470:
.xword 0x7FFFFFFFFFFFFFFF
L477:
.xword 0x7FFFFFFFFFFFFFFF
L478:
.xword 0x000000010444DA38
L481:
.xword 0x7FFFFFFFFFFFFFFF
L482:
.xword 0x7FFFFFFFFFFFFFFF
L483:
.xword 0x7FFFFFFFFFFFFFFF
L496:
.xword 0x7FFFFFFFFFFFFFFF
# End stub section
L502:
.section .rodata {#1}
line:
.byte 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
.section .text {#0}
.section .rodata {#1}
attr:
.byte 0x83, 0x6C, 0x00, 0x00, 0x00, 0x02, 0x68, 0x02, 0x77, 0x03, 0x76, 0x73, 0x6E, 0x6C, 0x00, 0x00, 0x00, 0x01, 0x6E, 0x10, 0x00, 0xE3, 0xBF, 0x3B, 0xF1, 0x35, 0x2A, 0xA5, 0x98, 0xE9, 0xE7, 0x07, 0x03, 0x91, 0x5A, 0xC2, 0x52, 0x6A, 0x68, 0x02, 0x77, 0x09, 0x62, 0x65, 0x68, 0x61, 0x76, 0x69, 0x6F, 0x75, 0x72, 0x6C, 0x00, 0x00, 0x00, 0x01, 0x77, 0x0E, 0x6C, 0x6F, 0x67, 0x67, 0x65, 0x72, 0x5F, 0x68, 0x61, 0x6E, 0x64, 0x6C, 0x65, 0x72, 0x6A, 0x6A
.section .text {#0}
.section .rodata {#1}
compile:
.byte 0x83, 0x6C, 0x00, 0x00, 0x00, 0x03, 0x68, 0x02, 0x77, 0x07, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x6B, 0x00, 0x05, 0x38, 0x2E, 0x36, 0x2E, 0x31, 0x68, 0x02, 0x77, 0x07, 0x6F, 0x70, 0x74, 0x69, 0x6F, 0x6E, 0x73, 0x6C, 0x00, 0x00, 0x00, 0x06, 0x77, 0x0A, 0x64, 0x65, 0x62, 0x75, 0x67, 0x5F, 0x69, 0x6E, 0x66, 0x6F, 0x68, 0x02, 0x77, 0x01, 0x69, 0x6B, 0x00, 0x28, 0x2F, 0x62, 0x75, 0x69, 0x6C, 0x64, 0x72, 0x6F, 0x6F, 0x74, 0x2F, 0x6F, 0x74, 0x70, 0x2F, 0x6C, 0x69, 0x62, 0x2F, 0x6B, 0x65, 0x72, 0x6E, 0x65, 0x6C, 0x2F, 0x73, 0x72, 0x63, 0x2F, 0x2E, 0x2E, 0x2F, 0x69, 0x6E, 0x63, 0x6C, 0x75, 0x64, 0x65, 0x77, 0x19, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x64, 0x6F, 0x63, 0x5F, 0x66, 0x75, 0x6E, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x77, 0x19, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x64, 0x6F, 0x63, 0x5F, 0x63, 0x61, 0x6C, 0x6C, 0x62, 0x61, 0x63, 0x6B, 0x77, 0x1C, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x6D, 0x69, 0x73, 0x73, 0x69, 0x6E, 0x67, 0x5F, 0x73, 0x70, 0x65, 0x63, 0x5F, 0x64, 0x6F, 0x63, 0x75, 0x6D, 0x65, 0x6E, 0x74, 0x65, 0x64, 0x77, 0x15, 0x77, 0x61, 0x72, 0x6E, 0x5F, 0x64, 0x65, 0x70, 0x72, 0x65, 0x63, 0x61, 0x74, 0x65, 0x64, 0x5F, 0x63, 0x61, 0x74, 0x63, 0x68, 0x6A, 0x68, 0x02, 0x77, 0x06, 0x73, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x6B, 0x00, 0x31, 0x2F, 0x62, 0x75, 0x69, 0x6C, 0x64, 0x72, 0x6F, 0x6F, 0x74, 0x2F, 0x6F, 0x74, 0x70, 0x2F, 0x6C, 0x69, 0x62, 0x2F, 0x6B, 0x65, 0x72, 0x6E, 0x65, 0x6C, 0x2F, 0x73, 0x72, 0x63, 0x2F, 0x6C, 0x6F, 0x67, 0x67, 0x65, 0x72, 0x5F, 0x73, 0x69, 0x6D, 0x70, 0x6C, 0x65, 0x5F, 0x68, 0x2E, 0x65, 0x72, 0x6C, 0x6A
.section .text {#0}
.section .rodata {#1}
md5:
.byte 0x52, 0xC2, 0x5A, 0x91, 0x03, 0x07, 0xE7, 0xE9, 0x98, 0xA5, 0x2A, 0x35, 0xF1, 0x3B, 0xBF, 0xE3
.section .text {#0}
