use crate::{EntityError, EntityResult};
use serde::{Deserialize, Serialize};

/// 属性種別コード
///
/// - 負値: system属性
/// - 正値: user属性
/// - 0: 無効
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AttributeCode(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeCodeParts {
    pub domain: u8,
    pub category: u8,
    pub subcategory: u8,
    pub item: u8,
}

impl AttributeCode {
    pub const INVALID: Self = Self(0);

    pub const fn new_unchecked(raw: i32) -> Self {
        Self(raw)
    }

    pub fn try_new(raw: i32) -> EntityResult<Self> {
        if raw == 0 {
            return Err(EntityError::InvalidAttributeCode);
        }
        Ok(Self(raw))
    }

    pub const fn raw(self) -> i32 {
        self.0
    }

    pub const fn is_system(self) -> bool {
        self.0 < 0
    }

    pub const fn is_user(self) -> bool {
        self.0 > 0
    }

    pub fn parts(self) -> Option<AttributeCodeParts> {
        let abs_value = self.0.abs();
        if abs_value > 99_99_99_99 {
            return None;
        }

        let domain = (abs_value / 1_000_000) as u8;
        let category = ((abs_value / 10_000) % 100) as u8;
        let subcategory = ((abs_value / 100) % 100) as u8;
        let item = (abs_value % 100) as u8;

        Some(AttributeCodeParts {
            domain,
            category,
            subcategory,
            item,
        })
    }
}
