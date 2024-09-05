#include <stdio.h>

#include "mailr/smtp.h"

int main(void)
{
    smtp_hello_world();

    printf("You are running RIOT on a(n) %s board.\n", RIOT_BOARD);
    printf("This board features a(n) %s CPU.\n", RIOT_CPU);

    return 0;
}