#ifndef MAILR_SMTP_H
#define MAILR_SMTP_H

#include "net/sock/tcp.h"
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define SMTP_SESSION_UNCONNECTED {0}

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
    const smtp_auth_credential_t *auth; /* Optional */
    const char *client_id;              /* Optional */
} smtp_connect_info_t;

typedef struct mailr_mailbox_t {
    const char *address;
    const char *name; /* Optional */
} mailr_mailbox_t;

typedef struct mailr_mailbox_slice_t {
    mailr_mailbox_t *data;
    size_t len;
} mailr_mailbox_slice_t;

typedef struct mailr_message_t {
    mailr_mailbox_t from; /* Optional (set both address and name to NULL) */
    mailr_mailbox_slice_t to;
    mailr_mailbox_slice_t cc;
    mailr_mailbox_slice_t bcc;
    const char *subject; /* Optional */
    const char *body;    /* Optional */
} mailr_message_t;

typedef struct mailr_envelope_receiver_addrs_t {
    const char **addrs;
    size_t len;
} mailr_envelope_receiver_addrs_t;

typedef struct mailr_envelope_t {
    const char *sender_addr; /* Optional */
    mailr_envelope_receiver_addrs_t receiver_addrs;
} mailr_envelope_t;

/*
 * Connect the SMTP client with the provided info.
 *
 * Returns
 *   0 on success.
 *   -EINVAL, if session, info, or any required info fields are NULL or invalid.
 *   -EISCONN, if the session is already connected.
 *   -ENOBUFS, if the provided buffer is too small.
 *   -EPROTO, if the operation fails.
 *   -EACCES, if authentication fails.
 *   -EOPNOTSUPP, if no mutually supported authentication mechanisms.
 *   and other errors from sock_tcp_connect, sock_tcp_read, and sock_tcp_write.
 */
int smtp_connect(smtp_session_t *session, smtp_connect_info_t *info);

/*
 * Terminate the SMTP session.
 *
 * Returns
 *   0 on success.
 *   -EINVAL, if session is NULL.
 *   -ENOTCONN, if the session is not connected.
 *   and other errors from sock_tcp_write.
 */
int smtp_close(smtp_session_t *session);

/*
 * Send an email message.
 *
 * Returns
 *   0 on success.
 *   -EINVAL, if session, mail, or any required mail fields are NULL or invalid.
 *   -ENOTCONN, if the session is not connected.
 *   -ENOBUFS, if the provided buffer is too small.
 *   -EPROTO, if the operation fails.
 *   and other errors from sock_tcp_read and sock_tcp_write.
 */
int smtp_send(smtp_session_t *session, const mailr_message_t *mail);

/*
 * Send a raw email message string.
 *
 * Returns
 *   0 on success.
 *   -EINVAL, if session, mail, or any required mail fields are NULL or invalid.
 *   -ENOTCONN, if the session is not connected.
 *   -ENOBUFS, if the provided buffer is too small.
 *   -EPROTO, if the operation fails.
 *   and other errors from sock_tcp_read and sock_tcp_write.
 */
int smtp_send_raw(smtp_session_t *session,
                  const mailr_envelope_t *envelope,
                  const char *data);

#endif