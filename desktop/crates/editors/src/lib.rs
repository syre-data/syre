pub mod common;
pub mod project;

pub mod types {
    use leptos::prelude::Signal;

    #[derive(derive_more::Deref, Clone, Copy)]
    pub struct InputDebounce(Signal<f64>);
    impl InputDebounce {
        pub fn new(signal: Signal<f64>) -> Self {
            Self(signal)
        }
    }
}
