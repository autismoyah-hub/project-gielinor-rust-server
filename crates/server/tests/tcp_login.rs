use gielinor_protocol::{rsa::PrivateKey, session::Accepted};
use gielinor_server::transport::{Limits, NetworkEvent, Transport};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};
fn hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn receive(server: &mut Transport, client: &mut TcpStream, bytes: &mut [u8]) {
    client.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut offset = 0;
    while offset < bytes.len() {
        server.poll(|| Ok(0x1020304050607080)).unwrap();
        match client.read(&mut bytes[offset..]) {
            Ok(0) => panic!("Peer closed"),
            Ok(n) => offset += n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => panic!("{e}"),
        };
        assert!(Instant::now() < deadline, "Socket response timed out");
        if offset < bytes.len() {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    client.set_nonblocking(false).unwrap();
}
#[test]
fn real_loopback_socket_handshake_and_controlled_admission() {
    let key = PrivateKey::from_be_bytes(
        &hex(include_str!(
            "../../../reference/protocol/login-fixture/rsa-exponent.hex"
        )),
        &hex(include_str!(
            "../../../reference/protocol/login-fixture/rsa-modulus.hex"
        )),
    )
    .unwrap();
    let crc = std::array::from_fn(|i| 0x80102030u32.wrapping_add(i as u32 * 0x010203));
    let mut server =
        Transport::bind("127.0.0.1:0".parse().unwrap(), key, crc, Limits::default()).unwrap();
    let mut client = TcpStream::connect(server.address().unwrap()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    client
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    client.write_all(&[14]).unwrap();
    let mut challenge = [0; 9];
    receive(&mut server, &mut client, &mut challenge);
    assert_eq!(challenge, [0, 16, 32, 48, 64, 80, 96, 112, 128]);
    let payload = hex(include_str!(
        "../../../reference/protocol/login-fixture/login.hex"
    ));
    let mut wire = vec![16];
    wire.extend((payload.len() as u16).to_be_bytes());
    wire.extend(payload);
    client.write_all(&wire).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let connection = loop {
        let events = server.poll(|| Ok(42)).unwrap();
        if let Some(id) = events.iter().find_map(|e| {
            if let NetworkEvent::Authenticate { connection, login } = e {
                assert_eq!(login.username, "parity_actor");
                Some(*connection)
            } else {
                None
            }
        }) {
            break id;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    // Controlled admission tests the real wire transition; account authentication is tested separately.
    server
        .accept_login(
            connection,
            Accepted {
                slot: 1,
                staff_level: 0,
                player_mod: false,
                member: false,
                account_hash: 1,
                user_id: 2,
                user_hash: 3,
                authenticator_code: None,
            },
        )
        .unwrap();
    let mut response = [0; 36];
    receive(&mut server, &mut client, &mut response);
    assert_eq!(&response[..2], &[2, 37]);
    server.disconnect(connection);
    assert!(server
        .poll(|| Ok(42))
        .unwrap()
        .iter()
        .any(|e| matches!(e, NetworkEvent::Disconnected { .. })));
}
