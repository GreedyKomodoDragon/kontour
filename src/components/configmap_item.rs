use dioxus::prelude::*;
use k8s_openapi::api::core::v1::ConfigMap;

#[derive(Clone)]
struct ConfigMapData {
    name: String,
    namespace: String,
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
    data: Vec<(String, String)>,
    binary_data_keys: Vec<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct ConfigMapItemProps {
    configmap: ConfigMap,
}

#[component]
pub fn ConfigMapItem(props: ConfigMapItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    let configmap_data = ConfigMapData {
        name: props.configmap.metadata.name.clone().unwrap_or_default(),
        namespace: props.configmap.metadata.namespace.clone().unwrap_or_default(),
        age: "1h".to_string(), // TODO: Calculate age
        labels: props.configmap.metadata.labels.as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        annotations: props.configmap.metadata.annotations.as_ref()
            .map(|annotations| annotations.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        data: props.configmap.data.as_ref()
            .map(|data| data.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        binary_data_keys: props.configmap.binary_data.as_ref()
            .map(|data| data.keys().cloned().collect())
            .unwrap_or_default(),
    };

    let data_keys_count = configmap_data.data.len() + configmap_data.binary_data_keys.len();

    rsx! {
        div {
            key: "{configmap_data.namespace}-{configmap_data.name}",
            class: "configmap-card",
            div {
                class: "configmap-header-card",
                div { class: "configmap-title",
                    h3 { "{configmap_data.name}" }
                    span { class: "status-badge status-info", "{data_keys_count} keys" }
                }
                div { class: "configmap-info-short",
                    span { class: "info-item-short", title: "Namespace", "{configmap_data.namespace}" }
                    span { class: "info-item-short", title: "Age", "{configmap_data.age}" }
                }
                div { class: "configmap-controls",
                    button {
                        class: "btn-icon expand-toggle",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            is_expanded.set(!is_expanded());
                        },
                        title: if is_expanded() { "Collapse" } else { "Expand" },
                        if is_expanded() { "🔼" } else { "🔽" }
                    }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                }
            }

            {is_expanded().then(|| rsx! {
                div { class: "configmap-details",
                    // Labels Section
                    {(!configmap_data.labels.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Labels" }
                            div { class: "labels-grid",
                                {configmap_data.labels.iter().map(|(key, value)| rsx! {
                                    div {
                                        key: "lbl-{configmap_data.namespace}-{configmap_data.name}-{key}",
                                        class: "label",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                })}
                            }
                        }
                    })}

                    // Annotations Section
                    {(!configmap_data.annotations.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Annotations" }
                            div { class: "labels-grid",
                                {configmap_data.annotations.iter().map(|(key, value)| rsx! {
                                    div {
                                        key: "anno-{configmap_data.namespace}-{configmap_data.name}-{key}",
                                        class: "label annotation",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                })}
                            }
                        }
                    })}

                    // Data Section
                    {(!configmap_data.data.is_empty()).then(|| rsx! {
                        div { class: "data-section",
                            h4 { "Data" }
                            div { class: "data-grid",
                                {configmap_data.data.iter().map(|(key, value)| rsx! {
                                    div {
                                        key: "data-{configmap_data.namespace}-{configmap_data.name}-{key}",
                                        class: "data-item",
                                        div { class: "data-key", "{key}" }
                                        pre { class: "data-value", "{value}" }
                                    }
                                })}
                            }
                        }
                    })}

                    // Binary Data Section
                    {(!configmap_data.binary_data_keys.is_empty()).then(|| rsx! {
                        div { class: "data-section",
                            h4 { "Binary Data Keys" }
                            div { class: "data-grid binary-keys",
                                {configmap_data.binary_data_keys.iter().map(|key| rsx! {
                                    div {
                                        key: "bindata-{configmap_data.namespace}-{configmap_data.name}-{key}",
                                        class: "data-item binary-item",
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
}
