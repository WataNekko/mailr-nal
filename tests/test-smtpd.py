#!/usr/bin/env python3

from contextlib import suppress
import os
import sys

try:
    import asyncio
    import logging
    from aiosmtpd.controller import UnthreadedController
    from aiosmtpd.handlers import Debugging
    from aiosmtpd.smtp import AuthResult, LoginPassword
    import ssl
except ModuleNotFoundError:
    raise SystemExit("Missing module\nRun:\npip install -r tests/requirements.txt")


def print_usage():
    program = os.path.basename(__file__)
    usage = f"""Usage:
    {program} [-h|--help]
Env:
    DEBUG       Enable debug log. Default 0.
    PORT        The port to listen on. Default 2525.

    CERT        The certificate file to use for TLS. Must be defined together with `KEY` to enable TLS.
    KEY         The key file to use for TLS. Must be defined together with `CERT` to enable TLS.

    AUTH_USER   Username of an authenticated credential to use for mocking auth.
                Must be defined together with `AUTH_PASS`. Default "mock".
    AUTH_PASS   Password of an authenticated credential to use for mocking auth.
                Must be defined together with `AUTH_USER`. Default "123456".
"""
    print(usage)


def main():
    if len(sys.argv) > 1:
        print_usage()
        return

    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)

    # Configure logging
    logging.basicConfig(level=logging.ERROR)
    log = logging.getLogger("mail.log")

    if os.getenv("DEBUG", "0") != "0":
        log.setLevel(logging.DEBUG)
        loop.set_debug(True)

    # TLS
    ssl_context = None
    if (SSL_CERT := os.getenv("CERT")) and (SSL_KEY := os.getenv("KEY")):
        # Load SSL certificate and key
        ssl_context = ssl.create_default_context(ssl.Purpose.CLIENT_AUTH)
        ssl_context.check_hostname = False
        ssl_context.load_cert_chain(SSL_CERT, SSL_KEY)
        log.debug("TLS enabled")

    # Mock authentication
    AUTH_USER, AUTH_PASS = os.getenv("AUTH_USER"), os.getenv("AUTH_PASS")

    if AUTH_USER and AUTH_PASS:
        AUTH_CREDENTIAL = LoginPassword(AUTH_USER.encode(), AUTH_PASS.encode())
    elif not AUTH_USER and not AUTH_PASS:
        AUTH_CREDENTIAL = LoginPassword(b"mock", b"123456")
    else:
        raise SystemExit("Provide both AUTH_USER and AUTH_PASS env, or none.")

    log.debug(f"Using authenticated credential: {AUTH_CREDENTIAL}")

    def auth(server, session, envelope, mechanism, auth_data):
        log.debug(f"auth_data={auth_data}")

        if not isinstance(auth_data, LoginPassword):
            return AuthResult(success=False, handled=False)

        if auth_data == AUTH_CREDENTIAL:
            return AuthResult(success=True)
        else:
            return AuthResult(success=False, handled=False)

    # Create SMTP server
    HOST = "localhost"
    PORT = int(os.getenv("PORT", 2525))
    controller = UnthreadedController(
        Debugging(),
        hostname=HOST,
        port=PORT,
        loop=loop,
        ssl_context=ssl_context,
        auth_require_tls=not ssl_context,  # https://github.com/aio-libs/aiosmtpd/issues/281
        authenticator=auth,
    )

    controller.begin()
    print(f"SMTP server listening on {HOST}:{PORT}")

    log.debug("Starting asyncio loop")
    with suppress(KeyboardInterrupt):
        loop.run_forever()
    log.debug("Completed asyncio loop")

    print("Stopping SMTP server")
    controller.end()
    loop.close()


if __name__ == "__main__":
    main()
