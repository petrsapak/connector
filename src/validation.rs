use std::fs::read_to_string;

use serde_json::Value;
use console::style;

pub fn invoke() -> Vec<(String, String, String)> {
    let config_file_paths = get_list_of_config_file_paths();

    let valid_config_file_paths: Vec<(String, String, String)> = config_file_paths
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
        .map(|config_file_path| (config_file_path.clone(), config_file_path.clone(), "".to_string()))
        .collect();

    return valid_config_file_paths;
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
