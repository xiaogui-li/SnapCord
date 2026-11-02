use tauri::command;

#[command]
pub fn get_screenshots_folder() -> Result<String, String> {
    // Use default Pictures/Screenshots folder
    let home = std::env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let screenshots = format!("{}\\Pictures\\Screenshots", home);
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&screenshots).map_err(|e| e.to_string())?;
    
    Ok(screenshots)
}

#[command]
pub fn open_file_in_explorer(path: String) -> Result<(), String> {
    use std::process::Command;

    Command::new("explorer")
        .arg("/select,")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open explorer: {}", e))?;



    Ok(())
}

#[command]
pub fn open_file_with_default_app(path: String) -> Result<(), String> {
    use std::process::Command;

    Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open file: {}", e))?;

    Ok(())
}

#[command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
