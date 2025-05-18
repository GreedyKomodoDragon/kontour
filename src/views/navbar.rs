use crate::Route;
// Import the new dialog component
use crate::components::kubeconfig_name_dialog::KubeconfigNameDialog;
use dioxus::{logger::tracing, prelude::*};

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    // Signal to store a list of added context names
    let mut filenames = use_signal(|| Vec::<String>::new());
    // State to manage the dialog: Option<original_filename>
    let mut dialog_state = use_signal::<Option<String>>(|| None);
    // State for resetting the input element via key
    let mut input_key = use_signal(|| 0);
    // State to track the currently selected context name
    let mut selected_context = use_signal(|| String::new()); // Initialize empty or with a default

    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        // Conditionally render the dialog
        if let Some(original_filename) = dialog_state() {
            KubeconfigNameDialog {
                original_filename: original_filename.clone(),
                on_close: move |result: Option<String>| {
                    if let Some(name) = result {
                        tracing::info!("Kubeconfig context named: {}", name);
                        let mut current_files = filenames.write();
                        // Add the name if it doesn't already exist
                        if !current_files.contains(&name) {
                            current_files.push(name.clone());
                            // Optionally, set the newly added context as selected
                            selected_context.set(name);
                        }
                        // TODO: Process file content associated with original_filename and store using 'name'
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
                        // Updated cluster selector
                        div { class: "cluster-selector",
                            label { r#for: "cluster-select", class: "sr-only", "Select Cluster" }
                            select {
                                id: "cluster-select",
                                name: "cluster",
                                class: "cluster-dropdown", // Use appropriate class from navbar.css if needed
                                value: "{selected_context}", // Bind select value to state
                                onchange: move |evt| {
                                    let new_selection = evt.value();
                                    tracing::info!("Selected context: {}", new_selection);
                                    selected_context.set(new_selection);
                                    // TODO: Add logic to switch context based on selection
                                },
                                // Default/Placeholder option (optional)
                                option { value: "", disabled: true, hidden: !selected_context.read().is_empty(), "Select Context" }
                                // Dynamically generate options from filenames signal
                                {
                                    filenames.read().iter().map(|name| rsx! {
                                        option { key: "{name}", value: "{name}", "{name}" }
                                    })
                                }
                            }
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
                                            // Set state to show the dialog with the original filename
                                            dialog_state.set(Some(file_name.clone()));

                                            // NOTE: File content reading would happen here or after naming
                                            // let file_content = file_engine.read_file(file_name).await;
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
                    // Existing Nav Groups (Restored)
                    div { class: "nav-group",
                        span { class: "nav-group-title", "CLUSTER" }
                        Link {
                            to: Route::Home {},
                            class: "nav-overview",
                            "Overview"
                        }
                        Link {
                            to: Route::Nodes {},
                            class: "nav-nodes",
                            "Nodes"
                        }
                        Link {
                            to: Route::Namespaces {},
                            class: "nav-namespaces",
                            "Namespaces"
                        }
                    }
                    div { class: "nav-group",
                        span { class: "nav-group-title", "WORKLOADS" }
                        Link {
                            to: Route::Pods {},
                            class: "nav-pods",
                            "Pods"
                        }
                        Link {
                            to: Route::Deployments {},
                            class: "nav-deployments",
                            "Deployments"
                        }
                        Link {
                            to: Route::StatefulSets {},
                            class: "nav-statefulsets",
                            "StatefulSets"
                        }
                        Link {
                            to: Route::DaemonSets {},
                            class: "nav-daemonsets",
                            "DaemonSets"
                        }
                        Link {
                            to: Route::Jobs {},
                            class: "nav-jobs",
                            "Jobs"
                        }
                    }
                    div { class: "nav-group",
                        span { class: "nav-group-title", "NETWORK" }
                        Link {
                            to: Route::Services {},
                            class: "nav-services",
                            "Services"
                        }
                        Link {
                            to: Route::Ingresses {},
                            class: "nav-ingress",
                            "Ingress"
                        }
                    }
                    div { class: "nav-group",
                        span { class: "nav-group-title", "STORAGE" }
                        Link {
                            to: Route::Pvcs {}, // Update route
                            class: "nav-pvcs",
                            "Persistent Volume Claims"
                        }
                        Link {
                            to: Route::ConfigMaps {}, // Update route
                            class: "nav-configmaps",
                            "Config Maps"
                        }
                        Link {
                            to: Route::Secrets {}, // Update route
                            class: "nav-secrets",
                            "Secrets"
                        }
                    }
                    div { class: "nav-group",
                        span { class: "nav-group-title", "SETTINGS" }
                        Link {
                            to: Route::Blog { id: 12 },
                            class: "nav-settings",
                            "Settings"
                        }
                    }
                }
            }
            // Main Content Outlet (Restored)
            div {
                class: "main-content",
                Outlet::<Route> {}
            }
        }
    }
}
