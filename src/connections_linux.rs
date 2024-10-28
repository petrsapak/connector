#[cfg(target_os = "linux")]
use {rustix::io::Errno, rustix::mount::mount, rustix::mount::MountFlags};

#[cfg(target_os = "linux")]
pub fn mount_share(
    source_path: &str,
    mount_target: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), Errno> {
    let mut user = username.as_ref().unwrap().splitn(2, "\\");
    let options = format!(
        "username={},password={},domain={}",
        user.next().unwrap(),
        password.as_ref().unwrap(),
        user.next().unwrap_or("")
    );
    let target_path = format!("{}/{}", mount_target.as_ref().unwrap(), source_path);

    mount(
        format!("//{}", source_path),
        target_path,
        "cifs",
        MountFlags::empty(),
        options,
    )
}
