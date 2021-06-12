mod system;
mod user;
mod canonical;

pub use system::SystemApps;
pub use user::{MimeApps, Rule as MimeappsRule, APPS};
pub use canonical::{CanonicalMimeApps, CANONICAL};
