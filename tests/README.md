## Testing

To run integrated tests and examples, ensure an SMTP server is listening unencrypted on local port 2525 and one listening with implicit TLS on port 5870. These ports can be changed with environment variables `NOTLS_PORT` and `PORT` respectively when running the tests. E.g.,

```sh
NOTLS_PORT=2525 PORT=5870 cargo test
```

For convenience, a test server is included, using the python [`aiosmtpd`](https://pypi.org/project/aiosmtpd/) package. Run with:

```sh
cd tests

# generate a TLS certificate to use, or use the generated one in `tests/data`
openssl req -x509 -newkey rsa:2048 -keyout data/key.pem -out data/cert.pem -days 365 -noenc -batch

# run SMTP servers for testing
PORT=5870 CERT=data/cert.pem KEY=data/key.pem test-smtpd.py # TLS
PORT=2525 test-smtpd.py # No TLS

# `test-smtpd.py --help` for details
```
