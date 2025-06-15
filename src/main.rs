use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use kube::Client;
use views::{
    ConfigMaps, CreatePod, CronJobs, DaemonSets, Deployments, Home, Ingresses, Jobs, Namespaces, Navbar,
    Nodes, Pods, Pvcs, Secrets, Services, StatefulSets, CreateNamespace, CreateDeployment, CreateStatefulSet,
    CreateDaemonSet, CreateCronJob, Insights
};

mod components;
mod contexts;
mod k8s;
mod views;
mod utils;

use contexts::{FilePathsContext, KubeconfigStorage, ClientReloadContext, create_client_from_path};
use utils::config;

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
    let current_kubeconfig_path = use_signal(|| config::DEFAULT_KUBECONFIG.to_string());
    
    // Create kubeconfig storage - ALWAYS at top level
    let kubeconfig_storage = use_context_provider(|| KubeconfigStorage::default());
    
    // ALWAYS provide client reload context - BEFORE any conditional logic
    use_context_provider(|| ClientReloadContext {
        current_path: current_kubeconfig_path,
    });
    
    // ALWAYS provide file paths context - BEFORE any conditional logic  
    let kubeconfig_paths = use_signal(|| vec![config::DEFAULT_KUBECONFIG.to_string()]);
    use_context_provider(|| FilePathsContext {
        kubeconfig_paths,
    });

    // Resource for managing the Kubernetes client based on the current path
    let client_resource = use_resource({
        let storage = kubeconfig_storage.clone();
        move || {
            let current_path = current_kubeconfig_path();
            println!("DEBUG: Client resource triggering with path: '{}'", current_path);
            let storage = storage.clone();
            async move { 
                println!("DEBUG: About to call create_client_from_path with: '{}'", current_path);
                let result = create_client_from_path(&current_path, &storage).await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
                match &result {
                    Ok(_) => println!("DEBUG: create_client_from_path result: Ok(client)"),
                    Err(e) => println!("DEBUG: create_client_from_path result: Err({})", e),
                }
                result
            }
        }
    });

    // Create a reactive client signal that updates when the resource state changes
    let client_signal = use_signal(|| None::<Client>);
    
    // Update the client signal whenever the resource state changes
    use_effect({
        let mut client_signal = client_signal.clone();
        let client_resource = client_resource.clone();
        move || {
            let client_ref = client_resource.read();
            let client_option: Option<Client> = match &*client_ref {
                None => None, // Still loading
                Some(Err(_)) => None, // Error - no client available
                Some(Ok(client)) => Some(client.clone()), // Success - client available
            };
            client_signal.set(client_option);
        }
    });
    
    use_context_provider(move || client_signal);

    // Read the current client resource state for conditional rendering
    let client_ref = client_resource.read();
    match &*client_ref {
        None => println!("DEBUG: Client resource state: None (loading)"),
        Some(Ok(_)) => println!("DEBUG: Client resource state: Some(Ok(_)) (success)"),
        Some(Err(e)) => println!("DEBUG: Client resource state: Some(Err(_)) (error: {})", e),
    }

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        // Show loading or error banners based on client state
        if let None = &*client_ref {
            // Loading banner
            div {
                class: "loading-banner",
                style: "background: linear-gradient(90deg, #eff6ff 0%, #dbeafe 100%); border: 2px solid #3b82f6; color: #1e40af; padding: 1.5rem; margin: 0; border-radius: 0; position: fixed; top: 0; left: 0; right: 0; z-index: 1000; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1); font-weight: 600;",
                div { 
                    style: "display: flex; align-items: center; gap: 12px;",
                    span { 
                        style: "font-size: 1.5rem;", 
                        "🔄" 
                    }
                    div {
                        div { 
                            style: "font-size: 1.1rem; margin-bottom: 4px;",
                            "Connecting to Kubernetes cluster..." 
                        }
                        div { 
                            style: "font-size: 0.9rem; color: #1d4ed8; font-weight: normal;",
                            "Loading kubeconfig and establishing connection" 
                        }
                    }
                }
            }
        }
        
        if let Some(Err(err)) = &*client_ref {
            // Error banner
            div {
                class: "error-banner",
                style: "background: linear-gradient(90deg, #fef2f2 0%, #fee2e2 100%); border: 2px solid #dc2626; color: #7f1d1d; padding: 1.5rem; margin: 0; border-radius: 0; position: fixed; top: 0; left: 0; right: 0; z-index: 1000; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1); font-weight: 600;",
                div { 
                    style: "display: flex; align-items: center; gap: 12px;",
                    span { 
                        style: "font-size: 1.5rem;", 
                        "⚠️" 
                    }
                    div {
                        div { 
                            style: "font-size: 1.1rem; margin-bottom: 4px;",
                            "Failed to connect to Kubernetes cluster" 
                        }
                        div { 
                            style: "font-size: 0.9rem; color: #991b1b; font-weight: normal;",
                            "Error: {err}" 
                        }
                    }
                }
            }
        }
        
        // Add top margin when banner is shown
        if !matches!(&*client_ref, Some(Ok(_))) {
            div { style: "margin-top: 6rem;" }
        }

        Router::<Route> {}
    }
}
