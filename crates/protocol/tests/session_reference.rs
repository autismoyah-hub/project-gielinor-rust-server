use gielinor_protocol::{
    framing::{self, OpcodeWidth},
    isaac::Isaac,
    rsa::PrivateKey,
    session::{Accepted, Event, Session, State},
    tables,
};
fn hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture() -> (PrivateKey, Vec<u8>, [u32; 23]) {
    let key = PrivateKey::from_be_bytes(
        &hex(include_str!(
            "../../../reference/protocol/login-fixture/rsa-exponent.hex"
        )),
        &hex(include_str!(
            "../../../reference/protocol/login-fixture/rsa-modulus.hex"
        )),
    )
    .unwrap();
    let payload = hex(include_str!(
        "../../../reference/protocol/login-fixture/login.hex"
    ));
    let mut wire = vec![14, 16];
    wire.extend((payload.len() as u16).to_be_bytes());
    wire.extend(payload);
    (
        key,
        wire,
        std::array::from_fn(|i| 0x80102030u32.wrapping_add(i as u32 * 0x010203)),
    )
}
#[test]
fn fragmented_login_authentication_and_pipelined_game_packet() {
    let (key, wire, crc) = fixture();
    for chunk in [1, 2, 3, 8, 64, 4096] {
        let mut session = Session::new(0x1020304050607080, crc);
        let mut logins = 0;
        let mut writes = 0;
        for part in wire.chunks(chunk) {
            for event in session.receive(part, &key).unwrap() {
                match event {
                    Event::Authenticate(login) => {
                        assert_eq!(login.username, "parity_actor");
                        logins += 1;
                    }
                    Event::Write(v) => {
                        assert_eq!(v.len(), 9);
                        writes += 1;
                    }
                    _ => panic!("unexpected login event"),
                }
            }
        }
        assert_eq!((logins, writes), (1, 1));
        assert_eq!(session.state(), State::Authenticating);
        let def = *tables::CLIENT
            .iter()
            .find(|d| matches!(d.size, framing::Size::Fixed(0)))
            .unwrap();
        let mut cipher = Some(Isaac::new(&[0x80000000, 239, 1, u32::MAX]).unwrap());
        let packet = framing::encode(def, &[], &mut cipher, OpcodeWidth::Byte).unwrap();
        assert!(session.receive(&packet, &key).unwrap().is_empty());
        let response = session
            .accept(Accepted {
                slot: 1,
                staff_level: 0,
                player_mod: false,
                member: false,
                account_hash: 1,
                user_id: 2,
                user_hash: 3,
                authenticator_code: None,
            })
            .unwrap();
        assert_eq!(response.len(), 36);
        assert_eq!(&response[..2], &[2, 37]);
        let events = session.receive(&[], &key).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0],Event::Game(p) if p.opcode==def.opcode));
        assert!(session
            .accept(Accepted {
                slot: 1,
                staff_level: 0,
                player_mod: false,
                member: false,
                account_hash: 1,
                user_id: 2,
                user_hash: 3,
                authenticator_code: None
            })
            .is_err());
    }
}
#[test]
fn seed_replay_and_wrong_cache_are_rejected_before_authentication() {
    let (key, wire, crc) = fixture();
    for (seed, crcs, status) in [(1, crc, 10), (0x1020304050607080, [0; 23], 68)] {
        let mut session = Session::new(seed, crcs);
        let events = session.receive(&wire, &key).unwrap();
        assert!(events
            .iter()
            .any(|v| matches!(v,Event::Rejected(s) if *s==status)));
        assert_eq!(session.state(), State::Closed);
        assert!(session.receive(&wire, &key).is_err());
    }
}

#[test]
fn login_success_length_includes_first_game_header_and_preserves_cipher_position() {
    let (key, wire, crc) = fixture();
    for token in [None, Some(0x12345678u32)] {
        let mut session = Session::new(0x1020304050607080, crc);
        session.receive(&wire, &key).unwrap();
        let mut bytes = session.accept(Accepted {
            slot: 1, staff_level: 0, player_mod: false, member: false,
            account_hash: 1, user_id: 2, user_hash: 3, authenticator_code: token,
        }).unwrap();
        let mut cipher = Isaac::new(&[0x80000032, 289, 51, 49]).unwrap();
        if let Some(token) = token {
            assert_eq!(bytes[2], 1);
            let decoded = std::array::from_fn(|i| bytes[3+i].wrapping_sub(cipher.next_u32() as u8));
            assert_eq!(u32::from_be_bytes(decoded), token);
        }
        let rebuild = vec![0; 4608];
        bytes.extend(session.encode(49, &rebuild).unwrap());
        // The native client's login stage reads the advertised prefix, but only
        // consumes account fields; the first game header belongs to the framer.
        assert_eq!(bytes[1] as usize, 34 + 3);
        assert_eq!(bytes[36].wrapping_sub(cipher.next_u32() as u8), 49);
        assert_eq!(u16::from_be_bytes([bytes[37],bytes[38]]) as usize, rebuild.len());
        assert_eq!(&bytes[39..], rebuild);
    }
}
