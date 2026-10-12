use gielinor_world::{
    combat_state::{npc_uid, Config},
    variables::{Transmit, Variables, Varp},
};
#[test]
fn temporary_combat_clocks_and_npc_uid_follow_reference_boundaries() {
    let varp = |id| Varp {
        id,
        permanent: false,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    let config = Config {
        last_npc: varp(1),
        last_pvp: varp(2),
        aggressive_npc: varp(3),
    };
    let mut vars = Variables::default();
    let source = npc_uid(65534, 40000);
    assert_eq!(source as u32, 0x9c40_fffe);
    assert!(config.denied(&vars, 8, source));
    assert!(!config.denied(&vars, 9, source));
    let updates = config.record(&mut vars, 20, source, true);
    assert_eq!(
        updates.iter().map(|u| (u.id, u.value)).collect::<Vec<_>>(),
        [(1, 20), (3, source)]
    );
    assert!(!config.denied(&vars, 28, source));
    assert!(config.denied(&vars, 28, npc_uid(1, 40000)));
    assert!(!config.denied(&vars, 29, npc_uid(1, 40000)));
    assert!(config.clear_on_death(&mut vars, 7, true).is_none());
    assert_eq!(
        config
            .clear_on_death(&mut vars, source, true)
            .unwrap()
            .value,
        0
    );
    assert_eq!(
        vars.get(3),
        source,
        "death clears the clock, not the aggressor uid"
    );
}
