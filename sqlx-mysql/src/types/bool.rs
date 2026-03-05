use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{
    protocol::text::{ColumnFlags, ColumnType},
    MySql, MySqlTypeInfo, MySqlValueRef,
};

/// Parse a string as boolean value.
/// Only accepts "true" or "false" (case-insensitive).
#[cfg(feature = "matrixone")]
fn parse_bool_str(s: &str) -> Result<bool, BoxDynError> {
    let s = s.trim();
    
    if s.eq_ignore_ascii_case("true") {
        return Ok(true);
    }
    if s.eq_ignore_ascii_case("false") {
        return Ok(false);
    }
    
    Err(format!("Expected 'true' or 'false', got '{}'", s).into())
}

impl Type<MySql> for bool {
    fn type_info() -> MySqlTypeInfo {
        // MySQL has no actual `BOOLEAN` type, the type is an alias of `TINYINT(1)`
        MySqlTypeInfo {
            flags: ColumnFlags::BINARY | ColumnFlags::UNSIGNED,
            max_size: Some(1),
            r#type: ColumnType::Tiny,
        }
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::Tiny
                | ColumnType::Short
                | ColumnType::Long
                | ColumnType::Int24
                | ColumnType::LongLong
                | ColumnType::Bit
        ) || {
            #[cfg(feature = "matrixone")]
            {
                // MatrixOne EXISTS returns VARCHAR "true"/"false"
                matches!(ty.r#type, ColumnType::String | ColumnType::VarString | ColumnType::VarChar)
            }
            #[cfg(not(feature = "matrixone"))]
            {
                false
            }
        }
    }
}

impl Encode<'_, MySql> for bool {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
        <i8 as Encode<MySql>>::encode(*self as i8, buf)
    }
}

impl Decode<'_, MySql> for bool {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        #[cfg(feature = "matrixone")]
        {
            // MatrixOne: try string parsing first (for EXISTS function returning VARCHAR)
            // then fall back to integer parsing (for actual BOOLEAN/TINYINT columns)
            if let Ok(text) = value.as_str() {
                if let Ok(result) = parse_bool_str(text) {
                    return Ok(result);
                }
                // If it looks like a string but isn't "true"/"false",
                // let it fall through to integer parsing (might be "1" or "0")
            }
        }

        // Standard MySQL TINYINT(1) parsing (also used by MatrixOne for BOOLEAN columns)
        Ok(<i8 as Decode<MySql>>::decode(value)? != 0)
    }
}
