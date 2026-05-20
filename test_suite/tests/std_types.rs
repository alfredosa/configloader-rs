use configloader::ConfigLoader;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::num::{NonZeroI32, NonZeroUsize};
use std::path::PathBuf;
use std::sync::Mutex;
use url::Url;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, ConfigLoader)]
struct StandardLibraryConfig {
    ip_addr: IpAddr,
    ipv4_addr: Ipv4Addr,
    ipv6_addr: Ipv6Addr,
    socket_addr: SocketAddr,
    path_buf: PathBuf,
    non_zero_i32: NonZeroI32,
    non_zero_usize: NonZeroUsize,
}

#[derive(Debug, PartialEq, ConfigLoader)]
struct UrlConfig {
    base_url: Url,
    callback_url: Url,
}

#[test]
fn loads_standard_library_from_str_types() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("STANDARD_LIBRARY_CONFIG_IP_ADDR", "127.0.0.1");
        std::env::set_var("STANDARD_LIBRARY_CONFIG_IPV4_ADDR", "192.168.1.10");
        std::env::set_var("STANDARD_LIBRARY_CONFIG_IPV6_ADDR", "::1");
        std::env::set_var("STANDARD_LIBRARY_CONFIG_SOCKET_ADDR", "127.0.0.1:8080");
        std::env::set_var("STANDARD_LIBRARY_CONFIG_PATH_BUF", "/tmp/configloader");
        std::env::set_var("STANDARD_LIBRARY_CONFIG_NON_ZERO_I32", "-7");
        std::env::set_var("STANDARD_LIBRARY_CONFIG_NON_ZERO_USIZE", "9");
    }

    let config = StandardLibraryConfig::load().unwrap();

    assert_eq!(config.ip_addr, "127.0.0.1".parse::<IpAddr>().unwrap());
    assert_eq!(config.ipv4_addr, Ipv4Addr::new(192, 168, 1, 10));
    assert_eq!(config.ipv6_addr, Ipv6Addr::LOCALHOST);
    assert_eq!(
        config.socket_addr,
        "127.0.0.1:8080".parse::<SocketAddr>().unwrap()
    );
    assert_eq!(config.path_buf, PathBuf::from("/tmp/configloader"));
    assert_eq!(config.non_zero_i32, NonZeroI32::new(-7).unwrap());
    assert_eq!(config.non_zero_usize, NonZeroUsize::new(9).unwrap());
}

#[test]
fn loads_url_types() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("URL_CONFIG_BASE_URL", "https://example.com/api");
        std::env::set_var(
            "URL_CONFIG_CALLBACK_URL",
            "https://example.com/oauth/callback?state=ready",
        );
    }

    let config = UrlConfig::load().unwrap();

    assert_eq!(config.base_url.as_str(), "https://example.com/api");
    assert_eq!(config.callback_url.scheme(), "https");
    assert_eq!(config.callback_url.domain(), Some("example.com"));
    assert_eq!(config.callback_url.path(), "/oauth/callback");
    assert_eq!(config.callback_url.query(), Some("state=ready"));
}
