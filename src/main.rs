use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use kube::Client;
use views::{
    ConfigMaps, CreatePod, CronJobs, DaemonSets, Deployments, Home, Ingresses, Jobs, Namespaces, Navbar,
    Nodes, Pods, Pvcs, Secrets, Services, StatefulSets, CreateNamespace, CreateDeployment, CreateStatefulSet,
    CreateDaemonSet, CreateCronJob, Insights
};

mod components;
mod k8s;
mod views;
mod utils;

// Context for sharing file paths
#[derive(Clone, Default)]
pub struct FilePathsContext {
    pub kubeconfig_paths: Vec<String>,
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
    let client = use_resource(|| async move { Client::try_default().await });

    let client_ref = client.read();

    match &*client_ref {
        None => {
            // Still loading
            return rsx!(div { "Loading Kubernetes client..." });
        }
        Some(Err(_err)) => {
            // Failed to initialize kube client
            rsx! {
                div {
                    // Show error banner
                    div {
                        class: "error-banner",
                        "⚠️ Failed to initialize Kubernetes client. Some features may not work."
                    }

                    document::Link { rel: "icon", href: FAVICON }
                    document::Link { rel: "stylesheet", href: MAIN_CSS }
                    document::Link { rel: "stylesheet", href: TAILWIND_CSS }

                    Router::<Route> {}
                }
            }
        }
        Some(Ok(client)) => {
            // Successful
            use_context_provider(|| client.clone());
            
            // Provide example file paths context
            use_context_provider(|| FilePathsContext {
                kubeconfig_paths: vec![],
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
