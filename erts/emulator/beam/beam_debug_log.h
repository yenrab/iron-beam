#ifndef BEAM_DEBUG_LOG_H
#define BEAM_DEBUG_LOG_H

#include "sys.h"
#include "erl_vm.h"
#include <stdio.h>
#include <inttypes.h>

/* Debug log file for comparing C and Rust BEAM decoding */
/* Declare as extern - will be defined in beam_load.c */
extern FILE *beam_debug_log;

/* Initialize debug logging - call once at startup */
void beam_debug_log_init(void);

/* Close debug logging */
void beam_debug_log_close(void);

#define BEAM_DEBUG_LOG(fmt, ...) do { \
    beam_debug_log_init(); \
    if (beam_debug_log != NULL) { \
        fprintf(beam_debug_log, "[Decoder] " fmt "\n", ##__VA_ARGS__); \
        fflush(beam_debug_log); \
    } \
} while(0)

#endif /* BEAM_DEBUG_LOG_H */

