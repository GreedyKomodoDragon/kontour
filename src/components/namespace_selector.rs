use dioxus::{prelude::*};
use k8s_openapi::api::core::v1::Namespace;
use kube::{api::ListParams, Api, Client};

#[derive(Props, PartialEq, Clone)]
pub struct NamespaceSelectorProps {
    selected_namespace: String,
    on_change: EventHandler<String>,
}

#[component]
pub fn NamespaceSelector(props: NamespaceSelectorProps) -> Element {
    let client_signal = use_context::<Signal<Option<Client>>>();

    // Signal for holding namespaces fetched from Kubernetes
    let mut namespaces = use_signal(|| Vec::<String>::new());

    // Fetch namespaces using `use_effect` - always call the hook but handle conditionals inside
    use_effect({        
        move || {
            if let Some(client) = &*client_signal.read() {
                let client = client.clone();
                spawn(async move {
                    let ns_api: Api<Namespace> = Api::all(client);
                    match ns_api.list(&ListParams::default()).await {
                        Ok(ns_list) => {
                            let mut ns_names = ns_list.items
                                .into_iter()
                                .filter_map(|ns| ns.metadata.name)
                                .collect::<Vec<_>>();
                            ns_names.insert(0, "All".to_string());
                            namespaces.set(ns_names);
                        }
                        Err(e) => {
                            println!("Error fetching namespaces: {:?}", e);
                        }
                    }
                });
            } else {
                // Set default namespaces when no client is available
                namespaces.set(vec!["All".to_string()]);
            }
        }
    });

    rsx! {
        select {
            class: "namespace-select",
            value: "{props.selected_namespace}",
            onchange: move |evt| {
                props.on_change.call(evt.value());
            },
            {namespaces().iter().map(|ns| {
                rsx! {
                    option {
                        value: "{ns}",
                        "{ns}"
                    }
                }
            })}
        }
    }
}
