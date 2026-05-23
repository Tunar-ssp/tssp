//! UTC timestamp value object used by the domain.

use crate::DomainError;

const MAX_REASONABLE_UNIX_SECONDS: i64 = 4_102_444_800;
const MAX_REASONABLE_UNIX_SECONDS_U64: u64 = 4_102_444_800;

/// UTC timestamp represented as whole seconds since the Unix epoch.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnixTimestamp(i64);

impl UnixTimestamp {
    /// Maximum supported timestamp in UTC seconds.
    pub const MAX_SECONDS: u64 = MAX_REASONABLE_UNIX_SECONDS_U64;

    /// Creates a timestamp within the supported operational range.
    ///
    /// The upper bound is 2100-01-01T00:00:00Z. It catches accidental millis or
    /// nanos being passed where seconds are expected.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::OutOfRange`] when `seconds` is negative or beyond
    /// the supported timestamp range.
    pub fn new(seconds: i64) -> Result<Self, DomainError> {
        if !(0..=MAX_REASONABLE_UNIX_SECONDS).contains(&seconds) {
            return Err(DomainError::OutOfRange {
                field: "unix timestamp",
                min: 0,
                max: MAX_REASONABLE_UNIX_SECONDS_U64,
                actual: seconds_to_u64_lossy(seconds),
            });
        }

        Ok(Self(seconds))
    }

    /// Returns seconds since the Unix epoch.
    #[must_use]
    pub const fn seconds(self) -> i64 {
        self.0
    }

    /// Returns the maximum supported timestamp.
    #[must_use]
    pub const fn max() -> Self {
        Self(MAX_REASONABLE_UNIX_SECONDS)
    }

    /// Returns seconds as an unsigned value.
    #[must_use]
    pub fn seconds_u64(self) -> u64 {
        seconds_to_u64_lossy(self.0)
    }
}

fn seconds_to_u64_lossy(seconds: i64) -> u64 {
    u64::try_from(seconds).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::UnixTimestamp;
    use crate::DomainError;

    #[test]
    fn current_epoch_seconds_are_valid() {
        assert!(UnixTimestamp::new(1_700_000_000).is_ok());
    }

    #[test]
    fn timestamp_accessors_return_seconds() {
        let timestamp = UnixTimestamp::new(1_700_000_000).unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(timestamp.seconds(), 1_700_000_000);
        assert_eq!(timestamp.seconds_u64(), 1_700_000_000);
        assert_eq!(
            UnixTimestamp::max().seconds_u64(),
            UnixTimestamp::MAX_SECONDS
        );
    }

    #[test]
    fn negative_time_is_rejected() {
        assert_eq!(
            UnixTimestamp::new(-1),
            Err(DomainError::OutOfRange {
                field: "unix timestamp",
                min: 0,
                max: 4_102_444_800,
                actual: 0
            })
        );
    }

    #[test]
    fn millisecond_input_is_rejected_by_upper_bound() {
        assert_eq!(
            UnixTimestamp::new(1_700_000_000_000),
            Err(DomainError::OutOfRange {
                field: "unix timestamp",
                min: 0,
                max: 4_102_444_800,
                actual: 1_700_000_000_000
            })
        );
    }
}
