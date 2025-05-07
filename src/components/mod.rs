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

mod namespace_selector;
pub use namespace_selector::NamespaceSelector;

mod status_selector;
pub use status_selector::StatusSelector;

mod search_input;
pub use search_input::SearchInput;