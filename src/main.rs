use windows_sys::Win32::Foundation::NO_ERROR;
use windows_sys::Win32::NetworkManagement::WNet;
use console::style;
use std::ffi::CString;
use std::fs::read_to_string;
use serde_json::Value;

fn main() -> std::io::Result<()> {

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const CONFIG_FILE_NAME: &str = "appconfig.json";
    let appsettings_content = read_to_string(CONFIG_FILE_NAME)?;
    let appsettings: Value = serde_json::from_str(&appsettings_content)?;
    let username = appsettings["username"].as_str().unwrap_or_default();

    cliclack::intro(format!(" Connector v{} ({})", VERSION, username))?;

    let servers_from_json = appsettings["servers"].as_array().cloned().unwrap_or_default();

    let mut multiselect = servers_from_json
        .iter()
        .fold(cliclack::multiselect("Select servers to connect to"), |multiselect, server| {
            let name = server["name"].as_str().unwrap_or_default();
            let description = server["description"].as_str().unwrap_or_default();
            multiselect.item(name, name, description)
        });

    let _servers = multiselect.interact()?;

    let _password = cliclack::password("Provide password for the servers")
        .mask('#')
        .interact()?;

    let number_of_servers = _servers.len();

    for (index, server) in _servers.into_iter().enumerate() {
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
                    .interact()?;
                if continue_with_next_server {
                    continue;
                } else {
                    break;
                }
            }
        }
    }

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