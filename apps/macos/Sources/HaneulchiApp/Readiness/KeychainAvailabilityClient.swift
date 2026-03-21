import Foundation
import Security

struct KeychainAvailabilityClient: Sendable {
    let isAvailable: @Sendable () async -> Bool

    static func mock(isAvailable: Bool) -> Self {
        Self(isAvailable: { isAvailable })
    }

    static let live = Self(
        isAvailable: {
            let query: [String: Any] = [
                kSecClass as String: kSecClassGenericPassword,
                kSecMatchLimit as String: kSecMatchLimitOne,
                kSecReturnAttributes as String: false,
            ]
            let status = SecItemCopyMatching(query as CFDictionary, nil)
            return status == errSecSuccess || status == errSecItemNotFound || status == errSecInteractionNotAllowed
        }
    )
}
