use signals::{emit, HandleState, Object, UserInterface};
use signals_macros::signal;

#[derive(Debug)]
pub enum CounterMessage {
    ValueChanged(i32),
}

#[derive(Default)]
pub struct Counter {
    value: i32,
}

#[signal(CounterMessage)]
impl Counter {
    pub fn set(&mut self, value: i32) {
        self.value = value;

        emit!(CounterMessage::ValueChanged(1));
    }
}

fn main() {
    let counter = Counter::default().spawn();

    counter.listen(|msg| {
        dbg!(msg);
    });

    counter.set(2);
}
