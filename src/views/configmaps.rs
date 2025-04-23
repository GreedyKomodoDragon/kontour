use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};

const CONFIGMAPS_CSS: Asset = asset!("/assets/styling/configmaps.css"); // Link to the new CSS

// --- Data Structures ---

#[derive(Clone, PartialEq)]
struct ConfigMapData {
    name: String,
    namespace: String,
    data: Vec<(String, String)>, // Key-value pairs
    binary_data_keys: Vec<String>, // Just the keys for binary data
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
}

// --- Sample Data ---

fn get_sample_configmaps() -> Vec<ConfigMapData> {
    vec![
        ConfigMapData {
            name: "nginx-config".into(),
            namespace: "default".into(),
            data: vec![
                ("nginx.conf".into(), "server {\n  listen 80;\n  server_name localhost;\n  location / {\n    root /usr/share/nginx/html;\n    index index.html index.htm;\n  }\n}".into()),
                ("proxy.conf".into(), "proxy_set_header Host $host;".into()),
            ],
            binary_data_keys: vec![],
            age: "3d".into(),
            labels: vec![("app".into(), "nginx".into())],
            annotations: vec![],
        },
        ConfigMapData {
            name: "app-settings".into(),
            namespace: "production".into(),
            data: vec![
                ("database.url".into(), "postgres://user:pass@db-service:5432/prod".into()),
                ("api.key".into(), "abc123xyz".into()),
                ("feature.flags".into(), "{\"new_dashboard\": true, \"beta_feature\": false}".into()),
            ],
            binary_data_keys: vec![],
            age: "2h".into(),
            labels: vec![("app".into(), "webapp".into())],
            annotations: vec![("last-updated-by".into(), "admin@example.com".into())],
        },
        ConfigMapData {
            name: "kube-root-ca.crt".into(),
            namespace: "default".into(), // Often present in many namespaces
            data: vec![
                ("ca.crt".into(), "-----BEGIN CERTIFICATE-----\nMIIC...<truncated>...END CERTIFICATE-----\n".into()),
            ],
            binary_data_keys: vec![],
            age: "45d".into(),
            labels: vec![("kubernetes.io/cluster-service".into(), "true".into())],
            annotations: vec![],
        },
        ConfigMapData {
            name: "binary-data-example".into(),
            namespace: "testing".into(),
            data: vec![
                ("config.yaml".into(), "key: value".into()),
            ],
            binary_data_keys: vec!["logo.png".into(), "init.gz".into()], // Keys of binary data
            age: "10m".into(),
            labels: vec![],
            annotations: vec![],
        },
    ]
}

// --- Component ---

#[component]
pub fn ConfigMaps() -> Element {
    let mut selected_namespace = use_signal(|| "all".to_string());
    let mut search_query = use_signal(String::new);
    let mut expanded_configmaps = use_signal(|| HashSet::<String>::new()); // Keyed by name+namespace
    let configmaps = use_signal(get_sample_configmaps);

    // --- Filtering Logic ---
    let filtered_configmaps = {
        let configmaps = configmaps.clone();
        let selected_namespace = selected_namespace.clone();
        let search_query = search_query.clone();

        use_signal(move || {
            let configmaps = configmaps.read();
            let query = search_query.read().to_lowercase();
            let current_ns = selected_namespace.read();

            configmaps.iter()
                .filter(|&cm| {
                    let ns_match = *current_ns == "all" || cm.namespace == *current_ns;
                    let search_match = query.is_empty()
                        || cm.name.to_lowercase().contains(&query)
                        || cm.namespace.to_lowercase().contains(&query)
                        || cm.data.iter().any(|(k, v)| k.to_lowercase().contains(&query) || v.to_lowercase().contains(&query)); // Search keys and values
                    ns_match && search_match
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    // --- Unique Namespaces for Filter ---
    let namespaces = use_memo(move || {
        let mut ns = configmaps.read().iter().map(|d| d.namespace.clone()).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        ns.sort();
        ns
    });

    rsx! {
        document::Link { rel: "stylesheet", href: CONFIGMAPS_CSS }
        div { class: "configmaps-container", // Use new CSS classes
            // --- Header ---
            div { class: "configmaps-header",
                div { class: "header-left",
                    h1 { "Config Maps" }
                    div { class: "header-controls",
                        // Search Input
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search ConfigMaps...",
                                value: "{search_query}",
                                oninput: move |evt| search_query.set(evt.value()),
                            }
                        }
                        // Namespace Select
                        select {
                            class: "namespace-select",
                            value: "{selected_namespace.read()}",
                            onchange: move |evt| selected_namespace.set(evt.value()),
                            option { value: "all", "All Namespaces" }
                            {namespaces.read().iter().map(|ns| rsx!{
                                option { key: "{ns}", value: "{ns}", "{ns}" }
                            })}
                        }
                        // Count
                        span { class: "configmap-count", "{filtered_configmaps.read().len()} ConfigMaps" }
                    }
                }
                // Header Actions
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Create ConfigMap" } // Placeholder
                    button { class: "btn btn-secondary", "Refresh" } // Placeholder
                }
            }

            // --- ConfigMaps Grid ---
            div { class: "configmaps-grid",
                {filtered_configmaps.read().iter().map(|cm| {
                    let cm_key = format!("{}-{}", cm.namespace, cm.name); // Unique key
                    let is_expanded = expanded_configmaps.read().contains(&cm_key);
                    let cm_key_clone = cm_key.clone();
                    let data_keys_count = cm.data.len();
                    let binary_keys_count = cm.binary_data_keys.len();

                    rsx! {
                        div {
                            key: "{cm_key}",
                            class: "configmap-card",
                            // --- Card Header ---
                            div {
                                class: "configmap-header-card",
                                div { class: "configmap-title",
                                    h3 { "{cm.name}" }
                                    // Maybe show data key count?
                                    span { class: "status-badge status-info", "{data_keys_count} keys" }
                                }
                                div { class: "configmap-info-short", // Show key info in header
                                     span { class: "info-item-short", title: "Namespace", "{cm.namespace}" }
                                     span { class: "info-item-short", title: "Age", "{cm.age}" }
                                }
                                div { class: "configmap-controls",
                                    // Expand/Collapse Button
                                    button {
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let mut set = expanded_configmaps.write();
                                            if set.contains(&cm_key_clone) {
                                                set.remove(&cm_key_clone);
                                            } else {
                                                set.insert(cm_key_clone.clone());
                                            }
                                        },
                                        title: if is_expanded { "Collapse" } else { "Expand" },
                                        if is_expanded { "🔼" } else { "🔽" }
                                    }
                                    // Placeholder Action Buttons
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                                }
                            }

                            // --- Expanded Details ---
                            {is_expanded.then(|| rsx! {
                                div { class: "configmap-details",
                                    // Basic Info Section (less relevant here, maybe just age/namespace if needed)
                                    // div { class: "info-section", ... }

                                    // Labels Section
                                    {(!cm.labels.is_empty()).then(|| rsx! {
                                        div { class: "labels-section",
                                            h4 { "Labels" }
                                            div { class: "labels-grid",
                                                {cm.labels.iter().map(|(key, value)| rsx! {
                                                    div { key: "lbl-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                                })}
                                            }
                                        }
                                    })}

                                     // Annotations Section
                                    {(!cm.annotations.is_empty()).then(|| rsx! {
                                        div { class: "labels-section", // Reuse label styling
                                            h4 { "Annotations" }
                                            div { class: "labels-grid",
                                                {cm.annotations.iter().map(|(key, value)| rsx! {
                                                    div { key: "anno-{key}", class: "label annotation",
                                                        span { class: "label-key", "{key}" }
                                                        span { class: "label-value", "{value}" }
                                                    }
                                                })}
                                            }
                                        }
                                    })}

                                    // Data Section
                                    {(!cm.data.is_empty()).then(|| rsx! {
                                        div { class: "data-section",
                                            h4 { "Data" }
                                            div { class: "data-grid",
                                                {cm.data.iter().map(|(key, value)| rsx! {
                                                    div { key: "data-{key}", class: "data-item",
                                                        div { class: "data-key", "{key}" }
                                                        pre { class: "data-value", "{value}" } // Use <pre> for formatting
                                                    }
                                                })}
                                            }
                                        }
                                    })}

                                     // Binary Data Section
                                    {(!cm.binary_data_keys.is_empty()).then(|| rsx! {
                                        div { class: "data-section", // Reuse data styling
                                            h4 { "Binary Data Keys" }
                                            div { class: "data-grid binary-keys", // Add class to differentiate if needed
                                                {cm.binary_data_keys.iter().map(|key| rsx! {
                                                    div { key: "bindata-{key}", class: "data-item binary-item",
                                                        div { class: "data-key", "{key}" }
                                                        div { class: "data-value binary-placeholder", i { "(binary data)" } }
                                                    }
                                                })}
                                            }
                                        }
                                    })}
                                }
                            })}
                        }
                    }
                })}
            }
        }
    }
}
