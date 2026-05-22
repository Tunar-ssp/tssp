//! Error types produced by pure domain validation.

use thiserror::Error;

/// A validation failure in a TSSP domain value.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    /// A required value was empty after normalization.
    #[error("{field} must not be empty")]
    Empty {
        /// Human-readable field name.
        field: &'static str,
    },

    /// A value exceeded a documented size limit.
    #[error("{field} is too long: {actual} exceeds {max}")]
    TooLong {
        /// Human-readable field name.
        field: &'static str,
        /// Maximum accepted length.
        max: usize,
        /// Actual supplied length.
        actual: usize,
    },

    /// A value used a byte or character that is not accepted by the rule.
    #[error("{field} contains an invalid character: {character:?}")]
    InvalidCharacter {
        /// Human-readable field name.
        field: &'static str,
        /// Rejected character.
        character: char,
    },

    /// A value did not match a required shape.
    #[error("{field} has invalid format")]
    InvalidFormat {
        /// Human-readable field name.
        field: &'static str,
    },

    /// A numeric value fell outside the accepted inclusive range.
    #[error("{field} must be between {min} and {max}, got {actual}")]
    OutOfRange {
        /// Human-readable field name.
        field: &'static str,
        /// Inclusive lower bound.
        min: u64,
        /// Inclusive upper bound.
        max: u64,
        /// Supplied value.
        actual: u64,
    },
}
