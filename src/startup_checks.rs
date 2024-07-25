use std::io;
use winreg::enums::*;
use winreg::RegKey;

pub fn add_to_path_evn_variable() -> io::Result<()> {
    let exe_path = std::env::current_exe().unwrap();
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env = hkcu.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment", KEY_READ | KEY_WRITE)?;
    let current_path: String = env.get_value("PATH")?;
    let updated_path = format!("{};{}", current_path, exe_path.parent().unwrap().to_str().unwrap());
    env.set_value("Path", &updated_path)?;
    Ok(())
}

pub fn is_path_env_variable_set() -> io::Result<bool> {
    let exe_path = std::env::current_exe().unwrap();
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env = hkcu.open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")?;
    let current_path: String = env.get_value("PATH")?;

    match current_path.contains(exe_path.parent().unwrap().to_str().unwrap()) {
        true => Ok(true),
        false => Ok(false),
    }
}