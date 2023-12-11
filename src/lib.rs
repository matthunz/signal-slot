pub use signals_macros::object;
use slotmap::{DefaultKey, SlotMap};
use std::{any::Any, cell::RefCell, marker::PhantomData, mem, ops::Deref, rc::Rc};

pub struct HandleState<O: Object> {
    key: DefaultKey,
    _marker: PhantomData<O>,
}

impl<O: Object> Clone for HandleState<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O: Object> Copy for HandleState<O> {}

impl<O: Object> HandleState<O> {
    pub fn update(&self, mut f: impl FnMut(&mut O) + 'static)
    where
        O: 'static,
    {
        Runtime::current().inner.borrow_mut().updates.push((
            self.key,
            Box::new(move |element| f(element.downcast_mut().unwrap())),
        ))
    }
}

pub struct Handle<O: Object> {
    key: DefaultKey,
    sender: O::Sender,
    _marker: PhantomData<O>,
}

impl<O: Object> Clone for Handle<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O: Object> Copy for Handle<O> {}

impl<O: Object> Handle<O> {
    pub fn update(&self, mut f: impl FnMut(&mut O) + 'static)
    where
        O: 'static,
    {
        Runtime::current().inner.borrow_mut().updates.push((
            self.key,
            Box::new(move |element| f(element.downcast_mut().unwrap())),
        ))
    }
}

impl<O: Object> Deref for Handle<O> {
    type Target = O::Sender;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

pub trait Object: Sized {
    type Sender: From<HandleState<Self>> + Copy;

    fn spawn(self) -> Handle<Self>
    where
        Self: 'static,
    {
        let key = Runtime::current().inner.borrow_mut().nodes.insert(Node {
            object: Rc::new(RefCell::new(self)),
            listeners: Vec::new(),
        });

        Handle {
            key,
            sender: HandleState {
                key,
                _marker: PhantomData,
            }
            .into(),
            _marker: PhantomData,
        }
    }
}

pub trait AnyObject {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<O> AnyObject for O
where
    O: Object + 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct Node {
    object: Rc<RefCell<dyn AnyObject>>,
    listeners: Vec<Box<dyn FnMut(&dyn Any)>>,
}

#[derive(Default)]
struct Inner {
    nodes: SlotMap<DefaultKey, Node>,
    updates: Vec<(DefaultKey, Box<dyn FnMut(&mut dyn Any)>)>,
    message_queue: Vec<(DefaultKey, Box<dyn Any>)>,
    current: Option<DefaultKey>,
}

#[derive(Clone, Default)]
pub struct Runtime {
    inner: Rc<RefCell<Inner>>,
}

impl Runtime {
    pub fn current() -> Self {
        thread_local! {
            static CURRENT: RefCell<Option<Runtime>> = RefCell::default();
        }

        CURRENT
            .try_with(|cell| {
                let mut current = cell.borrow_mut();
                if let Some(ui) = &*current {
                    ui.clone()
                } else {
                    let ui = Self::default();
                    *current = Some(ui.clone());
                    ui
                }
            })
            .unwrap()
    }

    pub fn emit(&self, msg: Box<dyn Any>) {
        let mut me = self.inner.borrow_mut();
        let key = me.current.unwrap();
        me.message_queue.push((key, msg));
    }

    pub fn run(&self) {
        let mut updates = mem::take(&mut self.inner.borrow_mut().updates);
        for (key, f) in &mut updates {
            let object = self.inner.borrow().nodes[*key].object.clone();
            self.inner.borrow_mut().current = Some(*key);
            f(object.borrow_mut().as_any_mut());
            self.inner.borrow_mut().current = None;
        }

        let mut message_queue = mem::take(&mut self.inner.borrow_mut().message_queue);
        for (key, msg) in &mut message_queue {
            for listener in &mut self.inner.borrow_mut().nodes[*key].listeners {
                listener(&**msg);
            }
        }
    }
}

pub struct Signal {}

impl Signal {
    pub fn bind<A, B>(&self, _a: A, _b: B) {}
}
