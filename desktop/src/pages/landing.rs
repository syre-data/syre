use leptos::*;
use leptos_router::*;
use serde::Serialize;
use std::io;
use syre_core::system::User;
use syre_local::error::IoSerde;
use web_sys::{FormData, SubmitEvent};

#[component]
pub fn Landing() -> impl IntoView {
    let user_count = create_resource(|| (), |_| async move { fetch_user_count().await });
    let fallback = move |errors: RwSignal<Errors>| {
        errors.with(|errors| {
            let errors = errors
                .iter()
                .map(|(_, error)| (*error).clone())
                .collect::<Vec<_>>();

            let [error] = &errors[..] else {
                panic!("invalid errors");
            };

            view! {
                <Show
                    when={
                        let error = error.clone();
                        move || {
                            matches!(
                                error.downcast_ref::<IoSerde>().unwrap(),
                                IoSerde::Io(io::ErrorKind::NotFound)
                            )
                        }
                    }

                    fallback=|| view! { <div>"The user manifest is corrupt."</div> }
                >
                    <Register/>
                </Show>
            }
        })
    };

    view! {
        <h1>Syre</h1>
        <Suspense fallback=Loading>
            <ErrorBoundary fallback>
                {move || {
                    user_count
                        .get()
                        .map(|count| {
                            count
                                .map(|count| {
                                    view! {
                                        <Show
                                            when=move || { count > 0 }
                                            fallback=|| view! { <Register/> }
                                        >
                                            <div>
                                                <A href="/register">"Sign up"</A>
                                                <A href="/login">"Log in"</A>
                                            </div>
                                        </Show>
                                    }
                                })
                        })
                }}

            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div>"Loading users..."</div> }
}

// TODO: Use `auth/register`, but currently running into issue with use navigate.
#[component]
pub fn Register() -> impl IntoView {
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(None);
    let form_ref = NodeRef::new();
    let register_user = {
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

            spawn_local(async move {
                match register(email, name).await {
                    Ok(_user) => {}

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

async fn fetch_user_count() -> Result<usize, IoSerde> {
    tauri_sys::core::invoke_result("user_count", ()).await
}
