// Type unification for generic type parameters
// Implements Robinson's unification algorithm with occurs check

use crate::types::{RecordType, Type};
use std::collections::HashMap;

/// Substitution map from type variables to types
pub type Substitution = HashMap<String, Type>;

/// Unify two types, producing a substitution if successful
pub fn unify(left: &Type, right: &Type) -> Result<Substitution, String> {
    unify_with_subs(left, right, &HashMap::new())
}

/// Unify with existing substitutions
fn unify_with_subs(left: &Type, right: &Type, subs: &Substitution) -> Result<Substitution, String> {
    let left = apply_substitution(left, subs);
    let right = apply_substitution(right, subs);

    match (&left, &right) {
        // Same type - trivially unify
        (l, r) if l == r => Ok(subs.clone()),

        // Type variable on left - bind it
        (Type::TypeVar(name), t) | (Type::Var(name), t) => {
            if occurs_check(name, t) {
                Err(format!("Infinite type: {} occurs in {:?}", name, t))
            } else {
                let mut new_subs = subs.clone();
                new_subs.insert(name.clone(), t.clone());
                Ok(new_subs)
            }
        }

        // Type variable on right - bind it
        (t, Type::TypeVar(name)) | (t, Type::Var(name)) => {
            if occurs_check(name, t) {
                Err(format!("Infinite type: {} occurs in {:?}", name, t))
            } else {
                let mut new_subs = subs.clone();
                new_subs.insert(name.clone(), t.clone());
                Ok(new_subs)
            }
        }

        // Function types - unify parameter and return types
        (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
            if params1.len() != params2.len() {
                return Err(format!(
                    "Function arity mismatch: {} vs {}",
                    params1.len(),
                    params2.len()
                ));
            }

            let mut current_subs = subs.clone();
            for (p1, p2) in params1.iter().zip(params2.iter()) {
                current_subs = unify_with_subs(p1, p2, &current_subs)?;
            }
            unify_with_subs(ret1, ret2, &current_subs)
        }

        // ADT types - unify if same name
        (Type::Adt(adt1), Type::Adt(adt2)) => {
            if adt1.name != adt2.name {
                return Err(format!("Type mismatch: {} vs {}", adt1.name, adt2.name));
            }
            if adt1.type_params.len() != adt2.type_params.len() {
                return Err(format!("Type parameter count mismatch for {}", adt1.name));
            }

            let mut current_subs = subs.clone();
            for (p1, p2) in adt1.type_params.iter().zip(adt2.type_params.iter()) {
                let t1 = Type::TypeVar(p1.clone());
                let t2 = Type::TypeVar(p2.clone());
                current_subs = unify_with_subs(&t1, &t2, &current_subs)?;
            }
            Ok(current_subs)
        }

        // Record types - unify if same name and fields match
        (Type::Record(rec1), Type::Record(rec2)) => {
            if rec1.name != rec2.name {
                return Err(format!(
                    "Record type mismatch: {} vs {}",
                    rec1.name, rec2.name
                ));
            }

            let mut current_subs = subs.clone();
            for (name, type1) in &rec1.fields {
                if let Some(type2) = rec2.field_type(name) {
                    current_subs = unify_with_subs(type1, type2, &current_subs)?;
                } else {
                    return Err(format!("Field {} missing in record {}", name, rec2.name));
                }
            }
            Ok(current_subs)
        }

        // Cannot unify
        _ => Err(format!("Cannot unify {:?} with {:?}", left, right)),
    }
}

/// Check if a type variable occurs in a type (prevents infinite types)
fn occurs_check(var: &str, ty: &Type) -> bool {
    match ty {
        Type::TypeVar(name) | Type::Var(name) => name == var,
        Type::Function(params, ret) => {
            params.iter().any(|p| occurs_check(var, p)) || occurs_check(var, ret)
        }
        Type::Adt(adt) => adt.type_params.iter().any(|p| p == var),
        Type::Record(rec) => rec.fields.iter().any(|(_, t)| occurs_check(var, t)),
        _ => false,
    }
}

/// Apply a substitution to a type
pub fn apply_substitution(ty: &Type, subs: &Substitution) -> Type {
    match ty {
        Type::TypeVar(name) | Type::Var(name) => {
            if let Some(t) = subs.get(name) {
                apply_substitution(t, subs)
            } else {
                ty.clone()
            }
        }
        Type::Function(params, ret) => Type::Function(
            params.iter().map(|p| apply_substitution(p, subs)).collect(),
            Box::new(apply_substitution(ret, subs)),
        ),
        Type::Adt(_adt) => {
            // For ADT, we don't substitute type params directly
            // as they are stored as strings
            ty.clone()
        }
        Type::Record(rec) => {
            let mut new_fields = Vec::new();
            for (name, field_type) in &rec.fields {
                new_fields.push((name.clone(), apply_substitution(field_type, subs)));
            }
            Type::Record(Box::new(RecordType {
                name: rec.name.clone(),
                fields: new_fields,
            }))
        }
        _ => ty.clone(),
    }
}

/// Instantiate a type scheme by replacing type variables with fresh ones
pub fn instantiate(ty: &Type, fresh_prefix: &str, counter: &mut usize) -> Type {
    let mut subs = HashMap::new();
    collect_type_vars(ty, &mut subs, fresh_prefix, counter);
    apply_substitution(ty, &subs)
}

fn collect_type_vars(ty: &Type, subs: &mut Substitution, prefix: &str, counter: &mut usize) {
    match ty {
        Type::TypeVar(name) | Type::Var(name) => {
            if !subs.contains_key(name) {
                let fresh = format!("{}_{}", prefix, counter);
                *counter += 1;
                subs.insert(name.clone(), Type::TypeVar(fresh));
            }
        }
        Type::Function(params, ret) => {
            for p in params {
                collect_type_vars(p, subs, prefix, counter);
            }
            collect_type_vars(ret, subs, prefix, counter);
        }
        Type::Adt(_adt) => {
            // Type params in ADTs are strings, not Type instances
            // We could instantiate them here if needed
        }
        Type::Record(rec) => {
            for (_, field_type) in &rec.fields {
                collect_type_vars(field_type, subs, prefix, counter);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_same_type() {
        let result = unify(&Type::Number, &Type::Number);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unify_type_var() {
        let result = unify(&Type::TypeVar("T".to_string()), &Type::Number);
        assert!(result.is_ok());
        let subs = result.unwrap();
        assert_eq!(subs.get("T"), Some(&Type::Number));
    }

    #[test]
    fn test_unify_different_types() {
        let result = unify(&Type::Number, &Type::String);
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_functions() {
        let f1 = Type::Function(
            vec![Type::TypeVar("T".to_string())],
            Box::new(Type::TypeVar("T".to_string())),
        );
        let f2 = Type::Function(vec![Type::Number], Box::new(Type::Number));
        let result = unify(&f1, &f2);
        assert!(result.is_ok());
        let subs = result.unwrap();
        assert_eq!(subs.get("T"), Some(&Type::Number));
    }

    #[test]
    fn test_apply_substitution() {
        let mut subs = HashMap::new();
        subs.insert("T".to_string(), Type::Number);

        let ty = Type::Function(
            vec![Type::TypeVar("T".to_string())],
            Box::new(Type::TypeVar("T".to_string())),
        );
        let result = apply_substitution(&ty, &subs);

        assert_eq!(
            result,
            Type::Function(vec![Type::Number], Box::new(Type::Number))
        );
    }
}
