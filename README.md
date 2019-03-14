kurobako
=========

"kurobako" is a Japanese translation of "black box".

- https://github.com/sigopt/evalset

```console
$ echo '[{"problem":'(cargo run -- problem ackley)', "optimizer":'(cargo run -- optimizer random)', "budget":10}]' | cargo run -- run | jq .q
```
