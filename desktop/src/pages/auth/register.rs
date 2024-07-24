use leptos::*;
use leptos_router::{use_navigate, *};
use serde::Serialize;
use syre_core::system::User;
use web_sys::{FormData, SubmitEvent};

#[component]
pub fn Register(#[prop(default = true)] login_link: bool) -> impl IntoView {
    let navigate = use_navigate();
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(None);
    let form_ref = NodeRef::new();
    let register_user = {
        let navigate = navigate.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            set_loading(true);
            set_error(None);

            let data = FormData::new_with_form(&form_ref.get().unwrap()).unwrap();
            let name = data.get("name").as_string().unwrap();
            let email = data.get("email").as_string().unwrap();

            let name = if name.trim().is_empty() {
                None
            } else {
                Some(name.trim().to_string())
            };

            if email.trim().is_empty() {
                set_error(Some("Email is required.".to_string()));
                return;
            }
            let email = email.trim().to_string();

            let navigate = navigate.clone();
            spawn_local(async move {
                match register(email, name).await {
                    Ok(_user) => {
                        navigate("/", Default::default());
                    }

                    Err(err) => {
                        set_error(Some(err));
                        set_loading(false);
                    }
                }
            });
        }
    };

    view! {
        <h1>"Sign up"</h1>
        <form node_ref=form_ref on:submit=register_user>
            <div>
                <label>"Email"</label>
                <input name="email" type="email" required autofocus/>
            </div>
            <div>
                <label>"Name"</label>
                <input name="name"/>
            </div>
            <div>
                <button disabled=move || loading()>"Sign up"</button>
                {if login_link {
                    view! { <A href="/login">"Log in"</A> }.into_view()
                } else {
                    view! {}.into_view()
                }}

            </div>
            <div>{error}</div>
        </form>
    }
}

async fn register(email: String, name: Option<String>) -> Result<User, String> {
    tauri_sys::core::invoke_result("register_user", RegisterArgs { email, name })
        .await
        .map_err(|err| match err {
            syre_local::Error::IoSerde(err) => {
                tracing::debug!(?err);
                "Could not load user manifest.".to_string()
            }
            syre_local::Error::Users(syre_local::error::Users::InvalidEmail(_)) => {
                "Invalid email.".to_string()
            }
            syre_local::Error::Users(syre_local::error::Users::DuplicateEmail(_)) => {
                "Email is already registered.".to_string()
            }
            err => {
                tracing::debug!(?err);
                "Could not create user.".to_string()
            }
        })
}

#[derive(Serialize)]
struct RegisterArgs {
    email: String,
    name: Option<String>,
}
