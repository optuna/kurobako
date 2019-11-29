kurobako
=========

[![kurobako](https://img.shields.io/crates/v/kurobako.svg)](https://crates.io/crates/kurobako)
[![Documentation](https://docs.rs/kurobako/badge.svg)](https://docs.rs/kurobako)
[![Actions Status](https://github.com/sile/kurobako/workflows/CI/badge.svg)](https://github.com/sile/kurobako/actions)
[![Coverage Status](https://coveralls.io/repos/github/sile/kurobako/badge.svg?branch=master)](https://coveralls.io/github/sile/kurobako?branch=master)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`kurobako` is a command-line tool to benchmark black-box optimization algorithms.

Features:
- It has the capability to support wide range of optimization problems:
  - various search sapce:
    - Continuous numerical, discrete numerical and categorical
    - Uniform distribution and log uniform distribution
    - Conditional
  - Constrainted problems
  - Multi-objective problems
- Simulating a concurrent environment in which an optimization process is executed by multiple workers simultaneously.
- Generating a text-based report (Markdown) from benchmarking results
- Plotting images from benchmarking results
- Reproducible 
- Easy to add user-defined optimization problems and solvers


Installation
------------

### Precompiled binaries

Precompiled binaries for Linux are available in the [releases] page.

```console
$ curl -L https://github.com/sile/kurobako/releases/download/${VERSION}/kurobako-${VERSION}.linux-amd64 -o kurobako
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


What is "kurobako"?
--------------------

"kurobako" is a Japanese translation of "black box".

