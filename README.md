kurobako
=========

[![kurobako](https://img.shields.io/crates/v/kurobako.svg)](https://crates.io/crates/kurobako)
[![Documentation](https://docs.rs/kurobako/badge.svg)](https://docs.rs/kurobako)
[![Actions Status](https://github.com/sile/kurobako/workflows/CI/badge.svg)](https://github.com/sile/kurobako/actions)
[![Coverage Status](https://coveralls.io/repos/github/sile/kurobako/badge.svg?branch=master)](https://coveralls.io/github/sile/kurobako?branch=master)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A black-box optimization benchmark framework.

"kurobako" is a Japanese translation of "black box".


Installation
------------

### Precompiled binaries

Precompiled binaries for Linux are available in the [releases] page.

```console
$ curl -L https://github.com/sile/kurobako/releases/download/${VERSION}/kurobako-${VERSION}.linux -o kurobako
$ chmod +x kurobako
$ ./kurobako -h
```

[releases]: https://github.com/sile/kurobako/releases

### Using Cargo

If you have already installed [Cargo][cargo], you can install `kurobako` by executing the following command:

```console
$ cargo install kurobako
```

[cargo]: https://doc.rust-lang.org/cargo/
