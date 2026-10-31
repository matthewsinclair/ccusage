import Foundation
import LocalAuthentication
import Security

#if SECOND_BUILD
let build = "second"
#else
let build = "first"
#endif
let password = Data("public-disposable-test-password".utf8)
func check(_ status: OSStatus) {
    if status != errSecSuccess {
        fputs("Keychain upgrade fixture failed (OSStatus \(status), build \(build))\n", stderr)
        exit(1)
    }
}
guard CommandLine.arguments.count == 3 else { exit(2) }
let mode = CommandLine.arguments[1]
let service = CommandLine.arguments[2]
guard service.hasPrefix("com.geodica.ccusage.upgrade-fixture.") else { exit(2) }
var query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: service,
    kSecAttrAccount as String: "fixture"
]
switch mode {
case "create":
    query[kSecValueData as String] = password
    check(SecItemAdd(query as CFDictionary, nil))
case "read":
    // Fail instead of asking for approval: the upgraded build must already be trusted.
    let context = LAContext()
    context.interactionNotAllowed = true
    query[kSecUseAuthenticationContext as String] = context
    query[kSecReturnData as String] = true
    query[kSecMatchLimit as String] = kSecMatchLimitOne
    var result: CFTypeRef?
    check(SecItemCopyMatching(query as CFDictionary, &result))
    guard let data = result as? Data, data == password else { exit(1) }
case "delete":
    let status = SecItemDelete(query as CFDictionary)
    if status != errSecItemNotFound { check(status) }
default:
    exit(2)
}
