# Pinned Linux source used only to derive the bounded UAPI header surface.

let crunch = import "lib.ncl" in
crunch.fetchTarball {
  url = "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.6.tar.xz",
  hash = "sha256-+GqM1/8Qw0N6Z2t45jjapVZrG0JysMs9PVi79GIK1Ec=",
  name = "linux-6.6-src",
}
