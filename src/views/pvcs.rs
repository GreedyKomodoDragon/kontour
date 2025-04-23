use dioxus::prelude::*;
use std::collections::HashSet;

const PVCS_CSS: Asset = asset!("/assets/styling/pvcs.css"); // Link to the new CSS

// --- Data Structures ---

#[derive(Clone, PartialEq)]
struct PvcData {
    name: String,
    namespace: String,
    status: String, // e.g., Bound, Pending, Lost
    volume: Option<String>, // Name of the bound PersistentVolume
    capacity: Option<String>, // e.g., "10Gi", "1Ti" (as reported by PV)
    access_modes: Vec<String>, // e.g., RWO, ROX, RWX
    storage_class: Option<String>,
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
    // Add volumeMode if needed (Filesystem, Block)
}

// --- Sample Data ---

fn get_sample_pvcs() -> Vec<PvcData> {
    vec![
        PvcData {
            name: "db-data-pvc".into(),
            namespace: "data".into(),
            status: "Bound".into(),
            volume: Some("pv-001".into()),
            capacity: Some("5Gi".into()),
            access_modes: vec!["RWO".into()], // ReadWriteOnce
            storage_class: Some("standard-ssd".into()),
            age: "5h".into(),
            labels: vec![("app".into(), "postgres-db".into())],
            annotations: vec![],
        },
        PvcData {
            name: "webapp-cache-pvc".into(),
            namespace: "production".into(),
            status: "Bound".into(),
            volume: Some("pv-002".into()),
            capacity: Some("1Gi".into()),
            access_modes: vec!["RWX".into()], // ReadWriteMany
            storage_class: Some("nfs-client".into()),
            age: "2h".into(),
            labels: vec![("app".into(), "webapp".into()), ("tier".into(), "cache".into())],
            annotations: vec![],
        },
        PvcData {
            name: "pending-pvc".into(),
            namespace: "default".into(),
            status: "Pending".into(),
            volume: None,
            capacity: None, // No capacity until bound
            access_modes: vec!["RWO".into()],
            storage_class: Some("fast-nvme".into()), // Requested class
            age: "1m".into(),
            labels: vec![],
            annotations: vec![],
        },
         PvcData {
            name: "shared-logs-pvc".into(),
            namespace: "monitoring".into(),
            status: "Bound".into(),
            volume: Some("pv-logs-001".into()),
            capacity: Some("100Gi".into()),
            access_modes: vec!["ROX".into(), "RWX".into()], // ReadOnlyMany, ReadWriteMany
            storage_class: Some("ceph-fs".into()),
            age: "6d".into(),
            labels: vec![("purpose".into(), "shared-logs".into())],
            annotations: vec![],
        },
    ]
}

// --- Component ---

#[component]
pub fn Pvcs() -> Element {
    let mut selected_namespace = use_signal(|| "all".to_string());
    let mut search_query = use_signal(String::new);
    let mut expanded_pvcs = use_signal(|| HashSet::<String>::new()); // Keyed by name+namespace
    let pvcs = use_signal(get_sample_pvcs);

    // --- Filtering Logic ---
    let filtered_pvcs = {
        let pvcs = pvcs.clone();
        let selected_namespace = selected_namespace.clone();
        let search_query = search_query.clone();

        use_signal(move || {
            let pvcs = pvcs.read();
            let query = search_query.read().to_lowercase();
            let current_ns = selected_namespace.read();

            pvcs.iter()
                .filter(|&pvc| {
                    let ns_match = *current_ns == "all" || pvc.namespace == *current_ns;
                    let search_match = query.is_empty()
                        || pvc.name.to_lowercase().contains(&query)
                        || pvc.namespace.to_lowercase().contains(&query)
                        || pvc.storage_class.as_deref().unwrap_or("").to_lowercase().contains(&query)
                        || pvc.volume.as_deref().unwrap_or("").to_lowercase().contains(&query);
                    ns_match && search_match
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    // --- Unique Namespaces for Filter ---
    let namespaces = use_memo(move || {
        let mut ns = pvcs.read().iter().map(|d| d.namespace.clone()).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        ns.sort();
        ns
    });

    rsx! {
        document::Link { rel: "stylesheet", href: PVCS_CSS }
        div { class: "pvcs-container", // Use new CSS classes
            // --- Header ---
            div { class: "pvcs-header",
                div { class: "header-left",
                    h1 { "Persistent Volume Claims" }
                    div { class: "header-controls",
                        // Search Input
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search PVCs...",
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
                        span { class: "pvc-count", "{filtered_pvcs.read().len()} PVCs" }
                    }
                }
                // Header Actions
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Create PVC" } // Placeholder
                    button { class: "btn btn-secondary", "Refresh" } // Placeholder
                }
            }

            // --- PVCs Grid ---
            div { class: "pvcs-grid",
                {filtered_pvcs.read().iter().map(|pvc| {
                    let pvc_key = format!("{}-{}", pvc.namespace, pvc.name); // Unique key
                    let is_expanded = expanded_pvcs.read().contains(&pvc_key);
                    let pvc_key_clone = pvc_key.clone();

                    // Determine status class
                    let status_class = match pvc.status.as_str() {
                        "Bound" => "status-bound", // Use specific or re-use running
                        "Pending" => "status-pending",
                        "Lost" => "status-lost", // Use specific or re-use failed
                        _ => "status-unknown",
                    };
                    let access_modes_str = pvc.access_modes.join(", ");

                    rsx! {
                        div {
                            key: "{pvc_key}",
                            class: "pvc-card",
                            // --- Card Header ---
                            div {
                                class: "pvc-header-card",
                                div { class: "pvc-title",
                                    h3 { "{pvc.name}" }
                                    span { class: "status-badge {status_class}", "{pvc.status}" }
                                }
                                div { class: "pvc-info-short", // Show key info in header
                                     span { class: "info-item-short", title: "Namespace", "{pvc.namespace}" }
                                     span { class: "info-item-short", title: "Capacity", "{pvc.capacity.as_deref().unwrap_or(\"-\")}" }
                                     span { class: "info-item-short", title: "Access Modes", "{access_modes_str}" }
                                     span { class: "info-item-short", title: "Storage Class", "{pvc.storage_class.as_deref().unwrap_or(\"<none>\")}" }
                                }
                                div { class: "pvc-controls",
                                    // Expand/Collapse Button
                                    button {
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let mut set = expanded_pvcs.write();
                                            if set.contains(&pvc_key_clone) {
                                                set.remove(&pvc_key_clone);
                                            } else {
                                                set.insert(pvc_key_clone.clone());
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
                                div { class: "pvc-details",
                                    // Basic Info Section
                                    div { class: "info-section",
                                        h4 { "Details" }
                                        div { class: "info-grid",
                                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{pvc.namespace}" } }
                                            div { class: "info-item", span { class: "info-label", "Status" } span { class: "info-value", "{pvc.status}" } }
                                            div { class: "info-item", span { class: "info-label", "Bound Volume" } span { class: "info-value", "{pvc.volume.as_deref().unwrap_or(\"-\")}" } }
                                            div { class: "info-item", span { class: "info-label", "Capacity" } span { class: "info-value", "{pvc.capacity.as_deref().unwrap_or(\"-\")}" } }
                                            div { class: "info-item", span { class: "info-label", "Access Modes" } span { class: "info-value", "{access_modes_str}" } }
                                            div { class: "info-item", span { class: "info-label", "Storage Class" } span { class: "info-value", "{pvc.storage_class.as_deref().unwrap_or(\"<none>\")}" } }
                                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{pvc.age}" } }
                                            // Add Volume Mode if needed
                                        }
                                    }

                                    // Labels Section
                                    {(!pvc.labels.is_empty()).then(|| rsx! {
                                        div { class: "labels-section",
                                            h4 { "Labels" }
                                            div { class: "labels-grid",
                                                {pvc.labels.iter().map(|(key, value)| rsx! {
                                                    div { key: "lbl-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                                })}
                                            }
                                        }
                                    })}

                                     // Annotations Section
                                    {(!pvc.annotations.is_empty()).then(|| rsx! {
                                        div { class: "labels-section", // Reuse label styling
                                            h4 { "Annotations" }
                                            div { class: "labels-grid",
                                                {pvc.annotations.iter().map(|(key, value)| rsx! {
                                                    div { key: "anno-{key}", class: "label annotation",
                                                        span { class: "label-key", "{key}" }
                                                        span { class: "label-value", "{value}" }
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
