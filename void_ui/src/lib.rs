use void_core::IGui;

#[derive(Default)]
pub struct VoidUi {}

impl IGui for VoidUi {
    fn show(&mut self, ui: &egui::Context) {
        egui::Window::new("Streamline CFD")
            .default_open(true)
            .resizable(true)
            .show(&ui, |ui| {
                if ui.add(egui::Button::new("Click me")).clicked() {
                    println!("PRESSED")
                }

                ui.label("Slider");
                // ui.add(egui::Slider::new(_, 0..=120).text("age"));
                ui.end_row();

                // proto_scene.egui(ui);
            });
    }
}
