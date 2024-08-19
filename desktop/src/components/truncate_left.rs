use leptos::*;

#[component]
pub fn TruncateLeft(children: Children, #[prop(optional)] class: TextProp) -> impl IntoView {
    let classes = {
        let class = class.clone();
        move || format!("truncate-rtl {}", class.get())
    };

    view! {
        <div class=classes>
            <span class="ltr inline-block">{children()}</span>
        </div>
    }
}
