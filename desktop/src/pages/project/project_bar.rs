use super::state;
use leptos::*;

#[component]
pub fn ProjectBar() -> impl IntoView {
    let project = expect_context::<state::Project>();

    view! {
        <div class="flex">
            <div>
                <PreviewSelector/>
            </div>
            <div class="grow">{project.properties().name()}</div>
            <div></div>
        </div>
    }
}

#[component]
fn PreviewSelector() -> impl IntoView {
    let workspace_state = expect_context::<state::Workspace>();
    let state = workspace_state.preview.clone();

    let toggle_assets = move |e: _| state.update(|state| state.assets = !state.assets);
    let toggle_analyses = move |e: _| state.update(|state| state.analyses = !state.analyses);
    let toggle_kind = move |e: _| state.update(|state| state.kind = !state.kind);
    let toggle_description =
        move |e: _| state.update(|state| state.description = !state.description);
    let toggle_tags = move |e: _| state.update(|state| state.tags = !state.tags);
    let toggle_metadata = move |e: _| state.update(|state| state.metadata = !state.metadata);

    view! {
        <form>
            <div>
                <label>
                    <input
                        type="checkbox"
                        name="assets"
                        on:input=toggle_assets
                        checked=move || state.with(|state| state.assets)
                    />

                    "Data"
                </label>
            </div>

            <div>
                <label>
                    <input
                        type="checkbox"
                        name="analyses"
                        on:input=toggle_analyses
                        checked=move || state.with(|state| state.analyses)
                    />

                    "Analyses"
                </label>
            </div>

            <div>
                <label>
                    <input
                        type="checkbox"
                        name="kind"
                        on:input=toggle_kind
                        checked=move || state.with(|state| state.kind)
                    />

                    "Type"
                </label>
            </div>

            <div>
                <label>
                    <input
                        type="checkbox"
                        name="description"
                        on:input=toggle_description
                        checked=move || state.with(|state| state.description)
                    />

                    "Description"
                </label>
            </div>

            <div>
                <label>
                    <input
                        type="checkbox"
                        name="tags"
                        on:input=toggle_tags
                        checked=move || state.with(|state| state.tags)
                    />

                    "Tags"
                </label>
            </div>

            <div>
                <label>
                    <input
                        type="checkbox"
                        name="metadata"
                        on:input=toggle_metadata
                        checked=move || state.with(|state| state.metadata)
                    />

                    "Metadata"
                </label>
            </div>

        </form>
    }
}
