//! Tab bar.
use indexmap::IndexMap;
use std::hash::Hash;
use yew::prelude::*;
use yew::virtual_dom::Key;

/// Convenience trait grouping required traits for a tab key.
pub trait TabKey = PartialEq + Eq + Hash + Into<Key> + Clone;

/// Information provided when on tab close.
pub struct TabCloseInfo<K>
where
    K: TabKey,
{
    /// Key of the tab requesting to be closed.
    pub closing: K,

    /// Key of the tab that would become active next.
    pub next: Option<K>,
}

/// Properties for [`TabBar`].
#[derive(Properties, PartialEq)]
pub struct TabBarProps<K>
where
    K: TabKey,
{
    #[prop_or_default]
    pub id: Option<AttrValue>,

    #[prop_or_default]
    pub class: Classes,

    /// Key-value map for the tabs.
    /// Values are displayed as the tab name,
    /// keys are used to indicate tab actions.
    pub tabs: IndexMap<K, String>,

    /// Active tab.
    #[prop_or_default]
    pub active: Option<K>,

    /// Callback to run when a tab is clicked.
    pub onclick_tab: Callback<K>,

    /// Callback to run when a tab close button is cliked.
    /// If not provided close button is not displayed,
    /// and tabs can not be closed.
    #[prop_or_default]
    pub onclick_tab_close: Option<Callback<TabCloseInfo<K>>>,
    // @todo
    // /// Allow tabs to be reordered.
    // #[prop_or(false)]
    // reorder: bool,
}

/// Tab bar.
#[function_component(TabBar)]
pub fn tab_bar<K: TabKey + Clone + 'static>(props: &TabBarProps<K>) -> Html {
    let onclick_tab = {
        let onclick_tab = props.onclick_tab.clone();

        move |key: K| -> Callback<MouseEvent> {
            Callback::from(move |_: MouseEvent| {
                onclick_tab.emit(key.clone());
            })
        }
    };

    let onclick_tab_close = {
        let active = props.active.clone();
        let tabs = props.tabs.clone();

        props.onclick_tab_close.clone().map(|onclick_tab_close| {
            move |key: K| -> Callback<MouseEvent> {
                let next = next_tab_on_close(&key, active.as_ref(), &tabs).cloned();

                Callback::from(move |e: MouseEvent| {
                    e.stop_propagation();
                    onclick_tab_close.emit(TabCloseInfo {
                        closing: key.clone(),
                        next: next.clone(),
                    });
                })
            }
        })
    };

    html! {
        <ol id={props.id.clone()}
            class={classes!("tab-list", props.class.clone())} >

            { props.tabs
                .iter()
                .map(|(k, v)| {
                    let mut class = classes!("clickable");
                    if Some(k) == props.active.as_ref() {
                        class.push("active");
                    }

                    html! {
                        <li key={k.clone()} {class}
                            onclick={onclick_tab.clone()(k.clone())}>
                            <div class={classes!("tab-container")}>
                                <span class={classes!("tab-name")}>{
                                    &v
                                }</span>

                                if let Some(onclick_tab_close) = onclick_tab_close.clone() {
                                    <button class={classes!("btn-tab-close")}
                                        onclick={onclick_tab_close(k.clone())}>{

                                        "x"
                                    }</button>
                                }
                            </div>
                        </li>
                    }
                })
                .collect::<Html>()
            }
        </ol>
    }
}

// ***************
// *** helpers ***
// ***************

/// Calculates the next tab to be active if the current one closes.
///
/// # Arguments
/// 1. Tab to be closed.
/// 2. Key of the currently active tab.
/// 3. All tabs.
///
/// # Returns
/// + Under normal circumstances
///   + If a non-active tab is being closed, do not change the active tab.
///   + If the active tab is being closed,
///   returns the tab to the right of the closing tab, if it exists,
///   then return the tab to the left, if it exists,
///   then returns `None` (no tab exist).
/// + If no active tab is found, or an invalid (non-existant) tab
/// is active, returns `None`.
/// + If the closing tab is not found, returns the active tab.
fn next_tab_on_close<'a, K: TabKey>(
    close: &'a K,
    active: Option<&'a K>,
    tabs: &'a IndexMap<K, String>,
) -> Option<&'a K> {
    let Some(active) = active else {
        // no active tab
        return None;
    };

    if !tabs.contains_key(active) {
        // active tab not found
        return None;
    };

    let Some(close_index) = tabs.get_index_of(close) else {
        // current tab not found, no effect
        return Some(active);
    };

    if close != active {
        // did not close active tab, do not change active
        return Some(active);
    }

    // closed the active tab
    if tabs.len() < 2 {
        // closed last project
        return None;
    }

    // next active project should be the next project to the right,
    // if it exists, otherwise the project to the left.
    let next_index = if close_index == tabs.len() - 1 {
        // end project, activate project to the left
        close_index - 1
    } else {
        // interior project, activate project to right
        close_index + 1
    };

    tabs.get_index(next_index).map(|(k, _)| k)
}

#[cfg(test)]
#[path = "./tab_bar_test.rs"]
mod tab_bar_test;
