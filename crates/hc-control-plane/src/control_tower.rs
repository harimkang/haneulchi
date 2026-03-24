use hc_domain::{AppSnapshot, SessionRuntimeState};
use serde::{Deserialize, Serialize};

use crate::reviews::ReviewQueueItem;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HeatStripProjection {
    pub running: u32,
    pub waiting_input: u32,
    pub review_ready: u32,
    pub blocked: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectCardProjection {
    pub project_id: String,
    pub state: String,
    pub session_count: u32,
    pub attention_count: u32,
    pub latest_summary: Option<String>,
    pub latest_commentary: Option<String>,
    pub heat_strip: HeatStripProjection,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RecentArtifactProjection {
    pub task_id: String,
    pub project_id: String,
    pub summary: String,
    pub jump_target: String,
    pub manifest_path: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ControlTowerProjection {
    pub project_cards: Vec<ProjectCardProjection>,
    pub recent_artifacts: Vec<RecentArtifactProjection>,
}

pub fn build_control_tower_projection(
    snapshot: &AppSnapshot,
    review_items: &[ReviewQueueItem],
) -> ControlTowerProjection {
    let project_cards = snapshot
        .projects
        .iter()
        .map(|project| {
            let sessions = snapshot
                .sessions
                .iter()
                .filter(|session| session.project_id == project.project_id)
                .collect::<Vec<_>>();

            let latest_session = sessions
                .iter()
                .max_by_key(|session| session.last_activity_at.as_deref().unwrap_or(""));
            let mut heat_strip = HeatStripProjection::default();

            for session in &sessions {
                match session.runtime_state {
                    SessionRuntimeState::Running => heat_strip.running += 1,
                    SessionRuntimeState::WaitingInput => heat_strip.waiting_input += 1,
                    SessionRuntimeState::ReviewReady => heat_strip.review_ready += 1,
                    SessionRuntimeState::Blocked | SessionRuntimeState::Error => {
                        heat_strip.blocked += 1;
                    }
                    _ => {}
                }
            }

            let state = if heat_strip.waiting_input > 0
                || heat_strip.review_ready > 0
                || project.attention_count > 0
            {
                "attention"
            } else if heat_strip.blocked > 0 {
                "error"
            } else if project.session_count == 0 {
                "idle"
            } else {
                "active"
            };

            ProjectCardProjection {
                project_id: project.project_id.clone(),
                state: state.to_string(),
                session_count: project.session_count,
                attention_count: project.attention_count,
                latest_summary: latest_session.and_then(|session| session.latest_summary.clone()),
                latest_commentary: latest_session
                    .and_then(|session| session.latest_commentary.clone()),
                heat_strip,
            }
        })
        .collect();

    let recent_artifacts = review_items
        .iter()
        .map(|item| RecentArtifactProjection {
            task_id: item.task_id.clone(),
            project_id: item.project_id.clone(),
            summary: item.summary.clone(),
            jump_target: "review_queue".to_string(),
            manifest_path: item.evidence_manifest_path.clone(),
        })
        .collect();

    ControlTowerProjection {
        project_cards,
        recent_artifacts,
    }
}
