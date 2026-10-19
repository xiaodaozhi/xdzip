//! Distinct, nonzero identities preserve every bit across the native boundary.
use std::num::{NonZeroU64, TryFromIntError};

macro_rules! identity {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU64);

        impl $name {
            /// Exact native representation; QML must receive an opaque string instead.
            pub const fn get(self) -> u64 {
                self.0.get()
            }
        }

        impl TryFrom<u64> for $name {
            type Error = TryFromIntError;
            fn try_from(value: u64) -> Result<Self, Self::Error> {
                NonZeroU64::try_from(value).map(Self)
            }
        }
    };
}

identity!(
    SessionId,
    "Runtime archive session identity, independent of its path."
);
identity!(
    EntryId,
    "Physical record identity within one session; duplicate names stay distinct."
);
identity!(QueryId, "Immutable query snapshot identity.");
identity!(
    QueryGeneration,
    "Nonzero UI request generation; exhaustion requires a new session."
);
identity!(
    SnapshotRevision,
    "Nonzero version of the immutable query result."
);
identity!(
    TaskId,
    "Runtime task identity; it cannot be substituted for an entry identity."
);
identity!(
    RequestId,
    "Command correlation identity, never reused within its runtime."
);

/// Demonstrate the type boundary without permitting lossy or implicit conversion.
///
/// ```compile_fail
/// use xdzip_domain::identity::{EntryId, TaskId};
/// let entry: EntryId = TaskId::try_from(1).unwrap();
/// ```
const _: () = ();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_identity_rejects_zero_and_preserves_large_values() {
        assert!(SessionId::try_from(0).is_err());
        for value in [1, (1_u64 << 53) + 1, u64::MAX] {
            assert_eq!(SessionId::try_from(value).unwrap().get(), value);
        }
    }
}
