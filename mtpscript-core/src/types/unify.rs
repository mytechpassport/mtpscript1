// Type unification for generic type parameters
// Placeholder implementation

use crate::types::Type;

pub fn unify(left: &Type, right: &Type) -> Result<Type, String> {
    if left == right {
        Ok(left.clone())
    } else {
        Err(format!("Cannot unify {:?} with {:?}", left, right))
    }
}
