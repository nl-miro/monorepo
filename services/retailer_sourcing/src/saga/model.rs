//! Saga identifiers.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::{Uuid, Variant, Version};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Uuid", into = "Uuid")]
pub(crate) struct UuidV7(Uuid);

pub(crate) type SagaId = UuidV7;

impl UuidV7 {
    pub(crate) fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

#[derive(Debug, Error)]
#[error("expected an RFC UUIDv7")]
pub(crate) struct InvalidSagaId;

impl TryFrom<Uuid> for UuidV7 {
    type Error = InvalidSagaId;

    fn try_from(id: Uuid) -> Result<Self, Self::Error> {
        if id.get_version() == Some(Version::SortRand) && id.get_variant() == Variant::RFC4122 {
            Ok(Self(id))
        } else {
            Err(InvalidSagaId)
        }
    }
}

impl From<UuidV7> for Uuid {
    fn from(id: UuidV7) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::SagaId;
    use uuid::Uuid;

    #[test]
    fn saga_ids_require_uuid_version_seven_and_the_rfc_variant() {
        let id = SagaId::new();
        let uuid = Uuid::from(id);
        assert_eq!(SagaId::try_from(uuid).unwrap(), id);
        assert!(SagaId::try_from(Uuid::nil()).is_err());
        let mut bytes = *uuid.as_bytes();
        bytes[8] = 0;
        assert!(SagaId::try_from(Uuid::from_bytes(bytes)).is_err());
    }
}
