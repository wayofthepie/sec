[![build](https://github.com/wayofthepie/sec/actions/workflows/build.yml/badge.svg)](https://github.com/wayofthepie/sec/actions/workflows/build.yml)
[![security audit](https://github.com/wayofthepie/sec/actions/workflows/security_audit.yml/badge.svg)](https://github.com/wayofthepie/sec/actions/workflows/security_audit.yml)
[![codecov](https://codecov.io/gh/wayofthepie/sec/branch/main/graph/badge.svg?token=TAVF5SW2KM)](https://codecov.io/gh/wayofthepie/sec)
# sec
Simple secret manager wrapping GPG.

# Build
`sec` relies on the rust [gpgme wrapper](https://github.com/gpg-rs/gpgme) lib. This needs the following native libs installed.

```console
sudo apt install libgpg-error-dev libgpgme-dev
```
