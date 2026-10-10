//! Typed desktop command model; sled is only a learner-local snapshot store.
use serde::{Serialize,Deserialize};
use std::collections::BTreeMap;
use thiserror::Error;
#[derive(Debug,Error)] pub enum UiError {
    #[error("invalid panel")] Invalid,
    #[error("stale revision")] Stale,
    #[error("empty history")] EmptyHistory,
    #[error("store outcome unknown: reopen and reconcile")] Indeterminate,
    #[error(transparent)] Storage(#[from]sled::Error),
    #[error(transparent)] Format(#[from]serde_json::Error),
}
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq)] pub struct Panel {pub id:String,pub name:String,pub enabled:bool}
#[derive(Clone,Debug,Default,Serialize,Deserialize,PartialEq)] pub struct View {pub revision:u64,pub panels:BTreeMap<String,Panel>}
#[derive(Clone,Debug,Default,Serialize,Deserialize)] pub struct Saved {pub view:View,pub undo:Vec<BTreeMap<String,Panel>>,pub redo:Vec<BTreeMap<String,Panel>>}
#[derive(Clone,Debug,Serialize,Deserialize)] #[serde(tag="kind",rename_all="snake_case")]
pub enum Command {Add{expected_revision:u64,id:String,name:String},Toggle{expected_revision:u64,id:String},Undo{expected_revision:u64},Redo{expected_revision:u64}}
impl Command {fn revision(&self)->u64 {match self {Self::Add{expected_revision,..}|Self::Toggle{expected_revision,..}|Self::Undo{expected_revision}|Self::Redo{expected_revision}=>*expected_revision}}}
fn validate_panels(panels:&BTreeMap<String,Panel>)->Result<(),UiError> {
    if panels.len()>128 || panels.iter().any(|(id,p)|id!=&p.id||!shared_kernel::valid_id(id)||p.name.trim().is_empty()||p.name.len()>80||p.name.chars().any(char::is_control)){return Err(UiError::Invalid);}Ok(())
}
fn validate_saved(saved:&Saved)->Result<(),UiError> {
    if saved.undo.len()>32||saved.redo.len()>32{return Err(UiError::Invalid);}
    validate_panels(&saved.view.panels)?;
    for panels in saved.undo.iter().chain(&saved.redo){validate_panels(panels)?;}
    Ok(())
}
/// Deterministic proposal; original state is untouched on any refusal.
pub fn propose(saved:&Saved,command:Command)->Result<Saved,UiError> {
    validate_saved(saved)?;
    if command.revision()!=saved.view.revision{return Err(UiError::Stale);}
    let mut next=saved.clone();
    match command {
        Command::Add{id,name,..}=>{
            if next.view.panels.contains_key(&id){return Err(UiError::Invalid);}
            next.undo.push(next.view.panels.clone());next.redo.clear();
            next.view.panels.insert(id.clone(),Panel{id,name,enabled:true});
        }
        Command::Toggle{id,..}=>{
            if !next.view.panels.contains_key(&id){return Err(UiError::Invalid);}
            next.undo.push(next.view.panels.clone());next.redo.clear();
            if let Some(panel)=next.view.panels.get_mut(&id){panel.enabled=!panel.enabled;}
        }
        Command::Undo{..}=>{let previous=next.undo.pop().ok_or(UiError::EmptyHistory)?;next.redo.push(next.view.panels.clone());next.view.panels=previous;}
        Command::Redo{..}=>{let following=next.redo.pop().ok_or(UiError::EmptyHistory)?;next.undo.push(next.view.panels.clone());next.view.panels=following;}
    }
    validate_panels(&next.view.panels)?;
    next.view.revision=next.view.revision.checked_add(1).ok_or(UiError::Invalid)?;
    if next.undo.len()>32{next.undo.remove(0);}if next.redo.len()>32{next.redo.remove(0);}
    Ok(next)
}
pub struct Model {saved:Saved,world:hecs::World,store:sled::Db,bytes:Option<Vec<u8>>,unknown:bool}
impl Model {
    pub fn open(store:sled::Db)->Result<Self,UiError> {
        let bytes=store.get("course-ui-v1")?.map(|b|b.to_vec());
        if bytes.as_ref().is_some_and(|b|b.len()>4*1024*1024){return Err(UiError::Invalid);}
        let saved=match &bytes {Some(b)=>serde_json::from_slice::<Saved>(b)?,None=>Saved::default()};
        validate_saved(&saved)?;
        let world=world_from(&saved.view);
        Ok(Self{saved,world,store,bytes,unknown:false})
    }
    pub fn view(&self)->&View {&self.saved.view}
    pub fn enabled_names(&self)->Vec<String> {
        let mut names:Vec<_>=self.world.query::<&Panel>().iter().filter(|(_,p)|p.enabled).map(|(_,p)|p.name.clone()).collect();names.sort();names
    }
    pub fn dispatch(&mut self,command:Command)->Result<View,UiError> {
        if self.unknown{return Err(UiError::Indeterminate);}
        let next=propose(&self.saved,command)?;let encoded=serde_json::to_vec(&next)?;
        let swapped=self.store.compare_and_swap("course-ui-v1",self.bytes.as_deref(),Some(encoded.as_slice()))?;
        if swapped.is_err(){return Err(UiError::Stale);}
        // CAS changed the store. A failed flush means uncertain durability; further edits refuse.
        if self.store.flush().is_err(){self.unknown=true;return Err(UiError::Indeterminate);}
        self.bytes=Some(encoded);self.world=world_from(&next.view);self.saved=next;Ok(self.saved.view.clone())
    }
}
fn world_from(view:&View)->hecs::World {let mut world=hecs::World::new();for panel in view.panels.values(){world.spawn((panel.clone(),));}world}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn failed_proposal_preserves_state() {let original=Saved::default();assert!(matches!(propose(&original,Command::Toggle{expected_revision:0,id:"missing".into()}),Err(UiError::Invalid)));assert_eq!(original.view.revision,0);}
    #[test] fn invalid_input_history_is_refused_before_proposal() {
        let mut saved=Saved::default();let mut invalid=BTreeMap::new();
        invalid.insert("x".into(),Panel{id:"different".into(),name:"X".into(),enabled:true});saved.undo.push(invalid);
        assert!(matches!(propose(&saved,Command::Add{expected_revision:0,id:"a".into(),name:"A".into()}),Err(UiError::Invalid)));
        assert_eq!(saved.view.revision,0);
    }
    #[test] fn undo_redo_and_stale_revision_are_explicit() {
        let add=propose(&Saved::default(),Command::Add{expected_revision:0,id:"cpu".into(),name:"CPU".into()}).unwrap();
        let undo=propose(&add,Command::Undo{expected_revision:1}).unwrap();assert!(undo.view.panels.is_empty());assert_eq!(undo.view.revision,2);
        let redo=propose(&undo,Command::Redo{expected_revision:2}).unwrap();assert_eq!(redo.view.panels.len(),1);
        assert!(matches!(propose(&redo,Command::Toggle{expected_revision:1,id:"cpu".into()}),Err(UiError::Stale)));
    }
    #[test] fn snapshot_reopens_with_history() {
        let db=sled::Config::new().temporary(true).open().unwrap();
        let mut model=Model::open(db.clone()).unwrap();model.dispatch(Command::Add{expected_revision:0,id:"cpu".into(),name:"CPU".into()}).unwrap();
        drop(model);let mut restored=Model::open(db).unwrap();assert_eq!(restored.enabled_names(),vec!["CPU"]);restored.dispatch(Command::Undo{expected_revision:1}).unwrap();assert!(restored.view().panels.is_empty());
    }
    #[test] fn concurrent_models_refuse_stale_snapshot() {
        let db=sled::Config::new().temporary(true).open().unwrap();let mut a=Model::open(db.clone()).unwrap();let mut b=Model::open(db).unwrap();
        a.dispatch(Command::Add{expected_revision:0,id:"a".into(),name:"A".into()}).unwrap();
        assert!(matches!(b.dispatch(Command::Add{expected_revision:0,id:"b".into(),name:"B".into()}),Err(UiError::Stale)));
    }
}
