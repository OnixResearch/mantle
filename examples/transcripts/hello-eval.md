# Evaluate the smallest Mantle derivation

This executable transcript evaluates the seed-free hello expression in isolated
store and state roots. Evaluation proves only the declared derivation shape; it
does not prove a build or release.

```mantle
mantle eval ../hello.ncl
```

```expect
"name"
hello
```
