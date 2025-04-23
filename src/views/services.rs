use dioxus::prelude::*;
use std::collections::HashSet;

const SERVICES_CSS: Asset = asset!("/assets/styling/services.css");

// --- Data Structures ---

#[derive(Clone, PartialEq)]
struct ServiceData {
    name: String,
    namespace: String,
    service_type: String, // e.g., ClusterIP, NodePort, LoadBalancer
    cluster_ip: String,
    external_ip: String, // Often <pending> or an IP/hostname
    ports: Vec<ServicePort>,
    age: String,
    selector: Vec<(String, String)>,
    labels: Vec<(String, String)>,
    // Add other relevant fields like session affinity, annotations
}

#[derive(Clone, PartialEq)]
struct ServicePort {
    name: Option<String>, // Changed to Option<String>
    protocol: String, // TCP, UDP, SCTP
    port: u32, // Port exposed by the service
    target_port: String, // Port on the pods (can be number or name)
    node_port: Option<u32>, // Only for NodePort/LoadBalancer
}


// --- Sample Data ---

fn get_sample_services() -> Vec<ServiceData> {
    vec![
        ServiceData {
            name: "kubernetes".into(),
            namespace: "default".into(),
            service_type: "ClusterIP".into(),
            cluster_ip: "10.96.0.1".into(),
            external_ip: "<none>".into(),
            ports: vec![
                ServicePort { name: Some("https".into()), protocol: "TCP".into(), port: 443, target_port: "6443".into(), node_port: None }, // Wrapped in Some()
            ],
            age: "45d".into(),
            selector: vec![], // Kubernetes service usually has no selector
            labels: vec![("component".into(), "apiserver".into()), ("provider".into(), "kubernetes".into())],
        },
        ServiceData {
            name: "nginx-service".into(),
            namespace: "default".into(),
            service_type: "LoadBalancer".into(),
            cluster_ip: "10.100.50.20".into(),
            external_ip: "pending".into(), // Or an actual IP if assigned
            ports: vec![
                ServicePort { name: Some("http".into()), protocol: "TCP".into(), port: 80, target_port: "80".into(), node_port: Some(31080) }, // Wrapped in Some()
            ],
            age: "3d".into(),
            selector: vec![("app".into(), "nginx".into())],
            labels: vec![("app".into(), "nginx".into())],
        },
        ServiceData {
            name: "prometheus-service".into(),
            namespace: "monitoring".into(),
            service_type: "NodePort".into(),
            cluster_ip: "10.105.10.5".into(),
            external_ip: "<none>".into(),
            ports: vec![
                 ServicePort { name: Some("web".into()), protocol: "TCP".into(), port: 9090, target_port: "9090".into(), node_port: Some(30090) }, // Wrapped in Some()
            ],
            age: "6d".into(),
            selector: vec![("app".into(), "prometheus".into())],
            labels: vec![("app".into(), "prometheus".into())],
        },
         ServiceData {
            name: "db-headless".into(),
            namespace: "data".into(),
            service_type: "ClusterIP".into(),
            cluster_ip: "<none>".into(), // Headless service
            external_ip: "<none>".into(),
            ports: vec![
                 ServicePort { name: Some("tcp-postgres".into()), protocol: "TCP".into(), port: 5432, target_port: "5432".into(), node_port: None }, // Wrapped in Some()
            ],
            age: "5h".into(),
            selector: vec![("app".into(), "postgres-db".into())],
            labels: vec![("app".into(), "postgres-db".into())],
        },
    ]
}


// --- Component ---

#[component]
pub fn Services() -> Element {
    let mut selected_namespace = use_signal(|| "all".to_string()); // Add mut
    let mut search_query = use_signal(String::new); // Add mut
    let mut selected_type = use_signal(|| "all".to_string()); // Add mut
    // Expansion state might be less common for services, but keep pattern for consistency
    let mut expanded_services = use_signal(|| HashSet::<String>::new());
    let services = use_signal(get_sample_services);

    // --- Filtering Logic ---
    let filtered_services = {
        let services = services.clone();
        let selected_namespace = selected_namespace.clone();
        let search_query = search_query.clone();
        let selected_type = selected_type.clone(); // Clone the type signal

        use_signal(move || {
            let services = services.read();
            let query = search_query.read().to_lowercase();
            let current_ns = selected_namespace.read(); // Read the signal value (String)
            let current_type = selected_type.read(); // Read the signal value (String)

            services.iter()
                .filter(|&svc| {
                    // Comparisons should work due to PartialEq<str> for String
                    let ns_match = *current_ns == "all" || svc.namespace == *current_ns;
                    let search_match = query.is_empty() || svc.name.to_lowercase().contains(&query) || svc.namespace.to_lowercase().contains(&query);
                    let type_match = *current_type == "all" || svc.service_type == *current_type; // Compare String with &str/String
                    ns_match && search_match && type_match // Combine filters
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    // --- Unique Namespaces for Filter ---
    let namespaces = use_memo(move || {
        let mut ns = services.read().iter().map(|d| d.namespace.clone()).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        ns.sort();
        ns
    });

    // --- Unique Service Types for Filter ---
    let service_types = use_memo(move || {
        let mut types = services.read().iter().map(|s| s.service_type.clone()).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        types.sort();
        types
    });


    rsx! {
        document::Link { rel: "stylesheet", href: SERVICES_CSS }
        div { class: "services-container",
            // --- Header ---
            div { class: "services-header",
                div { class: "header-left",
                    h1 { "Services" }
                    div { class: "header-controls",
                        // Search Input
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search services...",
                                value: "{search_query}", // Reads signal, already String
                                oninput: move |evt| search_query.set(evt.value()), // Use set directly
                            }
                        }
                        // Namespace Select
                        select {
                            class: "namespace-select",
                            value: "{selected_namespace.read()}", // Reads signal, already String
                            onchange: move |evt| selected_namespace.set(evt.value()), // Use set directly
                            option { value: "all", "All Namespaces" }
                            {namespaces.read().iter().map(|ns| rsx!{
                                option { key: "{ns}", value: "{ns}", "{ns}" }
                            })}
                        }
                        // Type Select - Added
                        select {
                            class: "type-select", // Add specific class if needed, reuse namespace-select style
                            value: "{selected_type.read()}", // Reads signal, already String
                            onchange: move |evt| selected_type.set(evt.value()), // Use set directly
                            option { value: "all", "All Types" }
                            {service_types.read().iter().map(|st| rsx!{
                                option { key: "{st}", value: "{st}", "{st}" }
                            })}
                        }
                        // Count
                        span { class: "service-count", "{filtered_services.read().len()} services" }
                    }
                }
                // Header Actions
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Create Service" } // Placeholder action
                    button { class: "btn btn-secondary", "Refresh" } // Placeholder action
                }
            }

            // --- Services Grid ---
            div { class: "services-grid",
                {filtered_services.read().iter().map(|svc| {
                    let is_expanded = expanded_services.read().contains(&svc.name);
                    let svc_name_clone = svc.name.clone();
                    // Service status is generally implicit (exists or not), maybe use type?
                    let status_class = match svc.service_type.as_str() {
                        "LoadBalancer" => "status-running", // Or a specific color
                        "NodePort" => "status-pending", // Or a specific color
                        _ => "status-unknown", // ClusterIP or others
                    };

                    rsx! {
                        div {
                            key: "{svc.name}",
                            class: "service-card",
                            // --- Card Header ---
                            div {
                                class: "service-header-card",
                                div { class: "service-title",
                                    h3 { "{svc.name}" }
                                    // Maybe show Type or ClusterIP in header?
                                    span { class: "status-badge {status_class}", "{svc.service_type}" }
                                }
                                div { class: "service-controls",
                                    // Expand/Collapse Button
                                    button {
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let mut set = expanded_services.write();
                                            if set.contains(&svc_name_clone) {
                                                set.remove(&svc_name_clone);
                                            } else {
                                                set.insert(svc_name_clone.clone());
                                            }
                                        },
                                        title: if is_expanded { "Collapse" } else { "Expand" },
                                        if is_expanded { "🔼" } else { "🔽" }
                                    }
                                    // Placeholder Action Buttons
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "View Endpoints", "🔗" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                                }
                            }

                            // --- Expanded Details ---
                            {is_expanded.then(|| rsx! {
                                div { class: "service-details",
                                    // Basic Info Row
                                    div { class: "service-info",
                                        div { class: "info-group",
                                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{svc.namespace}" } }
                                            div { class: "info-item", span { class: "info-label", "Type" } span { class: "info-value", "{svc.service_type}" } }
                                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{svc.age}" } }
                                        }
                                         div { class: "info-group",
                                            div { class: "info-item", span { class: "info-label", "Cluster IP" } span { class: "info-value", "{svc.cluster_ip}" } }
                                            div { class: "info-item", span { class: "info-label", "External IP" } span { class: "info-value", "{svc.external_ip}" } }
                                            // Add Session Affinity etc. if needed
                                        }
                                    }

                                    // Labels Section
                                    div { class: "labels-section",
                                        h4 { "Labels" }
                                        div { class: "labels-grid",
                                            {svc.labels.iter().map(|(key, value)| rsx! {
                                                div { key: "lbl-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                            })}
                                        }
                                    }

                                     // Selector Section
                                    div { class: "labels-section",
                                        h4 { "Selector" }
                                         if svc.selector.is_empty() {
                                             div { class: "labels-grid", span { class: "info-value", i { "None" } } }
                                        } else {
                                            div { class: "labels-grid",
                                                {svc.selector.iter().map(|(key, value)| rsx! {
                                                    div { key: "sel-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                                })}
                                            }
                                        }
                                    }

                                    // Ports Section
                                    div { class: "ports-section", // New section for ports
                                        h4 { "Ports" }
                                        div { class: "ports-grid", // Use a grid or list
                                            // Header Row (Optional)
                                            // div { class: "port-item port-header", ... }
                                            {svc.ports.iter().map(|port| rsx! {
                                                div {
                                                    key: "{port.port}-{port.protocol}", // Unique key
                                                    class: "port-item",
                                                    // Use as_deref and unwrap_or for efficient handling of Option<&String>
                                                    span { class: "port-detail port-name", "{port.name.as_deref().unwrap_or(\"-\")}" }
                                                    span { class: "port-detail port-number", "{port.port}" }
                                                    span { class: "port-detail port-protocol", "{port.protocol}" }
                                                    span { class: "port-detail port-target", "→ {port.target_port}" }
                                                    {port.node_port.map(|np| rsx!{ span { class: "port-detail port-nodeport", "(Node: {np})" } })}
                                                }
                                            })}
                                        }
                                    }
                                }
                            })}
                        }
                    }
                })}
            }
        }
    }
}
