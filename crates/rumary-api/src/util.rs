use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::OnceLock;

pub fn check_first_time() -> bool {
    static INIT: OnceLock<()> = OnceLock::new();
    // get_or_init возвращает ссылку на значение, если оно уже было, вернем false
    INIT.get().is_none() && {
        INIT.set(()).ok();
        true
    }
}

async fn get_server_ip4() -> Result<Ipv4Addr, Box<dyn std::error::Error>> {
    let ip_str = reqwest::get("https://api.ipify.org").await?.text().await?;
    let ip = Ipv4Addr::from_str(&ip_str)?;
    Ok(ip)
}

pub async fn get_domain() -> Result<String, Box<dyn std::error::Error>> {
    let ip = get_server_ip4().await?;
    let i: IpAddr = ip.into();
    let host = dns_lookup::lookup_addr(&i)?;
    Ok(host)
}

pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>()
        .join("/")
        .to_ascii_lowercase()
}

pub fn matches_rule(path: &str, rules: &[String]) -> bool {
    rules.iter().any(|rule| {
        let normalized_rule = normalize_path(rule);
        path == normalized_rule || path.starts_with(&format!("{normalized_rule}/"))
    })
}
