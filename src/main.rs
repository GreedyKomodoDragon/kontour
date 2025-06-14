use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use kube::Client;
use std::path::Path;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use views::{
    ConfigMaps, CreatePod, CronJobs, DaemonSets, Deployments, Home, Ingresses, Jobs, Namespaces, Navbar,
    Nodes, Pods, Pvcs, Secrets, Services, StatefulSets, CreateNamespace, CreateDeployment, CreateStatefulSet,
    CreateDaemonSet, CreateCronJob, Insights
};

mod components;
mod k8s;
mod views;
mod utils;

// Global storage for kubeconfig files content
static KUBECONFIG_STORAGE: std::sync::LazyLock<Arc<Mutex<HashMap<String, String>>>> = 
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

// Context for sharing file paths
#[derive(Clone, Default)]
pub struct FilePathsContext {
    pub kubeconfig_paths: Signal<Vec<String>>,
}

// Context for managing file content storage
#[derive(Clone)]
pub struct KubeconfigStorage {
    storage: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for KubeconfigStorage {
    fn default() -> Self {
        Self {
            storage: KUBECONFIG_STORAGE.clone(),
        }
    }
}

impl KubeconfigStorage {
    pub fn store_content(&self, name: String, content: String) {
        if let Ok(mut storage) = self.storage.lock() {
            storage.insert(name, content);
        }
    }

    pub fn get_content(&self, name: &str) -> Option<String> {
        if let Ok(storage) = self.storage.lock() {
            storage.get(name).cloned()
        } else {
            None
        }
    }
}

// Context for managing Kubernetes client reload
#[derive(Clone)]
pub struct ClientReloadContext {
    pub current_path: Signal<String>,
}

// Function to create a client from a kubeconfig path
async fn create_client_from_path(path: &str, storage: &KubeconfigStorage) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    if path == "default" {
        // Use default kubeconfig
        Client::try_default().await.map_err(|e| e.into())
    } else if Path::new(path).exists() {
        // Load from specific file - set the KUBECONFIG environment variable temporarily
        let original_kubeconfig = std::env::var("KUBECONFIG").ok();
        std::env::set_var("KUBECONFIG", path);
        
        let result = Client::try_default().await;
        
        // Restore original KUBECONFIG if it existed, or remove it
        match original_kubeconfig {
            Some(original) => std::env::set_var("KUBECONFIG", original),
            None => std::env::remove_var("KUBECONFIG"),
        }
        
        result.map_err(|e| e.into())
    } else if let Some(content) = storage.get_content(path) {
        // Load from stored content by writing to a temporary file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("kubeconfig_{}.yaml", path.replace("/", "_")));
        
        std::fs::write(&temp_file, content)?;
        
        let original_kubeconfig = std::env::var("KUBECONFIG").ok();
        std::env::set_var("KUBECONFIG", temp_file.to_string_lossy().to_string());
        
        let result = Client::try_default().await;
        
        // Clean up
        let _ = std::fs::remove_file(&temp_file);
        match original_kubeconfig {
            Some(original) => std::env::set_var("KUBECONFIG", original),
            None => std::env::remove_var("KUBECONFIG"),
        }
        
        result.map_err(|e| e.into())
    } else {
        Err(format!("Kubeconfig not found: {}", path).into())
    }
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
        #[route("/nodes")]
        Nodes {},
        #[route("/namespaces")]
        Namespaces {},
        #[route("/namespaces/create")]
        CreateNamespace {},
        #[route("/insights")]
        Insights {},
        #[route("/pods")]
        Pods {},
        #[route("/pods/create")]
        CreatePod {},
        #[route("/deployments")]
        Deployments {},
        #[route("/deployments/create")]
        CreateDeployment {},
        #[route("/statefulsets")]
        StatefulSets {},
        #[route("/statefulsets/create")]
        CreateStatefulSet {},
        #[route("/daemonsets")]
        DaemonSets {},
        #[route("/daemonsets/create")]
        CreateDaemonSet {},
        #[route("/cronjobs")]
        CronJobs {},
        #[route("/cronjobs/create")]
        CreateCronJob {},
        #[route("/jobs")]
        Jobs {},
        #[route("/services")]
        Services {},
        #[route("/ingresses")]
        Ingresses {},
        #[route("/pvcs")]
        Pvcs {},
        #[route("/configmaps")]
        ConfigMaps {},
        #[route("/secrets")] 
        Secrets {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new().with_title("Kontour")
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    // Signal to track the current kubeconfig path
    let current_kubeconfig_path = use_signal(|| "default".to_string());
    
    // Create kubeconfig storage
    let kubeconfig_storage = use_context_provider(|| KubeconfigStorage::default());

    // Resource for managing the Kubernetes client based on the current path
    let client_resource = use_resource({
        let storage = kubeconfig_storage.clone();
        move || {
            let current_path = current_kubeconfig_path();
            let storage = storage.clone();
            async move { 
                create_client_from_path(&current_path, &storage).await
            }
        }
    });

    let client_ref = client_resource.read();

    match &*client_ref {
        None => {
            // Still loading
            return rsx!(div { "Loading Kubernetes client..." });
        }
        Some(Err(err)) => {
            // Failed to initialize kube client
            rsx! {
                div {
                    // Show error banner
                    div {
                        class: "error-banner",
                        "⚠️ Failed to initialize Kubernetes client: {err}. Some features may not work."
                    }

                    document::Link { rel: "icon", href: FAVICON }
                    document::Link { rel: "stylesheet", href: MAIN_CSS }
                    document::Link { rel: "stylesheet", href: TAILWIND_CSS }

                    Router::<Route> {}
                }
            }
        }
        Some(Ok(client)) => {
            // Successful - provide contexts
            use_context_provider(|| client.clone());
            
            // Provide client reload context
            use_context_provider(|| ClientReloadContext {
                current_path: current_kubeconfig_path,
            });
            
            // Provide file paths context with a signal
            let kubeconfig_paths = use_signal(|| vec!["default".to_string()]);
            use_context_provider(|| FilePathsContext {
                kubeconfig_paths,
            });

            rsx! {
                document::Link { rel: "icon", href: FAVICON }
                document::Link { rel: "stylesheet", href: MAIN_CSS }
                document::Link { rel: "stylesheet", href: TAILWIND_CSS }

                Router::<Route> {}
            }
        }
    }
}
