use crate::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        
        div { class: "layout-container",
            div {
                id: "sidebar",
                class: "k8s-sidebar",
                div {
                    class: "sidebar-logo",
                    img { src: "/assets/kubernetes-logo.svg", alt: "Kubernetes Logo" }
                    span { "Kubernetes Dashboard" }
                }
                nav {
                    class: "sidebar-links",
                    div { class: "nav-group",
                        span { class: "nav-group-title", "CLUSTER" }
                        Link {
                            to: Route::Home {},
                            class: "nav-overview",
                            "Overview"
                        }
                        Link {
                            to: Route::Nodes {},
                            class: "nav-nodes",
                            "Nodes"
                        }
                        Link {
                            to: Route::Namespaces {},
                            class: "nav-namespaces",
                            "Namespaces"
                        }
                    }
                    div { class: "nav-group",
                        span { class: "nav-group-title", "WORKLOADS" }
                        Link {
                            to: Route::Pods {},
                            class: "nav-pods",
                            "Pods"
                        }
                        Link {
                            to: Route::Deployments {},
                            class: "nav-deployments",
                            "Deployments"
                        }
                        Link {
                            to: Route::StatefulSets {},
                            class: "nav-statefulsets",
                            "StatefulSets"
                        }
                        Link {
                            to: Route::DaemonSets {},
                            class: "nav-daemonsets",
                            "DaemonSets"
                        }
                    }
                    div { class: "nav-group",
                        span { class: "nav-group-title", "NETWORK" }
                        Link {
                            to: Route::Services {},
                            class: "nav-services",
                            "Services"
                        }
                        Link {
                            to: Route::Ingresses {},
                            class: "nav-ingress",
                            "Ingress"
                        }
                    }
                    div { class: "nav-group",
                        span { class: "nav-group-title", "STORAGE" }
                        Link {
                            to: Route::Pvcs {}, // Update route
                            class: "nav-pvcs",
                            "Persistent Volume Claims"
                        }
                        Link {
                            to: Route::ConfigMaps {}, // Update route
                            class: "nav-configmaps",
                            "Config Maps"
                        }
                        Link {
                            to: Route::Secrets {}, // Update route
                            class: "nav-secrets",
                            "Secrets"
                        }
                    }
                    div { class: "nav-group",
                        span { class: "nav-group-title", "SETTINGS" }
                        Link {
                            to: Route::Blog { id: 12 },
                            class: "nav-settings",
                            "Settings"
                        }
                    }
                }
            }
            div {
                class: "main-content",
                Outlet::<Route> {}
            }
        }
    }
}
