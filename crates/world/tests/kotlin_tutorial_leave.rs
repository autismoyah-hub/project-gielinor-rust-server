use gielinor_world::{inventory::{Catalog,Insert,Inventory,Item,StackType},tutorial_leave::Config,tutorial_progression::{self,Effect},variables::{Transmit,Varp,Variables}};
fn encode(inv:&Inventory)->String {
    inv.slots().iter().map(|item|item.map_or("_".into(),|item|format!("{}:{}:{}",item.id,item.count,item.vars))).collect::<Vec<_>>().join(";")
}
#[test]
fn original_leave_grants_bank_topup_and_completion_order_match() {
    let fixture=std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/tutorial-leave.expected")).expect("Run scripts/tutorial-leave-parity.ps1");
    let varp=|id|Varp{id,permanent:true,transmit:Transmit::Always,bit_protect:false,client_code:-1};
    let config=Config{progression:tutorial_progression::Config{step:varp(1),completed:varp(2)},items:std::array::from_fn(|i|i as i32+100),coins:995};
    let mut types=Catalog::default();types.stackable.extend(109..=114);types.stackable.insert(995);
    let mut cases=0;
    for row in fixture.lines() {
        let f:Vec<_>=row.split('|').collect();let cap=f[0].parse::<usize>().unwrap();let bank_case=f[1].parse::<usize>().unwrap();let raw=f[2].parse::<i32>().unwrap();let done=f[3].parse::<i32>().unwrap();
        let mut inv=Inventory::new(cap,StackType::Normal,false);inv.restore_slot(cap-1,Some(Item{id:42,count:1,vars:0})).unwrap();
        let mut bank=Inventory::new(usize::from(bank_case!=0),StackType::Always,false);
        if bank_case==1 {bank.insert(&types,Insert::new(42,1)).unwrap();}
        if bank_case>=2 {bank.insert(&types,Insert::new(995,[12,25,100][bank_case-2])).unwrap();}
        let mut vars=Variables::restore([(1,raw),(2,done)].into());
        config.grant(&mut inv,&mut bank,&types);
        let output=config.progression.finish_and_leave(&mut vars,true).into_iter().map(|effect|match effect {
            Effect::Variable(update)=>format!("var:{}:{}",update.id,update.value),Effect::End=>"end".into(),Effect::Focus{..}=>panic!("Unexpected focus"),
        }).collect::<Vec<_>>().join(",");
        assert_eq!(encode(&inv),f[4],"{row}");assert_eq!(encode(&bank),f[5],"{row}");
        let to=Config::destination();assert_eq!(format!("{},{},{}",to.x(),to.z(),to.level()),f[6]);
        assert_eq!(vars.get(1),f[7].parse::<i32>().unwrap());assert_eq!(vars.get(2),f[8].parse::<i32>().unwrap());assert_eq!(output,f[9],"{row}");cases+=1;
    }
    assert_eq!(cases,180);
}
