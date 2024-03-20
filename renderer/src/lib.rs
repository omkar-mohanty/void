use egui::Context;
use egui_wgpu::{wgpu, ScreenDescriptor};
use void_core::System;

pub mod gui;
pub mod renderer;

pub trait Gui {
    fn run(&mut self, context: &Context);
}

pub trait GuiRenderer: System + 'static {
    fn draw(&mut self, raw_input: egui::RawInput, gui: &mut dyn Gui) -> egui::FullOutput;
    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        texture_view: &wgpu::TextureView,
        screen_descriptor: ScreenDescriptor,
        full_output: egui::FullOutput,
    );
}
