use anyhow::{anyhow, Result};
use std::net::{IpAddr, Ipv4Addr};

#[cfg(target_os = "linux")]
use {
    regex::Regex, rustix::mount::mount, rustix::mount::MountFlags, std::fs, std::net::ToSocketAddrs,
};

#[cfg(target_os = "linux")]
pub fn mount_share(
    source_path: &str,
    mount_target: &str,
    username: &str,
    password: &str,
) -> Result<()> {
    let user: Vec<&str> = username.split("\\").collect();

    let options = if user.len() == 2 {
        format!(
            "domain={},username={},password={}",
            user[0], user[1], password
        )
    } else {
        format!("username={},password={}", user[0], password)
    };

    let target_path = format!("{}/{}", mount_target, source_path);

    if !fs::exists(target_path.clone()).expect("Cannot check if mount target exists") {
        fs::create_dir_all(target_path.clone()).expect("Cannot create mount target");
    }

    let domain_with_share: Vec<&str> = source_path.split("/").collect();
    let address = resolve_address_if_domain(domain_with_share[0])?;

    mount(
        format!("//{}/{}", address, domain_with_share[1]),
        target_path,
        "cifs",
        MountFlags::empty(),
        options,
    )
    .map_err(|err| err.into())
}

#[cfg(target_os = "linux")]
fn resolve_address_if_domain(address: &str) -> Result<IpAddr> {
    let ip_regex = Regex::new(r"\d+\.\d+\.\d+\.\d+").unwrap();

    if ip_regex.is_match(address) {
        return Ok(address.parse::<Ipv4Addr>().map(IpAddr::V4)?);
    }

    let mut socket_addresses = format!("{}:445", address).to_socket_addrs()?;

    let ipv4_address = socket_addresses
        .find(|addr| addr.is_ipv4())
        .ok_or(anyhow!("No IPv4 found"))?;

    return Ok(ipv4_address.ip());
}
