use slotmap::{DefaultKey, SlotMap};
use std::{any::Any, cell::RefCell, marker::PhantomData, ops::Deref, rc::Rc};

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

    pub fn bind<O2: Object>(
        &self,
        _other: &Handle<O2>,
        _f: impl FnMut(&O::Message) -> O2::Message,
    ) {
    }
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

    fn handle(&mut self, msg: Self::Message);

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

    fn handle_any(&mut self, msg: Box<dyn Any>);
}

impl<O> AnyObject for O
where
    O: Object + 'static,
    O::Message: 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_any(&mut self, msg: Box<dyn Any>) {
        self.handle(*msg.downcast().unwrap())
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
