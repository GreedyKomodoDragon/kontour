use dioxus::prelude::*;
use k8s_openapi::api::core::v1::Secret;

#[derive(Clone)]
struct SecretData {
    name: String,
    namespace: String,
    secret_type: String,
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
    data_keys: Vec<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct SecretItemProps {
    secret: Secret,
}

#[component]
pub fn SecretItem(props: SecretItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);
    let mut revealed_keys = use_signal(|| std::collections::HashSet::new());

    let secret_data = SecretData {
        name: props.secret.metadata.name.clone().unwrap_or_default(),
        namespace: props.secret.metadata.namespace.clone().unwrap_or_default(),
        secret_type: props
            .secret
            .type_
            .clone()
            .unwrap_or_else(|| "Opaque".to_string()),
        age: "1h".to_string(), // TODO: Calculate age
        labels: props
            .secret
            .metadata
            .labels
            .as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        annotations: props
            .secret
            .metadata
            .annotations
            .as_ref()
            .map(|annotations| {
                annotations
                    .iter()
                    .filter(|(k, _)| *k != "kubectl.kubernetes.io/last-applied-configuration")
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default(),
        data_keys: props
            .secret
            .data
            .as_ref()
            .map(|data| data.keys().cloned().collect())
            .unwrap_or_default(),
    };

    let data_keys_count = secret_data.data_keys.len();
    let key_base = format!("{}-{}", secret_data.namespace, secret_data.name);
    let data_keys = secret_data.data_keys.clone();

    rsx! {
        div {
            key: "{key_base}",
            class: "secret-card",
            div {
                class: "secret-header-card",
                div { class: "secret-title",
                    h3 { "{secret_data.name}" }
                    span { class: "status-badge status-info", "{data_keys_count} keys" }
                }
                div { class: "secret-info-short",
                    span { class: "info-item-short", title: "Namespace", "{secret_data.namespace}" }
                    span { class: "info-item-short", title: "Type", "{secret_data.secret_type}" }
                    span { class: "info-item-short", title: "Age", "{secret_data.age}" }
                }
                div { class: "secret-controls",
                    button {
                        class: "btn-icon expand-toggle",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            is_expanded.set(!is_expanded());
                        },
                        title: if is_expanded() { "Collapse" } else { "Expand" },
                        if is_expanded() { "🔼" } else { "🔽" }
                    }
                }
            }

            {is_expanded().then(|| rsx! {
                div { class: "secret-details",
                    // Details Section
                    div { class: "info-section",
                        h4 { "Details" }
                        div { class: "info-grid",
                            div { class: "info-item",
                                span { class: "info-label", "Namespace" }
                                span { class: "info-value", "{secret_data.namespace}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Type" }
                                span { class: "info-value", "{secret_data.secret_type}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Age" }
                                span { class: "info-value", "{secret_data.age}" }
                            }
                        }
                    }

                    // Labels Section
                    {(!secret_data.labels.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Labels" }
                            div { class: "labels-grid",
                                {secret_data.labels.iter().map(|(key, value)| rsx! {
                                    div {
                                        key: "lbl-{key_base}-{key}",
                                        class: "label",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                })}
                            }
                        }
                    })}

                    // Annotations Section
                    {(!secret_data.annotations.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Annotations" }
                            div { class: "labels-grid",
                                {secret_data.annotations.iter().map(|(key, value)| rsx! {
                                    div {
                                        key: "anno-{key_base}-{key}",
                                        class: "label annotation",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                })}
                            }
                        }
                    })}

                    // Data Section
                    {
                        (!data_keys.is_empty()).then(|| rsx! {
                        div { class: "data-section",
                            h4 { "Data" }
                            div { class: "data-grid",
                                {data_keys.iter().map(|key| {
                                    let key = key.clone();
                                    let key_for_closure = key.clone();
                                    let is_revealed = revealed_keys.read().contains(&key);
                                    let decoded_value = if is_revealed {
                                        props.secret.data.as_ref()
                                            .and_then(|data| data.get(&key))
                                            .map(|value| String::from_utf8(value.0.clone())
                                                .unwrap_or_else(|_| "(invalid UTF-8)".to_string()))
                                            .unwrap_or_else(|| "(error)".to_string())
                                    } else {
                                        String::new()
                                    };

                                    rsx! {
                                        div {
                                            key: "data-{key_base}-{key}",
                                            class: "data-item",
                                            div { class: "data-key",
                                                span { "{key}" }
                                                button {
                                                    class: "btn-icon btn-small",
                                                    onclick: move |evt| {
                                                        evt.stop_propagation();
                                                        let mut keys = revealed_keys.write();
                                                        if keys.contains(&key_for_closure) {
                                                            keys.remove(&key_for_closure);
                                                        } else {
                                                            keys.insert(key_for_closure.clone());
                                                        }
                                                    },
                                                    title: if is_revealed { "Hide Value" } else { "Show Value" },
                                                    if is_revealed { "👁️‍🗨️" } else { "👁️" }
                                                }
                                            }
                                            div {
                                                class: "data-value",
                                                if is_revealed {
                                                    pre { "{decoded_value}" }
                                                } else {
                                                   div { class: "secret-placeholder", i { "(value hidden)" } }
                                                }
                                            }
                                        }
                                    }
                                })}
                            }
                        }
                    })}
                }
            })}
        }
    }
}
