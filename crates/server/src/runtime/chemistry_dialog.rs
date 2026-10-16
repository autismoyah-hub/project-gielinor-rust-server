use gielinor_world::simulation::Action;
use super::*;
use gielinor_protocol::client_script::{self,Argument};
use gielinor_world::ui::Component;
impl<B:AccountBackend> Runtime<B>{
    pub(super) fn resume_chemistry_pause(&mut self,actor:ActorId,pause:gielinor_protocol::dialog::Pause)->io::Result<()>{
        let Some(active)=self.active.get(&actor) else{return Ok(());};let Some((token,expected))=active.chemistry_dialog else{return Ok(());};
        let component=Component(pause.component);
        if component!=expected || !active.ui.modals.contains_interface(component.interface()){return Ok(());}
        let Some(definition)=self.components.get(&pause.component) else{return Ok(());};
        if !active.ui.events.enabled(component,pause.subcomponent as i32,definition.events,1){return Ok(());}
        self.world.submit(actor,Action::ResumeChemistry{token,choice:pause.subcomponent as i32}).map_err(|e|io::Error::other(format!("{e:?}")))
    }
    pub(super) fn chemistry_dialog_event(&mut self,actor:ActorId,token:Option<u64>,remaining:u8,breaking:bool)->io::Result<()>{
        let Some(connection)=self.connection(actor) else{return Ok(());};
        let presentation=self.drop_presentation.ok_or_else(||io::Error::other("Shared choice presentation missing"))?;
        let Some(token)=token else{
            if self.active.get_mut(&actor).and_then(|a|a.chemistry_dialog.take()).is_some(){self.close_interface(actor,presentation.parent)?;}
            return Ok(());
        };
        self.active.get_mut(&actor).unwrap().chemistry_dialog=None;
        self.set_varbit(actor,presentation.unclamp,1)?;
        let packet=client_script::run(2379,&[]).map_err(|e|io::Error::other(format!("{e:?}")))?;
        self.transport.send(connection,packet.opcode,&packet.payload)?;
        self.open_interface(actor,Some(presentation.parent),presentation.choice_interface,false)?;
        let packet=client_script::run(58,&[Argument::String(if breaking{"Break this amulet?".into()}else{format!("Amulet of chemistry: {remaining} charges remaining")}),Argument::String(if breaking{"Break amulet|Cancel"}else{"Continue when the amulet crumbles|Stop when the amulet crumbles"}.into())]).map_err(|e|io::Error::other(format!("{e:?}")))?;
        self.transport.send(connection,packet.opcode,&packet.payload)?;
        self.set_interface_events(actor,presentation.choice_button,1,2,1)?;
        self.active.get_mut(&actor).unwrap().chemistry_dialog=Some((token,presentation.choice_button));Ok(())
    }
}

// Static worn buttons use nullable child/object fields; the mapped component and
// current equipment remain authoritative. Explicit stale item claims still fail.
pub(super) fn worn_item_claim_matches(button:&gielinor_protocol::buttons::Button,item:i32)->bool {
    button.slot == -1 && (button.object == -1 || button.object == item)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_static_neck_packet_uses_null_item_but_rejects_stale_explicit_claims() {
        let mut button=gielinor_protocol::buttons::button(&[1,131,0,17,255,255,255,255,2]).unwrap();
        assert_eq!((button.component,button.slot,button.object,button.op),((387<<16)|17,-1,-1,2));
        assert!(worn_item_claim_matches(&button,21163));
        button.object=21163;assert!(worn_item_claim_matches(&button,21163));
        button.object=21164;assert!(!worn_item_claim_matches(&button,21163));
        button.object=-1;button.slot=2;assert!(!worn_item_claim_matches(&button,21163));
    }
}
