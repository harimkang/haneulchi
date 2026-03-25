#[cfg(target_os = "macos")]
mod tests {
    use hc_storage::KeychainBoundary;

    /// A fixed service name used for all tests so items are grouped together
    /// and cannot collide with production keychain entries.
    const TEST_SERVICE: &str = "haneulchi.tests.keychain_boundary";

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    /// Delete a test item, ignoring "not found" so cleanup is always safe.
    fn cleanup(account: &str) {
        let _ =
            security_framework::passwords::delete_generic_password(TEST_SERVICE, account);
    }

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    /// Storing a value and then retrieving it must round-trip successfully.
    #[test]
    fn store_and_retrieve_roundtrip() {
        let account = "test-roundtrip-a1b2c3d4";
        let secret = b"super-secret-value";

        cleanup(account);

        KeychainBoundary::store(TEST_SERVICE, account, secret)
            .expect("store should succeed");

        let retrieved = KeychainBoundary::retrieve(TEST_SERVICE, account)
            .expect("retrieve should succeed");

        assert_eq!(retrieved, Some(secret.to_vec()));

        cleanup(account);
    }

    /// Retrieving a key that was never stored must return `Ok(None)`.
    #[test]
    fn retrieve_missing_key_returns_none() {
        let account = "test-missing-e5f6g7h8";

        // Make sure it really does not exist.
        cleanup(account);

        let result = KeychainBoundary::retrieve(TEST_SERVICE, account)
            .expect("retrieve for missing key should not error");

        assert_eq!(result, None);

        cleanup(account);
    }

    /// Overwriting an existing entry must not error and must reflect the new value.
    #[test]
    fn store_overwrites_existing_value() {
        let account = "test-overwrite-i9j0k1l2";
        let first = b"first-secret";
        let second = b"second-secret";

        cleanup(account);

        KeychainBoundary::store(TEST_SERVICE, account, first)
            .expect("first store should succeed");

        KeychainBoundary::store(TEST_SERVICE, account, second)
            .expect("overwrite store should succeed");

        let retrieved = KeychainBoundary::retrieve(TEST_SERVICE, account)
            .expect("retrieve after overwrite should succeed");

        assert_eq!(retrieved, Some(second.to_vec()));

        cleanup(account);
    }
}
