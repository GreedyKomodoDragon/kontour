use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
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
            let storage = storage.clone();
            async move { 
                create_client_from_path(&current_path, &storage).await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
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
            // Failed to initialize kube client - show error and empty UI
            // Don't use the router when there's no client
            rsx! {
                document::Link { rel: "icon", href: FAVICON }
                document::Link { rel: "stylesheet", href: MAIN_CSS }
                document::Link { rel: "stylesheet", href: TAILWIND_CSS }
                
                div { class: "error-container",
                    // Error banner at the top
                    div {
                        class: "error-banner",
                        style: "background-color: #fef2f2; border: 1px solid #fecaca; color: #dc2626; padding: 1rem; margin: 1rem; border-radius: 0.5rem;",
                        "⚠️ Failed to connect to Kubernetes cluster: {err}"
                    }
                    
                    // Show navbar and empty content (no router)
                    div {
                        style: "display: flex; height: 100vh;",
                        // Show navbar for file management
                        Navbar {}
                        
                        // Empty main content area with message
                        div {
                            style: "flex: 1; display: flex; align-items: center; justify-content: center; flex-direction: column; padding: 2rem;",
                            h2 { 
                                style: "color: #6b7280; font-size: 1.5rem; margin-bottom: 1rem;",
                                "No Kubernetes Connection" 
                            }
                            p { 
                                style: "color: #9ca3af; text-align: center; max-width: 500px;",
                                "Please select a valid kubeconfig file using the dropdown above, or fix the current configuration to continue."
                            }
                        }
                    }
                }
            }
        }
        Some(Ok(client)) => {
            // Successful - provide the client context and show normal UI
            use_context_provider(|| client.clone());

            rsx! {
                document::Link { rel: "icon", href: FAVICON }
                document::Link { rel: "stylesheet", href: MAIN_CSS }
                document::Link { rel: "stylesheet", href: TAILWIND_CSS }

                Router::<Route> {}
            }
        }
    }
}
