use egui::{Align2, Context};

pub fn void_gui(ctx: &Context) {
    egui::Window::new("Control Plane")
        .default_open(true)
        .resizable(true)
        .anchor(Align2::LEFT_TOP, [0.0, 0.0])
        .show(ctx, |ui| {
            if ui.add(egui::Button::new("Clicked")).clicked() {
                println!("Clicked");
            }
        });
}
