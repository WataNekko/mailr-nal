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

typedef struct smtp_auth_credential_t {
    const char *username;
    const char *password;
} smtp_auth_credential_t;

typedef struct smtp_connect_info_t {
    sock_tcp_t *sock;
    uint8_t *buffer;
    uintptr_t buffer_len;
    const sock_tcp_ep_t *remote;
    const struct smtp_auth_credential_t *auth;
} smtp_connect_info_t;

typedef struct mailr_mailbox_t {
    const char *address;
    const char *name;
} mailr_mailbox_t;

typedef struct mailr_mailbox_slice_t {
    mailr_mailbox_t *data;
    size_t len;
} mailr_mailbox_slice_t;

typedef struct mailr_message_t {
    mailr_mailbox_t from;
    mailr_mailbox_slice_t to;
    mailr_mailbox_slice_t cc;
    mailr_mailbox_slice_t bcc;
    const char *subject;
    const char *body;
} mailr_message_t;

typedef struct mailr_envelope_receiver_addrs_t {
    const char **addrs;
    size_t len;
} mailr_envelope_receiver_addrs_t;

typedef struct mailr_envelope_t {
    const char *sender_addr;
    mailr_envelope_receiver_addrs_t receiver_addrs;
} mailr_envelope_t;

int smtp_connect(smtp_session_t *session, smtp_connect_info_t *data);

int smtp_close(smtp_session_t *session);

int smtp_send(smtp_session_t *session, const mailr_message_t *mail);

int smtp_send_raw(smtp_session_t *session,
                  const mailr_envelope_t *envelope,
                  const char *data);

#endif