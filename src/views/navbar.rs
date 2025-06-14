use crate::{Route, contexts::{FilePathsContext, ClientReloadContext, KubeconfigStorage}};
// Import the new dialog component
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


#[component]
pub fn Navbar() -> Element {
    // Get file paths from context - provide default if not available
    let file_paths_context = use_context::<FilePathsContext>();
    
    // Get client reload context
    let client_reload_context = use_context::<ClientReloadContext>();
    
    // Get kubeconfig storage
    let kubeconfig_storage = use_context::<KubeconfigStorage>();
    
    // Check if we have a Kubernetes client available (optional, might not exist in error state)
    let has_client = try_use_context::<Client>().is_some();
    
    // Use the signal from context instead of local state
    let mut filenames = file_paths_context.kubeconfig_paths;
    
    // State to manage the dialog: Option<(original_filename, file_content)>
    let mut dialog_state = use_signal::<Option<(String, String)>>(|| None);
    // State for resetting the input element via key
    let mut input_key = use_signal(|| 0);
    // State to track the currently selected context name
    let mut selected_context = use_signal(|| {
        // If file paths are provided, select the first one by default
        let paths = filenames.read();
        paths.first().cloned().unwrap_or_default()
    });

    // Effect to reload client when selected context changes
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

    // Effect to update filenames when context changes - removed since we're using global context now

    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        // Conditionally render the dialog
        if let Some((original_filename, file_content)) = dialog_state() {
            KubeconfigNameDialog {
                original_filename: original_filename.clone(),
                on_close: move |result: Option<String>| {
                    if let Some(name) = result {
                        tracing::info!("Kubeconfig context named: {}", name);
                        
                        // Save the file content to persistent storage and get the file path
                        match file_utils::save_kubeconfig_file(&name, &file_content) {
                            Ok(file_path) => {
                                // Store the file path with the user-provided name
                                if let Err(e) = kubeconfig_storage.store_file_path(name.clone(), file_path) {
                                    tracing::error!("Failed to store kubeconfig file path: {}", e);
                                    return;
                                }
                                
                                // Add to the global file paths
                                let mut current_files = filenames.write();
                                if !current_files.contains(&name) {
                                    current_files.push(name.clone());
                                    // Set the newly added context as selected
                                    selected_context.set(name);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to save kubeconfig file: {}", e);
                                return;
                            }
                        }
                    } else {
                        tracing::debug!("Kubeconfig name dialog cancelled.");
                    }
                    // Hide the dialog
                    dialog_state.set(None);
                    // Increment key to force input re-render (reset)
                    input_key += 1;
                }
            }
        }

        div { class: "layout-container",
            div {
                id: "sidebar",
                class: "k8s-sidebar",
                div {
                    class: "sidebar-logo",
                    img { src: "/assets/kubernetes-logo.svg", alt: "Kubernetes Logo" }
                    span { "Kubernetes Dashboard" }
                }
                nav {
                    class: "sidebar-links",
                    // Add Cluster Management Section
                    div { class: "nav-group",
                        span { class: "nav-group-title", "CLUSTER MANAGEMENT" }
                        // Updated cluster selector with better file path display
                        div { class: "cluster-selector",
                            label { r#for: "cluster-select", class: "sr-only", "Select Cluster" }
                            select {
                                id: "cluster-select",
                                name: "cluster",
                                class: "cluster-dropdown",
                                value: "{selected_context}",
                                onchange: move |evt| {
                                    let new_selection = evt.value();
                                    tracing::info!("User selected context: {}", new_selection);
                                    selected_context.set(new_selection.clone());
                                    
                                    // The effect will automatically trigger client reload
                                },
                                // Default/Placeholder option
                                option { 
                                    value: "", 
                                    disabled: true, 
                                    selected: selected_context.read().is_empty(),
                                    "Select kubeconfig file"
                                }
                                // Dynamically generate options from filenames signal
                                {
                                    filenames.read().iter().map(|filepath| {
                                        let display_name = filepath.split('/').last().unwrap_or(filepath);
                                        rsx! {
                                            option { 
                                                key: "{filepath}", 
                                                value: "{filepath}",
                                                title: "{filepath}", // Show full path on hover
                                                "{display_name}"
                                            }
                                        }
                                    })
                                }
                            }
                            // Add a small info text showing the count
                            {(!filenames.read().is_empty()).then(|| rsx! {
                                div { class: "file-count-info",
                                    "{filenames.read().len()} kubeconfig file(s) loaded"
                                }
                            })}
                        }

                        // Hidden file input for kubeconfig
                        input {
                            // Add key attribute
                            key: "{input_key()}",
                            r#type: "file",
                            accept: ".yaml,.kubeconfig,.yml",
                            multiple: false, // Allow only single file selection
                            id: "kubeconfig-upload",
                            hidden: true, // Hide the default input element
                            onchange: move |evt| {
                                // Clone necessary variables
                                let mut dialog_state = dialog_state.clone();

                                spawn(async move { // Use spawn for potential async file reading
                                    if let Some(file_engine) = evt.files() {
                                        let files = file_engine.files();
                                        if let Some(file_name) = files.first() {
                                            tracing::debug!("Selected file: {}", file_name);
                                            
                                            // Read the file content
                                            if let Some(content) = file_engine.read_file(file_name).await {
                                                // Store the file content temporarily with the original filename
                                                let content_str = String::from_utf8_lossy(&content).to_string();
                                                
                                                // Show the dialog with both filename and content
                                                dialog_state.set(Some((file_name.clone(), content_str)));
                                                
                                                tracing::info!("File read successfully: {} bytes", content.len());
                                            } else {
                                                tracing::error!("Failed to read file {}", file_name);
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        // Label styled as a button to trigger the file input
                        label {
                            r#for: "kubeconfig-upload",
                            class: "add-kubeconfig-button", // Use button styling
                            tabindex: 0, // Make label focusable
                            role: "button", // ARIA role
                            "Add Kubeconfig"
                        }
                    }
                    // Navigation links - only show when client is available
                    if has_client {
                        // Existing Nav Groups (Restored)
                        div { class: "nav-group",
                            span { class: "nav-group-title", "CLUSTER" }
                            Link {
                                to: Route::Home {},
                                class: "nav-overview",
                                img { src: "{OVERVIEW}", alt: "", class: "nav-icon" }
                                "Overview"
                            }
                            Link {
                                to: Route::Insights {},
                                class: "nav-insights",
                                img { src: "{INSIGHTS}", alt: "", class: "nav-icon" }
                                "Insights"
                            }
                            Link {
                                to: Route::Nodes {},
                                class: "nav-nodes",
                                img { src: "{NODES}", alt: "", class: "nav-icon" }
                                "Nodes"
                            }
                            Link {
                                to: Route::Namespaces {},
                                class: "nav-namespaces",
                                img { src: "{NAMESPACE}", alt: "", class: "nav-icon" }
                                "Namespaces"
                            }
                        }
                    } else {
                        // Show message when no client is available
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
                    if has_client {
                        div { class: "nav-group",
                            span { class: "nav-group-title", "WORKLOADS" }
                            Link {
                                to: Route::Pods {},
                                class: "nav-pods",
                                img { src: "{POD}", alt: "", class: "nav-icon" }
                                "Pods"
                            }
                            Link {
                                to: Route::Deployments {},
                                class: "nav-deployments",
                                img { src: "{DEPLOYMENT}", alt: "", class: "nav-icon" }
                                "Deployments"
                            }
                            Link {
                                to: Route::StatefulSets {},
                                class: "nav-statefulsets",
                                img { src: "{STATEFULSETS}", alt: "", class: "nav-icon" }
                                "StatefulSets"
                            }
                            Link {
                                to: Route::DaemonSets {},
                                class: "nav-daemonsets",
                                img { src: "{DAEMONSETS}", alt: "", class: "nav-icon" }
                                "DaemonSets"
                            }
                            Link {
                                to: Route::CronJobs {},
                                class: "nav-cronjobs",
                                img { src: "{CRONJOB}", alt: "", class: "nav-icon" }
                                "CronJobs"
                            }
                            Link {
                                to: Route::Jobs {},
                                class: "nav-jobs",
                                img { src: "{JOB}", alt: "", class: "nav-icon" }
                                "Jobs"
                            }
                        }
                        div { class: "nav-group",
                            span { class: "nav-group-title", "NETWORK" }
                            Link {
                                to: Route::Services {},
                                class: "nav-services",
                                img { src: "{SERVICE}", alt: "", class: "nav-icon" }
                                "Services"
                            }
                            Link {
                                to: Route::Ingresses {},
                                class: "nav-ingress",
                                img { src: "{INGRESS}", alt: "", class: "nav-icon" }
                                "Ingress"
                            }
                        }
                        div { class: "nav-group",
                            span { class: "nav-group-title", "STORAGE" }
                            Link {
                                to: Route::Pvcs {},
                                class: "nav-pvcs",
                                img { src: "{PVC}", alt: "", class: "nav-icon" }
                                "Persistent Volume Claims"
                            }
                            Link {
                                to: Route::ConfigMaps {},
                                class: "nav-configmaps",
                                img { src: "{CONFIGMAP}", alt: "", class: "nav-icon" }
                                "Config Maps"
                            }
                            Link {
                                to: Route::Secrets {},
                                class: "nav-secrets",
                                img { src: "{SECRET}", alt: "", class: "nav-icon" }
                                "Secrets"
                            }
                        }
                    }
                    // div { class: "nav-group",
                    //     span { class: "nav-group-title", "SETTINGS" }
                    //     Link {
                    //         to: Route::Blog { id: 12 },
                    //         class: "nav-settings",
                    //         img { src: "/assets/icons/gear.png", alt: "", class: "nav-icon" }
                    //         "Settings"
                    //     }
                    // }
                }
            }
            // Main Content Outlet - only render when client is available
            if has_client {
                div {
                    class: "main-content",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
