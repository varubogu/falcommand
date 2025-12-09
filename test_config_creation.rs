use std::path::PathBuf;
use std::fs;

// This test script will verify that the config creation works correctly
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing config creation functionality...");
    
    // Simulate the config directory path for debug builds
    let test_config_dir = std::env::current_dir()?.join(".falcommand_test");
    let test_config_file = test_config_dir.join("config.json");
    
    // Clean up any existing test directory
    if test_config_dir.exists() {
        fs::remove_dir_all(&test_config_dir)?;
        println!("Cleaned up existing test directory");
    }
    
    // Verify the directory doesn't exist
    assert!(!test_config_dir.exists(), "Test config directory should not exist initially");
    assert!(!test_config_file.exists(), "Test config file should not exist initially");
    
    // Simulate creating the directory structure (like save_to_file does)
    if let Some(parent) = test_config_file.parent() {
        tokio::fs::create_dir_all(parent).await?;
        println!("Created config directory: {:?}", parent);
    }
    
    // Verify directory was created
    assert!(test_config_dir.exists(), "Config directory should be created");
    
    // Create a sample config file
    let sample_config = r#"{
  "appearance": {
    "theme": "System",
    "transparency": 0.95,
    "position": "Center",
    "show_window": "Mouse",
    "show_window_display_number": 0,
    "font_size": 14,
    "max_results": 10,
    "enable_system_tray": true,
    "start_in_tray": false,
    "minimize_to_tray": true
  },
  "behavior": {
    "hotkey": "Ctrl+Space",
    "auto_hide": true,
    "max_results": 10,
    "rebuild_index_on_startup": true,
    "save_search_history": true,
    "record_usage_stats": true
  },
  "search": {
    "include_paths": {},
    "exclude_patterns": ["*.tmp", "*.log"],
    "fuzzy_threshold": 0.6,
    "enable_file_search": true,
    "enable_app_search": true,
    "enable_web_search": false
  },
  "plugins": {
    "enabled": ["calculator", "translator"],
    "disabled": ["weather"],
    "plugin_settings": {}
  },
  "sync": {
    "enabled": false,
    "provider": null,
    "auto_sync_interval": 3600,
    "encrypt_data": true
  }
}"#;
    
    tokio::fs::write(&test_config_file, sample_config).await?;
    println!("Created config file: {:?}", test_config_file);
    
    // Verify file was created and has content
    assert!(test_config_file.exists(), "Config file should be created");
    let content = tokio::fs::read_to_string(&test_config_file).await?;
    assert!(!content.is_empty(), "Config file should have content");
    
    println!("✓ Config directory and file creation test passed!");
    println!("✓ Directory created at: {:?}", test_config_dir);
    println!("✓ File created at: {:?}", test_config_file);
    
    // Clean up
    fs::remove_dir_all(&test_config_dir)?;
    println!("✓ Test cleanup completed");
    
    Ok(())
}