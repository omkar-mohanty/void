use anyhow::Ok;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use void_core::{CmdSender, Subject, System};
use void_engine::{GuiEngineSubject, IoEngineSubject, RenderEngineSubject};
use void_io::{IoCmd, IoEngine};
use void_native::{create_mpsc_channel, MpscReceiver, MpscSender};
use void_render::{RenderCmd, RenderEngine};
use void_ui::{GuiCmd, GuiEngine, NativeGui};
use winit::{event_loop::EventLoop, window::WindowBuilder};

type SystemHandle = (UnboundedReceiver<String>, JoinHandle<()>);

pub struct App<'a> {
    io_cmd_sender: MpscSender<IoCmd>,
    io_engine: IoEngine<IoEngineSubject, MpscReceiver<IoCmd>>,
    render_engine: RenderEngine<'a, RenderEngineSubject, MpscReceiver<RenderCmd>>,
    gui_engine: GuiEngine<NativeGui, MpscReceiver<GuiCmd>, GuiEngineSubject>,
    event_loop: EventLoop<()>,
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

    let mut io_engine = IoEngine::new(
        context.clone(),
        Arc::clone(&window),
        io_engine_subject,
        io_cmd_receiver,
    );

    let mut gui_engine = GuiEngine::new(
        context.clone(),
        gui_cmd_receiver,
        gui_engine_subject,
        NativeGui {},
    );

    let mut render_engine = RenderEngine::new(
        Arc::clone(&window),
        context.clone(),
        render_engine_subject,
        render_cmd_receiver,
    )
    .await;

    Ok(App {
        io_engine,
        gui_engine,
        render_engine,
        io_cmd_sender,
        event_loop,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let app = init().await?;

    let App {
        io_cmd_sender,
        mut io_engine,
        mut render_engine,
        mut gui_engine,
        event_loop,
    } = app;
    tokio::spawn(async move {
        if let Err(msg) = render_engine.run().await {
            log::error!("Render : {}", msg);
        }
    });
    tokio::spawn(async move {
        if let Err(msg) = io_engine.run().await {
            log::error!("Io : {}", msg);
        }
    });
    tokio::spawn(async move {
        if let Err(msg) = gui_engine.run().await {
            log::error!("Gui : {}", msg);
        }
    });

    event_loop.run(
        |event, _ewlt| {
            if let Err(msg) = io_cmd_sender.send_blocking(IoCmd::WindowEvent(event)) {}
        },
    )?;
    Ok(())
}
