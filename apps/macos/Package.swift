// swift-tools-version: 6.2
import PackageDescription

let package = Package(
    name: "HaneulchiApp",
    platforms: [
        .macOS(.v15),
    ],
    products: [
        .executable(name: "HaneulchiApp", targets: ["HaneulchiApp"]),
    ],
    targets: [
        .executableTarget(
            name: "HaneulchiApp",
            path: "Sources/HaneulchiApp"
        ),
        .testTarget(
            name: "HaneulchiAppTests",
            dependencies: ["HaneulchiApp"],
            path: "Tests/HaneulchiAppTests"
        ),
    ]
)
