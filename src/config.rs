use std::{
    io::{Read, Write},
    path::Path,
    str::FromStr,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfig<P: AsRef<Path>> {
    ip: std::net::SocketAddr,
    file_path: P,
    chunk_size: u64,
}
impl<P: AsRef<Path>> ServerConfig<P> {
    pub fn file_path(&self) -> &P {
        &self.file_path
    }
    pub fn chunk_size(&self) -> u64 {
        self.chunk_size
    }
    pub fn ip(&self) -> &std::net::SocketAddr {
        &self.ip
    }
}
impl<'a, P: AsRef<Path> + From<&'a str>> Default for ServerConfig<P> {
    fn default() -> Self {
        Self {
            ip: std::net::SocketAddr::new(std::net::IpAddr::from_str("127.0.0.1").unwrap(), 3000),
            file_path: "test".into(),
            chunk_size: 1 * MB,
        }
    }
}
const MB: u64 = 1024 * 1024;
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ClientConfig<P: AsRef<Path>> {
    ip: std::net::SocketAddr,
    save_path: P,
}

impl<P: AsRef<Path>> ClientConfig<P> {
    pub fn save_path(&self) -> &P {
        &self.save_path
    }
    pub fn ip(&self) -> &std::net::SocketAddr {
        &self.ip
    }
}
impl<'a, P: AsRef<Path> + From<&'a str>> Default for ClientConfig<P> {
    fn default() -> Self {
        Self {
            ip: std::net::SocketAddr::new((std::net::Ipv4Addr::new(127, 0, 0, 1)).into(), 3000),
            save_path: "./recieve".into(),
        }
    }
}

impl<P: AsRef<Path> + serde::Serialize + serde::de::DeserializeOwned> Conf for ServerConfig<P> {}
impl<P: AsRef<Path> + serde::Serialize + serde::de::DeserializeOwned> Conf for ClientConfig<P> {}

pub trait Conf: serde::Serialize + serde::de::DeserializeOwned {
    fn read_from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let mut f = std::fs::OpenOptions::new().read(true).open(path)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;

        anyhow::Ok(toml::from_str(&buf)?)
    }
    fn set_config_to_path<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let s = toml::to_string_pretty(self)?;

        let path = path.as_ref();
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        f.write_all(s.as_bytes())?;
        f.flush()?;

        anyhow::Ok(())
    }
}
