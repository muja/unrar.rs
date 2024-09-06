#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(not(target_os = "linux"), path = "all.rs")]
mod os;

pub(crate) use os::*;