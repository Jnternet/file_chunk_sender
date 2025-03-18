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
