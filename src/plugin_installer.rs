//! Plugin installation module for PoshBuddy.

/// Plugin installer.
pub struct PluginInstaller;

impl PluginInstaller {
    /// Creates a new installer
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl Default for PluginInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
}
