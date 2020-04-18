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
- Generating a markdown report and PNG plots from benchmarking results
- Easy to add user-defined optimization problems and solvers
- Simulating a concurrent environment in which an optimization process is executed by multiple workers simultaneously
- Reproducible


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

### Dependencies

If you want to use `kurobako plot` command, you need to have installed `gnuplot` package on your environment.

Usage Example
--------------

```console
// Define solver.
$ kurobako solver random | tee solver.json
{"random":{}}

// Define problem.
$ curl -OL http://ml4aad.org/wp-content/uploads/2019/01/fcnet_tabular_benchmarks.tar.gz
$ tar xf fcnet_tabular_benchmarks.tar.gz && cd fcnet_tabular_benchmarks/
$ kurobako problem hpobench fcnet_protein_structure_data.hdf5 | tee problem.json
{"hpobench":{"dataset":"fcnet_protein_structure_data.hdf5"}}

// Run benchmark.
$ kurobako studies --solvers $(cat solver.json) --problems $(cat problem.json) | kurobako run > result.json
(ALL) [00:00:01] [STUDIES     10/10 100%] [ETA  0s] done

// Report the benchmark result.
$ cat result.json | kurobako report
...abbrev...

// Plot the benchmark result.
$ cat result.json | kurobako plot curve
(PLOT) [00:00:01] [1/1 100%] [ETA  0s] done (dir="images/curve/")
```

Build-in Solvers and Problems
-----------------------------

Solvers:
- Random Search
- [NSGA-II](https://ieeexplore.ieee.org/document/996017)
- [ASHA](https://arxiv.org/abs/1810.05934)
- [Optuna](https://github.com/optuna/optuna)

Problems:
- [NASBench](https://github.com/automl/nas_benchmarks)
- [HPOBench](https://github.com/automl/nas_benchmarks)
- [sigopt/evalset](https://github.com/sigopt/evalset)


Where does the name come from?
-----------------------------------

"kurobako" is a Japanese translation of "black box".


References
----------

- [Introduction to Kurobako: A Benchmark Tool for Hyperparameter Optimization Algorithms](https://medium.com/optuna/kurobako-a2e3f7b760c7)
