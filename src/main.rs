mod app;
mod db;
mod ssr;
mod view;
mod widgets;
use crate::app::App;
use lazy_static::lazy_static;
use std::collections::HashMap;
static APP_VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {
    // 配置文件目录
    static ref CONFIG_PATH: &'static str = {
        if let Some(path) = dirs::home_dir() {
            let config_path = format!("{}/.config/ssr-gtk", path.display());
            if !std::path::Path::new(&config_path).exists() {
                std::fs::create_dir(&config_path).unwrap_or(());
            }
            return Box::leak(Box::new(config_path));
        }
        ".config/ssr-gtk"
    };
    // 加密方式
    static ref METHOD_LIST: HashMap<u8,&'static str>= {
        let mut m = HashMap::new();
        m.insert(0, "none");
        m.insert(1, "table");
        m.insert(2, "rc4");
        m.insert(3, "rc4-md5");
        m.insert(4, "rc4-md5-6");
        m.insert(5, "aes-128-cfb");
        m.insert(6, "aes-192-cfb");
        m.insert(7, "aes-256-cfb");
        m.insert(8, "aes-128-ctr");
        m.insert(9, "aes-192-ctr");
        m.insert(10, "aes-256-ctr");
        m.insert(11, "bf-cfb");
        m.insert(12, "camellia-128-cfb");
        m.insert(13, "camellia-192-cfb");
        m.insert(14, "camellia-256-cfb");
        m.insert(15, "salsa20");
        m.insert(16, "chacha20");
        m.insert(17, "chacha20-ietf");
        m
    };
    // 协议
    static ref PROTOCOL_LIST: HashMap<u8,&'static str>= {
        let mut m = HashMap::new();
        m.insert(0,"origin");
        m.insert(1,"verify_simple");
        m.insert(2,"verify_sha1");
        m.insert(3,"auth_sha1");
        m.insert(4,"auth_sha1_v2");
        m.insert(5,"auth_sha1_v4");
        m.insert(6,"auth_aes128_sha1");
        m.insert(7,"auth_aes128_md5");
        m.insert(8,"auth_chain_a");
        m.insert(9,"auth_chain_b");
        m.insert(10,"auth_chain_c");
        m.insert(11,"auth_chain_d");
        m.insert(12,"auth_chain_e");
        m.insert(13,"auth_chain_f");
        m
    };
}

#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

#[macro_export]
macro_rules! upgrade_weak {
    ($x:ident, $r:expr) => {{
        match $x.upgrade() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_weak!($x, ())
    };
}

fn main() {
    gtk::init().expect("Error initializing gtk.");

    smol::run(async {
        App::run();
    });
}
