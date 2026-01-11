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
}
