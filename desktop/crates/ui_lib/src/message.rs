use leptos::{
    prelude::*,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum MessageKind {
    Success,
    Warning,
    Error,
    Info,
}

// NOTE: Can use [`enum_dispatch` crate](https://crates.io/crates/enum_dispatch) here
// if manually maintinaing becomes obnoxious.

#[derive(Copy, Clone)]
pub struct NoBody;

#[derive(derive_more::Deref, Clone)]
pub struct Body<B>(B);
pub struct Builder<B> {
    title: String,
    kind: MessageKind,
    body: B,
}

impl Builder<NoBody> {
    fn new(title: impl Into<String>, kind: MessageKind) -> Self {
        Self {
            title: title.into(),
            kind,
            body: NoBody,
        }
    }

    pub fn success(title: impl Into<String>) -> Self {
        Self::new(title, MessageKind::Success)
    }

    pub fn warning(title: impl Into<String>) -> Self {
        Self::new(title, MessageKind::Warning)
    }

    pub fn error(title: impl Into<String>) -> Self {
        Self::new(title, MessageKind::Error)
    }

    pub fn info(title: impl Into<String>) -> Self {
        Self::new(title, MessageKind::Info)
    }

    pub fn body<B>(self, body: B) -> Builder<Body<B>>
    where
        B: AsAnyView,
    {
        Builder {
            title: self.title,
            kind: self.kind,
            body: Body(body),
        }
    }
}

impl Builder<NoBody> {
    pub fn build(self) -> Message<NoBody> {
        self.into()
    }
}

// TODO: Could ideally unify the `build` and `build_*` methods
// into a trait using specialization.
// I couldn't get this working on initial attempts, though.
impl Builder<Body<String>> {
    pub fn build_str(self) -> Message<Body<String>> {
        self.into()
    }
}

impl<B> Builder<Body<B>>
where
    B: AsAnyView,
{
    /// # Note
    /// For `String` bodies use `buid_str`.
    pub fn build(self) -> Message<Body<Arc<B>>> {
        self.into()
    }
}

impl Into<Message<NoBody>> for Builder<NoBody> {
    fn into(self) -> Message<NoBody> {
        let id = (js_sys::Math::random() * (usize::MAX as f64)) as usize;
        Message {
            id,
            kind: self.kind,
            title: self.title,
            body: self.body,
        }
    }
}

impl Into<Message<Body<String>>> for Builder<Body<String>> {
    fn into(self) -> Message<Body<String>> {
        let id = (js_sys::Math::random() * (usize::MAX as f64)) as usize;
        Message {
            id,
            kind: self.kind,
            title: self.title,
            body: self.body,
        }
    }
}

impl<B> Into<Message<Body<Arc<B>>>> for Builder<Body<B>>
where
    B: AsAnyView,
{
    fn into(self) -> Message<Body<Arc<B>>> {
        let id = (js_sys::Math::random() * (usize::MAX as f64)) as usize;
        let Body(body) = self.body;
        Message {
            id,
            kind: self.kind,
            title: self.title,
            body: Body(Arc::new(body)),
        }
    }
}

#[derive(Clone)]
pub struct Message<B> {
    id: usize,
    kind: MessageKind,
    title: String,
    body: B,
}

impl<B> Message<B> {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn kind(&self) -> MessageKind {
        self.kind
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn body(&self) -> &B {
        &self.body
    }
}

#[derive(derive_more::From, Clone)]
pub enum MessageContainer {
    NoBody(Message<NoBody>),
    String(Message<Body<String>>),

    #[from(skip)]
    AnyView(Message<Body<Arc<dyn AsAnyView>>>),
}

impl MessageContainer {
    pub fn id(&self) -> usize {
        match self {
            Self::NoBody(message) => message.id(),
            Self::String(message) => message.id(),
            Self::AnyView(message) => message.id(),
        }
    }
}

impl<B> From<Message<Body<Arc<B>>>> for MessageContainer
where
    B: AsAnyView + 'static,
{
    fn from(message: Message<Body<Arc<B>>>) -> MessageContainer {
        let Message {
            id,
            kind,
            title,
            body: Body(body),
        } = message;

        MessageContainer::AnyView(Message {
            id,
            kind,
            title,
            body: Body(body as Arc<dyn AsAnyView>),
        })
    }
}

/// App wide messages.
#[derive(Clone, Copy, derive_more::Deref)]
pub struct Messages(RwSignal<Vec<MessageContainer>, LocalStorage>);
impl Messages {
    pub fn new() -> Self {
        Self(RwSignal::new_local(vec![]))
    }

    pub fn push_message(&self, message: impl Into<MessageContainer>) {
        self.0.write().push(message.into());
    }
}

pub trait AsAnyView {
    fn as_any_view(&self) -> AnyView;
}

impl<T> AsAnyView for T
where
    T: IntoAny + Clone,
{
    fn as_any_view(&self) -> AnyView {
        self.clone().into_any()
    }
}
