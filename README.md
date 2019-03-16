kurobako
=========

A black-box optimization benchmark framework.

"kurobako" is a Japanese translation of "black box".

Memo
-----

- https://github.com/sigopt/evalset

```console
$ echo '[{"problem":'(cargo run -- problem ackley)', "optimizer":'(cargo run -- optimizer random)', "budget":10}]' | cargo run -- run | jq .q
```
