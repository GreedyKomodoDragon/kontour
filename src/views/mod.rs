//! The views module contains the components for all Layouts and Routes for our app.
//! Each submodule corresponds to a specific section of the application,
//! encapsulating the layout and routing logic for that section.

mod home;
pub use home::Home;

mod blog;
pub use blog::Blog;

mod navbar;
pub use navbar::Navbar;

mod nodes;
pub use nodes::Nodes;

mod namespaces;
pub use namespaces::Namespaces;

mod pods;
pub use pods::Pods;
