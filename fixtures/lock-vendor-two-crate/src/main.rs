fn main() {
    let payload = b"123456789";
    assert_eq!(memchr::memchr(b'5', payload), Some(4));
    assert_eq!(crc64::crc64(0, payload), 0xe9c6d914c4b8d9ca);
    println!("offline two-crate consumer: verified");
}
