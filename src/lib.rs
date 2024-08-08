#[cfg(any(target_os = "linux", target_os = "android"))]
#[path = "linux.rs"]
mod platform;

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[path = "darwin.rs"]
mod platform;

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
)))]
mod platform {
    use crate::MemoryStats;
    use crate::Proc;

    pub fn memory_stats(_: Proc) -> Option<MemoryStats> {
        None
    }
}

pub enum Proc {
    This,
    Other(usize),
}

impl From<usize> for Proc {
    fn from(value: usize) -> Self {
        Self::Other(value)
    }
}

impl From<&str> for Proc {
    fn from(value: &str) -> Self {
        match value.parse::<usize>() {
            Ok(id) => Proc::Other(id),
            Err(_) => Proc::This,
        }
    }
}

impl From<String> for Proc {
    fn from(value: String) -> Self {
        match value.parse::<usize>() {
            Ok(id) => Proc::Other(id),
            Err(_) => Proc::This,
        }
    }
}

/// Statistics on the memory used by the specified process.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MemoryStats {
    /// The "physical" memory used, in bytes.
    pub physical_mem: usize,

    /// The "virtual" memory used, in bytes.
    pub virtual_mem: usize,
}

/// Returns a snapshot of the the memory used by the
/// specified process.
///
/// # Errors
///
/// If the current memory usage cannot be queried
/// or `memory_stats` is run on a unsupported platform,
/// `None` is returned.
pub fn memory_stats(proc: impl Into<Proc>) -> Option<MemoryStats> {
    platform::memory_stats(proc.into())
}
