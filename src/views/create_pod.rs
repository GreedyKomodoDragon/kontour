use dioxus::prelude::*;
use kube::Client;

const CREATE_POD_CSS: Asset = asset!("/assets/styling/create_pod.css");

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

#[component]
pub fn CreatePod() -> Element {
    let client = use_context::<Client>();
    let navigate = use_navigator();
    
    let mut name = use_signal(String::new);
    let mut namespace = use_signal(|| "default".to_string());
    let mut image = use_signal(String::new);
    
    // Resource requests/limits
    let mut cpu_request = use_signal(|| "100m".to_string());
    let mut memory_request = use_signal(|| "128Mi".to_string());
    let mut cpu_limit = use_signal(|| "200m".to_string());
    let mut memory_limit = use_signal(|| "256Mi".to_string());

    // Container ports
    let mut ports = use_signal(|| vec![ContainerPort::default()]);
    
    // Environment variables
    let mut env_vars = use_signal(|| vec![EnvVar::default()]);

    // Pod labels
    let mut labels = use_signal(|| vec![KeyValuePair::default()]);
    let mut annotations = use_signal(|| vec![KeyValuePair::default()]);

    let mut sections_state = use_signal(|| {
        vec![
            ("basic", false),
            ("labels", false),
            ("annotations", false),
            ("resources", false),
            ("ports", false),
            ("env", false),
        ].into_iter().collect::<std::collections::HashMap<&'static str, bool>>()
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

    let submit = move |_| {
        let name = name();
        let namespace = namespace();
        let image = image();
        
        spawn(async move {
            // TODO: Implement pod creation
            println!("Creating pod: {} in namespace: {} with image: {}", name, namespace, image);
            navigate.push("/pods");
        });
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CREATE_POD_CSS }
        div { class: "create-pod-container",
            h1 { class: "create-pod-title", "Create Pod" }
            
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
                            label { class: "form-label", "Pod Name" }
                            input {
                                class: "form-input",
                                placeholder: "Enter pod name",
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
                            label { class: "form-label", "Container Image" }
                            input {
                                class: "form-input",
                                placeholder: "e.g., nginx:latest",
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
                                            placeholder: "nginx",
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
                            class: "btn-secondary",
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
                                            placeholder: "kubernetes.io/description",
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
                                            placeholder: "My application pod",
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
                        })}
                        button {
                            class: "btn-secondary",
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
                                            placeholder: "80",
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

            div { class: "button-group",
                button {
                    class: "btn-create",
                    onclick: submit,
                    "Create Pod"
                }
                button {
                    class: "btn-cancel",
                    onclick: move |_| {
                        navigate.push("/pods");
                    },
                    "Cancel"
                }
            }
        }
    }
}
