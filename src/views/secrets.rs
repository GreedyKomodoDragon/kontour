use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};

const SECRETS_CSS: Asset = asset!("/assets/styling/secrets.css"); // Link to the new CSS

// --- Data Structures ---

#[derive(Clone, PartialEq)]
struct SecretData {
    name: String,
    namespace: String,
    secret_type: String, // e.g., Opaque, kubernetes.io/service-account-token, kubernetes.io/tls
    data_keys: Vec<String>, // Only the keys, always available
    actual_data: Option<Vec<(String, String)>>, // Key-value pairs, loaded on demand (or present in sample)
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
}

// --- Sample Data ---

fn get_sample_secrets() -> Vec<SecretData> {
    vec![
        SecretData {
            name: "db-credentials".into(),
            namespace: "data".into(),
            secret_type: "Opaque".into(),
            data_keys: vec!["username".into(), "password".into()],
            // Add dummy actual data
            actual_data: Some(vec![
                ("username".into(), "admin".into()),
                ("password".into(), "s3cr3tP@ssw0rd".into()),
            ]),
            age: "5h".into(),
            labels: vec![("app".into(), "postgres-db".into())],
            annotations: vec![],
        },
        SecretData {
            name: "api-key-secret".into(),
            namespace: "production".into(),
            secret_type: "Opaque".into(),
            data_keys: vec!["api-key".into()],
            // Add dummy actual data
            actual_data: Some(vec![
                ("api-key".into(), "veryLongAndSecureApiKeyGeneratedBySystemXYZ".into()),
            ]),
            age: "2h".into(),
            labels: vec![("app".into(), "webapp".into())],
            annotations: vec![],
        },
        SecretData {
            name: "default-token-xyz12".into(), // Example service account token
            namespace: "default".into(),
            secret_type: "kubernetes.io/service-account-token".into(),
            data_keys: vec!["ca.crt".into(), "namespace".into(), "token".into()],
            // Add dummy actual data (token is usually long)
            actual_data: Some(vec![
                ("ca.crt".into(), "-----BEGIN CERTIFICATE-----\nMIIC...<truncated>...END CERTIFICATE-----\n".into()),
                ("namespace".into(), "default".into()),
                ("token".into(), "eyJhbGciOiJSUzI1NiIsImtpZCI6I...<very long token>...xyz".into()),
            ]),
            age: "45d".into(),
            labels: vec![],
            annotations: vec![("kubernetes.io/service-account.name".into(), "default".into())],
        },
        SecretData {
            name: "example-tls-secret".into(),
            namespace: "production".into(),
            secret_type: "kubernetes.io/tls".into(),
            data_keys: vec!["tls.crt".into(), "tls.key".into()],
            // Add dummy actual data
            actual_data: Some(vec![
                 ("tls.crt".into(), "-----BEGIN CERTIFICATE-----\nMIID...<truncated cert>...END CERTIFICATE-----\n".into()),
                 ("tls.key".into(), "-----BEGIN PRIVATE KEY-----\nMIIE...<truncated key>...END PRIVATE KEY-----\n".into()),
            ]),
            age: "2h".into(),
            labels: vec![],
            annotations: vec![],
        },
    ]
}

// --- Component ---

#[component]
pub fn Secrets() -> Element {
    let mut selected_namespace = use_signal(|| "all".to_string());
    let mut search_query = use_signal(String::new);
    let mut expanded_secrets = use_signal(|| HashSet::<String>::new()); // Keyed by name+namespace
    let mut revealed_secrets = use_signal(|| HashSet::<String>::new()); // Track revealed secrets
    let secrets = use_signal(get_sample_secrets);

    // --- Filtering Logic ---
    let filtered_secrets = {
        let secrets = secrets.clone();
        let selected_namespace = selected_namespace.clone();
        let search_query = search_query.clone();

        use_signal(move || {
            let secrets = secrets.read();
            let query = search_query.read().to_lowercase();
            let current_ns = selected_namespace.read();

            secrets.iter()
                .filter(|&secret| {
                    let ns_match = *current_ns == "all" || secret.namespace == *current_ns;
                    let search_match = query.is_empty()
                        || secret.name.to_lowercase().contains(&query)
                        || secret.namespace.to_lowercase().contains(&query)
                        || secret.secret_type.to_lowercase().contains(&query)
                        || secret.data_keys.iter().any(|k| k.to_lowercase().contains(&query)); // Search keys
                    ns_match && search_match
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    // --- Unique Namespaces for Filter ---
    let namespaces = use_memo(move || {
        let mut ns = secrets.read().iter().map(|d| d.namespace.clone()).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        ns.sort();
        ns
    });

    rsx! {
        document::Link { rel: "stylesheet", href: SECRETS_CSS }
        div { class: "secrets-container", // Use new CSS classes
            // --- Header ---
            div { class: "secrets-header",
                div { class: "header-left",
                    h1 { "Secrets" }
                    div { class: "header-controls",
                        // Search Input
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search Secrets...",
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
                        span { class: "secret-count", "{filtered_secrets.read().len()} Secrets" }
                    }
                }
                // Header Actions
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Create Secret" } // Placeholder
                    button { class: "btn btn-secondary", "Refresh" } // Placeholder
                }
            }

            // --- Secrets Grid ---
            div { class: "secrets-grid",
                {filtered_secrets.read().iter().map(|secret| {
                    let secret_key = format!("{}-{}", secret.namespace, secret.name); // Unique key
                    let is_expanded = expanded_secrets.read().contains(&secret_key);
                    let is_revealed = revealed_secrets.read().contains(&secret_key); // Check if revealed
                    let secret_key_clone = secret_key.clone();
                    let secret_key_clone_reveal = secret_key.clone(); // Clone for reveal button
                    let data_keys_count = secret.data_keys.len();

                    rsx! {
                        div {
                            key: "{secret_key}",
                            class: "secret-card",
                            // --- Card Header ---
                            div {
                                class: "secret-header-card",
                                div { class: "secret-title",
                                    h3 { "{secret.name}" }
                                    span { class: "status-badge status-info", "{data_keys_count} keys" }
                                }
                                div { class: "secret-info-short", // Show key info in header
                                     span { class: "info-item-short", title: "Namespace", "{secret.namespace}" }
                                     span { class: "info-item-short", title: "Type", "{secret.secret_type}" }
                                     span { class: "info-item-short", title: "Age", "{secret.age}" }
                                }
                                div { class: "secret-controls",
                                    // Expand/Collapse Button
                                    button {
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let mut set = expanded_secrets.write();
                                            if set.contains(&secret_key_clone) {
                                                set.remove(&secret_key_clone);
                                            } else {
                                                set.insert(secret_key_clone.clone());
                                            }
                                        },
                                        title: if is_expanded { "Collapse" } else { "Expand" },
                                        if is_expanded { "🔼" } else { "🔽" }
                                    }

                                    // Show/Hide Data Button
                                    button {
                                        class: "btn-icon",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let mut revealed = revealed_secrets.write();
                                            if revealed.contains(&secret_key_clone_reveal) {
                                                revealed.remove(&secret_key_clone_reveal);
                                            } else {
                                                // In real app: trigger fetch if needed before inserting
                                                revealed.insert(secret_key_clone_reveal.clone());
                                            }
                                        },
                                        title: if is_revealed { "Hide Data" } else { "Show Data" },
                                        if is_revealed { "👁️‍🗨️" } else { "👁️" } // Different icons for revealed/hidden
                                    }
                                    // Placeholder Action Buttons
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                                }
                            }

                            // --- Expanded Details ---
                            {is_expanded.then(|| rsx! {
                                div { class: "secret-details",
                                    // Basic Info Section
                                    div { class: "info-section",
                                        h4 { "Details" }
                                        div { class: "info-grid",
                                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{secret.namespace}" } }
                                            div { class: "info-item", span { class: "info-label", "Type" } span { class: "info-value", "{secret.secret_type}" } }
                                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{secret.age}" } }
                                        }
                                    }

                                    // Labels Section
                                    {(!secret.labels.is_empty()).then(|| rsx! {
                                        div { class: "labels-section",
                                            h4 { "Labels" }
                                            div { class: "labels-grid",
                                                {secret.labels.iter().map(|(key, value)| rsx! {
                                                    div { key: "lbl-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                                })}
                                            }
                                        }
                                    })}

                                     // Annotations Section
                                    {(!secret.annotations.is_empty()).then(|| rsx! {
                                        div { class: "labels-section", // Reuse label styling
                                            h4 { "Annotations" }
                                            div { class: "labels-grid",
                                                {secret.annotations.iter().map(|(key, value)| rsx! {
                                                    div { key: "anno-{key}", class: "label annotation",
                                                        span { class: "label-key", "{key}" }
                                                        span { class: "label-value", "{value}" }
                                                    }
                                                })}
                                            }
                                        }
                                    })}

                                    // Data Section (Conditional Rendering)
                                    {(!secret.data_keys.is_empty()).then(|| rsx! {
                                        div { class: "data-section",
                                            h4 { "Data" }
                                            div { class: "data-grid",
                                                {
                                                    // If revealed and actual_data exists, show values
                                                    if is_revealed && secret.actual_data.is_some() {
                                                        rsx! { // Wrap the iterator in rsx!
                                                            {secret.actual_data.as_ref().unwrap().iter().map(|(key, value)| rsx! {
                                                                div { key: "data-{key}", class: "data-item",
                                                                    div { class: "data-key", "{key}" }
                                                                    pre { class: "data-value", "{value}" } // Use <pre> for revealed data
                                                                }
                                                            })}
                                                        }
                                                    } else {
                                                        // Otherwise, show keys with placeholder
                                                        rsx! { // Wrap the iterator in rsx!
                                                            {secret.data_keys.iter().map(|key| rsx! {
                                                                div { key: "data-{key}", class: "data-item",
                                                                    div { class: "data-key", "{key}" }
                                                                    div { class: "data-value secret-placeholder", i { "(value hidden)" } }
                                                                }
                                                            })}
                                                        }
                                                    }
                                                }
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
