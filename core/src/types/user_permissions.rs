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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UserPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl UserPermissions {
    pub fn new() -> UserPermissions {
        UserPermissions {
            read: false,
            write: false,
            execute: false,
        }
    }
}

#[cfg(test)]
#[path = "./user_permissions_test.rs"]
mod user_permissions_test;
