use file_chunk_sender::config;
use file_chunk_sender::config::Conf;
use file_chunk_sender::file_trans;
fn main() {
    let config = match config::ServerConfig::<String>::read_from_path("./ServerConf.toml") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("读取失败:{:?},初始化", e);
            let config: config::ServerConfig<String> = config::ServerConfig::default();
            config.set_config_to_path("./ServerConf.toml").unwrap();
            config
        }
    };
    let tcp_listener = std::net::TcpListener::bind(config.ip()).unwrap();
    let (mut tcp, _addr) = tcp_listener.accept().unwrap();
    let sent_size = file_trans::send_ds(&config, &mut tcp).unwrap();
    dbg!(sent_size);
}
