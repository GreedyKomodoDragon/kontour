use dioxus::prelude::*;
use kube::Client;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::error::{KubeconfigError, KubeconfigResult};
use crate::utils::config;

// Global storage for kubeconfig file paths (name -> file path mapping)
static KUBECONFIG_STORAGE: std::sync::LazyLock<Arc<Mutex<HashMap<String, String>>>> = 
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Context for sharing file paths across the application
#[derive(Clone, Default)]
pub struct FilePathsContext {
    pub kubeconfig_paths: Signal<Vec<String>>,
}

/// Context for managing kubeconfig file paths storage
#[derive(Clone)]
pub struct KubeconfigStorage {
    storage: Arc<Mutex<HashMap<String, String>>>, // name -> file_path mapping
}

impl Default for KubeconfigStorage {
    fn default() -> Self {
        Self {
            storage: KUBECONFIG_STORAGE.clone(),
        }
    }
}

impl KubeconfigStorage {
    /// Store kubeconfig file path with a given name
    pub fn store_file_path(&self, name: String, file_path: String) -> KubeconfigResult<()> {
        if let Ok(mut storage) = self.storage.lock() {
            storage.insert(name, file_path);
            Ok(())
        } else {
            Err(KubeconfigError::StorageError("Failed to acquire storage lock".to_string()))
        }
    }

    /// Retrieve kubeconfig file path by name
    pub fn get_file_path(&self, name: &str) -> KubeconfigResult<Option<String>> {
        if let Ok(storage) = self.storage.lock() {
            Ok(storage.get(name).cloned())
        } else {
            Err(KubeconfigError::StorageError("Failed to acquire storage lock".to_string()))
        }
    }

    /// Remove kubeconfig file path by name
    pub fn remove_file_path(&self, name: &str) -> KubeconfigResult<bool> {
        if let Ok(mut storage) = self.storage.lock() {
            Ok(storage.remove(name).is_some())
        } else {
            Err(KubeconfigError::StorageError("Failed to acquire storage lock".to_string()))
        }
    }

    /// List all stored kubeconfig names
    pub fn list_names(&self) -> KubeconfigResult<Vec<String>> {
        if let Ok(storage) = self.storage.lock() {
            Ok(storage.keys().cloned().collect())
        } else {
            Err(KubeconfigError::StorageError("Failed to acquire storage lock".to_string()))
        }
    }
}

/// Context for managing Kubernetes client reload
#[derive(Clone)]
pub struct ClientReloadContext {
    pub current_path: Signal<String>,
}

/// Function to create a Kubernetes client from a kubeconfig path
pub async fn create_client_from_path(
    name_or_path: &str, 
    storage: &KubeconfigStorage
) -> KubeconfigResult<Client> {
    if name_or_path == config::DEFAULT_KUBECONFIG {
        // Use default kubeconfig
        Client::try_default().await.map_err(|e| e.into())
    } else {
        // First check if this is a stored name that maps to a file path
        let file_path = if let Some(stored_path) = storage.get_file_path(name_or_path)? {
            stored_path
        } else if Path::new(name_or_path).exists() {
            // It's already a file path
            name_or_path.to_string()
        } else {
            return Err(KubeconfigError::NotFound(name_or_path.to_string()));
        };
        
        // Verify the file exists before trying to use it
        if !Path::new(&file_path).exists() {
            return Err(KubeconfigError::FileNotFound(file_path));
        }
        
        create_client_from_file_path(&file_path).await
    }
}

/// Create a client from a file path
async fn create_client_from_file_path(
    path: &str
) -> KubeconfigResult<Client> {
    let original_kubeconfig = std::env::var("KUBECONFIG").ok();
    std::env::set_var("KUBECONFIG", path);
    
    let result = Client::try_default().await;
    
    // Restore original KUBECONFIG if it existed, or remove it
    match original_kubeconfig {
        Some(original) => std::env::set_var("KUBECONFIG", original),
        None => std::env::remove_var("KUBECONFIG"),
    }
    
    result.map_err(|e| e.into())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kubeconfig_storage() {
        let storage = KubeconfigStorage::default();
        
        // Test storing and retrieving file paths
        storage.store_file_path("test".to_string(), "/path/to/test-config".to_string()).unwrap();
        let path = storage.get_file_path("test").unwrap();
        assert_eq!(path, Some("/path/to/test-config".to_string()));
        
        // Test listing names
        let names = storage.list_names().unwrap();
        assert!(names.contains(&"test".to_string()));
        
        // Test removing file path
        assert!(storage.remove_file_path("test").unwrap());
        let path = storage.get_file_path("test").unwrap();
        assert_eq!(path, None);
    }
}
