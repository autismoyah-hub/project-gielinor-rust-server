use gielinor_world::{npc::RandomSource, npc_melee_roll::Inputs};
struct Replay(std::collections::VecDeque<(u32,u32)>);
impl RandomSource for Replay {
    fn below(&mut self, bound:u32)->u32 {
        let (expected,value)=self.0.pop_front().expect("Unexpected draw");
        assert_eq!(bound,expected);assert!(value<bound);value
    }
}
#[test]
fn compiled_npc_melee_operations_match_rolls_and_rng_order() {
    let fixture=std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/npc-melee-roll.expected")).expect("Run npc-melee-roll-parity.ps1");
    assert_eq!(fixture.lines().count(),4096);
    for (case,line) in fixture.lines().enumerate() {
        let f:Vec<_>=line.split('|').collect();let a:Vec<_>=f[0].split(',').collect();
        let int=|i:usize|a[i].parse::<i32>().unwrap();
        let input=Inputs {attack_level:int(0),strength_level:int(1),attack_bonus:int(2),strength_bonus:int(3),defence_level:int(4),defence_bonus:int(5),defence_style:(int(6)!=0).then_some(int(6) as u8),defence_prayer:a[7].parse().unwrap(),defence_armour:a[8].parse().unwrap(),elysian:a[9]=="true",bulwark:a[10]=="true",justiciar:a[11]=="true"};
        let mut replay=Replay(f[4].split(',').map(|s|{let(b,v)=s.split_once(':').unwrap();(b.parse().unwrap(),v.parse().unwrap())}).collect());
        let result=input.roll(&mut replay);
        if f[3]=="ERR" {assert!(result.is_err(),"case {case}");} else {
            let result=result.unwrap();assert_eq!(result.chance,f[1].parse::<i32>().unwrap(),"chance {case}");assert_eq!(result.max_hit.unwrap_or(-1),f[2].parse::<i32>().unwrap(),"max {case}");assert_eq!(result.damage,f[3].parse::<i32>().unwrap(),"damage {case}");
        }
        assert!(replay.0.is_empty(),"Unconsumed draws {case}");
    }
}
