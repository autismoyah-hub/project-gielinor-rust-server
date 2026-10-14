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
fn active_connection(limits: Limits) -> (Transport, TcpStream, u64) {
    active_connection_mode(limits, false)
}
fn active_connection_mode(limits: Limits, tick_queued: bool) -> (Transport, TcpStream, u64) {
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
    let mut server = Transport::bind("127.0.0.1:0".parse().unwrap(), key, crc, limits).unwrap();
    if tick_queued {
        server.queue_until_tick().unwrap();
    }
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
    (server, client, connection)
}
#[test]
fn real_loopback_socket_handshake_and_controlled_admission() {
    let (mut server, mut client, connection) = active_connection(Limits::default());
    use gielinor_protocol::{
        framing::{Decoder, OpcodeWidth},
        isaac::Isaac,
        tables,
    };
    let mut decoder = Decoder::new(
        tables::SERVER,
        Some(Isaac::new(&[0x80000032, 289, 51, 49]).unwrap()),
        OpcodeWidth::Smart,
        65535,
        131072,
    );
    server.begin_batch().unwrap();
    assert!(server.begin_batch().is_err());
    server.send(connection, 3, &[]).unwrap(); // Low: camera reset.
    assert!(server.send(connection, 97, &[0]).is_err()); // Validation consumes no ISAAC.
    server.send(connection, 97, &[1, 2, 3]).unwrap(); // High.
    server.send(connection, 92, &[]).unwrap(); // Low: reset animations.
    server.send(connection, 143, &[7, 8, 9, 10]).unwrap(); // High smart opcode consumes two cipher words.
    server.send(connection, 97, &[4, 5, 6]).unwrap(); // High FIFO.
    server.send(connection, 83, &[]).unwrap();
    server.end_batch().unwrap();
    let mut batch = [0; 17];
    receive(&mut server, &mut client, &mut batch);
    decoder.feed(&batch).unwrap();
    for (op, payload) in [
        (97, vec![1, 2, 3]),
        (143, vec![7, 8, 9, 10]),
        (97, vec![4, 5, 6]),
        (3, vec![]),
        (92, vec![]),
        (83, vec![]),
    ] {
        let packet = decoder.next_packet().unwrap().unwrap();
        assert_eq!((packet.opcode, packet.payload), (op, payload));
    }
    assert!(decoder.next_packet().unwrap().is_none());
    server.begin_batch().unwrap();
    server.send(connection, 57, &[]).unwrap();
    // A graceful disconnect flushes queued logout before clearing the cipher.
    server.disconnect(connection);
    server.end_batch().unwrap();
    let mut logout = [0];
    receive(&mut server, &mut client, &mut logout);
    decoder.feed(&logout).unwrap();
    assert_eq!(decoder.next_packet().unwrap().unwrap().opcode, 57);
    assert_eq!(server.connection_count(), 0);
}

#[test]
fn queued_and_encoded_bytes_share_one_limit_and_overflow_closes_without_partial_output() {
    let limits = Limits {
        output_bytes: 1024,
        ..Limits::default()
    };
    let (mut server, mut client, connection) = active_connection(limits);
    server.send(connection, 3, &[]).unwrap(); // One encoded byte, not yet polled.
    server.begin_batch().unwrap();
    for _ in 0..255 {
        server.send(connection, 97, &[1, 2, 3]).unwrap();
    }
    // 1021 bytes retained: three more one-byte frames exactly fill the budget.
    for _ in 0..3 {
        server.send(connection, 92, &[]).unwrap();
    }
    assert_eq!(
        server.send(connection, 83, &[]).unwrap_err().kind(),
        std::io::ErrorKind::OutOfMemory
    );
    assert!(server.send(connection, 3, &[]).is_err());
    server.end_batch().unwrap();
    assert!(server
        .poll(|| Ok(42))
        .unwrap()
        .iter()
        .any(|e| matches!(e, NetworkEvent::Disconnected { .. })));
    let mut byte = [0];
    assert_eq!(
        client.read(&mut byte).unwrap(),
        0,
        "Overflow must discard the entire unsent output"
    );
    server.begin_batch().unwrap(); // Batch state recovered even though peer closed.
    server.end_batch().unwrap();
}

#[test]
fn normal_logout_flushes_high_priority_and_discards_pending_ui_without_advancing_isaac() {
    use gielinor_protocol::{
        framing::{Decoder, OpcodeWidth},
        isaac::Isaac,
        tables,
    };
    let (mut server, mut client, connection) = active_connection(Limits::default());
    server.begin_batch().unwrap();
    server.send(connection, 3, &[]).unwrap();
    server.send(connection, 97, &[1, 2, 3]).unwrap();
    server.send(connection, 92, &[]).unwrap();
    server.logout(connection).unwrap();
    server.end_batch().unwrap();
    let mut bytes = [0; 5];
    receive(&mut server, &mut client, &mut bytes);
    let mut decoder = Decoder::new(
        tables::SERVER,
        Some(Isaac::new(&[0x80000032, 289, 51, 49]).unwrap()),
        OpcodeWidth::Smart,
        65535,
        131072,
    );
    decoder.feed(&bytes).unwrap();
    let first = decoder.next_packet().unwrap().unwrap();
    assert_eq!((first.opcode, first.payload), (97, vec![1, 2, 3]));
    assert_eq!(decoder.next_packet().unwrap().unwrap().opcode, 57);
    assert!(decoder.next_packet().unwrap().is_none());
    assert_eq!(client.read(&mut [0]).unwrap(), 0);
}

#[test]
fn production_output_spans_network_pumps_and_each_post_tick_flush() {
    use gielinor_protocol::{
        framing::{Decoder, OpcodeWidth},
        isaac::Isaac,
        tables,
    };
    // The helper completes the real handshake in queued mode without a world tick.
    let (mut server, mut client, connection) = active_connection_mode(Limits::default(), true);
    assert!(server.queue_until_tick().is_err());
    let mut decoder = Decoder::new(
        tables::SERVER,
        Some(Isaac::new(&[0x80000032, 289, 51, 49]).unwrap()),
        OpcodeWidth::Smart,
        65535,
        131072,
    );
    for cycle in 0..2 {
        server.send(connection, 3, &[]).unwrap(); // A low-priority login/input response.
        server.poll(|| Ok(42)).unwrap();
        client.set_nonblocking(true).unwrap();
        assert_eq!(
            client.read(&mut [0]).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        client.set_nonblocking(false).unwrap();
        server.begin_batch().unwrap();
        server.send(connection, 97, &[cycle, 2, 3]).unwrap();
        server.send(connection, 83, &[]).unwrap();
        server.end_batch().unwrap();
        let mut bytes = [0; 6];
        receive(&mut server, &mut client, &mut bytes);
        decoder.feed(&bytes).unwrap();
        for expected in [97, 3, 83] {
            assert_eq!(decoder.next_packet().unwrap().unwrap().opcode, expected);
        }
        assert!(decoder.next_packet().unwrap().is_none());
    }
    // Logout can occur between ticks; discard low-priority responses and flush
    // high-priority state plus LOGOUT before destroying the session cipher.
    server.send(connection, 92, &[]).unwrap();
    server.send(connection, 97, &[4, 5, 6]).unwrap();
    server.logout(connection).unwrap();
    let mut bytes = [0; 5];
    receive(&mut server, &mut client, &mut bytes);
    decoder.feed(&bytes).unwrap();
    for expected in [97, 57] {
        assert_eq!(decoder.next_packet().unwrap().unwrap().opcode, expected);
    }
    assert!(decoder.next_packet().unwrap().is_none());
    assert_eq!(client.read(&mut [0]).unwrap(), 0);
}
