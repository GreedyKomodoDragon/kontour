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
    actual_data: Option<Vec<(String, String)>>,
}

#[derive(Props, PartialEq, Clone)]
pub struct SecretItemProps {
    secret: Secret,
}

#[component]
pub fn SecretItem(props: SecretItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);
    let mut is_revealed = use_signal(|| false);

    let secret_data = SecretData {
        name: props.secret.metadata.name.clone().unwrap_or_default(),
        namespace: props.secret.metadata.namespace.clone().unwrap_or_default(),
        secret_type: props.secret.type_.clone().unwrap_or_else(|| "Opaque".to_string()),
        age: "1h".to_string(), // TODO: Calculate age
        labels: props.secret.metadata.labels.as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        annotations: props.secret.metadata.annotations.as_ref()
            .map(|annotations| annotations.iter()
                .filter(|(k, _)| *k != "kubectl.kubernetes.io/last-applied-configuration")
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect())
            .unwrap_or_default(),
        data_keys: props.secret.data.as_ref()
            .map(|data| data.keys().cloned().collect())
            .unwrap_or_default(),
        actual_data: if is_revealed() {
            props.secret.data.as_ref().map(|data| {
                data.iter()
                    .filter_map(|(k, v)| {
                        String::from_utf8(v.0.clone()).ok()
                            .map(|decoded| (k.clone(), decoded))
                    })
                    .collect()
            })
        } else {
            None
        },
    };

    let data_keys_count = secret_data.data_keys.len();
    let key_base = format!("{}-{}", secret_data.namespace, secret_data.name);

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
                    button {
                        class: "btn-icon",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            is_revealed.set(!is_revealed());
                        },
                        title: if is_revealed() { "Hide Data" } else { "Show Data" },
                        if is_revealed() { "👁️‍🗨️" } else { "👁️" }
                    }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
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
                    {(!secret_data.data_keys.is_empty()).then(|| rsx! {
                        div { class: "data-section",
                            h4 { "Data" }
                            div { class: "data-grid",
                                {
                                    if is_revealed() && secret_data.actual_data.is_some() {
                                        rsx! {
                                            {secret_data.actual_data.as_ref().unwrap().iter().map(|(key, value)| rsx! {
                                                div {
                                                    key: "data-{key_base}-{key}",
                                                    class: "data-item",
                                                    div { class: "data-key", "{key}" }
                                                    pre { class: "data-value", "{value}" }
                                                }
                                            })}
                                        }
                                    } else {
                                        rsx! {
                                            {secret_data.data_keys.iter().map(|key| rsx! {
                                                div {
                                                    key: "data-{key_base}-{key}",
                                                    class: "data-item",
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
}
