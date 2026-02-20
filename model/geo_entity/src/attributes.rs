use crate::{AttributeCode, AttributeValue, EntityError, EntityResult, find_system_attribute_def};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Attributes {
    values: HashMap<AttributeCode, AttributeValue>,
}

impl Attributes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, code: AttributeCode, value: AttributeValue) -> EntityResult<()> {
        if code.is_system() {
            let Some(system_def) = find_system_attribute_def(code) else {
                return Err(EntityError::UnknownSystemAttributeCode(code.raw()));
            };

            let value_kind = value.kind();
            if system_def.value_kind != value_kind {
                return Err(EntityError::AttributeTypeMismatch {
                    code: code.raw(),
                    expected: system_def.value_kind.as_str(),
                    actual: value_kind.as_str(),
                });
            }
        }

        self.values.insert(code, value);
        Ok(())
    }

    pub fn get(&self, code: AttributeCode) -> Option<&AttributeValue> {
        self.values.get(&code)
    }

    pub fn remove(&mut self, code: AttributeCode) -> Option<AttributeValue> {
        self.values.remove(&code)
    }

    pub fn contains(&self, code: AttributeCode) -> bool {
        self.values.contains_key(&code)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
