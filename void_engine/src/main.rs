use std::{collections::HashMap, sync::Arc};
use anyhow::Ok;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use void_core::{CmdSender, Subject, System};
use void_engine::{GuiEngineSubject, IoEngineSubject, RenderEngineSubject};
use void_io::{IoCmd, IoEngine};
use void_native::{create_mpsc_channel, MpscSender};
use void_render::RenderEngine;
use void_ui::{GuiEngine, NativeGui};
use winit::{event_loop::EventLoop, window::WindowBuilder};
use tokio::sync::mpsc::unbounded_channel;

type SystemHandle = (UnboundedReceiver<String>, JoinHandle<()>);

pub struct App<'a> {
    io_sender: MpscSender<IoCmd>,
    systems: HashMap<&'a str, SystemHandle>,
    event_loop: EventLoop<()>,
}

impl<'a> App<'a> {
    pub fn run(mut self) -> anyhow::Result<()> {
        self.event_loop.run(move| event, _ewlt| {
            self.io_sender.send_blocking(IoCmd::WindowEvent(event)).unwrap();
            for sys in  self.systems.values_mut() {
                let recv = &mut sys.0;
                if let Some(msg) = recv.blocking_recv() {
                    println!("{}", msg);
                }
            }
        })?;
        Ok(())
    }
}

async fn init<'a>() -> anyhow::Result<App<'a>> {
    use void_engine::gui::*;
    use void_engine::render::*;

    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);
    let context = egui::Context::default();

    let (render_cmd_sender, render_cmd_receiver) = create_mpsc_channel();
    let (gui_cmd_sender, gui_cmd_receiver) = create_mpsc_channel();
    let (io_cmd_sender, io_cmd_receiver) = create_mpsc_channel();

    //Render engine event publisher
    let mut render_engine_subject = RenderEngineSubject::default();
    render_engine_subject.attach(GuiObserver {
        cmd_sender: gui_cmd_sender.clone(),
    });

    // Io Engine event publisher
    let mut io_engine_subject = IoEngineSubject::default();
    io_engine_subject.attach(GuiObserver {
        cmd_sender: gui_cmd_sender.clone(),
    });
    io_engine_subject.attach(RendererObserver {
        cmd_sender: render_cmd_sender.clone(),
    });

    // Gui Engine event publisher
    let mut gui_engine_subject = GuiEngineSubject::default();
    gui_engine_subject.attach(RendererObserver {
        cmd_sender: render_cmd_sender.clone(),
    });

    let mut systems = HashMap::new();

    let (io_handle, io_recv) = unbounded_channel();

    let mut io_engine = IoEngine::new(
        context.clone(),
        Arc::clone(&window),
        io_engine_subject,
        io_cmd_receiver,
    );
    let join_handle = tokio::spawn(async move {
        if let Err(msg) = io_engine.run().await {
            io_handle.send(msg.to_string());
        }
        io_handle.send("Hello".to_string());
    });

    systems.insert("Io", (io_recv, join_handle));

    let (gui_handle, gui_recv) = unbounded_channel();
    let mut gui_engine = GuiEngine::new(
        context.clone(),
        gui_cmd_receiver,
        gui_engine_subject,
        NativeGui {},
    );


    let join_handle = tokio::spawn(async move {
        if let Err(msg) = gui_engine.run().await {
            gui_handle.send(msg.to_string());
        }
        gui_handle.send("Hello".to_string());
    });

    systems.insert("Gui", (gui_recv, join_handle));

    let (render_handle, render_recv) = unbounded_channel();

    let mut render_engine = RenderEngine::new(
        Arc::clone(&window),
        context.clone(),
        render_engine_subject,
        render_cmd_receiver,
    )
    .await;

    let join_handle = tokio::spawn(async move {
        if let Err(msg) = render_engine.run().await {
            render_handle.send(msg.to_string());
        }
        render_handle.send("Hello".to_string());
    });

    systems.insert("Render", (render_recv, join_handle));

    Ok(App {
        io_sender: io_cmd_sender,
        event_loop,
        systems
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = init().await?;

    Ok(())
}
