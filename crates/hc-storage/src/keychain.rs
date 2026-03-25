/// macOS Keychain boundary. Metadata is stored in SQLite; actual secret values
/// are never persisted to the database. On non-macOS platforms this module is a no-op.
pub struct KeychainBoundary;

#[cfg(target_os = "macos")]
impl KeychainBoundary {
    /// Store a secret value in the macOS Keychain by service+account key.
    ///
    /// The secret value is passed directly to the Keychain and never logged or
    /// included in error messages.
    pub fn store(service: &str, account: &str, value: &[u8]) -> Result<(), crate::StorageError> {
        security_framework::passwords::set_generic_password(service, account, value)
            .map_err(|e| crate::StorageError::Keychain(format!("os status: {}", e.code())))
    }

    /// Retrieve a secret value from the macOS Keychain.
    ///
    /// Returns `Ok(None)` when the item does not exist (`errSecItemNotFound`).
    /// The retrieved bytes are returned as an owned `Vec<u8>`; the value is
    /// never included in any error message.
    pub fn retrieve(
        service: &str,
        account: &str,
    ) -> Result<Option<Vec<u8>>, crate::StorageError> {
        // -25300 maps to errSecItemNotFound in Security/SecBase.h
        const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

        match security_framework::passwords::get_generic_password(service, account) {
            Ok(bytes) => Ok(Some(bytes.to_vec())),
            Err(e) if e.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(None),
            Err(e) => Err(crate::StorageError::Keychain(format!(
                "os status: {}",
                e.code()
            ))),
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl KeychainBoundary {
    /// No-op on non-macOS platforms.
    pub fn store(
        _service: &str,
        _account: &str,
        _value: &[u8],
    ) -> Result<(), crate::StorageError> {
        Ok(())
    }

    /// No-op on non-macOS platforms; always returns `Ok(None)`.
    pub fn retrieve(
        _service: &str,
        _account: &str,
    ) -> Result<Option<Vec<u8>>, crate::StorageError> {
        Ok(None)
    }
}
