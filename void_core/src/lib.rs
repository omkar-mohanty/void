pub trait Event: Eq + PartialEq {}

pub trait EventSender {
    type E: Event;

    fn send_event(event: Self::E);
}

pub trait System {
    type E: Event;
    fn process_event(&mut self, event: Self::E);
}

