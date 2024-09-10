#include "net/sock/tcp.h"
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct smtp_session_t {
    sock_tcp_t *sock;
    uint8_t *buffer;
} smtp_session_t;

int32_t smtp_connect(struct smtp_session_t *session,
                     sock_tcp_t *sock,
                     unsigned char *buffer,
                     uintptr_t buffer_len,
                     const sock_tcp_ep_t *remote);
