use windows_sys::Win32::Foundation::NO_ERROR;
use windows_sys::Win32::NetworkManagement::WNet;
use console::style;
use std::ffi::CString;
use std::fs::read_to_string;
use serde_json::Value;

fn main() -> std::io::Result<()> {

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    cliclack::intro(style(format!(" Connector v{}", VERSION)).green().bold())?;

    let config_file_paths = get_list_of_config_file_paths();

    let valid_config_file_paths = config_file_paths
        .iter()
        .filter(|config_file_path| {
            let mut validation_spinner = cliclack::spinner();
            validation_spinner.start(format!("Validating {}", config_file_path));
            match validate_config_file(config_file_path) {
                Ok(_) => {
                    validation_spinner.stop(style(format!("Validated {}", config_file_path)).green().italic());
                    return true;
                },
                Err(error) => {
                    validation_spinner.stop(style(format!("{}", error)).yellow().italic());
                    return false;
                }
            }
        })
        .map(|config_file_path| (config_file_path.as_str(), config_file_path.as_str(), ""));

    let _selected_configurations = cliclack::multiselect("Select configurations to use")
        .items(&valid_config_file_paths.collect::<Vec<(_, _, _,)>>())
        .interact()?;

    for configuration in _selected_configurations {
        let configuration_content = read_to_string(&configuration)?;
        let configuration: Value = serde_json::from_str(&configuration_content)?;
        let username = configuration["username"].as_str().unwrap_or_default();
        let servers = configuration["servers"].as_array().cloned().unwrap_or_default();
        let available_servers = servers
            .iter()
            .map(|server|
                (server["name"].as_str().unwrap_or_default(),
                server["name"].as_str().unwrap_or_default(),
                server["description"].as_str().unwrap_or_default())
        );
        let _selected_servers = cliclack::multiselect(format!("Select servers to connect to (as {})", username))
            .items(&available_servers.collect::<Vec<(_, _, _)>>())
            .interact()?;
        let _password = cliclack::password("Provide password for the servers")
            .mask('*')
            .interact()?;

        let number_of_servers = _selected_servers.len();
        for (index, server) in _selected_servers.into_iter().enumerate() {
            let mut spinner = cliclack::spinner();
            spinner.start(format!("Connecting to {}", server));
            let connection_result = connect_to_server(&format!("\\\\{}", server), Some(username), Some(&_password));
            match connection_result {
                Ok(_) => {
                    spinner.stop(style(format!("Connected to {}", server)).green().bold());
                },
                Err(error_code) => {
                    spinner.stop(style(format!("Failed to connect to {}. Error code: {}.", server, error_code)).red().bold());
                    if index == number_of_servers - 1 {
                        cliclack::outro("Finished!")?;
                        return Ok(());
                    }
                    let continue_with_next_server = cliclack::Confirm::new("Would you like to continue?")
                        .interact().map_err(|e| e.to_string());
                    if continue_with_next_server.unwrap() {
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
    };

    cliclack::outro(style("Finished!").yellow())?;

    Ok(())

}

fn connect_to_server(server: &str, username: Option<&str>, password: Option<&str>) -> Result<(), i32>
{
    let mut resources = WNet::NETRESOURCEA {
        dwDisplayType: WNet::RESOURCEDISPLAYTYPE_SHAREADMIN,
        dwScope: WNet::RESOURCE_GLOBALNET,
        dwType: WNet::RESOURCETYPE_DISK,
        dwUsage: WNet::RESOURCEUSAGE_ALL,
        lpComment: std::ptr::null_mut(),
        lpLocalName: std::ptr::null_mut(),
        lpProvider: std::ptr::null_mut(),
        lpRemoteName: CString::new(server).unwrap().into_raw() as *mut u8,
    };

    let username = username.as_ref().map(|username| CString::new(*username).unwrap());
    let password = password.as_ref().map(|password| CString::new(*password).unwrap());

    let result = unsafe {
        let username_ptr = username
            .as_ref()
            .map(|username| username.as_ptr())
            .unwrap_or(std::ptr::null());
        let password_ptr = password
            .as_ref()
            .map(|password| password.as_ptr())
            .unwrap_or(std::ptr::null());
        WNet::WNetAddConnection2A(
            &mut resources as *mut WNet::NETRESOURCEA,
            password_ptr as *const u8,
            username_ptr as *const u8,
            WNet::CONNECT_TEMPORARY,
        )
    };

    if result == NO_ERROR {
        Ok(())
    } else {
        Err(result as i32)
    }
}

fn get_list_of_config_file_paths() -> Vec<String> {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let config_file_paths = exe_dir
        .read_dir()
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_file())
        .filter(|entry| entry.path().extension().unwrap_or_default() == "json")
        .map(|entry| entry.path().to_str().unwrap().to_string())
        .collect::<Vec<String>>();
    config_file_paths
}

fn validate_config_file(config_file_path: &str) -> Result<(), String> {
    let config_file_content = read_to_string(config_file_path).unwrap();

    let config_file: Result<Value, _> = serde_json::from_str(&config_file_content);
    let config_file = match config_file {
        Ok(config_file) => config_file,
        Err(error) => {
            return Err(format!("Validation of {} failed. Error: {}", config_file_path, error));
        }
    };

    let servers = config_file["servers"].as_array().cloned().unwrap_or_default();

    match servers {
        servers if servers.is_empty() => {
            Err(format!("Validation of {} failed. No servers found.", config_file_path))
        },
        _ => Ok(())
    }
}
