use hc_domain::ReviewItem;
use rusqlite::{Connection, OptionalExtension, params};

use crate::StorageError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewReviewItem {
    pub id: String,
    pub task_id: String,
    pub session_id: Option<String>,
    pub summary: String,
    pub created_at: String,
}

pub struct ReviewRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> ReviewRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn create_pending(&self, input: NewReviewItem) -> Result<ReviewItem, StorageError> {
        let review = ReviewItem::new_pending(
            input.id,
            input.task_id,
            input.session_id,
            input.summary,
            input.created_at,
        );

        self.save(review.clone())
    }

    pub fn save(&self, review: ReviewItem) -> Result<ReviewItem, StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO review_items (
                id,
                task_id,
                session_id,
                status,
                summary,
                touched_files_json,
                diff_summary,
                tests_summary,
                command_summary,
                hook_summary,
                evidence_summary,
                checklist_summary,
                warnings_json,
                evidence_manifest_path,
                review_checklist_result,
                created_at,
                decided_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(id) DO UPDATE SET
                task_id = excluded.task_id,
                session_id = excluded.session_id,
                status = excluded.status,
                summary = excluded.summary,
                touched_files_json = excluded.touched_files_json,
                diff_summary = excluded.diff_summary,
                tests_summary = excluded.tests_summary,
                command_summary = excluded.command_summary,
                hook_summary = excluded.hook_summary,
                evidence_summary = excluded.evidence_summary,
                checklist_summary = excluded.checklist_summary,
                warnings_json = excluded.warnings_json,
                evidence_manifest_path = excluded.evidence_manifest_path,
                review_checklist_result = excluded.review_checklist_result,
                created_at = excluded.created_at,
                decided_at = excluded.decided_at
            "#,
            params![
                &review.id,
                &review.task_id,
                &review.session_id,
                review.status.as_str(),
                &review.summary,
                serde_json::to_string(&review.touched_files)?,
                &review.diff_summary,
                &review.tests_summary,
                &review.command_summary,
                &review.hook_summary,
                &review.evidence_summary,
                &review.checklist_summary,
                serde_json::to_string(&review.warnings)?,
                &review.evidence_manifest_path,
                &review.review_checklist_result,
                &review.created_at,
                &review.decided_at,
            ],
        )?;

        self.connection.execute(
            "UPDATE tasks SET latest_review_id = ?1 WHERE id = ?2",
            params![&review.id, &review.task_id],
        )?;

        Ok(review)
    }

    pub fn list_for_task(&self, task_id: &str) -> Result<Vec<ReviewItem>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                id,
                task_id,
                session_id,
                status,
                summary,
                touched_files_json,
                diff_summary,
                tests_summary,
                command_summary,
                hook_summary,
                evidence_summary,
                checklist_summary,
                warnings_json,
                evidence_manifest_path,
                review_checklist_result,
                created_at,
                decided_at
            FROM review_items
            WHERE task_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;
        let mut rows = statement.query(params![task_id])?;
        let mut reviews = Vec::new();

        while let Some(row) = rows.next()? {
            reviews.push(review_from_row(row)?);
        }

        Ok(reviews)
    }

    pub fn latest_for_task(&self, task_id: &str) -> Result<Option<ReviewItem>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                id,
                task_id,
                session_id,
                status,
                summary,
                touched_files_json,
                diff_summary,
                tests_summary,
                command_summary,
                hook_summary,
                evidence_summary,
                checklist_summary,
                warnings_json,
                evidence_manifest_path,
                review_checklist_result,
                created_at,
                decided_at
            FROM review_items
            WHERE task_id = ?1
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )?;

        statement
            .query_row(params![task_id], |row| {
                let status = row.get::<_, String>("status")?;
                let touched_files_json = row.get::<_, String>("touched_files_json")?;
                let warnings_json = row.get::<_, String>("warnings_json")?;

                Ok((
                    row.get::<_, String>("id")?,
                    row.get::<_, String>("task_id")?,
                    row.get::<_, Option<String>>("session_id")?,
                    status,
                    row.get::<_, String>("summary")?,
                    touched_files_json,
                    row.get::<_, Option<String>>("diff_summary")?,
                    row.get::<_, Option<String>>("tests_summary")?,
                    row.get::<_, Option<String>>("command_summary")?,
                    row.get::<_, Option<String>>("hook_summary")?,
                    row.get::<_, Option<String>>("evidence_summary")?,
                    row.get::<_, Option<String>>("checklist_summary")?,
                    warnings_json,
                    row.get::<_, Option<String>>("evidence_manifest_path")?,
                    row.get::<_, Option<String>>("review_checklist_result")?,
                    row.get::<_, String>("created_at")?,
                    row.get::<_, Option<String>>("decided_at")?,
                ))
            })
            .optional()?
            .map(
                |(
                    id,
                    task_id,
                    session_id,
                    status,
                    summary,
                    touched_files_json,
                    diff_summary,
                    tests_summary,
                    command_summary,
                    hook_summary,
                    evidence_summary,
                    checklist_summary,
                    warnings_json,
                    evidence_manifest_path,
                    review_checklist_result,
                    created_at,
                    decided_at,
                )| {
                    Ok(ReviewItem {
                        id,
                        task_id,
                        session_id,
                        status: status.parse().map_err(StorageError::UnknownReviewStatus)?,
                        summary,
                        touched_files: serde_json::from_str(&touched_files_json)?,
                        diff_summary,
                        tests_summary,
                        command_summary,
                        hook_summary,
                        evidence_summary,
                        checklist_summary,
                        warnings: serde_json::from_str(&warnings_json)?,
                        evidence_manifest_path,
                        review_checklist_result,
                        created_at,
                        decided_at,
                    })
                },
            )
            .transpose()
    }
}

fn review_from_row(row: &rusqlite::Row<'_>) -> Result<ReviewItem, StorageError> {
    let status = row.get::<_, String>("status")?;
    let touched_files_json = row.get::<_, String>("touched_files_json")?;
    let warnings_json = row.get::<_, String>("warnings_json")?;

    Ok(ReviewItem {
        id: row.get("id")?,
        task_id: row.get("task_id")?,
        session_id: row.get("session_id")?,
        status: status.parse().map_err(StorageError::UnknownReviewStatus)?,
        summary: row.get("summary")?,
        touched_files: serde_json::from_str(&touched_files_json)?,
        diff_summary: row.get("diff_summary")?,
        tests_summary: row.get("tests_summary")?,
        command_summary: row.get("command_summary")?,
        hook_summary: row.get("hook_summary")?,
        evidence_summary: row.get("evidence_summary")?,
        checklist_summary: row.get("checklist_summary")?,
        warnings: serde_json::from_str(&warnings_json)?,
        evidence_manifest_path: row.get("evidence_manifest_path")?,
        review_checklist_result: row.get("review_checklist_result")?,
        created_at: row.get("created_at")?,
        decided_at: row.get("decided_at")?,
    })
}
