use alloc::string::{String, ToString};
use base_x;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RippleBinaryCodecError {
    #[error("decode failed, reason: {0}")]
    DecodeError(String),
}

pub type Result<T> = core::result::Result<T, RippleBinaryCodecError>;

impl From<base_x::DecodeError> for RippleBinaryCodecError {
    fn from(value: base_x::DecodeError) -> Self {
        Self::DecodeError(value.to_string())
    }
}
