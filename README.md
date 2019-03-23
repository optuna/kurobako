kurobako
=========

[![kurobako](https://img.shields.io/crates/v/kurobako.svg)](https://crates.io/crates/kurobako)
[![Documentation](https://docs.rs/kurobako/badge.svg)](https://docs.rs/kurobako)
[![Build Status](https://travis-ci.org/sile/kurobako.svg?branch=master)](https://travis-ci.org/sile/kurobako)
[![Code Coverage](https://codecov.io/gh/sile/kurobako/branch/master/graph/badge.svg)](https://codecov.io/gh/sile/kurobako/branch/master)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A black-box optimization benchmark framework.

"kurobako" is a Japanese translation of "black box".

Memo
-----

- https://github.com/sigopt/evalset

```console
$ echo '[{"problem":'(cargo run -- problem ackley)', "optimizer":'(cargo run -- optimizer random)', "budget":10}]' | cargo run -- run | jq .q

$ cargo run -- benchmark --problems (cargo run -- problem-suite sigopt auc) --optimizers (cargo run -- optimizer-suite)
```
