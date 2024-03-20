use egui::Context;

pub mod gui;
pub mod renderer;

pub trait Gui {
    fn run(&mut self, context: &Context);
}
