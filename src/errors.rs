pub fn get_error_explanation(error_code: i32) -> String {
    match error_code {
        5 => "Access denied".to_string(),
        53 => "Network path not found".to_string(),
        67 => "Network name cannot be found".to_string(),
        86 => "Invalid password".to_string(),
        1219 => "Multiple connections to a server or shared resource by the same user, using more than one user name, are not allowed".to_string(),
        1326 => "The username or password is incorrect".to_string(),
        _ => "Unknown error".to_string()
    }
}