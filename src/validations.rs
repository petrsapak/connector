use std::fs::read_to_string;

use serde_json::Value;
use console::style;

pub fn invoke() -> Vec<(String, String, String)> {
    let server_list_file_paths = get_list_of_server_file_paths();

    let valid_server_list_file_paths: Vec<(String, String, String)> = server_list_file_paths
        .iter()
        .filter(|server_list_file_path| {
            let mut validation_spinner = cliclack::spinner();
            validation_spinner.start(format!("Validating server list: {}", server_list_file_path));
            match validate_server_list_file(server_list_file_path) {
                Ok(_) => {
                    validation_spinner.stop(style(format!("Validated server list: {}", server_list_file_path)).green().italic());
                    return true;
                },
                Err(error) => {
                    validation_spinner.stop(style(format!("{}", error)).yellow().italic());
                    return false;
                }
            }
        })
        .map(|server_list_file_path| (server_list_file_path.clone(), server_list_file_path.clone(), "".to_string()))
        .collect();
    return valid_server_list_file_paths;
}

fn get_list_of_server_file_paths() -> Vec<String> {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let server_list_file_paths = exe_dir
        .read_dir()
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_file())
        .filter(|entry| entry.path().extension().unwrap_or_default() == "json")
        .map(|entry| entry.path().to_str().unwrap().to_string())
        .collect::<Vec<String>>();
    server_list_file_paths
}

fn validate_server_list_file(server_list_file_path: &str) -> Result<(), String> {
    let server_list_file_content = read_to_string(server_list_file_path).unwrap();

    let server_list_file: Result<Value, _> = serde_json::from_str(&server_list_file_content);
    let server_list_file = match server_list_file {
        Ok(config_file) => config_file,
        Err(error) => {
            return Err(format!("Validation of {} failed. Error: {}", server_list_file_path, error));
        }
    };

    let servers = server_list_file["servers"].as_array().cloned().unwrap_or_default();

    match servers {
        servers if servers.is_empty() => {
            Err(format!("Validation of {} failed. No servers found.", server_list_file_path))
        },
        _ => Ok(())
    }
}

pub fn validate_server_list_file_name(server_list_file_name: &str) -> Result<(), String> {
    let invalid_characters = vec!["\\", "/", ":", "*", "?", "\"", "<", ">", "|"];
    for invalid_character in invalid_characters {
        if server_list_file_name.contains(invalid_character) {
            return Err(format!("Invalid character in server list file name: {}", invalid_character));
        }
    }
    return Ok(());
}
