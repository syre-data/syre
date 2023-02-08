//! Step funnel.
use std::rc::Rc;
use yew::prelude::*;

// *******************
// *** Funnel Step ***
// *******************

/// State of the funnel step.
#[derive(Debug, Clone, PartialEq)]
pub struct FunnelStepState {
    /// Step index within the curent funnel.
    pub index: usize,

    /// If the step is currently active.
    pub active: bool,
}

/// Properties for a [`FunnelStep`]
#[derive(PartialEq, Properties)]
struct FunnelStepProps {
    /// Step index within the curent funnel.
    pub index: usize,
    children: Children,
}

/// A step within a [`Funnel`].
#[function_component(FunnelStep)]
fn funnel_step(props: &FunnelStepProps) -> Html {
    let funnel_state = use_context::<FunnelReducer>().expect("no context found");

    let active = funnel_state.active_step == props.index;
    let step_state = use_state(|| FunnelStepState {
        index: props.index,
        active,
    });

    {
        // update active state based on funnel
        let funnel_state = funnel_state.clone();
        let step_state = step_state.clone();
        let index = props.index.clone();

        use_effect_with_deps(
            move |funnel_state| {
                let active = funnel_state.active_step == index;
                step_state.set(FunnelStepState { index, active });

                || {}
            },
            funnel_state,
        );
    }

    html! {
        <ContextProvider<FunnelStepState> context={(*step_state).clone()}>
            { for props.children.iter() }
        </ContextProvider<FunnelStepState>>

    }
}

// **************
// *** Funnel ***
// **************

/// Actions for [`Funnel`].
pub enum FunnelAction {
    /// Go to next step, if available.
    Next,

    /// Go to previous step, if avialble.
    Previous,

    /// Go to provided step, clamping at the first and last step.
    GoTo(usize),
}

/// State of a [`Funnel`]
#[derive(Debug, Clone, PartialEq)]
pub struct FunnelState {
    /// Total number of steps in the funnel.
    n_steps: usize,

    /// Currently active step.
    active_step: usize,

    /// Whether each step has been visited or not.
    visited: Vec<bool>,
}

impl FunnelState {
    fn new(n_steps: usize) -> Self {
        Self {
            n_steps,
            active_step: 0,
            visited: vec![false; n_steps],
        }
    }
}

impl Default for FunnelState {
    fn default() -> Self {
        Self {
            n_steps: 0,
            active_step: 0,
            visited: Vec::new(),
        }
    }
}

impl Reducible for FunnelState {
    type Action = FunnelAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            FunnelAction::Next => {
                let mut active_step = self.active_step;
                if active_step < self.n_steps - 1 {
                    active_step += 1;
                }

                let mut visited = self.visited.clone();
                visited[active_step] = true;

                Self {
                    active_step,
                    visited,
                    ..(*self)
                }
            }
            FunnelAction::Previous => {
                let mut active_step = self.active_step;
                if active_step > 0 {
                    active_step -= 1;
                }

                let mut visited = self.visited.clone();
                visited[active_step] = true;

                Self {
                    active_step,
                    visited,
                    ..(*self)
                }
            }
            FunnelAction::GoTo(step) => {
                let mut active_step = self.active_step;
                if active_step > self.n_steps - 1 {
                    active_step = self.n_steps - 1;
                }

                let mut visited = self.visited.clone();
                visited[active_step] = true;

                Self {
                    active_step: step,
                    visited,
                    ..(*self)
                }
            }
        }
        .into()
    }
}

pub type FunnelReducer = UseReducerHandle<FunnelState>;

#[derive(PartialEq, Properties)]
pub struct FunnelProps {
    #[prop_or_default]
    pub class: Classes,
    pub children: Children,
}

/// A funnel is a set of steps.
#[function_component(Funnel)]
pub fn funnel(props: &FunnelProps) -> Html {
    let funnel_state = use_reducer(|| FunnelState::new(props.children.len()));
    html! {
        <ContextProvider<FunnelReducer> context={funnel_state}>
            <div class={classes!(props.class.clone())}>
            {
                props.children.iter().enumerate().map(|(index, child)| {
                    html_nested!{ <FunnelStep {index}>{ child }</FunnelStep> }
                }).collect::<Html>()
            }
            </div>
        </ContextProvider<FunnelReducer>>
    }
}

#[cfg(test)]
#[path = "./funnel_test.rs"]
mod funnel_test;
