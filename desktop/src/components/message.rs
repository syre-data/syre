use leptos::*;

pub struct Builder {
    title: String,
    body: Option<View>,
    kind: MessageKind,
}

impl Builder {
    fn new(title: impl Into<String>, kind: MessageKind) -> Self {
        Self {
            title: title.into(),
            body: None,
            kind,
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

    pub fn body(&mut self, body: impl IntoView) -> &mut Self {
        let _ = self.body.insert(body.into_view());
        self
    }

    pub fn build(self) -> Message {
        self.into()
    }
}

impl Into<Message> for Builder {
    fn into(self) -> Message {
        Message {
            title: self.title,
            body: self.body,
            kind: self.kind,
        }
    }
}

pub struct Message {
    title: String,
    body: Option<View>,
    kind: MessageKind,
}

#[component]
pub fn Message(title: String, #[prop(optional)] body: Option<String>) -> impl IntoView {
    view! {
        <div>
            <div>{title}</div>
            <div>{body}</div>
        </div>
    }
}

pub enum MessageKind {
    Success,
    Warning,
    Error,
    Info,
}
