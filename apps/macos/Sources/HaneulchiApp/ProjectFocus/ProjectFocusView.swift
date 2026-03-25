import SwiftUI

struct ProjectFocusView: View {
    struct Model: Equatable, Sendable {
        let deck: TerminalDeckView.Model
        let projectRoot: String?

        static let demo = Self(deck: .demo, projectRoot: nil)
        static let runtimeDemo = Self(deck: .runtimeDemo, projectRoot: nil)

        static func restored(_ bundle: TerminalRestoreBundle) -> Self {
            Self(deck: .restored(bundle), projectRoot: bundle.launch.currentDirectory)
        }

        static func bootstrap(
            selectedProjectRoot: String? = nil,
            restoreStore: TerminalSessionRestoreStore,
            recoverableSessions: [RecoverableSessionPayload] = []
        ) throws -> Self {
            let bundles = try restoreStore.load()
            let recoverableBundles = recoverableSessions
                .filter(\.isRecoverable)
                .map { TerminalRestoreBundle.genericShell(at: $0.cwd) }

            if let selectedProjectRoot {
                if let bundle = compatibleBundle(for: selectedProjectRoot, bundles: bundles) {
                    return .restored(bundle)
                }
                if let bundle = compatibleBundle(for: selectedProjectRoot, bundles: recoverableBundles) {
                    return .restored(bundle)
                }
                return .restored(.genericShell(at: selectedProjectRoot))
            }

            if let bundle = bundles.first {
                return .restored(bundle)
            }
            if let bundle = recoverableBundles.first {
                return .restored(bundle)
            }

            return .demo
        }

        private static func compatibleBundle(
            for selectedProjectRoot: String,
            bundles: [TerminalRestoreBundle]
        ) -> TerminalRestoreBundle? {
            bundles.first(where: { bundle in
                guard let currentDirectory = bundle.launch.currentDirectory else {
                    return false
                }
                return currentDirectory == selectedProjectRoot
                    || currentDirectory.hasPrefix(selectedProjectRoot + "/")
            })
        }
    }

    let model: Model
    let snapshot: AppShellSnapshot?
    let queuedFilePath: String?
    let onAction: (AppShellAction) -> Void
    @State private var workspaceState: ProjectFocusWorkspaceState
    private let fileIndex = ProjectFileIndex()

    init(
        model: Model,
        snapshot: AppShellSnapshot? = nil,
        queuedFilePath: String? = nil,
        onAction: @escaping (AppShellAction) -> Void = { _ in }
    ) {
        self.model = model
        self.snapshot = snapshot
        self.queuedFilePath = queuedFilePath
        self.onAction = onAction
        _workspaceState = State(initialValue: ProjectFocusWorkspaceState(projectRoot: model.projectRoot))
    }

    var body: some View {
        VStack(spacing: 0) {
            headerBar

            HStack(spacing: 0) {
                if let snapshot, !snapshot.sessions.isEmpty {
                    SessionStackView(
                        rows: SessionStackView.rows(from: snapshot),
                        onAction: onAction
                    )
                }

                if workspaceState.layoutPreset == .explorerTerminalInspector {
                    FilesPanelView(workspaceState: $workspaceState)
                }

                TerminalDeckView(
                    model: model.deck,
                    signalPresentation: focusedSessionSignal,
                    onQuickDispatch: {
                        onAction(.presentQuickDispatch(.projectFocus))
                    },
                    onSessionReady: { sessionID in
                        onAction(.terminalSessionReady(sessionID))
                    }
                )
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)

                if workspaceState.layoutPreset == .explorerTerminalInspector {
                    VStack(spacing: 0) {
                        if workspaceState.isEditing {
                            QuickEditView(workspaceState: $workspaceState)
                                .frame(maxWidth: .infinity, alignment: .topLeading)
                        } else {
                            QuickPreviewView(workspaceState: workspaceState) {
                                workspaceState.enterQuickEdit()
                            }
                            .frame(maxWidth: .infinity, alignment: .topLeading)
                        }

                        InspectorPanelView(
                            workspaceState: $workspaceState,
                            snapshot: snapshot,
                            onAction: onAction
                        )
                    }
                }
            }
        }
        .task(id: model.projectRoot) {
            guard let projectRoot = model.projectRoot else {
                return
            }

            workspaceState.layoutPreset = .explorerTerminalInspector
            workspaceState.fileEntries = (try? await fileIndex.index(rootPath: projectRoot)) ?? []
        }
        .onChange(of: queuedFilePath) { _, queuedFilePath in
            guard let queuedFilePath else {
                return
            }
            workspaceState.layoutPreset = .explorerTerminalInspector
            workspaceState.openFile(queuedFilePath)
        }
    }

    private var headerBar: some View {
        HStack(spacing: 8) {
            Text("Project Focus")
                .font(.headline)
            Spacer()
            Button("Full Terminal") {
                workspaceState.layoutPreset = .fullTerminal
            }
            .buttonStyle(.bordered)
            Button("Explorer + Inspector") {
                workspaceState.layoutPreset = .explorerTerminalInspector
            }
            .buttonStyle(.bordered)
        }
        .padding(.horizontal, HaneulchiChrome.Spacing.panelPadding)
        .padding(.vertical, 12)
    }

    private var focusedSessionSignal: SessionSignalPresentation? {
        guard let snapshot else {
            return nil
        }

        guard let focusedSession = snapshot.sessions.first(where: {
            $0.focusState == .focused || snapshot.app.focusedSessionID == $0.sessionID
        }) ?? snapshot.sessions.first else {
            return nil
        }

        return SessionSignalPresentation.from(session: focusedSession, isFocused: true)
    }
}
