//
// ssr.rs
// Copyright (C) 2019 gmg137 <gmg137@live.com>
// Distributed under terms of the GPLv3 license.
//
use crate::db::Data;
use image::load_from_memory;
use screenshot_rs::screenshot_area;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    net::ToSocketAddrs,
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use async_std::{io, future, net::TcpStream};

// 扫码添加
pub fn add_qrcode() -> Option<(u8, Vec<(String, Option<String>, Vec<SsrConfig>)>)> {
    let image_path = format!("{}/qrcode.png", crate::CONFIG_PATH.to_owned());
    screenshot_area(image_path.to_owned(), true);
    if let Ok(buffer) = fs::read(Path::new(&image_path)) {
        fs::remove_file(image_path).unwrap_or(());
        if let Ok(image) = load_from_memory(&buffer) {
            let decoder = bardecoder::default_decoder();
            let results = decoder.decode(image);
            for result in results {
                if let Ok(ssr_str) = result {
                    return add_ssr_url(ssr_str);
                }
            }
        }
    }
    None
}

// 添加订阅
pub async fn add_sub(url: String) -> Option<Vec<(String, Option<String>, Vec<SsrConfig>)>> {
    if url.starts_with("http://") || url.starts_with("https://") {
        let configs = ssr_sub_url_parse(&url).await.ok()?;
        let mut data = Data::new();
        return data.add_sub(url, configs.get(0)?.group.to_owned(), configs);
    }
    None
}

// 添加 SSR 链接
pub fn add_ssr_url(url: String) -> Option<(u8, Vec<(String, Option<String>, Vec<SsrConfig>)>)> {
    let config = ssr_url_parse(url)?;
    let mut data = Data::new();
    data.add_ssr_url(config)
}

// 检测 ssr-local 是否运行
pub fn is_run() -> bool {
    if let Ok(output) = Command::new("pidof").arg("ssr-local").output() {
        return output.status.success() && !output.stdout.is_empty();
    }
    false
}

// 测试延迟
pub async fn timeout(host: String, port: String) -> io::Result<u16> {
    let now = Instant::now();
    let addrs = format!("{}:{}", host, port).to_socket_addrs()?;
    for addr in addrs {
       let t = if TcpStream::connect(&addr).await.is_ok() {
            now.elapsed().as_millis() as u16
        } else {
            9999
        };
        if t < 9999 {
            return Ok(t);
        }
    }
    Err(io::Error::last_os_error())
}

// 关闭 SSR 连接
pub fn stop() -> bool {
    if let Ok(status) = Command::new("killall").arg("ssr-local").status() {
        return status.success();
    }
    false
}

// 启动 SSR 连接
pub fn run(config: &SsrConfig) -> bool {
    if let Ok(addrs) = format!("{}:{}", config.remote_addr, config.remote_port).to_socket_addrs() {
        for addr in addrs {
            let remote_addr = addr.ip().to_string();
            if let Ok(status) = Command::new("ssr-local")
                .arg("-s")
                .arg(remote_addr)
                .arg("-p")
                .arg(config.remote_port.to_owned())
                .arg("-l")
                .arg(config.local_port.to_owned())
                .arg("-b")
                .arg(config.local_addr.to_owned())
                .arg("-k")
                .arg(config.password.to_owned())
                .arg("-m")
                .arg(config.method.to_owned())
                .arg("-t")
                .arg(config.timeout.to_owned())
                .arg("-o")
                .arg(config.obfs.to_owned())
                .arg("-g")
                .arg(config.obfsparam.to_owned())
                .arg("-O")
                .arg(config.protocol.to_owned())
                .arg("-G")
                .arg(config.protoparam.to_owned())
                .arg("-f")
                .arg(format!("{}/pid.txt", crate::CONFIG_PATH.to_owned()))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .status()
            {
                return status.success();
            }
        }
    }
    false
}

// SSR 配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsrConfig {
    // 服务器地址
    pub remote_addr: String,
    // 服务器端口
    pub remote_port: String,
    // 绑定地址
    pub local_addr: String,
    // 本地绑定端口
    pub local_port: String,
    // 超时
    pub timeout: String,
    // 加密方法
    pub method: String,
    // 密码
    pub password: String,
    // 协议
    pub protocol: String,
    // 协议参数
    pub protoparam: String,
    // 混淆
    pub obfs: String,
    // 混淆参数
    pub obfsparam: String,
    // 标签
    pub remarks: String,
    // 分组
    pub group: String,
    // 网络延迟
    pub delay: String,
}

impl Default for SsrConfig {
    fn default() -> Self {
        SsrConfig {
            remote_addr: String::new(),
            remote_port: String::new(),
            local_addr: String::from("127.0.0.1"),
            local_port: String::from("1080"),
            timeout: String::from("300"),
            method: String::new(),
            password: String::new(),
            protocol: String::new(),
            protoparam: String::new(),
            obfs: String::new(),
            obfsparam: String::new(),
            remarks: String::from("未命名"),
            group: String::from("默认"),
            delay: String::from("0 ms"),
        }
    }
}

impl SsrConfig {
    pub fn set_remote_addr(mut self, remote_addr: &str) -> Self {
        self.remote_addr = remote_addr.to_owned();
        self
    }
    pub fn set_remote_port(mut self, remote_port: &str) -> Self {
        self.remote_port = remote_port.to_owned();
        self
    }
    pub fn set_local_addr(mut self, local_addr: &str) -> Self {
        self.local_addr = local_addr.to_owned();
        self
    }
    pub fn set_local_port(mut self, local_port: &str) -> Self {
        self.local_port = local_port.to_owned();
        self
    }
    pub fn set_timeout(mut self, timeout: &str) -> Self {
        self.timeout = timeout.to_owned();
        self
    }
    pub fn set_method(mut self, method: &str) -> Self {
        self.method = method.to_owned();
        self
    }
    pub fn set_password(mut self, password: &str) -> Self {
        self.password = password.to_owned();
        self
    }
    pub fn set_protocol(mut self, protocol: &str) -> Self {
        self.protocol = protocol.to_owned();
        self
    }
    pub fn set_protoparam(mut self, protoparam: &str) -> Self {
        self.protoparam = protoparam.to_owned();
        self
    }
    pub fn set_obfs(mut self, obfs: &str) -> Self {
        self.obfs = obfs.to_owned();
        self
    }
    pub fn set_obfsparam(mut self, obfsparam: &str) -> Self {
        self.obfsparam = obfsparam.to_owned();
        self
    }
    pub fn set_remarks(mut self, remarks: &str) -> Self {
        self.remarks = remarks.to_owned();
        self
    }
    pub fn set_group(mut self, group: &str) -> Self {
        self.group = group.to_owned();
        self
    }
    pub fn set_delay(mut self, delay: u16) -> Self {
        self.delay = format!("{} ms", delay);
        self
    }
}

// 解析 SSR 链接
pub fn ssr_url_parse(url: String) -> Option<SsrConfig> {
    if url.starts_with("ssr://") {
        let (_, ends) = url.split_at(6);
        if let Ok(body) = base64::decode_config(ends, base64::URL_SAFE) {
            let body = String::from_utf8_lossy(&body);
            let body_vec = body.split(':').collect::<Vec<&str>>();
            if body_vec.len() > 5 {
                let body = format!("ssr://{}", body_vec[5]);
                if let Ok(body) = url::Url::parse(&body) {
                    let password = String::from_utf8_lossy(
                        &base64::decode_config(&body.host_str().unwrap_or(""), base64::URL_SAFE)
                            .unwrap_or_else(|_| vec![0u8]),
                    )
                    .to_string();
                    let remote_addr = body_vec[0].to_string();
                    let remote_port = body_vec[1].to_string();
                    let protocol = body_vec[2].to_string();
                    let method = body_vec[3].to_string();
                    let obfs = body_vec[4].to_string();
                    let mut obfsparam = "".to_string();
                    let mut protoparam = "".to_string();
                    let mut remarks = "".to_string();
                    let mut group = "".to_string();
                    for (k, v) in body.query_pairs() {
                        match k {
                            Cow::Borrowed("obfsparam") => obfsparam = v.to_string(),
                            Cow::Borrowed("protoparam") => protoparam = v.to_string(),
                            Cow::Borrowed("remarks") => remarks = v.to_string(),
                            Cow::Borrowed("group") => group = v.to_string(),
                            _ => (),
                        };
                    }
                    obfsparam = String::from_utf8_lossy(
                        &base64::decode_config(&obfsparam, base64::URL_SAFE)
                            .unwrap_or_else(|_| vec![]),
                    )
                    .to_string();
                    protoparam = String::from_utf8_lossy(
                        &base64::decode_config(&protoparam, base64::URL_SAFE)
                            .unwrap_or_else(|_| vec![]),
                    )
                    .to_string();
                    remarks = String::from_utf8_lossy(
                        &base64::decode_config(&remarks, base64::URL_SAFE)
                            .unwrap_or_else(|_| "未命名".as_bytes().to_vec()),
                    )
                    .to_string();
                    group = String::from_utf8_lossy(
                        &base64::decode_config(&group, base64::URL_SAFE)
                            .unwrap_or_else(|_| "默认".as_bytes().to_vec()),
                    )
                    .to_string();
                    return Some(SsrConfig {
                        remote_addr,
                        remote_port,
                        local_addr: "127.0.0.1".to_owned(),
                        local_port: "1080".to_owned(),
                        timeout: "300".to_owned(),
                        protocol,
                        method,
                        obfs,
                        obfsparam,
                        protoparam,
                        remarks,
                        group,
                        password,
                        delay: String::from("0 ms"),
                    });
                }
            }
        }
    }
    None
}

// 解析 SSR 定阅链接
pub async fn ssr_sub_url_parse(url: &str) -> Result<Vec<SsrConfig>, surf::Exception> {
    let body: String = surf::get(url).recv_string().await?;
    let body = String::from_utf8_lossy(
        &base64::decode_config(&body, base64::URL_SAFE).unwrap_or_else(|_| vec![]),
    )
    .to_string();
    let body = body.split('\n').collect::<Vec<&str>>();
    let mut vec: Vec<SsrConfig> = Vec::new();
    body.iter().for_each(|url| {
        if let Some(config) = ssr_url_parse(url.to_string()) {
            vec.push(config);
        }
    });
    Ok(vec)
}
