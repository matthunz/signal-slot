use slotmap::{DefaultKey, SlotMap};
use std::{any::Any, cell::RefCell, marker::PhantomData, ops::Deref, rc::Rc};

pub use signals_macros::signal;

pub struct HandleState<O: Object> {
    key: DefaultKey,
    _marker: PhantomData<O>,
}

impl<O: Object> HandleState<O> {
    pub fn update(&self, mut f: impl FnMut(&mut O) + 'static)
    where
        O: 'static,
    {
        UserInterface::current().inner.borrow_mut().updates.push((
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

impl<O: Object> Handle<O> {
    pub fn update(&self, mut f: impl FnMut(&mut O) + 'static)
    where
        O: 'static,
    {
        UserInterface::current().inner.borrow_mut().updates.push((
            self.key,
            Box::new(move |element| f(element.downcast_mut().unwrap())),
        ))
    }

    pub fn listen(&self, _f: impl FnMut(&O::Message)) {}
}

impl<O: Object> Deref for Handle<O> {
    type Target = O::Sender;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

pub trait Object: Sized {
    type Message;
    type Sender: From<HandleState<Self>>;

    fn emit(&mut self, msg: Self::Message);

    fn spawn(self) -> Handle<Self>
    where
        Self: 'static,
    {
        let key = UserInterface::current()
            .inner
            .borrow_mut()
            .objects
            .insert(Box::new(self));

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
    O::Message: 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
struct Inner {
    objects: SlotMap<DefaultKey, Box<dyn AnyObject>>,
    updates: Vec<(DefaultKey, Box<dyn FnMut(&mut dyn Any)>)>,
}

#[derive(Clone, Default)]
pub struct UserInterface {
    inner: Rc<RefCell<Inner>>,
}

impl UserInterface {
    pub fn current() -> Self {
        thread_local! {
            static CURRENT: RefCell<Option<UserInterface>> = RefCell::default();
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
}

#[macro_export]
macro_rules! emit {
    ($e:expr) => {};
}
