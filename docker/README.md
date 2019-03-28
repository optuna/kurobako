kurobako-docker
================

Build
-----

```console
$ cargo build --release --target=x86_64-unknown-linux-musl
$ cp ../target/x86_64-unknown-linux-musl/release/kurobako .
$ docker build -t kurobako .
```
