use signals::{HandleState, Object};

#[derive(Clone)]
pub enum CounterMessage {
    ValueChanged(i32),
}

pub struct CounterSender {
    handle: HandleState<Counter>,
}

impl From<HandleState<Counter>> for CounterSender {
    fn from(value: HandleState<Counter>) -> Self {
        Self { handle: value }
    }
}

impl CounterSender {
    pub fn set(&self, value: i32) {
        self.handle.update(move |me| {
            me.value = value;
        });
    }
}

#[derive(Default)]
pub struct Counter {
    value: i32,
}

impl Object for Counter {
    type Message = CounterMessage;
    type Sender = CounterSender;

    fn handle(&mut self, _msg: Self::Message) {}
}

fn main() {
    let a = Counter::default().spawn();
    let b = Counter::default().spawn();

    a.bind(&b, |msg| msg.clone());

    a.set(2);
}
