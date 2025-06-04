use crate::Route;
// Import the new dialog component
use crate::components::kubeconfig_name_dialog::KubeconfigNameDialog;
use dioxus::{logger::tracing, prelude::*};

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
            // Main Content Outlet (Restored)
            div {
                class: "main-content",
                Outlet::<Route> {}
            }
        }
    }
}
