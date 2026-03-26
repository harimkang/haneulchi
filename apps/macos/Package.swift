// swift-tools-version: 6.1
import Foundation
import PackageDescription

let packageDirectory = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let workspaceRoot = packageDirectory
    .deletingLastPathComponent()
    .deletingLastPathComponent()
let vendorLibraryDirectory = packageDirectory
    .appendingPathComponent("Vendor/lib", isDirectory: true)
    .path
let pluginLibraryDirectory = packageDirectory
    .appendingPathComponent(
        ".build/plugins/outputs/macos/HaneulchiApp/destination/HCCoreFFIBuildPlugin/hc-ffi-build",
        isDirectory: true,
    )
    .path
let hcFFILinkerSettings: [LinkerSetting] = [
    .unsafeFlags([
        "-L", pluginLibraryDirectory,
        "-L", vendorLibraryDirectory,
    ]),
    .linkedLibrary("hc_ffi"),
]

let package = Package(
    name: "HaneulchiApp",
    platforms: [
        .macOS(.v15),
    ],
    products: [
        .library(name: "HaneulchiAppUI", targets: ["HaneulchiApp"]),
        .executable(name: "HaneulchiApp", targets: ["HaneulchiAppExecutable"]),
    ],
    dependencies: [
        .package(url: "https://github.com/migueldeicaza/SwiftTerm.git", exact: "1.12.0"),
    ],
    targets: [
        .plugin(
            name: "HCCoreFFIBuildPlugin",
            capability: .buildTool(),
        ),
        .target(
            name: "HCCoreFFI",
            path: "Vendor/HCCoreFFI",
            publicHeadersPath: "include",
        ),
        .target(
            name: "HaneulchiApp",
            dependencies: [
                "HCCoreFFI",
                .product(name: "SwiftTerm", package: "SwiftTerm"),
            ],
            path: "Sources/HaneulchiApp",
            linkerSettings: hcFFILinkerSettings,
            plugins: [
                "HCCoreFFIBuildPlugin",
            ],
        ),
        .executableTarget(
            name: "HaneulchiAppExecutable",
            dependencies: [
                "HaneulchiApp",
            ],
            path: "Sources/HaneulchiAppExecutable",
        ),
        .testTarget(
            name: "HaneulchiAppTests",
            dependencies: ["HaneulchiApp"],
            path: "Tests/HaneulchiAppTests",
            linkerSettings: hcFFILinkerSettings,
        ),
    ],
)
