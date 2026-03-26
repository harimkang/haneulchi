import Foundation
import HCCoreFFI

enum CoreBridgeError: Error, Equatable {
    case invalidStringPayload
    case invalidBytesPayload
    case invalidRuntimeInfo
    case invalidSessionSnapshot
    case invalidSessionDetails
    case invalidSpawnResponse
    case operationFailed(String)
}

struct CoreBridge: Sendable {
    let provisionTaskWorkspace: @Sendable (String, String, String?) throws
        -> ProvisionedTaskWorkspace
    let runtimeInfo: @Sendable () throws -> TerminalBackendDescriptor
    let spawnSession: @Sendable (TerminalSessionLaunchRequest) throws -> TerminalSessionSnapshot
    let drainSession: @Sendable (String) throws -> Data
    let writeSession: @Sendable (String, Data) throws -> Void
    let resizeSession: @Sendable (String, TerminalGridSize) throws -> Void
    let terminateSession: @Sendable (String) throws -> Void
    let snapshotSession: @Sendable (String) throws -> TerminalSessionSnapshot
    let stateSnapshot: @Sendable () throws -> AppShellSnapshot
    let stateSnapshotJSON: @Sendable () throws -> String
    let sessionsList: @Sendable () throws -> [AppShellSnapshot.SessionSummary]
    let sessionDetails: @Sendable (String) throws -> SessionDetailsPayload
    let prepareIsolatedLaunch: @Sendable (String, String, String, String, String) throws
        -> WorkflowStatusPayload.BootstrapSummary
    let focusSession: @Sendable (String) throws -> Void
    let takeoverSession: @Sendable (String) throws -> Void
    let releaseTakeoverSession: @Sendable (String) throws -> Void
    let resolveAttention: @Sendable (String) throws -> Void
    let dismissAttention: @Sendable (String) throws -> Void
    let snoozeAttention: @Sendable (String) throws -> Void
    let dispatchSend: @Sendable (String, String?, String) throws -> Void
    let startLocalControlServer: @Sendable () throws -> Void
    let reconcileAutomation: @Sendable () throws -> Void
    let workflowValidate: @Sendable (String) throws -> Data
    let workflowReload: @Sendable (String) throws -> Data
    let inventorySummary: @Sendable (String) throws -> InventorySummaryPayload
    let inventoryList: @Sendable (String) throws -> [InventoryRowPayload]
    let terminalSettings: @Sendable () throws -> TerminalSettingsPayload?
    let resolveLaunchEnvironment: @Sendable () throws -> [String: String]
    let runtimeInfoSummary: @Sendable () throws -> RuntimeInfoSummaryPayload
    let listDegradedIssues: @Sendable (RecoveryContextPayload) throws -> [DegradedIssuePayload]
    let loadAppState: @Sendable () throws -> AppStatePayload?
    let saveAppState: @Sendable (String, String?, String?) throws -> Void
    let listRecoverableSessions: @Sendable (String) throws -> [RecoverableSessionPayload]
    let setWorktreePinned: @Sendable (String, Bool) throws -> Void
    let updateWorktreeLifecycle: @Sendable (String, String) throws -> Void

    init(
        provisionTaskWorkspace: @escaping @Sendable (String, String, String?) throws
            -> ProvisionedTaskWorkspace = { _, _, _ in
                throw CoreBridgeError.operationFailed("task_workspace_provision_unavailable")
            },
        runtimeInfo: @escaping @Sendable () throws -> TerminalBackendDescriptor,
        spawnSession: @escaping @Sendable (TerminalSessionLaunchRequest) throws
            -> TerminalSessionSnapshot,
        drainSession: @escaping @Sendable (String) throws -> Data,
        writeSession: @escaping @Sendable (String, Data) throws -> Void,
        resizeSession: @escaping @Sendable (String, TerminalGridSize) throws -> Void,
        terminateSession: @escaping @Sendable (String) throws -> Void,
        snapshotSession: @escaping @Sendable (String) throws -> TerminalSessionSnapshot,
        stateSnapshot: @escaping @Sendable () throws -> AppShellSnapshot = {
            throw CoreBridgeError.operationFailed("state_snapshot_unavailable")
        },
        stateSnapshotJSON: @escaping @Sendable () throws -> String = {
            throw CoreBridgeError.operationFailed("state_snapshot_unavailable")
        },
        sessionsList: @escaping @Sendable () throws -> [AppShellSnapshot.SessionSummary] = {
            throw CoreBridgeError.operationFailed("sessions_list_unavailable")
        },
        sessionDetails: @escaping @Sendable (String) throws -> SessionDetailsPayload = { _ in
            throw CoreBridgeError.operationFailed("session_details_unavailable")
        },
        prepareIsolatedLaunch: @escaping @Sendable (String, String, String, String, String) throws
            -> WorkflowStatusPayload.BootstrapSummary = { _, _, _, _, _ in
                throw CoreBridgeError.operationFailed("prepare_isolated_launch_unavailable")
            },
        focusSession: @escaping @Sendable (String) throws -> Void = { _ in
            throw CoreBridgeError.operationFailed("session_focus_unavailable")
        },
        takeoverSession: @escaping @Sendable (String) throws -> Void = { _ in
            throw CoreBridgeError.operationFailed("session_takeover_unavailable")
        },
        releaseTakeoverSession: @escaping @Sendable (String) throws -> Void = { _ in
            throw CoreBridgeError.operationFailed("session_release_takeover_unavailable")
        },
        resolveAttention: @escaping @Sendable (String) throws -> Void = { _ in
            throw CoreBridgeError.operationFailed("attention_resolve_unavailable")
        },
        dismissAttention: @escaping @Sendable (String) throws -> Void = { _ in
            throw CoreBridgeError.operationFailed("attention_dismiss_unavailable")
        },
        snoozeAttention: @escaping @Sendable (String) throws -> Void = { _ in
            throw CoreBridgeError.operationFailed("attention_snooze_unavailable")
        },
        dispatchSend: @escaping @Sendable (String, String?, String) throws -> Void = { _, _, _ in
            throw CoreBridgeError.operationFailed("dispatch_send_unavailable")
        },
        startLocalControlServer: @escaping @Sendable () throws -> Void = {},
        reconcileAutomation: @escaping @Sendable () throws -> Void = {
            throw CoreBridgeError.operationFailed("reconcile_unavailable")
        },
        workflowValidate: @escaping @Sendable (String) throws -> Data = { _ in
            throw CoreBridgeError.operationFailed("workflow_validate_unavailable")
        },
        workflowReload: @escaping @Sendable (String) throws -> Data = { _ in
            throw CoreBridgeError.operationFailed("workflow_reload_unavailable")
        },
        inventorySummary: @escaping @Sendable (String) throws -> InventorySummaryPayload = { _ in
            throw CoreBridgeError.operationFailed("inventory_summary_unavailable")
        },
        inventoryList: @escaping @Sendable (String) throws -> [InventoryRowPayload] = { _ in
            throw CoreBridgeError.operationFailed("inventory_list_unavailable")
        },
        terminalSettings: @escaping @Sendable () throws -> TerminalSettingsPayload? = {
            throw CoreBridgeError.operationFailed("terminal_settings_unavailable")
        },
        resolveLaunchEnvironment: @escaping @Sendable () throws -> [String: String] = { [:] },
        runtimeInfoSummary: @escaping @Sendable () throws -> RuntimeInfoSummaryPayload = {
            throw CoreBridgeError.operationFailed("runtime_info_summary_unavailable")
        },
        listDegradedIssues: @escaping @Sendable (RecoveryContextPayload) throws
            -> [DegradedIssuePayload] = { _ in
                throw CoreBridgeError.operationFailed("list_degraded_issues_unavailable")
            },
        loadAppState: @escaping @Sendable () throws -> AppStatePayload? = {
            nil
        },
        saveAppState: @escaping @Sendable (String, String?, String?) throws -> Void = { _, _, _ in
        },
        listRecoverableSessions: @escaping @Sendable (String) throws
            -> [RecoverableSessionPayload] = { _ in
                []
            },
        setWorktreePinned: @escaping @Sendable (String, Bool) throws -> Void = { _, _ in },
        updateWorktreeLifecycle: @escaping @Sendable (String, String) throws -> Void = { _, _ in },
    ) {
        self.provisionTaskWorkspace = provisionTaskWorkspace
        self.runtimeInfo = runtimeInfo
        self.spawnSession = spawnSession
        self.drainSession = drainSession
        self.writeSession = writeSession
        self.resizeSession = resizeSession
        self.terminateSession = terminateSession
        self.snapshotSession = snapshotSession
        self.stateSnapshot = stateSnapshot
        self.stateSnapshotJSON = stateSnapshotJSON
        self.sessionsList = sessionsList
        self.sessionDetails = sessionDetails
        self.prepareIsolatedLaunch = prepareIsolatedLaunch
        self.focusSession = focusSession
        self.takeoverSession = takeoverSession
        self.releaseTakeoverSession = releaseTakeoverSession
        self.resolveAttention = resolveAttention
        self.dismissAttention = dismissAttention
        self.snoozeAttention = snoozeAttention
        self.dispatchSend = dispatchSend
        self.startLocalControlServer = startLocalControlServer
        self.reconcileAutomation = reconcileAutomation
        self.workflowValidate = workflowValidate
        self.workflowReload = workflowReload
        self.inventorySummary = inventorySummary
        self.inventoryList = inventoryList
        self.terminalSettings = terminalSettings
        self.resolveLaunchEnvironment = resolveLaunchEnvironment
        self.runtimeInfoSummary = runtimeInfoSummary
        self.listDegradedIssues = listDegradedIssues
        self.loadAppState = loadAppState
        self.saveAppState = saveAppState
        self.listRecoverableSessions = listRecoverableSessions
        self.setWorktreePinned = setWorktreePinned
        self.updateWorktreeLifecycle = updateWorktreeLifecycle
    }

    static let live = Self(
        provisionTaskWorkspace: { projectRoot, taskID, baseRoot in
            let root = try CStringBox(projectRoot)
            let task = try CStringBox(taskID)
            let payload = try root.withPointer { rootPointer in
                try task.withPointer { taskPointer in
                    if let baseRoot, !baseRoot.isEmpty {
                        let base = try CStringBox(baseRoot)
                        return try base.withPointer { basePointer in
                            try stringPayloadData(
                                hc_task_provision_workspace_json(
                                    rootPointer,
                                    taskPointer,
                                    basePointer,
                                ),
                            )
                        }
                    }
                    return try stringPayloadData(
                        hc_task_provision_workspace_json(rootPointer, taskPointer, nil),
                    )
                }
            }
            return try decodeProvisionedTaskWorkspace(from: payload)
        },
        runtimeInfo: {
            let data = try stringPayloadData(hc_runtime_info_json())

            guard let descriptor = try? JSONDecoder().decode(
                TerminalBackendDescriptor.self,
                from: data,
            ) else {
                throw CoreBridgeError.invalidRuntimeInfo
            }

            return descriptor
        },
        spawnSession: { request in
            let payload = try JSONEncoder().encode(request)
            let json = String(decoding: payload, as: UTF8.self)
            let config = try CStringBox(json)
            do {
                let responseData = try config.withPointer { pointer in
                    try stringPayloadData(hc_terminal_session_spawn_json(pointer))
                }
                let sessionID = try decodeSpawnSessionID(from: responseData)
                let session = try CStringBox(sessionID)
                let snapshotData = try session.withPointer { pointer in
                    try stringPayloadData(hc_terminal_session_snapshot_json(pointer))
                }
                return try decodeSessionSnapshot(from: snapshotData)
            } catch {
                throw error
            }
        },
        drainSession: { sessionID in
            let session = try CStringBox(sessionID)
            let payload = session.withPointer { hc_terminal_session_drain($0) }
            defer { hc_bytes_free(payload) }

            guard let pointer = payload.ptr else {
                return Data()
            }

            return Data(bytes: pointer, count: payload.len)
        },
        writeSession: { sessionID, data in
            let session = try CStringBox(sessionID)
            let result = session.withPointer { pointer in
                data.withUnsafeBytes { rawBuffer in
                    hc_terminal_session_write(
                        pointer,
                        rawBuffer.baseAddress?.assumingMemoryBound(to: UInt8.self),
                        rawBuffer.count,
                    )
                }
            }

            guard result == 0 else {
                throw CoreBridgeError.operationFailed("session_write_failed")
            }
        },
        resizeSession: { sessionID, geometry in
            let session = try CStringBox(sessionID)
            let result = session.withPointer {
                hc_terminal_session_resize(
                    $0,
                    UInt16(geometry.cols),
                    UInt16(geometry.rows),
                )
            }

            guard result == 0 else {
                throw CoreBridgeError.operationFailed("session_resize_failed")
            }
        },
        terminateSession: { sessionID in
            let session = try CStringBox(sessionID)
            let result = session.withPointer { hc_terminal_session_terminate($0) }

            guard result == 0 else {
                throw CoreBridgeError.operationFailed("session_terminate_failed")
            }
        },
        snapshotSession: { sessionID in
            let session = try CStringBox(sessionID)
            let payload = try session.withPointer { pointer in
                try stringPayloadData(hc_terminal_session_snapshot_json(pointer))
            }
            return try decodeSessionSnapshot(from: payload)
        },
        stateSnapshot: {
            let payload = try stringPayloadData(hc_state_snapshot_json())
            return try decodeAppShellSnapshot(from: payload)
        },
        stateSnapshotJSON: {
            let payload = try stringPayloadData(hc_state_snapshot_json())
            return String(decoding: payload, as: UTF8.self)
        },
        sessionsList: {
            let payload = try stringPayloadData(hc_sessions_list_json())
            return try decodeSessionSummaries(from: payload)
        },
        sessionDetails: { sessionID in
            let session = try CStringBox(sessionID)
            let payload = try session.withPointer { pointer in
                try stringPayloadData(hc_session_details_json(pointer))
            }
            return try decodeSessionDetails(from: payload)
        },
        prepareIsolatedLaunch: { projectRoot, projectName, taskID, taskTitle, workspaceRoot in
            let root = try CStringBox(projectRoot)
            let name = try CStringBox(projectName)
            let task = try CStringBox(taskID)
            let title = try CStringBox(taskTitle)
            let workspace = try CStringBox(workspaceRoot)
            let payload = try root.withPointer { rootPointer in
                try name.withPointer { namePointer in
                    try task.withPointer { taskPointer in
                        try title.withPointer { titlePointer in
                            try workspace.withPointer { workspacePointer in
                                try stringPayloadData(
                                    hc_task_prepare_isolated_launch_json(
                                        rootPointer,
                                        namePointer,
                                        taskPointer,
                                        titlePointer,
                                        workspacePointer,
                                    ),
                                )
                            }
                        }
                    }
                }
            }
            return try decodeWorkflowBootstrapSummary(from: payload)
        },
        focusSession: { sessionID in
            let session = try CStringBox(sessionID)
            let result = session.withPointer { hc_session_focus($0) }
            guard result == 0 else {
                throw CoreBridgeError.operationFailed("session_focus_failed")
            }
        },
        takeoverSession: { sessionID in
            let session = try CStringBox(sessionID)
            let result = session.withPointer { hc_session_takeover($0) }
            guard result == 0 else {
                throw CoreBridgeError.operationFailed("session_takeover_failed")
            }
        },
        releaseTakeoverSession: { sessionID in
            let session = try CStringBox(sessionID)
            let result = session.withPointer { hc_session_release_takeover($0) }
            guard result == 0 else {
                throw CoreBridgeError.operationFailed("session_release_takeover_failed")
            }
        },
        resolveAttention: { attentionID in
            let attention = try CStringBox(attentionID)
            _ = try attention.withPointer { pointer in
                try stringPayloadData(hc_attention_resolve_json(pointer))
            }
        },
        dismissAttention: { attentionID in
            let attention = try CStringBox(attentionID)
            _ = try attention.withPointer { pointer in
                try stringPayloadData(hc_attention_dismiss_json(pointer))
            }
        },
        snoozeAttention: { attentionID in
            let attention = try CStringBox(attentionID)
            _ = try attention.withPointer { pointer in
                try stringPayloadData(hc_attention_snooze_json(pointer))
            }
        },
        dispatchSend: { targetSessionID, taskID, payload in
            let session = try CStringBox(targetSessionID)
            let payloadBox = try CStringBox(payload)
            let task = try taskID.map(CStringBox.init)
            _ = try session.withPointer { sessionPointer in
                try payloadBox.withPointer { payloadPointer in
                    if let task {
                        return try task.withPointer { taskPointer in
                            try stringPayloadData(
                                hc_dispatch_send_json(
                                    sessionPointer,
                                    taskPointer,
                                    true,
                                    payloadPointer,
                                ),
                            )
                        }
                    }
                    return try stringPayloadData(
                        hc_dispatch_send_json(sessionPointer, nil, true, payloadPointer),
                    )
                }
            }
        },
        startLocalControlServer: {
            _ = try stringPayloadData(hc_api_server_start_json(nil))
        },
        reconcileAutomation: {
            _ = try stringPayloadData(hc_reconcile_now_json())
        },
        workflowValidate: { projectRoot in
            let root = try CStringBox(projectRoot)
            return try root.withPointer { pointer in
                try stringPayloadData(hc_workflow_validate_json(pointer))
            }
        },
        workflowReload: { projectRoot in
            let root = try CStringBox(projectRoot)
            return try root.withPointer { pointer in
                try stringPayloadData(hc_workflow_reload_json(pointer))
            }
        },
        inventorySummary: { projectID in
            let pid = try CStringBox(projectID)
            let data = try pid.withPointer { pointer in
                try stringPayloadData(hc_inventory_summary_json(pointer))
            }
            guard let payload = try? JSONDecoder().decode(InventorySummaryPayload.self, from: data)
            else {
                throw CoreBridgeError.operationFailed("inventory_summary_decode_failed")
            }
            return payload
        },
        inventoryList: { projectID in
            let pid = try CStringBox(projectID)
            let data = try pid.withPointer { pointer in
                try stringPayloadData(hc_inventory_list_json(pointer))
            }
            guard let payload = try? JSONDecoder().decode([InventoryRowPayload].self, from: data)
            else {
                throw CoreBridgeError.operationFailed("inventory_list_decode_failed")
            }
            return payload
        },
        terminalSettings: {
            let data = try stringPayloadData(hc_terminal_settings_json())
            guard let payload = try? JSONDecoder().decode(TerminalSettingsPayload.self, from: data)
            else {
                throw CoreBridgeError.operationFailed("terminal_settings_decode_failed")
            }
            return payload
        },
        resolveLaunchEnvironment: {
            let data = try stringPayloadData(hc_resolve_secret_env_json())
            guard let payload = try? JSONDecoder().decode([String: String].self, from: data) else {
                throw CoreBridgeError.operationFailed("resolve_secret_env_decode_failed")
            }
            return payload
        },
        runtimeInfoSummary: {
            let data = try stringPayloadData(hc_runtime_info_summary_json())
            guard let payload = try? JSONDecoder()
                .decode(RuntimeInfoSummaryPayload.self, from: data)
            else {
                throw CoreBridgeError.operationFailed("runtime_info_summary_decode_failed")
            }
            return payload
        },
        listDegradedIssues: { context in
            let encoded = try JSONEncoder().encode(context)
            let json = String(decoding: encoded, as: UTF8.self)
            let contextBox = try CStringBox(json)
            let data = try contextBox.withPointer { pointer in
                try stringPayloadData(hc_degraded_issues_json(pointer))
            }
            return (try? JSONDecoder().decode([DegradedIssuePayload].self, from: data)) ?? []
        },
        loadAppState: {
            let data = try stringPayloadData(hc_load_app_state_json())
            return try? JSONDecoder().decode(AppStatePayload.self, from: data)
        },
        saveAppState: { route, projectId, sessionId in
            let payload = serde_encode_app_state(
                route: route,
                projectId: projectId,
                sessionId: sessionId,
            )
            let box = try CStringBox(payload)
            _ = try box.withPointer { pointer in
                try stringPayloadData(hc_save_app_state_json(pointer))
            }
        },
        listRecoverableSessions: { projectId in
            let pid = try CStringBox(projectId)
            let data = try pid.withPointer { pointer in
                try stringPayloadData(hc_list_recoverable_sessions_json(pointer))
            }
            return (try? JSONDecoder().decode([RecoverableSessionPayload].self, from: data)) ?? []
        },
        setWorktreePinned: { worktreeID, isPinned in
            let worktree = try CStringBox(worktreeID)
            _ = try worktree.withPointer { pointer in
                try stringPayloadData(hc_set_worktree_pinned_json(pointer, isPinned ? 1 : 0))
            }
        },
        updateWorktreeLifecycle: { worktreeID, state in
            let worktree = try CStringBox(worktreeID)
            let lifecycle = try CStringBox(state)
            _ = try worktree.withPointer { worktreePointer in
                try lifecycle.withPointer { statePointer in
                    try stringPayloadData(hc_update_worktree_lifecycle_json(
                        worktreePointer,
                        statePointer,
                    ))
                }
            }
        },
    )

    static func mockLiveSession(outputChunks: [String]) -> Self {
        let state = MockLiveSessionState(outputChunks: outputChunks)

        return Self(
            provisionTaskWorkspace: { projectRoot, taskID, baseRoot in
                ProvisionedTaskWorkspace(
                    taskID: taskID,
                    worktreeID: "wt_\(taskID)",
                    workspaceRoot: "\(projectRoot)/worktrees/\(taskID)",
                    baseRoot: baseRoot ?? ".",
                    branchName: "hc/\(taskID)",
                )
            },
            runtimeInfo: {
                TerminalBackendDescriptor(
                    rendererID: "swiftterm",
                    transport: "ffi_c_abi",
                    demoMode: false,
                )
            },
            spawnSession: { request in
                try state.spawn(request)
            },
            drainSession: { sessionID in
                try state.drain(sessionID: sessionID)
            },
            writeSession: { sessionID, data in
                try state.write(sessionID: sessionID, data: data)
            },
            resizeSession: { sessionID, geometry in
                try state.resize(sessionID: sessionID, geometry: geometry)
            },
            terminateSession: { sessionID in
                try state.terminate(sessionID: sessionID)
            },
            snapshotSession: { sessionID in
                try state.snapshot(sessionID: sessionID)
            },
            stateSnapshot: {
                AppShellSnapshot.empty(activeRoute: .projectFocus)
            },
            stateSnapshotJSON: {
                "{}"
            },
            sessionsList: {
                []
            },
            sessionDetails: { sessionID in
                SessionDetailsPayload(
                    sessionID: sessionID,
                    title: "Mock Session",
                    workflowBinding: .init(
                        state: .ok,
                        path: "/tmp/demo/WORKFLOW.md",
                        lastGoodHash: "sha256:mock",
                        lastReloadAt: "2026-03-22T00:00:00Z",
                        lastError: nil,
                    ),
                    recentEvents: [],
                )
            },
            prepareIsolatedLaunch: { _, _, _, _, workspaceRoot in
                WorkflowStatusPayload.BootstrapSummary(
                    workspaceRoot: workspaceRoot,
                    baseRoot: ".",
                    sessionCwd: workspaceRoot,
                    renderedPromptPath: workspaceRoot + "/prompt.rendered.md",
                    phaseSequence: ["resolve", "normalize", "workspace", "paths", "prompt"],
                    hookPhaseResults: [],
                    outcomeCode: "launch_prepared",
                    warningCodes: [],
                    claimReleased: false,
                    launchExitCode: nil,
                    lastKnownGoodHash: "sha256:mock",
                )
            },
            focusSession: { _ in },
            takeoverSession: { _ in },
            releaseTakeoverSession: { _ in },
            resolveAttention: { _ in },
            dismissAttention: { _ in },
            snoozeAttention: { _ in },
            dispatchSend: { _, _, _ in },
            startLocalControlServer: {},
            reconcileAutomation: {},
            workflowValidate: { _ in Data("{}".utf8) },
            workflowReload: { _ in Data("{}".utf8) },
            inventorySummary: { _ in
                InventorySummaryPayload(
                    total: 0,
                    inUse: 0,
                    recoverable: 0,
                    safeToDelete: 0,
                    stale: 0,
                )
            },
            inventoryList: { _ in [] },
            terminalSettings: {
                TerminalSettingsPayload(
                    shell: "/bin/zsh",
                    defaultCols: 220,
                    defaultRows: 50,
                    scrollbackLines: 5000,
                    fontName: "",
                    theme: "",
                    cursorStyle: "",
                )
            },
            resolveLaunchEnvironment: { [:] },
            runtimeInfoSummary: {
                RuntimeInfoSummaryPayload(
                    socketPath: nil,
                    transport: "ffi_c_abi",
                    status: "running",
                )
            },
            listDegradedIssues: { _ in [] },
            loadAppState: { nil },
            saveAppState: { _, _, _ in },
            listRecoverableSessions: { _ in [] },
            setWorktreePinned: { _, _ in },
            updateWorktreeLifecycle: { _, _ in },
        )
    }
}

private func serde_encode_app_state(route: String, projectId: String?,
                                    sessionId: String?) -> String
{
    var dict: [String: Any] = ["route": route]
    if let projectId { dict["last_project_id"] = projectId }
    if let sessionId { dict["last_session_id"] = sessionId }
    if let data = try? JSONSerialization.data(withJSONObject: dict),
       let json = String(data: data, encoding: .utf8)
    {
        return json
    }
    return "{}"
}

private func stringPayloadData(_ payload: HcString) throws -> Data {
    defer {
        hc_string_free(payload)
    }

    guard let pointer = payload.ptr else {
        throw CoreBridgeError.invalidStringPayload
    }

    guard let json = String(validatingCString: pointer) else {
        throw CoreBridgeError.invalidStringPayload
    }

    if
        let data = json.data(using: .utf8),
        let response = try? JSONDecoder().decode(CoreBridgeErrorPayload.self, from: data),
        let error = response.error
    {
        throw CoreBridgeError.operationFailed(error)
    }

    return Data(json.utf8)
}

private func decodeSessionSnapshot(from data: Data) throws -> TerminalSessionSnapshot {
    guard let snapshot = try? JSONDecoder().decode(TerminalSessionSnapshot.self, from: data) else {
        throw CoreBridgeError.invalidSessionSnapshot
    }

    return snapshot
}

private func decodeAppShellSnapshot(from data: Data) throws -> AppShellSnapshot {
    guard let snapshot = try? JSONDecoder().decode(AppShellSnapshot.self, from: data) else {
        throw CoreBridgeError.invalidRuntimeInfo
    }

    return snapshot
}

private func decodeProvisionedTaskWorkspace(from data: Data) throws -> ProvisionedTaskWorkspace {
    guard let workspace = try? JSONDecoder().decode(ProvisionedTaskWorkspace.self, from: data)
    else {
        throw CoreBridgeError.invalidRuntimeInfo
    }

    return workspace
}

private func decodeSessionSummaries(from data: Data) throws -> [AppShellSnapshot.SessionSummary] {
    guard let sessions = try? JSONDecoder().decode(
        [AppShellSnapshot.SessionSummary].self,
        from: data,
    ) else {
        throw CoreBridgeError.invalidRuntimeInfo
    }

    return sessions
}

private func decodeSessionDetails(from data: Data) throws -> SessionDetailsPayload {
    guard let payload = try? JSONDecoder().decode(SessionDetailsPayload.self, from: data) else {
        throw CoreBridgeError.invalidSessionDetails
    }

    return payload
}

private func decodeWorkflowBootstrapSummary(from data: Data) throws
    -> WorkflowStatusPayload.BootstrapSummary
{
    guard let payload = try? JSONDecoder().decode(WorkflowStatusPayload.BootstrapSummary.self, from: data)
    else {
        throw CoreBridgeError.invalidRuntimeInfo
    }

    return payload
}

private struct SpawnSessionResponse: Decodable {
    let sessionID: String

    enum CodingKeys: String, CodingKey {
        case sessionID = "session_id"
    }
}

func decodeSpawnSessionID(from data: Data) throws -> String {
    guard let response = try? JSONDecoder().decode(SpawnSessionResponse.self, from: data) else {
        throw CoreBridgeError.invalidSpawnResponse
    }

    return response.sessionID
}

private struct CoreBridgeErrorPayload: Decodable {
    let error: String?
}

private struct CStringBox {
    private let storage: [CChar]

    init(_ string: String) throws {
        storage = Array(string.utf8CString)
    }

    func withPointer<T>(_ body: (UnsafePointer<CChar>) throws -> T) rethrows -> T {
        try storage.withUnsafeBufferPointer { buffer in
            try body(buffer.baseAddress!)
        }
    }
}

private final class MockLiveSessionState: @unchecked Sendable {
    private let lock = NSLock()
    private var snapshot: TerminalSessionSnapshot?
    private var pendingChunks: [Data]

    init(outputChunks: [String]) {
        pendingChunks = outputChunks.map { Data($0.utf8) }
    }

    func spawn(_ request: TerminalSessionLaunchRequest) throws -> TerminalSessionSnapshot {
        let snapshot = TerminalSessionSnapshot(
            sessionID: "mock-session",
            launch: request,
            geometry: request.geometry,
            running: true,
            exitCode: nil,
        )

        lock.lock()
        self.snapshot = snapshot
        lock.unlock()

        return snapshot
    }

    func drain(sessionID: String) throws -> Data {
        lock.lock()
        defer { lock.unlock() }

        guard snapshot?.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        if pendingChunks.isEmpty {
            return Data()
        }

        return pendingChunks.removeFirst()
    }

    func write(sessionID: String, data: Data) throws {
        lock.lock()
        defer { lock.unlock() }

        guard snapshot?.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        pendingChunks.append(data)
    }

    func resize(sessionID: String, geometry: TerminalGridSize) throws {
        lock.lock()
        defer { lock.unlock() }

        guard var snapshot, snapshot.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        snapshot = TerminalSessionSnapshot(
            sessionID: snapshot.sessionID,
            launch: snapshot.launch,
            geometry: geometry,
            running: snapshot.running,
            exitCode: snapshot.exitCode,
        )
        self.snapshot = snapshot
    }

    func terminate(sessionID: String) throws {
        lock.lock()
        defer { lock.unlock() }

        guard var snapshot, snapshot.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        snapshot = TerminalSessionSnapshot(
            sessionID: snapshot.sessionID,
            launch: snapshot.launch,
            geometry: snapshot.geometry,
            running: false,
            exitCode: 0,
        )
        self.snapshot = snapshot
    }

    func snapshot(sessionID: String) throws -> TerminalSessionSnapshot {
        lock.lock()
        defer { lock.unlock() }

        guard let snapshot, snapshot.sessionID == sessionID else {
            throw CoreBridgeError.operationFailed("session_not_found")
        }

        return snapshot
    }
}
