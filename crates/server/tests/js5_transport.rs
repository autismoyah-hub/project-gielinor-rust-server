use gielinor_protocol::{js5, rsa::PrivateKey};
use gielinor_server::{
    js5::Provider,
    transport::{Limits, Transport},
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, Read, Write},
    net::TcpStream,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
struct Memory {
    groups: BTreeMap<(u8, u16), Arc<[u8]>>,
    calls: Rc<RefCell<Vec<(u8, u16)>>>,
}
impl Provider for Memory {
    fn response(&mut self, a: u8, g: u16) -> io::Result<Option<Arc<[u8]>>> {
        self.calls.borrow_mut().push((a, g));
        self.groups
            .get(&(a, g))
            .map(|b| Some(b.clone()))
            .ok_or_else(|| io::Error::other("missing test group"))
    }
}
#[test]
fn fragmented_js5_handshake_promotes_urgent_and_streams_xor_groups_with_backpressure() {
    let calls = Rc::new(RefCell::new(vec![]));
    let first = js5::response(1, 1, &(0..130_000).map(|i| i as u8).collect::<Vec<_>>());
    let second = js5::response(1, 2, &[7; 2048]);
    let mut server = Transport::bind(
        "127.0.0.1:0".parse().unwrap(),
        PrivateKey::from_be_bytes(&[1], &[255, 255]).unwrap(),
        [0; 23],
        Limits {
            output_bytes: 1024,
            ..Limits::default()
        },
    )
    .unwrap();
    server
        .configure_js5(Box::new(Memory {
            groups: BTreeMap::from([
                ((1, 1), Arc::from(first.clone())),
                ((1, 2), Arc::from(second.clone())),
            ]),
            calls: calls.clone(),
        }))
        .unwrap();
    let mut client = TcpStream::connect(server.address().unwrap()).unwrap();
    client.set_nonblocking(true).unwrap();
    client.write_all(&[15, 0]).unwrap();
    assert!(server.poll(|| Ok(1)).unwrap().is_empty());
    let mut header = vec![];
    header.extend(239u32.to_be_bytes());
    header.extend([0; 16]);
    let mut tail = header[1..].to_vec();
    // Prefetch 1 is promoted, not transmitted twice. Two low-priority messages
    // release queued prefetch immediately, avoiding wall-clock dependent tests.
    tail.extend([
        0, 1, 0, 1, 0, 1, 0, 2, 1, 1, 0, 1, 3, 0, 0, 0, 3, 0, 0, 0, 4, 90, 7, 9,
    ]);
    client.write_all(&tail).unwrap();
    let mut expected = vec![0];
    expected.extend(first.into_iter().chain(second).map(|v| v ^ 90));
    let mut received = vec![];
    let deadline = Instant::now() + Duration::from_secs(3);
    while received.len() < expected.len() {
        let events = server.poll(|| Ok(1)).unwrap();
        assert!(
            events.is_empty(),
            "cache connection emitted a game lifecycle event"
        );
        let mut buffer = [0; 8192];
        match client.read(&mut buffer) {
            Ok(0) => panic!("early disconnect"),
            Ok(n) => received.extend_from_slice(&buffer[..n]),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => panic!("{e}"),
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(received, expected);
    assert_eq!(*calls.borrow(), vec![(1, 1), (1, 2)]);
    client.write_all(&[5, 0, 0, 0]).unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    while server.connection_count() != 0 {
        server.poll(|| Ok(1)).unwrap();
        assert!(Instant::now() < deadline);
    }
    let mut outdated = TcpStream::connect(server.address().unwrap()).unwrap();
    outdated.set_nonblocking(true).unwrap();
    let mut request = vec![15];
    request.extend(238u32.to_be_bytes());
    request.extend([0; 16]);
    outdated.write_all(&request).unwrap();
    let mut status = [0];
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        server.poll(|| Ok(1)).unwrap();
        match outdated.read(&mut status) {
            Ok(1) => break,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
            other => panic!("{other:?}"),
        };
        assert!(Instant::now() < deadline);
    }
    assert_eq!(status, [6]);
}
