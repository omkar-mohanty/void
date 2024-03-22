use egui::Context;
use void_core::*;
use void_native::{MpscReceiver, MpscSender, NativeEvent};

use crate::{Gui, GuiEngine, GuiEvent};

impl<S> System
    for GuiEngine<MpscSender<GuiEvent>, MpscReceiver<NativeEvent>, GuiEvent, NativeEvent, S>
where
    S: Gui,
{
    type Sender = MpscSender<GuiEvent>;
    type Receiver = MpscReceiver<NativeEvent>;
    type EventUp = GuiEvent;
    type EventDown = NativeEvent;

    async fn run(&mut self, func: impl FnOnce(Self::EventDown) -> Self::EventUp) -> Result<()> {
        let event = self.event_receiver.receieve_event().await.unwrap();
        let gui_event = func(event);
        self.handle_event(gui_event).await;
        Ok(())
    }
}

impl<S: Gui> GuiEngine<MpscSender<GuiEvent>, MpscReceiver<NativeEvent>, GuiEvent, NativeEvent, S> {
    pub fn new(
        context: Context,
        event_sender: MpscSender<GuiEvent>,
        event_receiver: MpscReceiver<NativeEvent>,
        state: S,
    ) -> Self {
        Self {
            context,
            event_sender,
            event_receiver,
            state,
        }
    }

    async fn handle_event(&mut self, gui_event: GuiEvent) {
        use GuiEvent::*;
        match gui_event {
            Input(input) => {
                let output = self.update(input);
                self.event_sender
                    .send_event(GuiEvent::Output(output))
                    .await
                    .unwrap();
            }
            Output(_) => {}
        };
    }
}
