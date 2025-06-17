use crate::{Route, contexts::{FilePathsContext, ClientReloadContext, KubeconfigStorage}};
use crate::components::kubeconfig_name_dialog::KubeconfigNameDialog;
use crate::utils::file_utils;
use dioxus::{logger::tracing, prelude::*};
use kube::Client;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

// Import the asset macro for static assets
const OVERVIEW: Asset = asset!("/assets/images/overview.svg");
const NODES: Asset = asset!("/assets/images/nodes.svg");
const NAMESPACE: Asset = asset!("/assets/images/namespace.svg");
const DEPLOYMENT: Asset = asset!("/assets/images/deployment.svg");
const POD: Asset = asset!("/assets/images/pod.svg");
const STATEFULSETS: Asset = asset!("/assets/images/statefulset.svg");
const DAEMONSETS: Asset = asset!("/assets/images/daemonset.svg");
const CRONJOB: Asset = asset!("/assets/images/cronjob.svg");
const JOB: Asset = asset!("/assets/images/job.svg");
const SERVICE: Asset = asset!("/assets/images/service.svg");
const INGRESS: Asset = asset!("/assets/images/ingress.svg");
const PVC: Asset = asset!("/assets/images/pvc.svg");
const CONFIGMAP: Asset = asset!("/assets/images/configmap.svg");
const SECRET: Asset = asset!("/assets/images/secret.svg");
const INSIGHTS: Asset = asset!("/assets/images/insights.svg");

// Navigation item data structure
#[derive(Clone)]
struct NavItem {
    route: Route,
    icon: Asset,
    label: &'static str,
    class: &'static str,
}

#[component]
pub fn Navbar() -> Element {
    // Get contexts
    let file_paths_context = use_context::<FilePathsContext>();
    let client_reload_context = use_context::<ClientReloadContext>();
    let kubeconfig_storage = use_context::<KubeconfigStorage>();
    let client_signal = try_use_context::<Signal<Option<Client>>>();
    
    let has_client = client_signal
        .map(|signal| signal.read().is_some())
        .unwrap_or(false);
    
    // State management
    let mut filenames = file_paths_context.kubeconfig_paths;
    let mut dialog_state = use_signal::<Option<(String, String)>>(|| None);
    let mut input_key = use_signal(|| 0);
    let mut selected_context = use_signal(|| {
        let paths = filenames.read();
        paths.first().cloned().unwrap_or_default()
    });

    // Effects
    use_effect({
        let mut selected_context = selected_context.clone();
        move || {
            let paths = filenames.read();
            let should_update = {
                let current_selection = selected_context.read();
                current_selection.is_empty() || !paths.contains(&current_selection)
            };
            if should_update {
                if let Some(first_path) = paths.first() {
                    selected_context.set(first_path.clone());
                }
            }
        }
    });

    use_effect({
        let mut current_path = client_reload_context.current_path;
        move || {
            let new_path = selected_context();
            if !new_path.is_empty() {
                tracing::info!("Context changed to: {}", new_path);
                current_path.set(new_path);
            }
        }
    });

    // Navigation items configuration
    let cluster_nav_items = vec![
        NavItem { route: Route::Home {}, icon: OVERVIEW, label: "Overview", class: "nav-overview" },
        NavItem { route: Route::Insights {}, icon: INSIGHTS, label: "Insights", class: "nav-insights" },
        NavItem { route: Route::Nodes {}, icon: NODES, label: "Nodes", class: "nav-nodes" },
        NavItem { route: Route::Namespaces {}, icon: NAMESPACE, label: "Namespaces", class: "nav-namespaces" },
    ];

    let workload_nav_items = vec![
        NavItem { route: Route::Pods {}, icon: POD, label: "Pods", class: "nav-pods" },
        NavItem { route: Route::Deployments {}, icon: DEPLOYMENT, label: "Deployments", class: "nav-deployments" },
        NavItem { route: Route::StatefulSets {}, icon: STATEFULSETS, label: "StatefulSets", class: "nav-statefulsets" },
        NavItem { route: Route::DaemonSets {}, icon: DAEMONSETS, label: "DaemonSets", class: "nav-daemonsets" },
        NavItem { route: Route::CronJobs {}, icon: CRONJOB, label: "CronJobs", class: "nav-cronjobs" },
        NavItem { route: Route::Jobs {}, icon: JOB, label: "Jobs", class: "nav-jobs" },
    ];

    let network_nav_items = vec![
        NavItem { route: Route::Services {}, icon: SERVICE, label: "Services", class: "nav-services" },
        NavItem { route: Route::Ingresses {}, icon: INGRESS, label: "Ingress", class: "nav-ingress" },
    ];

    let storage_nav_items = vec![
        NavItem { route: Route::Pvcs {}, icon: PVC, label: "Persistent Volume Claims", class: "nav-pvcs" },
        NavItem { route: Route::ConfigMaps {}, icon: CONFIGMAP, label: "Config Maps", class: "nav-configmaps" },
        NavItem { route: Route::Secrets {}, icon: SECRET, label: "Secrets", class: "nav-secrets" },
    ];

    // Helper function to render navigation group
    let render_nav_group = |title: &str, items: &[NavItem]| {
        rsx! {
            div { class: "nav-group",
                span { class: "nav-group-title", "{title}" }
                {items.iter().map(|item| rsx! {
                    Link {
                        key: "{item.label}",
                        to: item.route.clone(),
                        class: "{item.class}",
                        img { src: "{item.icon}", alt: "", class: "nav-icon" }
                        "{item.label}"
                    }
                })}
            }
        }
    };

    // File upload handler
    let handle_file_upload = move |evt: Event<FormData>| {
        let mut dialog_state = dialog_state.clone();
        spawn(async move {
            if let Some(file_engine) = evt.files() {
                let files = file_engine.files();
                if let Some(file_name) = files.first() {
                    if let Some(content) = file_engine.read_file(file_name).await {
                        let content_str = String::from_utf8_lossy(&content).to_string();
                        dialog_state.set(Some((file_name.clone(), content_str)));
                        tracing::info!("File read successfully: {} bytes", content.len());
                    } else {
                        tracing::error!("Failed to read file {}", file_name);
                    }
                }
            }
        });
    };

    // Dialog close handler
    let handle_dialog_close = move |result: Option<String>| {
        if let Some(name) = result {
            tracing::info!("Kubeconfig context named: {}", name);
            
            // Get the file content from dialog state
            if let Some((_, file_content)) = dialog_state() {
                match file_utils::save_kubeconfig_file(&name, &file_content) {
                    Ok(file_path) => {
                        if let Err(e) = kubeconfig_storage.store_file_path(name.clone(), file_path) {
                            tracing::error!("Failed to store kubeconfig file path: {}", e);
                            return;
                        }
                        
                        let mut current_files = filenames.write();
                        if !current_files.contains(&name) {
                            current_files.push(name.clone());
                            selected_context.set(name);
                        }
                    }
                    Err(e) => tracing::error!("Failed to save kubeconfig file: {}", e),
                }
            }
        }
        dialog_state.set(None);
        input_key += 1;
    };

    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        // Dialog
        if let Some((original_filename, _)) = dialog_state() {
            KubeconfigNameDialog {
                original_filename: original_filename.clone(),
                on_close: handle_dialog_close
            }
        }

        div { class: "layout-container",
            div { id: "sidebar", class: "k8s-sidebar",
                div { class: "sidebar-logo",
                    img { src: "/assets/kubernetes-logo.svg", alt: "Kubernetes Logo" }
                    span { "Kubernetes Dashboard" }
                }
                
                nav { class: "sidebar-links",
                    // Cluster Management Section
                    div { class: "nav-group",
                        span { class: "nav-group-title", "CLUSTER MANAGEMENT" }
                        
                        div { class: "cluster-selector",
                            label { r#for: "cluster-select", class: "sr-only", "Select Cluster" }
                            select {
                                id: "cluster-select",
                                class: "cluster-dropdown",
                                value: "{selected_context}",
                                onchange: move |evt| {
                                    let new_selection = evt.value();
                                    tracing::info!("User selected context: {}", new_selection);
                                    selected_context.set(new_selection);
                                },
                                option { 
                                    value: "", 
                                    disabled: true, 
                                    selected: selected_context.read().is_empty(),
                                    "Select kubeconfig file"
                                }
                                {filenames.read().iter().map(|filepath| {
                                    let display_name = filepath.split('/').last().unwrap_or(filepath);
                                    rsx! {
                                        option { 
                                            key: "{filepath}", 
                                            value: "{filepath}",
                                            title: "{filepath}",
                                            "{display_name}"
                                        }
                                    }
                                })}
                            }
                            
                            if !filenames.read().is_empty() {
                                div { class: "file-count-info",
                                    "{filenames.read().len()} kubeconfig file(s) loaded"
                                }
                            }
                        }

                        input {
                            key: "{input_key()}",
                            r#type: "file",
                            accept: ".yaml,.kubeconfig,.yml",
                            id: "kubeconfig-upload",
                            hidden: true,
                            onchange: handle_file_upload
                        }
                        
                        label {
                            r#for: "kubeconfig-upload",
                            class: "add-kubeconfig-button",
                            tabindex: 0,
                            role: "button",
                            "Add Kubeconfig"
                        }
                    }
                    
                    // Navigation sections
                    if has_client {
                        {render_nav_group("CLUSTER", &cluster_nav_items)}
                        {render_nav_group("WORKLOADS", &workload_nav_items)}
                        {render_nav_group("NETWORK", &network_nav_items)}
                        {render_nav_group("STORAGE", &storage_nav_items)}
                    } else {
                        div { class: "nav-group",
                            div { 
                                style: "padding: 1rem; color: #9ca3af; text-align: center; font-size: 0.875rem;",
                                "🔌 No cluster connection"
                            }
                            div { 
                                style: "padding: 0 1rem; color: #6b7280; text-align: center; font-size: 0.75rem; line-height: 1.4;",
                                "Select a valid kubeconfig above to enable navigation"
                            }
                        }
                    }
                }
            }
            
            div { class: "main-content",
                Outlet::<Route> {}
            }
        }
    }
}
