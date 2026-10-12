use gielinor_protocol::js5::Command;
use gielinor_server::js5::{Client, Provider};
use std::{io, sync::Arc, time::Instant};
fn size(group: u16) -> usize {
    [8, 8191, 8192, 8200][group as usize % 4]
}
struct Memory;
impl Provider for Memory {
    fn response(&mut self, archive: u8, group: u16) -> io::Result<Option<Arc<[u8]>>> {
        let mut bytes = vec![0; size(group)];
        bytes[0] = archive;
        bytes[1..3].copy_from_slice(&group.to_be_bytes());
        Ok(Some(bytes.into()))
    }
}
#[test]
fn actual_rsprot_priority_promotion_limits_and_idle_prefetch_order_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/js5-queues.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let (actions, expected) = line.split_once('|').unwrap();
        let now = Instant::now();
        let mut client = Client::new(now);
        let mut memory = Memory;
        for (step, (action, expected)) in actions.split(';').zip(expected.split(';')).enumerate() {
            let result = match &action[..1] {
                "U" | "P" => {
                    if client
                        .push(Command::Request {
                            archive: 1,
                            group: action[1..].parse().unwrap(),
                            urgent: action.starts_with('U'),
                        })
                        .is_ok()
                    {
                        "1".into()
                    } else {
                        "0".into()
                    }
                }
                "L" | "H" => {
                    client
                        .push(Command::Priority {
                            high: action == "H",
                        })
                        .unwrap();
                    "-".into()
                }
                "T" => {
                    if client
                        .transfer_prefetch(&mut memory, action[1..].parse().unwrap())
                        .unwrap()
                    {
                        "1".into()
                    } else {
                        "0".into()
                    }
                }
                "D" => {
                    let bytes = client.output(&mut memory, now, 4_000_000).unwrap();
                    let mut position = 0;
                    let mut groups = Vec::new();
                    while position < bytes.len() {
                        assert_eq!(bytes[position], 1);
                        let group = u16::from_be_bytes(
                            bytes[position + 1..position + 3].try_into().unwrap(),
                        );
                        groups.push(group.to_string());
                        position += size(group);
                    }
                    assert_eq!(position, bytes.len());
                    groups.join(",")
                }
                _ => panic!("Invalid fixture command"),
            };
            assert_eq!(result, expected, "case {case}, step {step}: {action}");
        }
    }
}
