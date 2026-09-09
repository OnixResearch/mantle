nix develop -c sh -eu -c '
  export CARGO_TARGET_DIR=/home/brittonr/.cache/mantle-v4-target-20260908
  export CARGO_INCREMENTAL=0
  export CARGO_BUILD_JOBS=4
  export TMPDIR=/home/brittonr/.cache/mantle-v4-20260908/tmp
  export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
  cargo test --locked --offline -p mantle --bin mantle copy_selected_source_tree
  cargo test --locked --offline -p mantle --bin mantle source_built_mantle_source
'
