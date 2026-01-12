use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq)]
pub struct RecordType {
    pub name: String,
    pub fields: Vec<(String, super::Type)>, // (field_name, field_type)
}

impl RecordType {
    pub fn new(name: String, fields: Vec<(String, super::Type)>) -> Self {
        RecordType { name, fields }
    }

    /// Get the type of a field by name
    pub fn field_type(&self, field_name: &str) -> Option<&super::Type> {
        self.fields
            .iter()
            .find(|(name, _)| name == field_name)
            .map(|(_, typ)| typ)
    }

    /// Check if the record has a field
    pub fn has_field(&self, field_name: &str) -> bool {
        self.fields.iter().any(|(name, _)| name == field_name)
    }

    /// Get all field names
    pub fn field_names(&self) -> Vec<&str> {
        self.fields.iter().map(|(name, _)| name.as_str()).collect()
    }

    /// Compute a content-based hash for deterministic schema folding
    /// Two structurally identical records will have the same hash regardless of name
    pub fn content_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // Sort fields by name for deterministic ordering
        let mut sorted_fields: Vec<_> = self.fields.iter().collect();
        sorted_fields.sort_by_key(|(name, _)| name.as_str());

        for (field_name, field_type) in sorted_fields {
            hasher.update(field_name.as_bytes());
            hasher.update(b":");
            hasher.update(Self::type_to_bytes(field_type));
            hasher.update(b";");
        }

        format!("{:x}", hasher.finalize())
    }

    fn type_to_bytes(typ: &super::Type) -> Vec<u8> {
        match typ {
            super::Type::Number => b"number".to_vec(),
            super::Type::Boolean => b"boolean".to_vec(),
            super::Type::String => b"string".to_vec(),
            super::Type::Decimal => b"decimal".to_vec(),
            super::Type::TypeVar(name) => format!("typevar:{}", name).as_bytes().to_vec(),
            super::Type::Record(rec) => {
                format!("record:{}", rec.content_hash()).as_bytes().to_vec()
            }
            super::Type::Adt(adt) => format!("adt:{}", adt.content_hash()).as_bytes().to_vec(),
            super::Type::Json => b"json".to_vec(),
            super::Type::Var(name) => format!("var:{}", name).as_bytes().to_vec(),
            super::Type::Function(_, _) => b"function".to_vec(),
        }
    }
}
