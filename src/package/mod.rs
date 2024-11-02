mod package_manager;
mod error;
mod utilities;
mod zypper;
mod dnf;

pub use package_manager::*;
pub use error::Error;
pub use utilities::get_package_manager;