use egui::{Align2, Context};

use crate::integration::Controller;

pub fn nullus_gui(ctx: &Context, controller: &dyn Controller) {
    egui::Window::new("Control Plane")
        .default_open(true)
        .resizable(true)
        .anchor(Align2::LEFT_TOP, [0.0, 0.0])
        .show(ctx, |ui| {
            if ui.add(egui::Button::new("Clicked")).clicked() {
                println!("Clicked");
            }
        });
    controller.process_events(ctx);
}
