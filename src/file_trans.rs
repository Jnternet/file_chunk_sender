use indicatif::TermLike;
use std::{
    fs::File,
    io::{Read, Write},
    net::TcpStream,
};
/// 发送文件分块函数
/// 参数：
/// - stream: 可变的 TcpStream 引用
/// - file_path: 要发送的文件路径
/// - chunk_size: 每个分块的大小（字节数）
pub fn send_file_chunks(
    stream: &mut TcpStream,
    file_path: &str,
    chunk_size: usize,
) -> Result<(), std::io::Error> {
    // 打开要发送的文件
    let mut file = File::open(file_path)?;
    // 创建读取缓冲区
    let mut buffer = vec![0u8; chunk_size];

    loop {
        // 读取指定大小的数据块
        let bytes_read = file.read(&mut buffer)?;

        // 文件读取完毕时退出循环
        if bytes_read == 0 {
            break;
        }

        // 发送块长度（8字节，大端序）
        stream.write_all(&(bytes_read as u64).to_be_bytes())?;

        // 发送实际数据
        stream.write_all(&buffer[..bytes_read])?;
    }

    // 发送结束标志（长度为0的块）
    stream.write_all(&0u64.to_be_bytes())?;
    Ok(())
}

/// 接收文件分块函数
/// 参数：
/// - stream: 可变的 TcpStream 引用
/// - save_path: 文件保存路径
pub fn receive_file_chunks(stream: &mut TcpStream, save_path: &str) -> Result<(), std::io::Error> {
    // 创建目标文件
    let mut file = File::create(save_path)?;
    // 用于接收块长度的缓冲区
    let mut len_bytes = [0u8; 8];

    loop {
        // 读取块长度头
        stream.read_exact(&mut len_bytes)?;
        let chunk_len = u64::from_be_bytes(len_bytes);

        // 收到长度为0表示传输结束
        if chunk_len == 0 {
            break;
        }

        // 创建数据缓冲区
        let mut chunk = vec![0u8; chunk_len as usize];
        // 读取完整数据块
        stream.read_exact(&mut chunk)?;

        // 写入到文件
        file.write_all(&chunk)?;
    }

    Ok(())
}
//这一对里面相较于其他两个，多了一个总大小获取
pub fn send_file_chunks_simple_pb(
    stream: &mut TcpStream,
    file_path: &str,
    chunk_size: usize,
) -> Result<(), std::io::Error> {
    // 打开要发送的文件
    let mut file = File::open(file_path)?;
    // 创建读取缓冲区
    let mut buffer = vec![0u8; chunk_size];
    //获取总长度
    let total_size = file.metadata()?.len();
    stream.write_all(&total_size.to_be_bytes())?;

    let pb = indicatif::ProgressBar::new(total_size);
    let term = indicatif::InMemoryTerm::new(1, 1);
    pb.inc(0);

    loop {
        // 读取指定大小的数据块
        let bytes_read = file.read(&mut buffer)?;

        // 文件读取完毕时退出循环
        if bytes_read == 0 {
            break;
        }

        // 发送块长度（8字节，大端序）
        stream.write_all(&(bytes_read as u64).to_be_bytes())?;

        // 发送实际数据
        stream.write_all(&buffer[..bytes_read])?;

        // std::thread::sleep(std::time::Duration::from_millis(100));

        term.clear_line()?;
        pb.inc(bytes_read as u64);
    }

    // 发送结束标志（长度为0的块）
    stream.write_all(&0u64.to_be_bytes())?;
    Ok(())
}

pub fn receive_file_chunks_simple_pb(
    stream: &mut TcpStream,
    save_path: &str,
) -> Result<(), std::io::Error> {
    // 创建目标文件
    let mut file = File::create(save_path)?;
    // 用于接收块长度的缓冲区
    let mut len_bytes = [0u8; 8];
    //获取总长度
    stream.read_exact(&mut len_bytes)?;
    let total_size = u64::from_be_bytes(len_bytes);

    // eprintln!("总长度：{}", total_size);

    let pb = indicatif::ProgressBar::new(total_size);
    let term = indicatif::InMemoryTerm::new(1, 1);
    pb.inc(0);

    loop {
        // 读取块长度头
        stream.read_exact(&mut len_bytes)?;
        let chunk_len = u64::from_be_bytes(len_bytes);

        // 收到长度为0表示传输结束
        if chunk_len == 0 {
            break;
        }

        // 创建数据缓冲区
        let mut chunk = vec![0u8; chunk_len as usize];
        // 读取完整数据块
        stream.read_exact(&mut chunk)?;

        // 写入到文件
        file.write_all(&chunk)?;
        //进度条前进写入的进度
        term.clear_line()?;
        pb.inc(chunk_len);
    }

    Ok(())
}

use indicatif::{ProgressBar, ProgressStyle};
/// 发送文件分块函数（带进度条）
pub fn send_file_chunks_pb(
    stream: &mut TcpStream,
    file_path: &str,
    chunk_size: usize,
) -> Result<(), std::io::Error> {
    let mut file = File::open(file_path)?;
    let total_size = file.metadata()?.len();

    // 初始化进度条
    let progress = ProgressBar::new(total_size);
    progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut buffer = vec![0u8; chunk_size];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        // 发送块长度和数据
        stream.write_all(&(bytes_read as u64).to_be_bytes())?;
        stream.write_all(&buffer[..bytes_read])?;

        // std::thread::sleep(std::time::Duration::from_millis(100));

        // 更新进度条
        progress.inc(bytes_read as u64);
    }

    stream.write_all(&0u64.to_be_bytes())?;
    progress.finish_with_message("发送完成");
    Ok(())
}

/// 接收文件分块函数（带进度条）
pub fn receive_file_chunks_pb(
    stream: &mut TcpStream,
    save_path: &str,
) -> Result<(), std::io::Error> {
    let mut file = File::create(save_path)?;
    let mut len_bytes = [0u8; 8];

    // 初始化进度条（无总长度）
    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec})",
        )
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    progress.enable_steady_tick(std::time::Duration::from_millis(100));

    loop {
        stream.read_exact(&mut len_bytes)?;
        let chunk_len = u64::from_be_bytes(len_bytes);
        if chunk_len == 0 {
            break;
        }

        let mut chunk = vec![0u8; chunk_len as usize];
        stream.read_exact(&mut chunk)?;
        file.write_all(&chunk)?;

        // 更新进度条
        progress.inc(chunk_len);
    }

    progress.finish_with_message("接收完成");
    Ok(())
}
use std::path::Path;

use crate::config::{ClientConfig, ServerConfig};
pub fn send<P: AsRef<Path>>(
    conf: &crate::config::ServerConfig<P>,
    tcp: &mut TcpStream,
) -> anyhow::Result<u64> {
    //准备要发送的文件
    let mut f = std::fs::OpenOptions::new()
        .create(false)
        .read(true)
        .open(conf.file_path())?;

    //每个分块的大小
    let chunk_size = conf.chunk_size();
    //块缓冲区(一定要初始化，否则长度为0！！！！)
    let mut buf: Vec<u8> = vec![0u8; chunk_size as usize];
    //获取总大小
    let total_size = f.metadata()?.len();
    //初始化进度条
    let progress = ProgressBar::new(total_size);
    progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    //初始化终端
    let term = indicatif::InMemoryTerm::new(1, 1);

    //协议：总大小+块大小+（n * 块） 直到发送的大小大于等于总大小停止
    tcp.write_all(&total_size.to_be_bytes())?;
    tcp.write_all(&chunk_size.to_be_bytes())?;
    let mut sent_size = 0;
    while sent_size <= total_size {
        f.read_exact(&mut buf)?;
        tcp.write_all(&mut buf)?;
        sent_size += chunk_size;

        // std::thread::sleep(std::time::Duration::from_millis(100));
        // crate::pause();

        term.clear_line()?;
        progress.inc(chunk_size);
    }
    anyhow::Ok(sent_size)
}

pub fn receive<P: AsRef<Path>>(conf: &ClientConfig<P>, tcp: &mut TcpStream) -> anyhow::Result<u64> {
    //准备要写入的文件
    let mut f = std::fs::OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(conf.save_path())?;
    //长度缓冲区
    let mut len_buf = [0u8; 8];
    //获取总大小
    tcp.read_exact(&mut len_buf)?;
    let total_size = u64::from_be_bytes(len_buf);
    //块大小
    tcp.read_exact(&mut len_buf)?;
    let chunk_size = u64::from_be_bytes(len_buf);
    //块缓冲区
    let mut buf: Vec<u8> = vec![0u8; chunk_size as usize];
    //初始化进度条
    let progress = ProgressBar::new(total_size);
    progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    //初始化终端
    let term = indicatif::InMemoryTerm::new(1, 1);

    let mut saved_size = 0;
    while saved_size <= total_size {
        tcp.read_exact(&mut buf)?;
        f.write_all(&mut buf)?;
        saved_size += chunk_size;

        term.clear_line()?;
        progress.inc(chunk_size);
    }
    anyhow::Ok(saved_size)
}
pub fn send_ds<P: AsRef<Path>>(
    conf: &crate::config::ServerConfig<P>,
    tcp: &mut TcpStream,
) -> anyhow::Result<u64> {
    let mut f = std::fs::File::open(conf.file_path())?;
    let chunk_size = conf.chunk_size();
    let total_size = f.metadata()?.len();
    let progress = ProgressBar::new(total_size);
    // ... 进度条初始化 ...
    progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    tcp.write_all(&total_size.to_be_bytes())?;
    tcp.write_all(&chunk_size.to_be_bytes())?;

    let mut sent_size = 0;
    let mut buf = vec![0u8; chunk_size as usize];
    while sent_size < total_size {
        let remaining = total_size - sent_size;
        let read_size = remaining.min(chunk_size);
        buf.resize(read_size as usize, 0);
        f.read_exact(&mut buf)?;
        tcp.write_all(&buf)?;
        sent_size += read_size;
        progress.inc(read_size);
        // std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(sent_size)
}
pub fn receive_ds<P: AsRef<Path>>(
    conf: &ClientConfig<P>,
    tcp: &mut TcpStream,
) -> anyhow::Result<u64> {
    let mut f = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(conf.save_path())?;

    let mut len_buf = [0u8; 8];
    tcp.read_exact(&mut len_buf)?;
    let total_size = u64::from_be_bytes(len_buf);
    tcp.read_exact(&mut len_buf)?;
    let chunk_size = u64::from_be_bytes(len_buf);

    let progress = ProgressBar::new(total_size);
    // ... 进度条初始化 ...
    progress.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut saved_size = 0;
    let mut buf = vec![0u8; chunk_size as usize];
    while saved_size < total_size {
        let remaining = total_size - saved_size;
        let read_size = remaining.min(chunk_size) as usize;
        buf.resize(read_size, 0);
        tcp.read_exact(&mut buf)?;
        f.write_all(&buf)?;
        saved_size += read_size as u64;
        progress.inc(read_size as u64);
    }
    Ok(saved_size)
}

pub fn send_rem<P: AsRef<Path>>(
    conf: &ServerConfig<P>,
    tcp: &mut TcpStream,
) -> anyhow::Result<u64> {
    let f = std::fs::File::open(conf.file_path())?;
    let total_size = f.metadata()?.len();
    todo!()
}
