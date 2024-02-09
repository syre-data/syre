use std::rc::Rc;
use yew::prelude::*;

/// Identifies if the component has been mounted.
#[hook]
pub fn use_is_mounted() -> Rc<dyn Fn() -> bool> {
    let is_mounted = use_mut_ref(|| false);

    {
        let is_mounted = is_mounted.clone();
        use_effect_with((), move |_| {
            *is_mounted.borrow_mut() = true;

            // destructor
            move || {
                *is_mounted.borrow_mut() = false;
            }
        });
    }

    Rc::new(move || {
        let is_mounted = *is_mounted.borrow();
        is_mounted
    })
}
