fn escaped() {
    let _ = std::net::TcpStream::connect("example.invalid:443");
}
