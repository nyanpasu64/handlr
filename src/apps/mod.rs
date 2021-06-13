mod canonical;
mod system;
mod user;

pub use canonical::{CanonicalMimeApps, CANONICAL};
pub use system::SystemApps;
pub use user::{MimeApps, Rule as MimeappsRule, APPS};
