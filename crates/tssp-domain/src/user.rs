//! User identity and roles.

use std::fmt;

/// Stable user identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    /// Creates a validated user id.
    ///
    /// # Errors
    ///
    /// Returns [`crate::DomainError`] when the id is empty or invalid.
    pub fn new(value: impl Into<String>) -> Result<Self, crate::DomainError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(crate::DomainError::Empty { field: "user id" });
        }
        if trimmed.len() > 64 {
            return Err(crate::DomainError::TooLong {
                field: "user id",
                max: 64,
                actual: trimmed.len(),
            });
        }
        if !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        {
            return Err(crate::DomainError::InvalidFormat { field: "user id" });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Access role for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    /// Full administrative access.
    Admin,
    /// Standard user; manages own files only.
    User,
}

impl UserRole {
    /// Parses a role from storage.
    ///
    /// # Errors
    ///
    /// Returns [`crate::DomainError`] when the value is unknown.
    pub fn parse(value: &str) -> Result<Self, crate::DomainError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "admin" => Ok(Self::Admin),
            "user" => Ok(Self::User),
            _ => Err(crate::DomainError::InvalidFormat { field: "role" }),
        }
    }

    /// Returns the canonical storage representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }
}

/// Resource visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Only owner and admins.
    #[default]
    Private,
    /// Accessible via public link / public listing.
    Public,
}

impl Visibility {
    /// Parses visibility from storage.
    ///
    /// # Errors
    ///
    /// Returns [`crate::DomainError`] when the value is unknown.
    pub fn parse(value: &str) -> Result<Self, crate::DomainError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "private" => Ok(Self::Private),
            "public" => Ok(Self::Public),
            _ => Err(crate::DomainError::InvalidFormat {
                field: "visibility",
            }),
        }
    }

    /// Returns the canonical storage representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Public => "public",
        }
    }
}

/// Display name for login (unique, case-insensitive in storage).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserName(String);

impl UserName {
    /// Creates a validated display/login name.
    ///
    /// # Errors
    ///
    /// Returns [`crate::DomainError`] when the name is invalid.
    pub fn new(value: impl Into<String>) -> Result<Self, crate::DomainError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(crate::DomainError::Empty { field: "user name" });
        }
        if trimmed.len() > 64 {
            return Err(crate::DomainError::TooLong {
                field: "user name",
                max: 64,
                actual: trimmed.len(),
            });
        }
        if !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        {
            return Err(crate::DomainError::InvalidFormat { field: "user name" });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
