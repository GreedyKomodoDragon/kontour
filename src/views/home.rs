use dioxus::prelude::*;

const OVERVIEW_CSS: Asset = asset!("/assets/styling/overview.css");

/// The Home page component that renders the Kubernetes dashboard overview
#[component]
pub fn Home() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: OVERVIEW_CSS }
        
        div { class: "overview-container",
            div { class: "overview-header",
                h1 { "Cluster Overview" }
            }
            
            // Cluster Status Cards
            div { class: "cluster-status",
                div { class: "status-card",
                    h3 { "Cluster Status" }
                    p { class: "status-value status-healthy", "Healthy" }
                    p { class: "status-subtext", "Last checked 2 minutes ago" }
                }
                div { class: "status-card",
                    h3 { "Node Count" }
                    p { class: "status-value", "4" }
                    p { class: "status-subtext", "All nodes operational" }
                }
                div { class: "status-card",
                    h3 { "Pod Count" }
                    p { class: "status-value", "24/30" }
                    p { class: "status-subtext", "Running/Total" }
                }
                div { class: "status-card",
                    h3 { "Namespace Count" }
                    p { class: "status-value", "6" }
                    p { class: "status-subtext", "Active namespaces" }
                }
            }

            // Resource Usage Section
            div { class: "resource-section",
                h2 { "Resource Usage" }
                div { class: "resource-grid",
                    div { class: "resource-card",
                        div { class: "resource-header",
                            h3 { class: "resource-title", "CPU Usage" }
                        }
                        div { class: "resource-stats",
                            div { class: "stat-item",
                                span { class: "stat-value", "65%" }
                                span { class: "stat-label", "Utilization" }
                            }
                            div { class: "stat-item",
                                span { class: "stat-value", "12/16" }
                                span { class: "stat-label", "Cores Used" }
                            }
                        }
                    }
                    div { class: "resource-card",
                        div { class: "resource-header",
                            h3 { class: "resource-title", "Memory Usage" }
                        }
                        div { class: "resource-stats",
                            div { class: "stat-item",
                                span { class: "stat-value", "78%" }
                                span { class: "stat-label", "Utilization" }
                            }
                            div { class: "stat-item",
                                span { class: "stat-value", "47/64" }
                                span { class: "stat-label", "GB Used" }
                            }
                        }
                    }
                    div { class: "resource-card",
                        div { class: "resource-header",
                            h3 { class: "resource-title", "Storage Usage" }
                        }
                        div { class: "resource-stats",
                            div { class: "stat-item",
                                span { class: "stat-value", "45%" }
                                span { class: "stat-label", "Utilization" }
                            }
                            div { class: "stat-item",
                                span { class: "stat-value", "450/1000" }
                                span { class: "stat-label", "GB Used" }
                            }
                        }
                    }
                }
            }

            // Recent Events Section
            div { class: "resource-section",
                h2 { "Recent Events" }
                table { class: "events-table",
                    thead {
                        tr {
                            th { "Time" }
                            th { "Type" }
                            th { "Resource" }
                            th { "Message" }
                        }
                    }
                    tbody {
                        tr {
                            td { "2m ago" }
                            td { class: "status-healthy", "Normal" }
                            td { "Pod/nginx-deployment-123" }
                            td { "Successfully pulled image" }
                        }
                        tr {
                            td { "5m ago" }
                            td { class: "status-warning", "Warning" }
                            td { "Node/worker-1" }
                            td { "High CPU usage detected" }
                        }
                        tr {
                            td { "10m ago" }
                            td { class: "status-healthy", "Normal" }
                            td { "Deployment/web-app" }
                            td { "Scaled replicas to 3" }
                        }
                        tr {
                            td { "15m ago" }
                            td { class: "status-error", "Error" }
                            td { "PersistentVolume/data-01" }
                            td { "Volume mount failed" }
                        }
                    }
                }
            }
        }
    }
}
