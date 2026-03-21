import Foundation
import SQLite3

enum TaskSearchState: String, Codable, Equatable, Sendable {
    case inbox
    case ready
    case running
    case review
    case blocked
    case done
}

enum TaskAutomationMode: String, Codable, Equatable, Sendable {
    case manual
    case assisted
    case autoEligible = "auto-eligible"
}

struct TaskSearchProjectionStore: Sendable {
    struct Row: Equatable, Sendable {
        let taskID: String
        let projectID: String
        let title: String
        let state: TaskSearchState
        let automationMode: TaskAutomationMode
        let linkedSessionID: String?
    }

    let search: @Sendable (String) throws -> [Row]
    let upsert: @Sendable ([Row]) throws -> Void
    let createDraft: @Sendable (String, String?) throws -> Row

    static var inMemory: Self {
        makeStore(path: nil)
    }

    static var liveDefault: Self {
        fileBacked(fileURL: defaultFileURL)
    }

    static func fileBacked(fileURL: URL) -> Self {
        makeStore(path: fileURL.path)
    }

    private static func makeStore(path: String?) -> Self {
        if let sqliteStore = try? sqliteStore(path: path) {
            return sqliteStore
        }

        if path != nil, let fallbackStore = try? sqliteStore(path: nil) {
            return fallbackStore
        }

        return unavailableStore()
    }

    private static var defaultFileURL: URL {
        let applicationSupport =
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Library/Application Support")

        return applicationSupport
            .appendingPathComponent("Haneulchi", isDirectory: true)
            .appendingPathComponent("task-search.sqlite")
    }
}

private final class SQLiteTaskSearchDatabase: @unchecked Sendable {
    private let lock = NSLock()
    private let connection: OpaquePointer?

    init(path: String?) throws {
        var db: OpaquePointer?
        let targetPath = path ?? ":memory:"

        if sqlite3_open(targetPath, &db) != SQLITE_OK {
            defer { sqlite3_close(db) }
            throw SQLiteTaskSearchError.openFailed(message: Self.errorMessage(for: db))
        }

        connection = db
        try execute(
            """
            CREATE TABLE IF NOT EXISTS task_search_projection (
                task_id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                state TEXT NOT NULL,
                automation_mode TEXT NOT NULL,
                linked_session_id TEXT
            );
            """
        )
    }

    deinit {
        sqlite3_close(connection)
    }

    func upsert(_ rows: [TaskSearchProjectionStore.Row]) throws {
        lock.lock()
        defer { lock.unlock() }

        try execute("BEGIN IMMEDIATE TRANSACTION;")
        do {
            let statement = try prepare(
                """
                INSERT INTO task_search_projection (
                    task_id,
                    project_id,
                    title,
                    state,
                    automation_mode,
                    linked_session_id
                ) VALUES (?, ?, ?, ?, ?, ?)
                ON CONFLICT(task_id) DO UPDATE SET
                    project_id = excluded.project_id,
                    title = excluded.title,
                    state = excluded.state,
                    automation_mode = excluded.automation_mode,
                    linked_session_id = excluded.linked_session_id;
                """
            )
            defer { sqlite3_finalize(statement) }

            for row in rows {
                try bind(row.taskID, at: 1, in: statement)
                try bind(row.projectID, at: 2, in: statement)
                try bind(row.title, at: 3, in: statement)
                try bind(row.state.rawValue, at: 4, in: statement)
                try bind(row.automationMode.rawValue, at: 5, in: statement)
                try bind(row.linkedSessionID, at: 6, in: statement)

                guard sqlite3_step(statement) == SQLITE_DONE else {
                    throw SQLiteTaskSearchError.statementFailed(message: Self.errorMessage(for: connection))
                }

                sqlite3_reset(statement)
                sqlite3_clear_bindings(statement)
            }

            try execute("COMMIT;")
        } catch {
            try? execute("ROLLBACK;")
            throw error
        }
    }

    func search(query: String) throws -> [TaskSearchProjectionStore.Row] {
        lock.lock()
        defer { lock.unlock() }

        let trimmedQuery = query.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        let statement: OpaquePointer?

        if trimmedQuery.isEmpty {
            statement = try prepare(
                """
                SELECT task_id, project_id, title, state, automation_mode, linked_session_id
                FROM task_search_projection
                ORDER BY title COLLATE NOCASE ASC
                LIMIT 50;
                """
            )
        } else {
            statement = try prepare(
                """
                SELECT task_id, project_id, title, state, automation_mode, linked_session_id
                FROM task_search_projection
                WHERE lower(title) LIKE ? OR lower(task_id) LIKE ?
                ORDER BY title COLLATE NOCASE ASC
                LIMIT 50;
                """
            )

            let pattern = "%\(trimmedQuery)%"
            try bind(pattern, at: 1, in: statement)
            try bind(pattern, at: 2, in: statement)
        }

        defer { sqlite3_finalize(statement) }

        var results: [TaskSearchProjectionStore.Row] = []
        while sqlite3_step(statement) == SQLITE_ROW {
            let taskID = String(cString: sqlite3_column_text(statement, 0))
            let projectID = String(cString: sqlite3_column_text(statement, 1))
            let title = String(cString: sqlite3_column_text(statement, 2))
            let stateRaw = String(cString: sqlite3_column_text(statement, 3))
            let automationModeRaw = String(cString: sqlite3_column_text(statement, 4))
            let linkedSessionID = sqlite3_column_type(statement, 5) == SQLITE_NULL
                ? nil
                : String(cString: sqlite3_column_text(statement, 5))

            guard
                let state = TaskSearchState(rawValue: stateRaw),
                let automationMode = TaskAutomationMode(rawValue: automationModeRaw)
            else {
                throw SQLiteTaskSearchError.statementFailed(message: "Unexpected enum value in task_search_projection")
            }

            results.append(.init(
                taskID: taskID,
                projectID: projectID,
                title: title,
                state: state,
                automationMode: automationMode,
                linkedSessionID: linkedSessionID
            ))
        }

        return results
    }

    func createDraft(title: String, projectID: String?) throws -> TaskSearchProjectionStore.Row {
        let taskID = "task_\(UUID().uuidString.replacingOccurrences(of: "-", with: "").prefix(8))"
        let row = TaskSearchProjectionStore.Row(
            taskID: taskID,
            projectID: projectID ?? "local",
            title: title,
            state: .inbox,
            automationMode: .manual,
            linkedSessionID: nil
        )
        try upsert([row])
        return row
    }

    private func execute(_ sql: String) throws {
        guard sqlite3_exec(connection, sql, nil, nil, nil) == SQLITE_OK else {
            throw SQLiteTaskSearchError.statementFailed(message: Self.errorMessage(for: connection))
        }
    }

    private func prepare(_ sql: String) throws -> OpaquePointer? {
        var statement: OpaquePointer?
        guard sqlite3_prepare_v2(connection, sql, -1, &statement, nil) == SQLITE_OK else {
            throw SQLiteTaskSearchError.statementFailed(message: Self.errorMessage(for: connection))
        }

        return statement
    }

    private func bind(_ value: String, at index: Int32, in statement: OpaquePointer?) throws {
        guard sqlite3_bind_text(statement, index, value, -1, SQLITE_TRANSIENT) == SQLITE_OK else {
            throw SQLiteTaskSearchError.statementFailed(message: Self.errorMessage(for: connection))
        }
    }

    private func bind(_ value: String?, at index: Int32, in statement: OpaquePointer?) throws {
        guard let value else {
            guard sqlite3_bind_null(statement, index) == SQLITE_OK else {
                throw SQLiteTaskSearchError.statementFailed(message: Self.errorMessage(for: connection))
            }
            return
        }

        try bind(value, at: index, in: statement)
    }

    private static func errorMessage(for connection: OpaquePointer?) -> String {
        guard let connection, let message = sqlite3_errmsg(connection) else {
            return "unknown sqlite error"
        }

        return String(cString: message)
    }
}

private extension TaskSearchProjectionStore {
    static func sqliteStore(path: String?) throws -> Self {
        if let path {
            let parent = URL(fileURLWithPath: path).deletingLastPathComponent()
            try FileManager.default.createDirectory(at: parent, withIntermediateDirectories: true)
        }

        let database = try SQLiteTaskSearchDatabase(path: path)
        return Self(
            search: { query in
                try database.search(query: query)
            },
            upsert: { rows in
                try database.upsert(rows)
            },
            createDraft: { title, projectID in
                try database.createDraft(title: title, projectID: projectID)
            }
        )
    }

    static func unavailableStore() -> Self {
        Self(
            search: { _ in
                throw SQLiteTaskSearchError.statementFailed(message: "task search store unavailable")
            },
            upsert: { _ in
                throw SQLiteTaskSearchError.statementFailed(message: "task search store unavailable")
            },
            createDraft: { _, _ in
                throw SQLiteTaskSearchError.statementFailed(message: "task search store unavailable")
            }
        )
    }
}

private enum SQLiteTaskSearchError: Error {
    case openFailed(message: String)
    case statementFailed(message: String)
}

private let SQLITE_TRANSIENT = unsafeBitCast(-1, to: sqlite3_destructor_type.self)
