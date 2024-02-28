//! Excel template builder.
use super::steps::{InputBuilder, OutputBuilder, TemplateBuilder, TemplateReview};
use std::path::PathBuf;
use std::rc::Rc;
use syre_core::project::excel_template::{InputParameters, OutputParameters, TemplateParameters};
use syre_core::project::{AssetProperties, ExcelTemplate};
use syre_core::system::template;
use syre_core::types::ResourceId;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExcelTemplateBuilderProps {
    pub path: PathBuf,
    pub oncreate: Callback<ExcelTemplate>,
}

#[function_component(ExcelTemplateBuilder)]
pub fn excel_template_builder(props: &ExcelTemplateBuilderProps) -> Html {
    let builder = use_reducer(|| TemplateState::new());
    let template_node_ref = use_node_ref();
    let input_node_ref = use_node_ref();
    let output_node_ref = use_node_ref();
    let review_node_ref = use_node_ref();

    use_effect_with(builder.step.clone(), {
        let template_node_ref = template_node_ref.clone();
        let input_node_ref = input_node_ref.clone();
        let output_node_ref = output_node_ref.clone();
        let review_node_ref = review_node_ref.clone();

        move |step| {
            let node_ref = match step {
                BuilderStep::Template => template_node_ref,
                BuilderStep::Input => input_node_ref,
                BuilderStep::Output => output_node_ref,
                BuilderStep::Review => review_node_ref,
            };

            let elm = node_ref
                .cast::<web_sys::HtmlElement>()
                .expect("could not cast node ref as element");

            elm.scroll_into_view();
        }
    });

    let onsubmit_template = use_callback((), {
        let builder = builder.dispatcher();
        move |template, _| {
            builder.dispatch(TemplateStateAction::SetTemplate(template));
        }
    });

    let onsubmit_input = use_callback((), {
        let builder = builder.dispatcher();
        move |input, _| {
            builder.dispatch(TemplateStateAction::SetInput(input));
        }
    });

    let onsubmit_output = use_callback((), {
        let builder = builder.dispatcher();
        move |(path, properties), _| {
            builder.dispatch(TemplateStateAction::SetOutput { path, properties });
        }
    });

    let create_template = use_callback(
        (builder.clone(), props.oncreate.clone()),
        move |_, (builder, oncreate)| {
            let Some(template) = builder.template.as_ref() else {
                builder.dispatch(TemplateStateAction::SetStep(BuilderStep::Template));
                return;
            };

            let Some(input) = builder.input.as_ref() else {
                builder.dispatch(TemplateStateAction::SetStep(BuilderStep::Input));
                return;
            };

            let Some(output) = builder.output.as_ref() else {
                builder.dispatch(TemplateStateAction::SetStep(BuilderStep::Output));
                return;
            };

            let template = ExcelTemplate {
                rid: ResourceId::new(),
                name: None,
                description: None,
                template: template.clone(),
                input: input.clone(),
                output: output.clone(),
            };

            oncreate.emit(template);
        },
    );

    let mut template_step_class = classes!("builder-step", "excel-template", "flex");
    let mut input_step_class = classes!("builder-step", "input-data");
    let mut output_step_class = classes!("builder-step", "output-asset");
    let mut review_step_class = classes!("builder-step", "review");

    match builder.step {
        BuilderStep::Template => template_step_class.push("active"),
        BuilderStep::Input => input_step_class.push("active"),
        BuilderStep::Output => output_step_class.push("active"),
        BuilderStep::Review => review_step_class.push("active"),
    }

    html! {
        <div class={"excel-template-builder"}>
            <div ref={template_node_ref}
                class={template_step_class}>

                <TemplateBuilder
                    path={props.path.clone()}
                    onsubmit={onsubmit_template}
                    template={builder.template.clone()} />
            </div>

            <div ref={input_node_ref}
                class={input_step_class}>

                <InputBuilder
                    input={builder.input.clone()}
                    onsubmit={onsubmit_input} />
            </div>

            <div ref={output_node_ref}
                class={output_step_class}>

                <OutputBuilder
                    path={builder.output.clone().map(|output| output.path)}
                    properties={builder.output.clone().map(|output| output.properties)}
                    onsubmit={onsubmit_output} />
            </div>

            <div ref={review_node_ref}
                class={review_step_class}>

                <TemplateReview onaccept={create_template} />
            </div>
        </div>
    }
}

#[derive(PartialEq, Clone, Default, Debug)]
pub enum BuilderStep {
    #[default]
    Template,
    Input,
    Output,
    Review,
}

pub enum TemplateStateAction {
    SetStep(BuilderStep),
    SetTemplate(TemplateParameters),
    SetInput(InputParameters),
    SetOutput {
        path: PathBuf,
        properties: AssetProperties,
    },
}

#[derive(PartialEq, Clone, Debug)]
struct TemplateState {
    pub step: BuilderStep,
    pub template: Option<TemplateParameters>,
    pub input: Option<InputParameters>,
    pub output: Option<OutputParameters>,
}

impl TemplateState {
    pub fn new() -> Self {
        Self {
            step: BuilderStep::default(),
            template: None,
            input: None,
            output: None,
        }
    }
}

impl Reducible for TemplateState {
    type Action = TemplateStateAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            TemplateStateAction::SetStep(step) => current.step = step,

            TemplateStateAction::SetTemplate(template) => {
                current.template = Some(template);
                current.step = BuilderStep::Input;
            }

            TemplateStateAction::SetInput(params) => {
                current.input = Some(params);
                current.step = BuilderStep::Output;
            }

            TemplateStateAction::SetOutput { path, properties } => {
                current.output = Some(OutputParameters { path, properties });
                current.step = BuilderStep::Review;
            }
        }

        current.into()
    }
}
