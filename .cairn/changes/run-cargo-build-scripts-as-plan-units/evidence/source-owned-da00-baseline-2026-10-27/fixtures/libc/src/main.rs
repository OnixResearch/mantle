fn main() {
    assert!(unsafe { libc::getpid() } > 0);
}
