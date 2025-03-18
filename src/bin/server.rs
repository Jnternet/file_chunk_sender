fn main() {
    let tcp_listener = std::net::TcpListener::bind("127.0.0.1:3000").unwrap();
    let (mut tcp, _addr) = tcp_listener.accept().unwrap();
    let file_path = "./test.wav";
    let chunk_size = 1 * MB;
    file_chunk_sender::send_file_chunks_pb(&mut tcp, file_path, chunk_size).unwrap();
}

const MB: usize = 1024 * 1024;
