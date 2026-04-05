# crunch stdlib: conversion helpers.
#
# Enum-to-string functions for use in Nickel code that needs to
# interpolate system or algorithm names into strings.

{
  system_to_string
    | doc "Convert a System enum tag to its string representation."
    = fun sys => sys |> match {
      'x86_64-linux => "x86_64-linux",
      'aarch64-linux => "aarch64-linux",
      'x86_64-darwin => "x86_64-darwin",
      'aarch64-darwin => "aarch64-darwin",
    },

  hash_algo_to_string
    | doc "Convert a HashAlgo enum tag to its string representation."
    = fun algo => algo |> match {
      'md5 => "md5",
      'sha1 => "sha1",
      'sha256 => "sha256",
      'sha512 => "sha512",
      'blake3 => "blake3",
    },

  hash_mode_to_string
    | doc "Convert a HashMode enum tag to its string representation."
    = fun mode => mode |> match {
      'flat => "flat",
      'recursive => "recursive",
    },

  addressing_mode_to_string
    | doc "Convert an addressing mode enum tag to its string representation."
    = fun mode => mode |> match {
      'input-addressed => "input-addressed",
      'content-addressed => "content-addressed",
    },
}
