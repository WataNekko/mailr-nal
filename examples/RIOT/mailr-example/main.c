#include <stdio.h>

#include "mailr/smtp.h"
#include "net/af.h"
#include "net/ipv6/addr.h"
#include "net/netif.h"
#include "net/sock/tcp.h"

int add_local_ipv6_addr(netif_t *netif)
{
    (void)netif;

#ifdef ADD_LOCAL_IPV6_ADDR
    ipv6_addr_t addr;
    uint8_t prefix_len = 64;
    uint16_t flags = GNRC_NETIF_IPV6_ADDRS_FLAGS_STATE_VALID;

    if (ipv6_addr_from_str(&addr, ADD_LOCAL_IPV6_ADDR) == NULL) {
        printf("error: unable to parse IPv6 address.\n");
        return 1;
    }

    flags |= (prefix_len << 8U);

    if (netif_set_opt(netif, NETOPT_IPV6_ADDR, flags, &addr, sizeof(addr)) < 0) {
        printf("error: unable to add IPv6 address\n");
        return 1;
    }

    printf("Added ipv6 address [");
    ipv6_addr_print(&addr);
    printf("] to netif %d\n", netif_get_id(netif));
#endif

    return 0;
}

int set_remote_ep(netif_t *netif, sock_tcp_ep_t *remote)
{
    remote->netif = netif_get_id(netif);
    remote->port = SMTP_SERVER_PORT;

    if (ipv6_addr_from_str((ipv6_addr_t *)&remote->addr, SMTP_SERVER_IPV6_ADDR) == NULL) {
        printf("error: unable to parse IPv6 address.\n");
        return 1;
    }

    return 0;
}

int main(void)
{
    int res;

    netif_t *netif = netif_iter(NULL);
    sock_tcp_ep_t remote = SOCK_IPV6_EP_ANY;

    if (add_local_ipv6_addr(netif) != 0) {
        return 1;
    }

    if (set_remote_ep(netif, &remote) != 0) {
        return 1;
    }

    // Connecting to the server

    smtp_session_t session;
    sock_tcp_t sock;
    uint8_t buffer[BUFFER_SIZE];

    smtp_connect_info_t connect_info = {
        .sock = &sock,
        .buffer = buffer,
        .buffer_len = BUFFER_SIZE,
        .remote = &remote};

    printf("Connecting to SMTP server at [");
    ipv6_addr_print((ipv6_addr_t *)&remote.addr);
    printf("]:%d through netif %d\n\n", remote.port, remote.netif);

    res = smtp_connect(&session, &connect_info);
    if (res < 0) {
        printf("Connect failed with error %d", res);
        return 1;
    }

    // Sending email

    mailr_mailbox_t to[] = {{"Jones@foo.com", "Jones"}, {.address = "John@foo.com"}};
    mailr_mailbox_t cc[] = {{.address = "Green@foo.com", .name = "Green"}};

    mailr_message_t mail = {
        .from = {"Smith@bar.com"},
        .to = {to, 2},
        .cc = {cc, 1},
        .subject = "Test mail",
        .body = "Blah blah blah...\r\n..etc. etc. etc."};

    printf("Sending email: \"%s\"\n\n", mail.subject);
    res = smtp_send(&session, &mail);
    if (res < 0) {
        printf("Send mail failed with error %d", res);
        return 1;
    }

    // Sending second email

    mailr_mailbox_t bcc[] = {{.address = "Brown@foo.com"}};

    mail.bcc.data = bcc;
    mail.bcc.len = 1;
    mail.subject = "Another test mail";

    printf("Sending another mail: \"%s\"\n\n", mail.subject);
    res = smtp_send(&session, &mail);
    if (res < 0) {
        printf("Send mail failed with error %d", res);
        return 1;
    }

    // Sending raw data

    const char *receiver_addrs[] = {"janedoe@foo.com", "bar@baz.org"};
    mailr_envelope_t envelope = {
        .sender_addr = "johndoe@foo.com",
        .receiver_addrs = {receiver_addrs, 2}};

    const char *raw_msg_data = "From:<johndoe@foo.com>\r\n"
                               "To:<janedoe@foo.com>\r\n"
                               "Subject: Raw mail sending\r\n"
                               "\r\n"
                               "Blah blah blah...\r\n"
                               "..etc. etc. etc.";

    printf("Sending raw data: \"%s\"\n\n", raw_msg_data);
    res = smtp_send_raw(&session, &envelope, raw_msg_data);
    if (res < 0) {
        printf("Send mail failed with error %d", res);
        return 1;
    }

    puts("Mails sent");

    // Closing the session

    res = smtp_close(&session);
    if (res < 0) {
        printf("Error occurred while closing %d", res);
        return 1;
    }

    puts("SMTP session successfully terminated.");

    return 0;
}