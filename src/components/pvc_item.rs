use dioxus::prelude::*;
use k8s_openapi::api::core::v1::PersistentVolumeClaim;

#[derive(Clone)]
struct PvcData {
    name: String,
    namespace: String,
    status: String,
    volume: Option<String>,
    capacity: Option<String>,
    access_modes: Vec<String>,
    storage_class: Option<String>,
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
}

#[derive(Props, PartialEq, Clone)]
pub struct PvcItemProps {
    pvc: PersistentVolumeClaim,
}

#[component]
pub fn PvcItem(props: PvcItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    let pvc_data = PvcData {
        name: props.pvc.metadata.name.clone().unwrap_or_default(),
        namespace: props.pvc.metadata.namespace.clone().unwrap_or_default(),
        status: props.pvc.status.as_ref()
            .and_then(|s| s.phase.clone())
            .unwrap_or_else(|| "Unknown".to_string()),
        volume: props.pvc.spec.as_ref()
            .and_then(|s| s.volume_name.clone()),
        capacity: props.pvc.status.as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("storage"))
            .map(|q| q.0.clone()),
        access_modes: props.pvc.spec.as_ref()
            .and_then(|s| s.access_modes.clone())
            .unwrap_or_default(),
        storage_class: props.pvc.spec.as_ref()
            .and_then(|s| s.storage_class_name.clone()),
        age: "1h".to_string(), // TODO: Calculate age
        labels: props.pvc.metadata.labels.as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        annotations: props.pvc.metadata.annotations.as_ref()
            .map(|annotations| annotations.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
    };

    let status_class = match pvc_data.status.as_str() {
        "Bound" => "status-bound",
        "Pending" => "status-pending",
        "Lost" => "status-lost",
        _ => "status-unknown",
    };

    let access_modes_str = pvc_data.access_modes.join(", ");

    rsx! {
        div {
            key: "{pvc_data.name}",
            class: "pvc-card",
            div {
                class: "pvc-header-card",
                div { class: "pvc-title",
                    h3 { "{pvc_data.name}" }
                    span { class: "status-badge {status_class}", "{pvc_data.status}" }
                }
                div { class: "pvc-info-short",
                    span { class: "info-item-short", title: "Namespace", "{pvc_data.namespace}" }
                    span { class: "info-item-short", title: "Capacity", "{pvc_data.capacity.as_deref().unwrap_or(\"-\")}" }
                    span { class: "info-item-short", title: "Access Modes", "{access_modes_str}" }
                    span { class: "info-item-short", title: "Storage Class", "{pvc_data.storage_class.as_deref().unwrap_or(\"<none>\")}" }
                }
                div { class: "pvc-controls",
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
                div { class: "pvc-details",
                    // Basic Info Section
                    div { class: "info-section",
                        h4 { "Details" }
                        div { class: "info-grid",
                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{pvc_data.namespace}" } }
                            div { class: "info-item", span { class: "info-label", "Status" } span { class: "info-value", "{pvc_data.status}" } }
                            div { class: "info-item", span { class: "info-label", "Bound Volume" } span { class: "info-value", "{pvc_data.volume.as_deref().unwrap_or(\"-\")}" } }
                            div { class: "info-item", span { class: "info-label", "Capacity" } span { class: "info-value", "{pvc_data.capacity.as_deref().unwrap_or(\"-\")}" } }
                            div { class: "info-item", span { class: "info-label", "Access Modes" } span { class: "info-value", "{access_modes_str}" } }
                            div { class: "info-item", span { class: "info-label", "Storage Class" } span { class: "info-value", "{pvc_data.storage_class.as_deref().unwrap_or(\"<none>\")}" } }
                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{pvc_data.age}" } }
                        }
                    }

                    // Labels Section
                    {(!pvc_data.labels.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Labels" }
                            div { class: "labels-grid",
                                {pvc_data.labels.iter().map(|(key, value)| {
                                    rsx!(
                                        div {
                                            key: "lbl-{key}",
                                            class: "label",
                                            span { class: "label-key", "{key}" }
                                            span { class: "label-value", "{value}" }
                                        }
                                    )
                                })}
                            }
                        }
                    })}

                    // Annotations Section
                    {(!pvc_data.annotations.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Annotations" }
                            div { class: "labels-grid",
                                {pvc_data.annotations.iter().map(|(key, value)| {
                                    rsx!(
                                        div {
                                            key: "anno-{key}",
                                            class: "label annotation",
                                            span { class: "label-key", "{key}" }
                                            span { class: "label-value", "{value}" }
                                        }
                                    )
                                })}
                            }
                        }
                    })}
                }
            })}
        }
    }
}