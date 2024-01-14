use windows_sys::Win32::Foundation::NO_ERROR;
use windows_sys::Win32::NetworkManagement::WNet;
use console::style;
use std::ffi::CString;
use std::fs::read_to_string;
use serde_json::Value;

mod new_configuration;
mod validation;

fn main() -> std::io::Result<()> {

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    cliclack::intro(style(format!(" Connector v{}", VERSION)).green().bold())?;

    let mut valid_config_file_paths: Vec<(String, String, String)> = validation::invoke();
    let mut selected_configurations: Vec<String> = Vec::new();

    match valid_config_file_paths.len() {
        0 => {
            let start_first_time_setup = cliclack::Confirm::new(style("No valid configuration files found. Would you like to create one?").yellow().bold()).interact()?;
            match start_first_time_setup {
                true => {
                    let _ = new_configuration::invoke();
                    valid_config_file_paths = validation::invoke();
                },
                false => {
                    cliclack::outro(style("Finished!").yellow().italic())?;
                    return Ok(());
                }
            }
        },
        1 => {
            selected_configurations.push(valid_config_file_paths.first().unwrap().0.clone());
        }
        _ => {
            selected_configurations = cliclack::multiselect("Which server list would you like to use?")
                .items(&valid_config_file_paths)
                .interact()?;
        }
    }

    let number_of_selected_configurations = selected_configurations.len();

    for configuration in selected_configurations {
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
        let number_of_selected_servers = _selected_servers.len();

        let _password = cliclack::password("Provide password for the servers")
            .mask('*')
            .interact()?;

        for (index, server) in _selected_servers.into_iter().enumerate() {
            let mut spinner = cliclack::spinner();
            spinner.start(format!("Connecting to {}...", server));
            let connection_result = connect_to_server(&format!("\\\\{}", server), Some(username), Some(&_password));
            match connection_result {
                Ok(_) => {
                    spinner.stop(style(format!("Connected to {}.", server)).green().bold());
                },
                Err(error_code) => {
                    spinner.stop(style(format!("Failed to connect to {}. Error code: {}.", server, error_code)).red().italic());
                    if index == number_of_selected_servers - 1 && index == number_of_selected_configurations {
                        cliclack::outro(style("Finished!").yellow().italic())?;
                        return Ok(());
                    }
                    let continue_with_next_server = cliclack::Confirm::new("Would you like to continue?")
                        .interact()?;
                    if continue_with_next_server {
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
    };

    cliclack::outro(style("Finished!").green().bold())?;

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