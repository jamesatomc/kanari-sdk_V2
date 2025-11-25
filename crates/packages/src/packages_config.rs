// Copyright (c) Kanari Network
// SPDX-License-Identifier: Apache-2.0

/// Package configuration
#[derive(Debug, Clone)]
pub struct PackageConfig {
    pub name: &'static str,
    pub directory: &'static str,
    pub address: &'static str,
}

/// All framework packages
const PACKAGES: &[PackageConfig] = &[
    PackageConfig { name: "MoveStdlib", directory: "move-stdlib", address: "0x1" },
    PackageConfig { name: "KanariSystem", directory: "kanari-system", address: "0x2" },
    // เพิ่ม packages ใหม่ที่นี่:
    // PackageConfig { name: "MyPackage", directory: "my-package", address: "0x3" },
];

pub fn get_package_configs() -> Vec<PackageConfig> {
    PACKAGES.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_configs() {
        let configs = get_package_configs();
        assert!(configs.len() >= 2);
        
        let stdlib = configs.iter().find(|p| p.name == "MoveStdlib").unwrap();
        assert_eq!(stdlib.directory, "move-stdlib");
        assert_eq!(stdlib.address, "0x1");
        
        let system = configs.iter().find(|p| p.name == "KanariSystem").unwrap();
        assert_eq!(system.directory, "kanari-system");
        assert_eq!(system.address, "0x2");
    }
}
