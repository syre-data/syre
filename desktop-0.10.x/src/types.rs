use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[derive(Clone)]
pub struct RcEq<T>(Rc<T>);
impl<T> RcEq<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T> PartialEq for RcEq<T> {
    fn eq(&self, other: &RcEq<T>) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> PartialEq<Rc<T>> for RcEq<T> {
    fn eq(&self, other: &Rc<T>) -> bool {
        Rc::ptr_eq(&self.0, other)
    }
}

impl<T> Deref for RcEq<T> {
    type Target = Rc<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RcEq<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
