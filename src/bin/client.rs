fn main() {
    let mut tcp_stream = std::net::TcpStream::connect("127.0.0.1:3000").unwrap();
    let path = "./receive.wav";
    file_chunk_sender::receive_file_chunks_pb(&mut tcp_stream, path).unwrap();
}
