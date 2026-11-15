/// Shares a Vec of events between the output port and the test via Rc<RefCell>,
/// so the test can verify the port received the event.
use instinct_core::event::{Event, WorldOutput};
use std::cell::RefCell;
use std::rc::Rc;

pub struct SharedOutput {
    pub events: Rc<RefCell<Vec<Event>>>,
}

impl WorldOutput for SharedOutput {
    fn emit(&mut self, event: &Event) {
        self.events.borrow_mut().push(event.clone());
    }
}
