use egui::{Context, RawInput};

struct GuiEngine {
    context: Context,
}

impl GuiEngine {
    pub fn run_ui(&self,raw_input: RawInput, gui: impl FnOnce(&Context)) {
        let full_output = self.context.run(raw_input, |ui| {
            gui(&self.context)
        });
    }
}
