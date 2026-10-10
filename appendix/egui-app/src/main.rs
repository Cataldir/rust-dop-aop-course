struct Native {model:desktop_core::Model,error:String}
impl eframe::App for Native {
    fn update(&mut self,ctx:&eframe::egui::Context,_:&mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx,|ui|{
            ui.heading("Rust-owned panel state");
            let revision=self.model.view().revision;
            if ui.button("Add CPU panel").clicked() {
                if let Err(e)=self.model.dispatch(desktop_core::Command::Add{expected_revision:revision,id:"cpu".into(),name:"CPU".into()}){self.error=e.to_string();}
            }
            if ui.button("Undo").clicked() {
                if let Err(e)=self.model.dispatch(desktop_core::Command::Undo{expected_revision:revision}){self.error=e.to_string();}
            }
            for name in self.model.enabled_names(){ui.label(name);}ui.label(&self.error);
        });
    }
}
fn main()->eframe::Result<()> {
    let store=sled::open("learner-snapshot").expect("open learner-only snapshot");
    let model=desktop_core::Model::open(store).expect("validate snapshot");
    eframe::run_native("Course egui appendix",eframe::NativeOptions::default(),Box::new(move|_|Ok(Box::new(Native{model,error:String::new()}))))
}