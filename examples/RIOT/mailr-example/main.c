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

    smtp_session_t session;
    sock_tcp_t sock;
    uint8_t buffer[BUFFER_SIZE];

    printf("Connecting to SMTP server at [");
    ipv6_addr_print((ipv6_addr_t *)&remote.addr);
    printf("]:%d through netif %d\n", remote.port, remote.netif);

    res = smtp_connect(&session, &sock, buffer, BUFFER_SIZE, &remote);
    if (res < 0) {
        printf("Connect failed with error %d", res);
        return 1;
    }

    puts("Email sent");

    res = smtp_close(&session);
    if (res < 0) {
        printf("Error occurred while closing %d", res);
        return 1;
    }

    puts("SMTP session successfully terminated.");

    return 0;
}