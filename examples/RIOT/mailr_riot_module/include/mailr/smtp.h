#ifndef MAILR_SMTP_H
#define MAILR_SMTP_H

#include "net/sock/tcp.h"
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct smtp_ehlo_info_t {
    uint8_t extensions;
} smtp_ehlo_info_t;

typedef struct smtp_session_t {
    sock_tcp_t *sock;
    uint8_t *buffer;
    size_t buffer_len;
    smtp_ehlo_info_t ehlo_info;
} smtp_session_t;

int32_t smtp_connect(smtp_session_t *session,
                     sock_tcp_t *sock,
                     uint8_t *buffer,
                     uintptr_t buffer_len,
                     const sock_tcp_ep_t *remote);

int32_t smtp_close(smtp_session_t *session);

#endif