pub mod builtins;
pub mod decimal;
pub mod primitives;

pub use decimal::Decimal;

use crate::errors::compile::CompileError;

#[derive(Debug, Clone, PartialEq)]
pub enum AdtVariant {
    Unit(String),             // Variant name only
    Tuple(String, Vec<Type>), // Variant name + tuple of types
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdtType {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<AdtVariant>,
}

impl AdtType {
    pub fn content_hash(&self) -> String {
        // Simplified - should compute SHA-256 of variant list
        format!("adt_{}_{}", self.name, self.variants.len())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitives
    Number,
    Boolean,
    String,
    Decimal,

    // ADTs
    Adt(Box<AdtType>),

    // Type variables (for generics)
    Var(String),
}

impl Type {
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Type::Number | Type::Boolean | Type::String | Type::Decimal
        )
    }

    pub fn size_bits(&self) -> usize {
        match self {
            Type::Number => 64,
            Type::Boolean => 1,
            Type::String => 0,    // Variable size
            Type::Decimal => 128, // Approximation
            Type::Adt(_) => 0,    // Variable
            Type::Var(_) => 0,    // Unknown
        }
    }

    // Built-in types
    pub fn option(inner: Type) -> Type {
        Type::Adt(Box::new(AdtType {
            name: "Option".to_string(),
            type_params: vec!["T".to_string()],
            variants: vec![
                AdtVariant::Unit("None".to_string()),
                AdtVariant::Tuple("Some".to_string(), vec![inner]),
            ],
        }))
    }

    pub fn result(ok_type: Type, err_type: Type) -> Type {
        Type::Adt(Box::new(AdtType {
            name: "Result".to_string(),
            type_params: vec!["T".to_string(), "E".to_string()],
            variants: vec![
                AdtVariant::Tuple("Ok".to_string(), vec![ok_type]),
                AdtVariant::Tuple("Err".to_string(), vec![err_type]),
            ],
        }))
    }
}

pub struct TypeContext {
    types: std::collections::HashMap<String, Type>,
}

impl TypeContext {
    pub fn with_builtins() -> Self {
        let mut ctx = TypeContext {
            types: std::collections::HashMap::new(),
        };

        // Add built-in types
        ctx.types.insert("number".to_string(), Type::Number);
        ctx.types.insert("boolean".to_string(), Type::Boolean);
        ctx.types.insert("string".to_string(), Type::String);
        ctx.types.insert("Decimal".to_string(), Type::Decimal);

        // Option and Result are generic, so we don't add them here
        // They need to be instantiated with type parameters

        ctx
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn insert(&mut self, name: String, typ: Type) {
        self.types.insert(name, typ);
    }
}
