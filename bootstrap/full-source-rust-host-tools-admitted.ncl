# Exact logical identities of independently materialized full-source Rust host tools.
#
# Each path must have a canonical artifact attestation and construction receipt
# before `mantle bootstrap full-source-rust-host-tools` admits it.

let { StorePath, .. } = import "contracts.ncl" in

{
  make = "/mantle/store/fb8jzr2f951mvr57i8azmq0dm49y12lm-make-4.4.1-full-source-gcc10-v1" | StorePath,
  linux_headers = "/mantle/store/bmcpsbg4x9zb7x8rq0cfsyblcvyrw9h0-linux-headers-6.6-full-source-gcc10-v1" | StorePath,
  busybox = "/mantle/store/x3rn4spfi90yrfgxvk7q3iyx07kaqpxb-busybox-1.37.0-full-source-gcc10-v1" | StorePath,
}
