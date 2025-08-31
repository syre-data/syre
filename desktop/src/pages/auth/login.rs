use leptos::{prelude::*, task::spawn_local};
use leptos_router::{components::A, hooks::use_navigate};
use serde::Serialize;
use syre_core::system::User;
use syre_desktop_lib::command::auth::error;
use syre_desktop_ui_components::{Autofocus, Logo};
use syre_local as local;
use web_sys::{FormData, SubmitEvent};

#[component]
pub fn Login() -> impl IntoView {
    let navigate = use_navigate();
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None);
    let form_ref = NodeRef::new();

    let login_user = {
        let navigate = navigate.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            set_loading(true);
            set_error(None);

            let data = FormData::new_with_form(&form_ref.get().unwrap()).unwrap();
            let email = data.get("email").as_string().unwrap();

            if email.trim().is_empty() {
                set_error(Some("Email is required".to_string()));
                return;
            }
            let email = email.trim().to_string();

            let navigate = navigate.clone();
            spawn_local(async move {
                match login(email).await {
                    Ok(_user) => {
                        navigate("/", Default::default());
                    }

                    Err(err) => {
                        let msg = match err {
                            error::Login::GetUser(err) => {
                                format!("Could not get user: {err:?}.")
                            }
                            error::Login::InvalidCredentials => "Invalid credentials.".to_string(),
                            error::Login::SetActiveUser(err) => {
                                format!("Could not set active user: {err:?}.")
                            }
                        };
                        set_error(Some(msg));
                        set_loading(false);
                    }
                }
            });
        }
    };

    view! {
        <div class="h-screen w-screen flex flex-col justify-center items-center gap-y-4">
            <div class="flex flex-col items-center w-20">
                <Logo attr:class="w-full" />
                <h1 class="font-primary text-4xl">"Syre"</h1>
            </div>
            <div class="flex justify-center w-1/2">
                <form node_ref=form_ref on:submit=login_user>
                    <div>
                        <label>
                            <span class="block">"Email"</span>
                            <Autofocus>
                                <input
                                    type="email"
                                    name="email"
                                    class="input-simple"
                                    required=true
                                    autofocus
                                />
                            </Autofocus>
                        </label>
                    </div>
                    <div class="text-sm">{error}</div>
                    <div class="pt-4 flex gap-x-4 justify-center">
                        <button disabled=loading class="btn btn-primary">
                            "Login"
                        </button>
                        <A href="/register" attr:class="btn btn-secondary">
                            "Sign up"
                        </A>
                    </div>
                </form>
            </div>
        </div>
    }
}

async fn login(email: String) -> Result<User, error::Login> {
    tauri_sys::core::invoke_result("login", LoginArgs { email }).await
}

#[derive(Serialize)]
struct LoginArgs {
    email: String,
}
