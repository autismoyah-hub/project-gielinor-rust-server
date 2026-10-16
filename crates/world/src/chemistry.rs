//! Ordinary amulet-of-chemistry state belongs to the character, not the amulet.
//! Wiki Amulet of chemistry revision15338661. Alchemist's amulet is separate.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct State { remaining:u8, stop_on_crumble:bool }
impl Default for State {fn default()->Self {Self{remaining:5,stop_on_crumble:false}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Proc {pub extra_dose:bool,pub crumbled:bool,pub stop:bool}
impl State {
    pub fn restore(remaining:i32,stop:i32)->Option<Self> {
        if !(1..=5).contains(&remaining) || !(0..=1).contains(&stop) {return None;}
        Some(Self{remaining:remaining as u8,stop_on_crumble:stop!=0})
    }
    pub fn remaining(self)->u8 {self.remaining}
    pub fn stop_on_crumble(self)->bool {self.stop_on_crumble}
    pub fn set_stop_on_crumble(&mut self,value:bool) {self.stop_on_crumble=value;}
    pub fn break_amulet(&mut self) {self.remaining=5;}
    /// Caller must validate live equipment, recipe eligibility and the complete
    /// transaction before drawing, and atomically publish equipment/output/state.
    pub fn roll(&mut self,eligible:bool,random:&mut impl crate::npc::RandomSource)->Proc {
        if !eligible || random.below(100)>=5 {return Proc{extra_dose:false,crumbled:false,stop:false};}
        self.remaining-=1;let crumbled=self.remaining==0;
        if crumbled {self.remaining=5;}
        Proc{extra_dose:true,crumbled,stop:crumbled && self.stop_on_crumble}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Draw{value:u32,calls:usize}
    impl crate::npc::RandomSource for Draw {fn below(&mut self,upper:u32)->u32 {assert_eq!(upper,100);self.calls+=1;self.value}}
    #[test]
    fn exact_five_percent_no_ineligible_draw_and_per_character_depletion() {
        for n in 0..100 {let mut state=State::default();let mut rng=Draw{value:n,calls:0};
            assert_eq!(state.roll(false,&mut rng),Proc{extra_dose:false,crumbled:false,stop:false});assert_eq!(rng.calls,0);
            let result=state.roll(true,&mut rng);assert_eq!(result.extra_dose,n<5);assert!(!result.crumbled);assert_eq!(rng.calls,1);assert_eq!(state.remaining(),if n<5 {4}else{5});
        }
        for stop in [false,true] {let mut state=State::default();state.set_stop_on_crumble(stop);let mut rng=Draw{value:0,calls:0};
            for i in 1..=5 {let p=state.roll(true,&mut rng);assert!(p.extra_dose);assert_eq!(p.crumbled,i==5);assert_eq!(p.stop,i==5 && stop);}
            assert_eq!(state.remaining(),5);assert_eq!(state.stop_on_crumble(),stop);
        }
    }
    #[test]
    fn restoration_and_break_preserve_setting_and_reject_invalid_state() {
        for remaining in 1..=5 {for stop in 0..=1 {let mut s=State::restore(remaining,stop).unwrap();s.break_amulet();assert_eq!(s.remaining(),5);assert_eq!(s.stop_on_crumble(),stop==1);}}
        for (n,s) in [(0,0),(6,0),(-1,0),(5,-1),(5,2)] {assert!(State::restore(n,s).is_none());}
    }
}
