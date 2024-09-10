#include <stdio.h>

#include "mailr/smtp.h"
#include "net/af.h"
#include "net/ipv6/addr.h"
#include "net/netif.h"
#include "net/sock/tcp.h"

int main(void)
{
    netif_t *netif = netif_iter(NULL);

    sock_tcp_ep_t remote = SOCK_IPV6_EP_ANY;
    remote.netif = netif_get_id(netif);
    remote.port = SMTP_SERVER_PORT;

    if (ipv6_addr_from_str((ipv6_addr_t *)&remote.addr, SMTP_SERVER_IPV6_ADDR) == NULL) {
        printf("error: unable to parse IPv6 address.\n");
        return 1;
    }

    sock_tcp_t sock;
    unsigned char buffer[BUFFER_SIZE];
    smtp_session_t session;

    int res = smtp_connect(&session, &sock, buffer, BUFFER_SIZE, &remote);
    if (res < 0) {
        printf("Connect failed with error %d", res);
        return 1;
    }

    puts("Email sent");

    return 0;
}