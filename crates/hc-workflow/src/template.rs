use std::collections::BTreeMap;

use regex::Regex;

use crate::WorkflowError;

const ALLOWED_VARIABLES: &[&str] = &[
    "task.id",
    "task.key",
    "task.title",
    "task.description",
    "task.description_markdown",
    "task.priority",
    "task.column",
    "task.labels_csv",
    "project.id",
    "project.name",
    "project.repo_root",
    "session.id",
    "session.mode",
    "session.workspace_root",
    "session.cwd",
    "session.branch",
    "session.adapter_name",
    "workflow.name",
    "workflow.max_slots",
    "workflow.contract_hash",
    "review.required",
    "review.checklist_markdown",
    "review.checklist_plain",
    "now.iso8601",
    "now.date",
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RenderContext {
    values: BTreeMap<&'static str, String>,
}

impl RenderContext {
    pub(crate) fn with(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.values.insert(key, value.into());
        self
    }

    fn lookup(&self, key: &str) -> String {
        self.values.get(key).cloned().unwrap_or_default()
    }
}

pub(crate) fn validate_template_variables(template_body: &str) -> Result<(), WorkflowError> {
    let pattern = Regex::new(r#"(\\)?\{\{\s*([a-zA-Z0-9_.]+)\s*\}\}"#).expect("template regex");

    for captures in pattern.captures_iter(template_body) {
        if captures.get(1).is_some() {
            continue;
        }

        let variable = captures.get(2).expect("variable capture").as_str();

        if !ALLOWED_VARIABLES.contains(&variable) {
            return Err(WorkflowError::UnknownTemplateVariable(variable.to_string()));
        }
    }

    Ok(())
}

pub(crate) fn render_template(
    template_body: &str,
    context: &RenderContext,
) -> Result<String, WorkflowError> {
    validate_template_variables(template_body)?;

    let escaped_open = "__HC_ESCAPED_OPEN__";
    let normalized = template_body.replace(r"\{{", escaped_open);
    let pattern = Regex::new(r"\{\{\s*([a-zA-Z0-9_.]+)\s*\}\}").expect("template regex");

    let rendered = pattern.replace_all(&normalized, |captures: &regex::Captures<'_>| {
        let key = captures.get(1).expect("variable capture").as_str();
        context.lookup(key)
    });

    Ok(rendered.replace(escaped_open, "{{"))
}
