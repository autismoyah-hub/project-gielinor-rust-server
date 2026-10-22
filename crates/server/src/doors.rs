//! Resolve the generic single/double door script registrations from packed types.
use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, symbols::Symbols,
};
use gielinor_world::doors::{Catalog, Door, Kind};
use std::collections::BTreeMap;
pub fn resolve(
    content: &Symbols,
    params: &Symbols,
    parameter_types: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<Catalog, String> {
    let symbol = |names: &Symbols, name: &str| {
        names
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing door symbol {name}"))
    };
    let mut groups = BTreeMap::new();
    for (name, kind) in [
        ("closed_single_door", Kind::ClosedSingle),
        ("opened_single_door", Kind::OpenedSingle),
        ("closed_left_door", Kind::ClosedLeft),
        ("closed_right_door", Kind::ClosedRight),
        ("opened_left_door", Kind::OpenedLeft),
        ("opened_right_door", Kind::OpenedRight),
    ] {
        groups.insert(symbol(content, name)?, kind);
    }
    let next = symbol(params, "next_loc_stage")? as u32;
    let open = symbol(params, "opensound")? as u32;
    let close = symbol(params, "closesound")? as u32;
    let mut result = Catalog::new();
    for (&id, ty) in &defs.locations {
        let Some(&kind) = groups.get(&ty.content_group) else {
            continue;
        };
        // OpLocHandler checks the type's operation before launching a script.
        if ty.ops[0].is_none() {
            continue;
        }
        let int = |key: u32| -> Result<i32, String> {
            match ty.params.get(&key) {
                Some(Param::Integer(v)) => Ok(*v),
                Some(_) => Err(format!("Noninteger door parameter {key} on {id}")),
                None => parameter_types
                    .get(&key)
                    .and_then(|p| p.default_int)
                    .ok_or_else(|| format!("Missing door parameter {key} on {id}")),
            }
        };
        let next =
            u32::try_from(int(next)?).map_err(|_| format!("Invalid next door stage on {id}"))?;
        if !defs.locations.contains_key(&next) {
            return Err(format!("Unknown door stage {next} on {id}"));
        }
        // DoubleDoorScript uses opensound even for closing both halves.
        let sound = u16::try_from(int(if kind == Kind::OpenedSingle {
            close
        } else {
            open
        })?)
        .map_err(|_| format!("Invalid door sound on {id}"))?;
        result.insert(id, Door { kind, next, sound });
    }
    Ok(result)
}

/// Attested target239.1 families; no quest or unrelated gate handler is inferred.
pub fn resolve_picket(content:&Symbols,params:&Symbols,locations:&Symbols,defs:&Definitions)->Result<Catalog,String> {
    for(name,id)in [("closed_left_picketgate",27),("closed_right_picketgate",28),("opened_left_picketgate",29),("opened_right_picketgate",30)] {
        if content.get(name)!=Some(&id){return Err(format!("Unattested picket content {name}"));}
    }
    for(name,id)in [("next_loc_stage",65533),("opensound",65534),("closesound",65532)] {
        if params.get(name)!=Some(&id){return Err(format!("Unattested picket parameter {name}"));}
    }
    let mut result=Catalog::new();
    for (closed,opened,closed_name,opened_name,left,close_sound) in [
        (1558,1559,"fencegate_l","openfencegate_l",true,66),
        (1560,1567,"fencegate_r","openfencegate_r",false,66),
        (12986,12988,"qip_sheep_shearer_fencegate_l","qip_sheep_shearer_openfencegate_l",true,60),
        (12987,12989,"qip_sheep_shearer_fencegate_r","qip_sheep_shearer_openfencegate_r",false,60),
        (883,23918,"rustic_fencegate_l","rustic_openfencegate_l",true,60),
        (23917,23919,"rustic_fencegate_r","rustic_openfencegate_r",false,60),
    ] {
        if locations.get(closed_name)!=Some(&(closed as i32)) || locations.get(opened_name)!=Some(&(opened as i32)){return Err("Unattested picket location symbol".into());}
        for (id,next,kind,group,op,sound_key,sound) in [
            (closed,opened,if left{Kind::GateClosedLeft}else{Kind::GateClosedRight},if left{27}else{28},"Open",65534,67),
            (opened,closed,if left{Kind::GateOpenedLeft}else{Kind::GateOpenedRight},if left{29}else{30},"Close",65532,close_sound),
        ] {
            let ty=defs.locations.get(&id).ok_or("Missing picket stage")?;
            if ty.content_group!=group || ty.name!="Gate" || ty.ops[0].as_deref()!=Some(op) || ty.ops[1..].iter().any(Option::is_some)
                || ty.collision.width!=1 || ty.collision.length!=1 || !ty.multi.is_empty() || ty.multi_default!=-1 || ty.multi_varbit!=-1 || ty.multi_varp!=-1
                || !matches!(ty.params.get(&65533),Some(Param::Integer(v))if *v==next as i32)
                || !matches!(ty.params.get(&sound_key),Some(Param::Integer(v))if *v==sound as i32) {return Err(format!("Unattested picket stage {id}"));}
            result.insert(id,Door{kind,next,sound});
        }
    }
    Ok(result)
}

#[cfg(test)]
mod picket_tests {
    use super::*;
    fn fixture()->(Symbols,Symbols,Symbols,Definitions) {
        let content=[("closed_left_picketgate".into(),27),("closed_right_picketgate".into(),28),("opened_left_picketgate".into(),29),("opened_right_picketgate".into(),30)].into();
        let params=[("next_loc_stage".into(),65533),("opensound".into(),65534),("closesound".into(),65532)].into();
        let mut loc=Symbols::new();
        let mut d=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for(c,o,cn,on,left,sound)in [(1558,1559,"fencegate_l","openfencegate_l",true,66),(1560,1567,"fencegate_r","openfencegate_r",false,66),(12986,12988,"qip_sheep_shearer_fencegate_l","qip_sheep_shearer_openfencegate_l",true,60),(12987,12989,"qip_sheep_shearer_fencegate_r","qip_sheep_shearer_openfencegate_r",false,60),(883,23918,"rustic_fencegate_l","rustic_openfencegate_l",true,60),(23917,23919,"rustic_fencegate_r","rustic_openfencegate_r",false,60)] {
            loc.insert(cn.into(),c);loc.insert(on.into(),o);
            for(id,next,group,op,key,s)in [(c,o,if left{27}else{28},"Open",65534,67),(o,c,if left{29}else{30},"Close",65532,sound)] {
                let mut ty=gielinor_cache::locations::LocationType::decode(&[0]).unwrap();ty.name="Gate".into();ty.content_group=group;ty.ops[0]=Some(op.into());ty.params.insert(65533,Param::Integer(next));ty.params.insert(key,Param::Integer(s));d.locations.insert(id as u32,ty);
            }
        }
        (content,params,loc,d)
    }
    #[test]
    fn picket_binding_all_twelve_stages_and_late_profile_drift_are_checked() {
        let(c,p,l,d)=fixture();let result=resolve_picket(&c,&p,&l,&d).unwrap();assert_eq!(result.len(),12);
        assert_eq!(result[&12989].sound,60);assert_eq!(result[&1567].sound,66);
        for mode in 0..10 {
            let(mut c,mut p,mut l,mut d)=fixture();
            match mode {
                0=>{c.remove("opened_right_picketgate");},
                1=>{p.insert("opensound".into(),65532);},
                2=>{l.insert("rustic_openfencegate_r".into(),23920);},
                3=>{d.locations.remove(&23919);},
                4=>d.locations.get_mut(&23919).unwrap().ops[0]=Some("Open".into()),
                5=>{d.locations.get_mut(&23919).unwrap().params.insert(65533,Param::Integer(1558));},
                6=>{d.locations.get_mut(&23919).unwrap().params.remove(&65532);},
                7=>d.locations.get_mut(&23919).unwrap().multi_varbit=4153,
                8=>d.locations.get_mut(&23919).unwrap().collision.width=2,
                _=>d.locations.get_mut(&23919).unwrap().content_group=29,
            }
            assert!(resolve_picket(&c,&p,&l,&d).is_err(),"late drift {mode}");
        }
    }
}
