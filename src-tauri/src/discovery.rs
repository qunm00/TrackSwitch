use crate::blueutil::Device;

pub fn find_trackpad(devices: &[Device]) -> Option<Device> {
    devices
        .iter()
        .find(|d| d.name.to_lowercase().contains("trackpad"))
        .cloned()
}

pub fn normalize_mac(addr: &str) -> String {
    let cleaned: String = addr.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let lower = cleaned.to_lowercase();
    let mut out = String::new();
    for (i, ch) in lower.chars().enumerate() {
        if i > 0 && i % 2 == 0 {
            out.push(':');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueutil::Device;

    #[test]
    fn finds_magic_trackpad_by_name() {
        let devices = vec![
            Device {
                name: "Magic Keyboard".into(),
                address: "11:22:33:44:55:66".into(),
                paired: true,
                connected: false,
            },
            Device {
                name: "Magic Trackpad".into(),
                address: "aa:bb:cc:dd:ee:ff".into(),
                paired: true,
                connected: false,
            },
        ];
        let t = find_trackpad(&devices).unwrap();
        assert_eq!(t.address, "aa:bb:cc:dd:ee:ff");
    }

    #[test]
    fn normalizes_mac() {
        assert_eq!(normalize_mac("AA-BB-CC-DD-EE-FF"), "aa:bb:cc:dd:ee:ff");
    }

    #[test]
    fn normalizes_mac_without_separators() {
        assert_eq!(normalize_mac("aabbccddeeff"), "aa:bb:cc:dd:ee:ff");
    }
}
