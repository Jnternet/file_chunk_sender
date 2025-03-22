use file_chunk_sender::config;
use file_chunk_sender::config::Conf;
use file_chunk_sender::file_trans;
fn main() {
    let config = match config::ClientConfig::<String>::read_from_path("./ClientConf.toml") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("读取失败{:?}初始化", e);
            let config = config::ClientConfig::<String>::default();
            config.set_config_to_path("./ClientConf.toml").unwrap();
            config
        }
    };
    let mut tcp = std::net::TcpStream::connect(config.ip()).unwrap();
    let saved_size = file_trans::receive_ds(&config, &mut tcp).unwrap();
    dbg!(saved_size);
}
