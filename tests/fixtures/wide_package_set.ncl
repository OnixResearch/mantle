let crunch = import "lib.ncl" in
{
  alpha = {
    name = "alpha",
    builder = "/bin/sh",
    args = ["-c", "echo alpha > $out"],
    inputs = [],
  } | crunch.Derivation,

  beta = {
    name = "beta",
    builder = "/bin/sh",
    args = ["-c", "echo beta > $out"],
    inputs = [],
  } | crunch.Derivation,

  gamma = {
    name = "gamma",
    builder = "/bin/sh",
    args = ["-c", "echo gamma > $out"],
    inputs = [],
  } | crunch.Derivation,

  delta = {
    name = "delta",
    builder = "/bin/sh",
    args = ["-c", "echo delta > $out"],
    inputs = [],
  } | crunch.Derivation,

  epsilon = {
    name = "epsilon",
    builder = "/bin/sh",
    args = ["-c", "echo epsilon > $out"],
    inputs = [],
  } | crunch.Derivation,

  zeta = {
    name = "zeta",
    builder = "/bin/sh",
    args = ["-c", "echo zeta > $out"],
    system = 'x86_64-linux,
    inputs = [],
  } | crunch.Derivation,

  eta = {
    name = "eta",
    builder = "/bin/sh",
    args = ["-c", "echo eta > $out"],
    inputs = [],
  } | crunch.Derivation,

  theta = {
    name = "theta",
    builder = "/bin/sh",
    args = ["-c", "echo theta > $out"],
    inputs = [],
  } | crunch.Derivation,

  iota = {
    name = "iota",
    builder = "/bin/sh",
    args = ["-c", "echo iota > $out"],
    inputs = [],
  } | crunch.Derivation,

  kappa = {
    name = "kappa",
    builder = "/bin/sh",
    args = ["-c", "echo kappa > $out"],
    inputs = [],
  } | crunch.Derivation,

  lambda = {
    name = "lambda",
    builder = "/bin/sh",
    args = ["-c", "echo lambda > $out"],
    addressing_mode = 'content-addressed,
    inputs = [],
  } | crunch.Derivation,

  mu = {
    name = "mu",
    builder = "/bin/sh",
    args = ["-c", "echo mu > $out"],
    inputs = [],
  } | crunch.Derivation,

  nu = {
    name = "nu",
    builder = "/bin/sh",
    args = ["-c", "echo nu > $out"],
    inputs = [],
  } | crunch.Derivation,

  xi = {
    name = "xi",
    builder = "/bin/sh",
    args = ["-c", "echo xi > $out"],
    inputs = [],
  } | crunch.Derivation,

  omicron = {
    name = "omicron",
    builder = "/bin/sh",
    args = ["-c", "echo omicron > $out"],
    inputs = [],
  } | crunch.Derivation,

  pi = {
    name = "pi",
    builder = "/bin/sh",
    args = ["-c", "echo pi > $out"],
    inputs = [],
  } | crunch.Derivation,
}
