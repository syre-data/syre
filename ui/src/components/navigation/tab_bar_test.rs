use super::*;
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::Fake;
use syre_core::types::ResourceId;

#[test]
fn next_tab_on_close_should_work() {
    let tabs = create_tabs(None);
    println!("{:#?}", tabs.keys());

    // --- normal cases ---
    // close active interior tab
    let active_index = tabs.len() - 4;
    let (active, _) = tabs.get_index(active_index).expect("tab should exist");
    let (next_exp, _) = tabs.get_index(active_index + 1).expect("tab should exist");
    let next = next_tab_on_close(active, Some(active), &tabs);
    assert_eq!(Some(next_exp), next, "next should be tab to the right");

    // close active last tab
    let active_index = tabs.len() - 1;
    let (active, _) = tabs.get_index(active_index).expect("tab should exist");
    let (next_exp, _) = tabs.get_index(active_index - 1).expect("tab should exist");
    let next = next_tab_on_close(active, Some(active), &tabs);
    assert_eq!(Some(next_exp), next, "next should be tab to the left");

    // close inactive tab
    let active_index = tabs.len() - 4;
    let (active, _) = tabs.get_index(active_index).expect("tab should exist");
    let (close, _) = tabs.get_index(active_index + 1).expect("tab should exist");
    let next = next_tab_on_close(close, Some(active), &tabs);
    assert_eq!(Some(active), next, "active should not be changed");

    // active tab not set
    let close_index = tabs.len() - 4;
    let (close, _) = tabs.get_index(close_index + 1).expect("tab should exist");
    let next = next_tab_on_close(close, None, &tabs);
    assert_eq!(None, next, "next should be undetermined");

    // close last tab
    let tabs_last = create_tabs(Some(1));
    let active_index = 0;
    let (active, _) = tabs_last.get_index(active_index).expect("tab should exist");
    let next = next_tab_on_close(active, Some(active), &tabs_last);
    assert_eq!(None, next, "next should be undetermined");

    // --- exceptional cases ---
    // active tab not found
    let close_index = tabs.len() - 4;
    let (close, _) = tabs.get_index(close_index + 1).expect("tab should exist");
    let active = ResourceId::new();
    let next = next_tab_on_close(close, Some(&active), &tabs);
    assert_eq!(None, next, "next should be undetermined");

    // close tab not found
    let (active, _) = tabs.get_index(close_index + 1).expect("tab should exist");
    let close = ResourceId::new();
    let next = next_tab_on_close(&close, Some(active), &tabs);
    assert_eq!(Some(active), next, "active tab should not be changed");
}

// ***************
// *** helpers ***
// ***************

fn create_tabs(num: Option<usize>) -> IndexMap<ResourceId, String> {
    let rng = if let Some(num) = num {
        num..(num + 1)
    } else {
        5..20
    };

    Words(EN, rng)
        .fake::<Vec<String>>()
        .into_iter()
        .map(|title| (ResourceId::new(), title))
        .collect::<IndexMap<ResourceId, String>>()
}
