use std::fs::File;
use std::io::Write;
use std::path::Path;

use console::style;
use serde_json::Value;
use serde_json::json;

use crate::validations;

pub fn create_server_list() -> std::io::Result<()> {

    let mut add_new_server_list = true;

    while add_new_server_list {
        let new_server_list_name: String = cliclack::input("Enter configuration name")
            .placeholder("your-server_list-name")
            .interact()?;

        let name_validation_result = validations::validate_server_list_file_name(&new_server_list_name);
        match name_validation_result {
            Ok(_) => {},
            Err(error) => {
                cliclack::outro(style(format!("{}", error)).yellow().italic())?;
                continue;
            }
        }

        let new_username: String = cliclack::input("Enter username")
            .placeholder("your-username")
            .interact()?;

        let mut add_new_server = true;
        let mut servers: Vec<Value> = Vec::new();

        while add_new_server {
            let server_name: String = cliclack::input("Enter server name")
                .placeholder("my-server")
                .interact()?;

            let server_description: String = cliclack::input(format!("Enter description for {}", server_name))
                .placeholder("description example")
                .required(false)
                .interact()?;

            servers.push(json!({
                "name": server_name,
                "description": server_description
            }));

            add_new_server = cliclack::Confirm::new(style("Would you like to add another server?").yellow().bold()).interact()?;
        }

        let new_server_list = json!({
            "username": new_username,
            "servers": servers
        });

        let json_string = serde_json::to_string_pretty(&new_server_list)?;
        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();
        let new_server_list_path = Path::new(&exe_dir).join(format!("{}.json", new_server_list_name));
        let mut file = File::create(new_server_list_path)?;
        file.write_all(json_string.as_bytes())?;

        add_new_server_list = cliclack::Confirm::new(style("Would you like to add another configuration?").yellow().bold()).interact()?;

    }

    return Ok(())

}