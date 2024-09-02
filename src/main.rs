use console::style;
use std::fs::read_to_string;
use serde_json::Value;

mod server_list;
mod validations;
mod errors;
mod connections;
mod startup_checks;

fn main() -> std::io::Result<()> {

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    cliclack::intro(style(format!(" Connector v{}", VERSION)).green().bold())?;

    let valid_server_list_file_paths: Vec<(String, String, String)> = validations::invoke();

    let env_variable_path_check = startup_checks::is_path_env_variable_set();
    match env_variable_path_check {
        Ok(true) => (),
        Ok(false) => {
            let add_to_path = cliclack::Confirm::new(style("The PATH environment variable is not set. Would you like to set it now?").yellow().bold()).interact()?;
            match add_to_path {
                true => {
                    let add_to_path_env_result = startup_checks::add_to_path_evn_variable();
                    match add_to_path_env_result {
                        Ok(_) => cliclack::outro(style("The PATH environment variable has been set.").green().bold())?,
                        Err(error) => cliclack::outro(style(format!("Failed to set the PATH environment variable. {}. You need to have admin rights.", error)).red().bold())?

                    }
                },
                false => ()
            }
        },
        Err(_) => ()
    }

    let mut selected_server_lists: Vec<String> = Vec::new();

    match valid_server_list_file_paths.len() {
        0 => {
            let start_first_time_setup = cliclack::Confirm::new(style("No valid server list files found. Would you like to create one?").yellow().bold()).interact()?;
            match start_first_time_setup {
                true => {
                    let _ = server_list::create_server_list();
                    validations::invoke();
                },
                false => {
                    cliclack::outro(style("Finished!").yellow().italic())?;
                    return Ok(());
                }
            }
        },
        1 => {
            selected_server_lists.push(valid_server_list_file_paths.first().unwrap().0.clone());
        }
        _ => {
            selected_server_lists = cliclack::multiselect("Which list of servers would you like to use?")
                .items(&valid_server_list_file_paths)
                .interact()?;
        }
    }

    let number_of_selected_configurations = selected_server_lists.len();

    for configuration in selected_server_lists {
        let configuration_content = read_to_string(&configuration)?;
        let configuration: Value = serde_json::from_str(&configuration_content)?;
        let username = configuration["username"].as_str().unwrap_or_default();
        let servers = configuration["servers"].as_array().cloned().unwrap_or_default();
        let available_servers_from_file = servers
            .iter()
            .map(|server|
                (server["name"].as_str().unwrap_or_default(),
                 server["name"].as_str().unwrap_or_default(),
                 server["description"].as_str().unwrap_or_default())
        );

        let available_servers_default = vec![("select all","select all","")];

        let mut available_servers = Vec::new();
        available_servers.extend(available_servers_default);
        available_servers.extend(available_servers_from_file.clone());

        let mut _selected_servers = Vec::new();

        let _selected_servers_by_user = cliclack::multiselect(format!("Select servers to connect to (as {})", username))
            .items(&available_servers.iter().map(|&(a, b, c)| (a, b, c)).collect::<Vec<_>>())
            .interact()?;

        if _selected_servers_by_user.contains(&"select all") {
            _selected_servers = available_servers_from_file.into_iter().map(|server| server.0.to_string()).collect();
        } else {
            _selected_servers = _selected_servers_by_user.iter().map(|&server| server.to_string()).collect();
        }


        let number_of_selected_servers = _selected_servers.len();

        let _password = cliclack::password("Provide password for the servers")
            .mask('*')
            .interact()?;

        for (index, server) in _selected_servers.into_iter().enumerate() {
            let mut spinner = cliclack::spinner();
            let index_for_display = index + 1;
            spinner.start(format!("[{}\\{}] Connecting to {}...", index_for_display, number_of_selected_servers, server));
            let connection_result = connections::create_connection(&format!("\\\\{}", server), Some(username), Some(&_password));
            match connection_result {
                Ok(_) => {
                    spinner.stop(style(format!("[{}\\{}] Connected to {}.", index_for_display, number_of_selected_servers, server)).green().bold());
                },
                Err(error_code) => {
                    spinner.stop(style(format!("[{}\\{}] Failed to connect to {}.\r\n   Error code: {}: {}.", index_for_display, number_of_selected_servers, server, error_code, errors::get_error_explanation(error_code))).red().italic());
                    if index == number_of_selected_servers - 1 && index == number_of_selected_configurations {
                        cliclack::outro(style("Finished!").yellow().italic())?;
                        return Ok(());
                    }
                    let continue_with_next_server = cliclack::Confirm::new(format!("Remaining servers in the queue: {}. Would you like to continue?", number_of_selected_servers - index_for_display)).interact()?;
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
