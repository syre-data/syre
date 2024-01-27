#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// User permissions
///
/// # Fields
/// + `read`: Read permission.
/// 	    Can user view resource properties?
/// + `write`: Write permission.
/// 	    Can user edit resource properties?
/// + `execute`: Execute permission.
///	    Only applicable for Containers, if user can run analysis.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct UserPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl UserPermissions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_permissions(read: bool, write: bool, execute: bool) -> Self {
        Self {
            read,
            write,
            execute,
        }
    }
}
