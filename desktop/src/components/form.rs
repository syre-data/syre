use leptos::*;
use std::str::FromStr;

/// `<input type="number" ... /> wrapper.
/// Handles `step` and validation UI.
#[component]
pub fn InputNumber(
    /// Read signal.
    /// Attached to `prop:value`.
    #[prop(into)]
    value: Signal<String>,

    #[prop(into)] oninput: Callback<String>,
    #[prop(optional)] min: Option<f64>,
    #[prop(optional)] max: Option<f64>,
    #[prop(optional, into)] placeholder: MaybeProp<String>,
    #[prop(default = false)] required: bool,
    #[prop(optional, into)] class: MaybeProp<String>,
) -> impl IntoView {
    let step = move || {
        value.with(|value| match value.split_once('.') {
            None => {
                let magnitude = value.chars().rev().take_while(|c| *c == '0').count();
                10_f64.powi(magnitude as i32)
            }
            Some((_, decs)) => 10_f64.powi(-(decs.len() as i32)),
        })
    };

    let is_invalid = move || {
        value
            .with(|value| {
                let value = value.trim_start_matches("0");
                serde_json::Number::from_str(value)
            })
            .is_err()
    };

    view! {
        <input
            type="number"
            class:error=is_invalid
            prop:value=value
            on:input=move |e| {
                let v = event_target_value(&e);
                tracing::debug!(?v);
                oninput(v)
            }
            min=min
            max=max
            step=step
            placeholder=placeholder
            class=class
            required=required
        />
    }
}

pub mod debounced {
    use leptos::*;

    #[component]
    pub fn InputText(
        #[prop(into)] value: MaybeSignal<String>,
        #[prop(into)] oninput: Callback<String>,
        #[prop(into)] debounce: MaybeSignal<f64>,
        #[prop(into, optional)] placeholder: MaybeProp<String>,
        #[prop(into, optional)] minlength: MaybeProp<usize>,
        #[prop(optional, into)] class: MaybeProp<String>,
    ) -> impl IntoView {
        let (input_value, set_input_value) = create_signal(value::State::set_from_state(value()));
        let input_value = leptos_use::signal_debounced(input_value, debounce);

        let _ = watch(
            value,
            move |value, _, _| {
                set_input_value(value::State::set_from_state(value.clone()));
            },
            false,
        );

        create_effect(move |_| {
            input_value.with(|value| {
                if value.was_set_from_input() {
                    oninput(value.value().clone());
                }
            })
        });

        view! {
            <input
                prop:value=move || { input_value.with(|value| { value.value().clone() }) }

                on:input=move |e| {
                    let v = event_target_value(&e);
                    set_input_value(value::State::set_from_input(v))
                }

                placeholder=placeholder
                minlength=minlength
                class=class
            />
        }
    }

    #[component]
    pub fn TextArea(
        #[prop(into)] value: MaybeSignal<String>,
        #[prop(into)] oninput: Callback<String>,
        #[prop(into)] debounce: MaybeSignal<f64>,
        #[prop(into, optional)] placeholder: MaybeProp<String>,
        #[prop(into, optional)] class: MaybeProp<String>,
    ) -> impl IntoView {
        let (input_value, set_input_value) = create_signal(value::State::set_from_state(value()));
        let input_value = leptos_use::signal_debounced(input_value, debounce);

        let _ = watch(
            value,
            move |value, _, _| {
                set_input_value(value::State::set_from_state(value.clone()));
            },
            false,
        );

        create_effect(move |_| {
            input_value.with(|value| {
                if value.was_set_from_input() {
                    oninput(value.value().clone());
                }
            })
        });

        // TODO: Update from source does not update value.
        view! {
            <textarea
                on:input=move |e| {
                    let v = event_target_value(&e);
                    set_input_value(value::State::set_from_input(v))
                }

                placeholder=placeholder
                class=class
            >

                {input_value.with(|value| value.value().clone())}
            </textarea>
        }
    }

    pub mod value {
        /// Value and source.
        #[derive(derive_more::Deref, Clone, Debug)]
        pub struct State<T> {
            /// Source of the value.
            source: Source,

            #[deref]
            value: T,
        }

        impl<T> State<T> {
            pub fn set_from_state(value: T) -> Self {
                Self {
                    source: Source::State,
                    value,
                }
            }

            pub fn set_from_input(value: T) -> Self {
                Self {
                    source: Source::Input,
                    value,
                }
            }

            pub fn source(&self) -> &Source {
                &self.source
            }

            pub fn value(&self) -> &T {
                &self.value
            }

            pub fn was_set_from_state(&self) -> bool {
                self.source == Source::State
            }

            pub fn was_set_from_input(&self) -> bool {
                self.source == Source::Input
            }
        }

        /// Source of current value.
        #[derive(PartialEq, Clone, Debug)]
        pub enum Source {
            /// Value state.
            State,

            /// User input.
            Input,
        }
    }
}
