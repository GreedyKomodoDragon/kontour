//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component  to be used in our app.

mod hero;
pub mod kubeconfig_name_dialog;

mod pod_item;
pub use pod_item::PodItem;

mod deployment_item;
pub use deployment_item::DeploymentItem;

mod statefulset_item;
pub use statefulset_item::StatefulSetItem;

mod daemonset_item;
pub use daemonset_item::DaemonSetItem;

mod namespace_selector;
pub use namespace_selector::NamespaceSelector;

mod status_selector;
pub use status_selector::StatusSelector;

mod search_input;
pub use search_input::SearchInput;

mod service_item;
pub use service_item::ServiceItem;

mod ingress_item;
pub use ingress_item::IngressItem;

mod pvc_item;
pub use pvc_item::PvcItem;

mod configmap_item;
pub use configmap_item::ConfigMapItem;

mod secret_item;
pub use secret_item::SecretItem;

mod job_item;
pub use job_item::JobItem;

mod cronjob_item;
pub use cronjob_item::CronJobItem;

mod pod_containers;
pub use pod_containers::*;

mod namespace_item;
pub use namespace_item::{NamespaceItem, ResourceQuota, LimitRange, NamespaceItemProps};