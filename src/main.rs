use console::style;
use std::fs::read_to_string;
use serde_json::Value;

mod configurations;
mod validations;
mod errors;
mod connections;

fn main() -> std::io::Result<()> {

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    cliclack::intro(style(format!(" Connector v{}", VERSION)).green().bold())?;

    let mut valid_config_file_paths: Vec<(String, String, String)> = validations::invoke();
    let mut selected_configurations: Vec<String> = Vec::new();

    match valid_config_file_paths.len() {
        0 => {
            let start_first_time_setup = cliclack::Confirm::new(style("No valid configuration files found. Would you like to create one?").yellow().bold()).interact()?;
            match start_first_time_setup {
                true => {
                    let _ = configurations::create_configuration();
                    valid_config_file_paths = validations::invoke();
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
            let connection_result = connections::create_connection(&format!("\\\\{}", server), Some(username), Some(&_password));
            match connection_result {
                Ok(_) => {
                    spinner.stop(style(format!("Connected to {}.", server)).green().bold());
                },
                Err(error_code) => {
                    spinner.stop(style(format!("Failed to connect to {}.\r\n   Error code: {}: {}.", server, error_code, errors::get_error_explanation(error_code))).red().italic());
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
