use dioxus::prelude::*;
use k8s_openapi::{
    api::apps::v1::{StatefulSet, StatefulSetSpec},
    api::core::v1::{
        Container, PersistentVolumeClaim, PersistentVolumeClaimSpec, PodSpec, PodTemplateSpec,
    },
    apimachinery::pkg::api::resource::Quantity,
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::{api::PostParams, Api, Client};
use std::collections::BTreeMap;

const CREATE_FORMS_CSS: Asset = asset!("/assets/styling/create_forms.css");

#[derive(Default, Clone)]
struct ContainerPort {
    container_port: String,
    protocol: String,
}

#[derive(Default, Clone)]
struct EnvVar {
    name: String,
    value: String,
}

#[derive(Default, Clone)]
struct KeyValuePair {
    key: String,
    value: String,
}

#[derive(Default, Clone)]
struct VolumeMount {
    name: String,
    mount_path: String,
}

#[derive(Default, Clone)]
struct VolumeClaimTemplate {
    name: String,
    storage_size: String,
    storage_class: String,
}

#[component]
pub fn CreateStatefulSet() -> Element {
    let client_signal = use_context::<Signal<Option<Client>>>();
    let navigate = use_navigator();

    let mut name = use_signal(String::new);
    let mut namespace = use_signal(|| "default".to_string());
    let mut replicas = use_signal(|| "1".to_string());
    let mut image = use_signal(String::new);
    let mut service_name = use_signal(String::new);

    // Resource requests/limits
    let mut cpu_request = use_signal(|| "100m".to_string());
    let mut memory_request = use_signal(|| "128Mi".to_string());
    let mut cpu_limit = use_signal(|| "200m".to_string());
    let mut memory_limit = use_signal(|| "256Mi".to_string());

    // Container ports
    let mut ports = use_signal(|| vec![ContainerPort::default()]);

    // Environment variables
    let mut env_vars = use_signal(|| vec![EnvVar::default()]);

    // Labels and selectors
    let mut labels = use_signal(|| vec![KeyValuePair::default()]);
    let mut selectors = use_signal(|| vec![KeyValuePair::default()]);
    let mut annotations = use_signal(|| vec![KeyValuePair::default()]);

    // Volume claim templates
    let mut volume_claims = use_signal(|| {
        vec![VolumeClaimTemplate {
            name: "data".to_string(),
            storage_size: "1Gi".to_string(),
            storage_class: "standard".to_string(),
            ..Default::default()
        }]
    });

    // Volume mounts
    let mut volume_mounts = use_signal(|| {
        vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/data".to_string(),
            ..Default::default()
        }]
    });

    let mut sections_state = use_signal(|| {
        vec![
            ("basic", false),
            ("labels", false),
            ("selectors", false),
            ("annotations", false),
            ("resources", false),
            ("ports", false),
            ("env", false),
            ("storage", false),
        ]
        .into_iter()
        .collect::<std::collections::HashMap<&'static str, bool>>()
    });

    let mut toggle_section = move |section: &'static str| {
        let current = sections_state.read().get(section).copied().unwrap_or(false);
        sections_state.write().insert(section, !current);
    };

    let section_class = move |section: &'static str| {
        let is_open = sections_state.read().get(section).copied().unwrap_or(false);
        if is_open {
            "section section-open"
        } else {
            "section"
        }
    };

    let mut error = use_signal(|| None::<String>);

    let submit = move |_| {
        let name = name().clone();
        let statefulset_name = name.clone();
        let namespace = namespace();
        let replicas_str = replicas();
        let image = image();
        let service_name = service_name();
        
        if let Some(client) = &*client_signal.read() {
            let client = client.clone();

        // Basic validation
        if name.is_empty() {
            error.set(Some("StatefulSet name is required".to_string()));
            return;
        }
        if image.is_empty() {
            error.set(Some("Container image is required".to_string()));
            return;
        }
        if service_name.is_empty() {
            error.set(Some("Service name is required".to_string()));
            return;
        }

        let replicas = match replicas_str.parse::<i32>() {
            Ok(r) if r >= 0 => r,
            _ => {
                error.set(Some("Replicas must be a non-negative number".to_string()));
                return;
            }
        };

        error.set(None);

        spawn(async move {
            use k8s_openapi::api::core::v1::ResourceRequirements;

            // Create resource requirements
            let mut requests = BTreeMap::new();
            let mut limits = BTreeMap::new();

            if !cpu_request().is_empty() {
                requests.insert("cpu".to_string(), Quantity(cpu_request()));
            }
            if !memory_request().is_empty() {
                requests.insert("memory".to_string(), Quantity(memory_request()));
            }
            if !cpu_limit().is_empty() {
                limits.insert("cpu".to_string(), Quantity(cpu_limit()));
            }
            if !memory_limit().is_empty() {
                limits.insert("memory".to_string(), Quantity(memory_limit()));
            }

            let resources = if !requests.is_empty() || !limits.is_empty() {
                Some(ResourceRequirements {
                    requests: if requests.is_empty() {
                        None
                    } else {
                        Some(requests)
                    },
                    limits: if limits.is_empty() {
                        None
                    } else {
                        Some(limits)
                    },
                    claims: None,
                })
            } else {
                None
            };

            // Convert container ports
            let ports = if ports().is_empty() {
                None
            } else {
                Some(
                    ports()
                        .into_iter()
                        .filter_map(|p| {
                            if p.container_port.is_empty() {
                                return None;
                            }
                            Some(k8s_openapi::api::core::v1::ContainerPort {
                                container_port: p.container_port.parse().ok()?,
                                protocol: Some(p.protocol),
                                ..Default::default()
                            })
                        })
                        .collect(),
                )
            };

            // Convert environment variables
            let env = if env_vars().is_empty() {
                None
            } else {
                Some(
                    env_vars()
                        .into_iter()
                        .filter_map(|e| {
                            if e.name.is_empty() {
                                return None;
                            }
                            Some(k8s_openapi::api::core::v1::EnvVar {
                                name: e.name,
                                value: Some(e.value),
                                value_from: None,
                            })
                        })
                        .collect(),
                )
            };

            // Convert volume mounts
            let volume_mounts = volume_mounts()
                .into_iter()
                .filter_map(|vm| {
                    if vm.name.is_empty() || vm.mount_path.is_empty() {
                        return None;
                    }
                    Some(k8s_openapi::api::core::v1::VolumeMount {
                        name: vm.name,
                        mount_path: vm.mount_path,
                        ..Default::default()
                    })
                })
                .collect::<Vec<_>>();

            // Convert volume claim templates
            let volume_claim_templates = volume_claims()
                .into_iter()
                .filter_map(|vc| {
                    if vc.name.is_empty() || vc.storage_size.is_empty() {
                        return None;
                    }
                    Some(PersistentVolumeClaim {
                        metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                            name: Some(vc.name),
                            ..Default::default()
                        },
                        spec: Some(PersistentVolumeClaimSpec {
                            access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                            resources: Some(
                                k8s_openapi::api::core::v1::VolumeResourceRequirements {
                                    requests: Some({
                                        let mut m = BTreeMap::new();
                                        m.insert("storage".to_string(), Quantity(vc.storage_size));
                                        m
                                    }),
                                    limits: None,
                                },
                            ),
                            storage_class_name: if vc.storage_class.is_empty() {
                                None
                            } else {
                                Some(vc.storage_class)
                            },
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                })
                .collect::<Vec<_>>();

            // Convert labels and selectors
            let mut label_map = BTreeMap::new();
            for label in labels() {
                if !label.key.is_empty() && !label.value.is_empty() {
                    label_map.insert(label.key, label.value);
                }
            }

            let mut selector_map = BTreeMap::new();
            for selector in selectors() {
                if !selector.key.is_empty() && !selector.value.is_empty() {
                    selector_map.insert(selector.key, selector.value);
                }
            }

            let mut annotation_map = BTreeMap::new();
            for annotation in annotations() {
                if !annotation.key.is_empty() && !annotation.value.is_empty() {
                    annotation_map.insert(annotation.key, annotation.value);
                }
            }

            let statefulset = StatefulSet {
                metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                    name: Some(name),
                    namespace: Some(namespace.clone()),
                    labels: if label_map.is_empty() {
                        None
                    } else {
                        Some(label_map.clone())
                    },
                    annotations: if annotation_map.is_empty() {
                        None
                    } else {
                        Some(annotation_map)
                    },
                    ..Default::default()
                },
                spec: Some(StatefulSetSpec {
                    replicas: Some(replicas),
                    service_name,
                    selector: LabelSelector {
                        match_labels: if selector_map.is_empty() {
                            None
                        } else {
                            Some(selector_map)
                        },
                        match_expressions: None,
                    },
                    template: PodTemplateSpec {
                        metadata: Some(
                            k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                                labels: Some(label_map),
                                ..Default::default()
                            },
                        ),
                        spec: Some(PodSpec {
                            containers: vec![Container {
                                name: statefulset_name,
                                image: Some(image),
                                ports,
                                env,
                                resources,
                                volume_mounts: if volume_mounts.is_empty() {
                                    None
                                } else {
                                    Some(volume_mounts)
                                },
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                    },
                    volume_claim_templates: Some(volume_claim_templates),
                    ..Default::default()
                }),
                status: None,
            };

            let statefulsets: Api<StatefulSet> = Api::namespaced(client, &namespace);
            match statefulsets
                .create(&PostParams::default(), &statefulset)
                .await
            {
                Ok(_) => {
                    navigate.push("/statefulsets");
                }
                Err(e) => {
                    error.set(Some(format!("Failed to create statefulset: {}", e)));
                }
            }
        });
        } else {
            error.set(Some("Kubernetes client not available".to_string()));
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CREATE_FORMS_CSS }
        div { class: "create-statefulset-container",
            h1 { class: "create-statefulset-title", "Create StatefulSet" }

            // Basic Info Section
            div { class: section_class("basic"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("basic"),
                    h2 { class: "section-title", "Basic Information" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "form-grid",
                        div { class: "form-group",
                            label { class: "form-label", "StatefulSet Name" }
                            input {
                                class: "form-input",
                                placeholder: "Enter statefulset name",
                                value: "{name}",
                                oninput: move |evt| name.set(evt.value().clone())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Namespace" }
                            input {
                                class: "form-input",
                                value: "{namespace}",
                                oninput: move |evt| namespace.set(evt.value().clone())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Service Name" }
                            input {
                                class: "form-input",
                                placeholder: "Enter headless service name",
                                value: "{service_name}",
                                oninput: move |evt| service_name.set(evt.value().clone())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Replicas" }
                            input {
                                class: "form-input",
                                r#type: "number",
                                min: "0",
                                value: "{replicas}",
                                oninput: move |evt| replicas.set(evt.value().clone())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Container Image" }
                            input {
                                class: "form-input",
                                placeholder: "e.g., mysql:8.0",
                                value: "{image}",
                                oninput: move |evt| image.set(evt.value().clone())
                            }
                        }
                    }
                }
            }

            // Labels Section
            div { class: section_class("labels"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("labels"),
                    h2 { class: "section-title", "Labels" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        div { class: "form-row form-header",
                            div { class: "form-group",
                                label { class: "form-label", "Key" }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Value" }
                            }
                        }
                        {labels().iter().enumerate().map(|(i, label)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-row",
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "app",
                                            value: "{label.key}",
                                            oninput: move |evt| {
                                                let mut new_labels = labels();
                                                new_labels[i].key = evt.value().clone();
                                                labels.set(new_labels);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "mysql",
                                            value: "{label.value}",
                                            oninput: move |evt| {
                                                let mut new_labels = labels();
                                                new_labels[i].value = evt.value().clone();
                                                labels.set(new_labels);
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                        button {
                            class: "create-form-btn create-form-btn-secondary",
                            onclick: move |_| {
                                let mut new_labels = labels();
                                new_labels.push(KeyValuePair::default());
                                labels.set(new_labels);
                            },
                            "Add Label"
                        }
                    }
                }
            }

            // Selector Labels Section
            div { class: section_class("selectors"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("selectors"),
                    h2 { class: "section-title", "Selector Labels" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        div { class: "form-row form-header",
                            div { class: "form-group",
                                label { class: "form-label", "Key" }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Value" }
                            }
                        }
                        {selectors().iter().enumerate().map(|(i, selector)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-row",
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "app",
                                            value: "{selector.key}",
                                            oninput: move |evt| {
                                                let mut new_selectors = selectors();
                                                new_selectors[i].key = evt.value().clone();
                                                selectors.set(new_selectors);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "mysql",
                                            value: "{selector.value}",
                                            oninput: move |evt| {
                                                let mut new_selectors = selectors();
                                                new_selectors[i].value = evt.value().clone();
                                                selectors.set(new_selectors);
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                        button {
                            class: "create-form-btn create-form-btn-secondary",
                            onclick: move |_| {
                                let mut new_selectors = selectors();
                                new_selectors.push(KeyValuePair::default());
                                selectors.set(new_selectors);
                            },
                            "Add Selector"
                        }
                    }
                }
            }

            // Annotations Section
            div { class: section_class("annotations"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("annotations"),
                    h2 { class: "section-title", "Annotations" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        div { class: "form-row form-header",
                            div { class: "form-group",
                                label { class: "form-label", "Key" }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Value" }
                            }
                        }
                        {annotations().iter().enumerate().map(|(i, annotation)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-row",
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "version",
                                            value: "{annotation.key}",
                                            oninput: move |evt| {
                                                let mut new_annotations = annotations();
                                                new_annotations[i].key = evt.value().clone();
                                                annotations.set(new_annotations);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "v1.0",
                                            value: "{annotation.value}",
                                            oninput: move |evt| {
                                                let mut new_annotations = annotations();
                                                new_annotations[i].value = evt.value().clone();
                                                annotations.set(new_annotations);
                                            }
                                        }
                                    }
                                }
                            }
                        })}                                button {
                                    class: "create-form-btn create-form-btn-secondary",
                                    onclick: move |_| {
                                        let mut new_annotations = annotations();
                                        new_annotations.push(KeyValuePair::default());
                                        annotations.set(new_annotations);
                                    },
                                    "Add Annotation"
                                }
                    }
                }
            }

            // Storage Section
            div { class: section_class("storage"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("storage"),
                    h2 { class: "section-title", "Storage Configuration" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        h3 { class: "section-subtitle", "Volume Claim Templates" }
                        {volume_claims().iter().enumerate().map(|(i, vc)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-grid",
                                    div { class: "form-group",
                                        label { class: "form-label", "Volume Name" }
                                        input {
                                            class: "form-input",
                                            placeholder: "data",
                                            value: "{vc.name}",
                                            oninput: move |evt| {
                                                let mut new_claims = volume_claims();
                                                new_claims[i].name = evt.value().clone();
                                                volume_claims.set(new_claims);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        label { class: "form-label", "Storage Size" }
                                        input {
                                            class: "form-input",
                                            placeholder: "1Gi",
                                            value: "{vc.storage_size}",
                                            oninput: move |evt| {
                                                let mut new_claims = volume_claims();
                                                new_claims[i].storage_size = evt.value().clone();
                                                volume_claims.set(new_claims);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        label { class: "form-label", "Storage Class" }
                                        input {
                                            class: "form-input",
                                            placeholder: "standard",
                                            value: "{vc.storage_class}",
                                            oninput: move |evt| {
                                                let mut new_claims = volume_claims();
                                                new_claims[i].storage_class = evt.value().clone();
                                                volume_claims.set(new_claims);
                                            }
                                        }
                                    }
                                }
                            }
                        })}

                        h3 { class: "section-subtitle", "Volume Mounts" }
                        {volume_mounts().iter().enumerate().map(|(i, vm)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-grid",
                                    div { class: "form-group",
                                        label { class: "form-label", "Volume Name" }
                                        input {
                                            class: "form-input",
                                            placeholder: "data",
                                            value: "{vm.name}",
                                            oninput: move |evt| {
                                                let mut new_mounts = volume_mounts();
                                                new_mounts[i].name = evt.value().clone();
                                                volume_mounts.set(new_mounts);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        label { class: "form-label", "Mount Path" }
                                        input {
                                            class: "form-input",
                                            placeholder: "/data",
                                            value: "{vm.mount_path}",
                                            oninput: move |evt| {
                                                let mut new_mounts = volume_mounts();
                                                new_mounts[i].mount_path = evt.value().clone();
                                                volume_mounts.set(new_mounts);
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }

            // Resource Section
            div { class: section_class("resources"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("resources"),
                    h2 { class: "section-title", "Container Resources" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "form-grid",
                        div { class: "form-group",
                            label { class: "form-label", "CPU Request" }
                            input {
                                class: "form-input",
                                placeholder: "100m",
                                value: "{cpu_request}",
                                oninput: move |evt| cpu_request.set(evt.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Memory Request" }
                            input {
                                class: "form-input",
                                placeholder: "128Mi",
                                value: "{memory_request}",
                                oninput: move |evt| memory_request.set(evt.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "CPU Limit" }
                            input {
                                class: "form-input",
                                placeholder: "200m",
                                value: "{cpu_limit}",
                                oninput: move |evt| cpu_limit.set(evt.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Memory Limit" }
                            input {
                                class: "form-input",
                                placeholder: "256Mi",
                                value: "{memory_limit}",
                                oninput: move |evt| memory_limit.set(evt.value().clone())
                            }
                        }
                    }
                }
            }

            // Ports Section
            div { class: section_class("ports"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("ports"),
                    h2 { class: "section-title", "Container Ports" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        div { class: "form-row form-header",
                            div { class: "form-group",
                                label { class: "form-label", "Port" }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Protocol" }
                            }
                        }
                        {ports().iter().enumerate().map(|(i, port)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-row",
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "3306",
                                            value: "{port.container_port}",
                                            oninput: move |evt| {
                                                let mut new_ports = ports();
                                                new_ports[i].container_port = evt.value().clone();
                                                ports.set(new_ports);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        select {
                                            class: "form-input",
                                            value: "{port.protocol}",
                                            onchange: move |evt| {
                                                let mut new_ports = ports();
                                                new_ports[i].protocol = evt.value().clone();
                                                ports.set(new_ports);
                                            },
                                            option { value: "TCP", "TCP" }
                                            option { value: "UDP", "UDP" }
                                        }
                                    }
                                }
                            }
                        })}
                        button {
                            class: "btn-secondary",
                            onclick: move |_| {
                                let mut new_ports = ports();
                                new_ports.push(ContainerPort::default());
                                ports.set(new_ports);
                            },
                            "Add Port"
                        }
                    }
                }
            }

            // Environment Variables Section
            div { class: section_class("env"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("env"),
                    h2 { class: "section-title", "Environment Variables" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        div { class: "form-row form-header",
                            div { class: "form-group",
                                label { class: "form-label", "Name" }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Value" }
                            }
                        }
                        {env_vars().iter().enumerate().map(|(i, env)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-row",
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            value: "{env.name}",
                                            oninput: move |evt| {
                                                let mut new_envs = env_vars();
                                                new_envs[i].name = evt.value().clone();
                                                env_vars.set(new_envs);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            value: "{env.value}",
                                            oninput: move |evt| {
                                                let mut new_envs = env_vars();
                                                new_envs[i].value = evt.value().clone();
                                                env_vars.set(new_envs);
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                        button {
                            class: "btn-secondary",
                            onclick: move |_| {
                                let mut new_envs = env_vars();
                                new_envs.push(EnvVar::default());
                                env_vars.set(new_envs);
                            },
                            "Add Environment Variable"
                        }
                    }
                }
            }

            {error().map(|err| rsx!(
                div { class: "error-message", "{err}" }
            ))}

            div { class: "button-group",
                button {
                    class: "create-form-btn create-form-btn-primary",
                    onclick: submit,
                    "Create StatefulSet"
                }
                button {
                    class: "create-form-btn create-form-btn-secondary",
                    onclick: move |_| {
                        navigate.push("/statefulsets");
                    },
                    "Cancel"
                }
            }
        }
    }
}
