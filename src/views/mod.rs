//! The views module contains the components for all Layouts and Routes for our app.
//! Each submodule corresponds to a specific section of the application,
//! encapsulating the layout and routing logic for that section.

mod blog;
pub use blog::Blog;

mod deployments;
pub use deployments::Deployments;

mod statefulsets;
pub use statefulsets::StatefulSets;

mod daemonsets;
pub use daemonsets::DaemonSets;

mod services;
pub use services::Services;

mod ingresses;
pub use ingresses::Ingresses;

mod pvcs; // Add module
pub use pvcs::Pvcs; // Add export

mod configmaps; // Add module
pub use configmaps::ConfigMaps; // Add export

mod secrets; // Add module
pub use secrets::Secrets; // Add export

mod home;
pub use home::Home;

mod namespaces;
pub use namespaces::Namespaces;

mod navbar;
pub use navbar::Navbar;

mod nodes;
pub use nodes::Nodes;

mod pods;
pub use pods::Pods;
