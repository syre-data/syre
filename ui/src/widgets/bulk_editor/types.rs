//! Types for bulk editing.

#[derive(PartialEq, Clone)]
pub enum BulkValue<T>
where
    T: PartialEq + Clone,
{
    Equal(T),
    Mixed,
}

impl<T> BulkValue<T>
where
    T: PartialEq + Clone,
{
    pub fn is_equal(&self) -> bool {
        match self {
            Self::Equal(_) => true,
            Self::Mixed => false,
        }
    }

    pub fn is_mixed(&self) -> bool {
        !self.is_equal()
    }
}
