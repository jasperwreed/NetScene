use netscene_lib::{Device};

#[test]
fn parse_linux_style() {
    let sample = "? (192.168.1.1) at aa:bb:cc:dd:ee:ff [ether] on eth0\n? (192.168.1.2) at 11:22:33:44:55:66 [ether] on eth0";
    let devices = netscene_lib::parse_arp_output(sample);
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0], Device { ip: "192.168.1.1".into(), mac: "aa:bb:cc:dd:ee:ff".into() });
    assert_eq!(devices[1].mac, "11:22:33:44:55:66");
}

#[test]
fn parse_macos_style() {
    let sample = "? (192.168.1.3) at 77-88-99-aa-bb-cc on en0 ifscope [ethernet]";
    let devices = netscene_lib::parse_arp_output(sample);
    assert_eq!(devices, vec![Device { ip: "192.168.1.3".into(), mac: "77:88:99:aa:bb:cc".into() }]);
}
