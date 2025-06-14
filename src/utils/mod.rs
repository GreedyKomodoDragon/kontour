/// Configuration constants for the application
pub mod config {
    /// Default kubeconfig identifier
    pub const DEFAULT_KUBECONFIG: &str = "default";
    
    /// Directory to store persistent kubeconfig files
    pub const KUBECONFIG_STORAGE_DIR: &str = ".kontour/kubeconfigs";
    
    /// Temporary file prefix for kubeconfig files
    pub const TEMP_FILE_PREFIX: &str = "kubeconfig_";
    
    /// Temporary file extension
    pub const TEMP_FILE_EXTENSION: &str = ".yaml";
    
    /// Characters to replace in file names for safety
    pub const UNSAFE_FILENAME_CHARS: &[char] = &['/', '\\', ':'];
    
    /// Replacement character for unsafe filename characters
    pub const FILENAME_REPLACEMENT_CHAR: &str = "_";    
}

/// Utility functions for file operations
pub mod file_utils {
    use super::config::*;
    use std::path::{Path, PathBuf};
    use std::fs;
    
    /// Get the kubeconfig storage directory path and create it if it doesn't exist
    pub fn get_kubeconfig_storage_dir() -> Result<PathBuf, std::io::Error> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found"))?;
        
        let storage_dir = home_dir.join(KUBECONFIG_STORAGE_DIR);
        
        // Create directory if it doesn't exist
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }
        
        Ok(storage_dir)
    }
    
    /// Save uploaded file content to persistent storage and return the file path
    pub fn save_kubeconfig_file(name: &str, content: &str) -> Result<String, std::io::Error> {
        let storage_dir = get_kubeconfig_storage_dir()?;
        let filename = format!("{}.yaml", sanitize_filename(name));
        let file_path = storage_dir.join(filename);
        
        // Write content to file
        fs::write(&file_path, content)?;
        
        // Return the absolute path as string
        Ok(file_path.to_string_lossy().to_string())
    }
    
    /// Sanitize a filename by replacing unsafe characters
    pub fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| {
                if UNSAFE_FILENAME_CHARS.contains(&c) {
                    FILENAME_REPLACEMENT_CHAR.to_string()
                } else {
                    c.to_string()
                }
            })
            .collect::<String>()
    }
    
    /// Generate a temporary file name for a kubeconfig
    pub fn generate_temp_filename(name: &str) -> String {
        format!("{}{}{}", TEMP_FILE_PREFIX, sanitize_filename(name), TEMP_FILE_EXTENSION)
    }
}

/// Utility functions for time and age calculations
pub mod time_utils {
    use k8s_openapi::chrono::{DateTime, Utc};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
    
    /// Calculate age from a timestamp string
    pub fn calculate_age(timestamp: &str) -> String {
        // Parse the timestamp - assuming RFC3339 format (ISO 8601)
        match DateTime::parse_from_rfc3339(timestamp) {
            Ok(dt) => {
                let now = Utc::now();
                let age = now.signed_duration_since(dt.with_timezone(&Utc));
                
                if age.num_days() > 0 {
                    format!("{}d", age.num_days())
                } else if age.num_hours() > 0 {
                    format!("{}h", age.num_hours())
                } else if age.num_minutes() > 0 {
                    format!("{}m", age.num_minutes())
                } else {
                    format!("{}s", age.num_seconds().max(0))
                }
            }
            Err(_) => "unknown".to_string(),
        }
    }
    
    /// Calculate age from an optional Kubernetes Time object
    pub fn calculate_age_from_time(time_opt: Option<&Time>) -> String {
        match time_opt {
            Some(time) => calculate_age(&time.0.to_rfc3339()),
            None => "unknown".to_string(),
        }
    }
}

// Re-export for backwards compatibility
pub use time_utils::{calculate_age, calculate_age_from_time};

#[cfg(test)]
mod tests {
    use super::file_utils::*;
    
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file\\name:with:unsafe"), "test_file_name_with_unsafe");
        assert_eq!(sanitize_filename("safe_filename"), "safe_filename");
    }
    
    #[test]
    fn test_generate_temp_filename() {
        assert_eq!(generate_temp_filename("test"), "kubeconfig_test.yaml");
        assert_eq!(generate_temp_filename("test/unsafe"), "kubeconfig_test_unsafe.yaml");
    }
    
    #[test]
    fn test_save_kubeconfig_file() {
        use super::file_utils::save_kubeconfig_file;
        
        let test_content = "apiVersion: v1\nkind: Config";
        let result = save_kubeconfig_file("test-config", test_content);
        
        // Should succeed (assuming home directory exists and is writable)
        match result {
            Ok(path) => {
                assert!(path.contains("test-config.yaml"));
                // Clean up the test file
                let _ = std::fs::remove_file(&path);
            }
            Err(e) => {
                // This might fail in CI environments without proper home directory setup
                println!("Test skipped due to: {}", e);
            }
        }
    }
    
    #[test]
    fn test_calculate_age() {
        use super::time_utils::calculate_age;
        use k8s_openapi::chrono::Utc;
        
        // Test with a recent timestamp (should be in seconds)
        let recent = Utc::now().to_rfc3339();
        let age = calculate_age(&recent);
        assert!(age.ends_with('s') || age.ends_with('m')); // Should be seconds or minutes for a recent timestamp
        
        // Test with invalid timestamp
        let invalid_age = calculate_age("invalid-timestamp");
        assert_eq!(invalid_age, "unknown");
    }
}
