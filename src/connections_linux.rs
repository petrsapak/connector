#[cfg(target_os = "linux")]
use {rustix::io::Errno, rustix::mount::mount, rustix::mount::MountFlags, std::fs};

#[cfg(target_os = "linux")]
pub fn mount_share(
    source_path: &str,
    mount_target: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), Errno> {
    let user: Vec<&str> = username.unwrap().split("\\").collect();

    let options = if user.len() == 2 {
        format!(
            "domain={},username={},password={}",
            user[0],
            user[1],
            password.unwrap()
        )
    } else {
        format!("username={},password={}", user[0], password.unwrap())
    };

    let target_path = format!("{}/{}", mount_target.unwrap(), source_path);

    if !fs::exists(target_path.clone()).expect("Cannot check if mount target exists") {
        fs::create_dir_all(target_path.clone()).expect("Cannot create mount target");
    }

    mount(
        format!("//{}", source_path),
        target_path,
        "cifs",
        MountFlags::empty(),
        options,
    )
}