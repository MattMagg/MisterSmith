//! Subject taxonomy for NATS-style hierarchical message routing.
//!
//! Provides type-safe subject construction and validation for all 14 subject
//! patterns defined in the data model.

use crate::errors::TransportError;

/// Validates a subject segment: must be non-empty and contain no wildcards or spaces.
fn validate_segment(segment: &str, name: &str) -> Result<(), TransportError> {
    if segment.is_empty() {
        return Err(TransportError::SubjectInvalid(format!(
            "{name} must not be empty"
        )));
    }
    if segment.contains('*') || segment.contains('>') {
        return Err(TransportError::SubjectInvalid(format!(
            "{name} must not contain wildcards: {segment}"
        )));
    }
    if segment.contains(' ') {
        return Err(TransportError::SubjectInvalid(format!(
            "{name} must not contain spaces: {segment}"
        )));
    }
    Ok(())
}

/// Type-safe builder for subject strings following the hierarchical taxonomy.
pub struct SubjectTaxonomy;

impl SubjectTaxonomy {
    // --- Agent subjects ---

    /// `agents.{agent_id}.commands.{type}` — commands directed to a specific agent.
    pub fn agent_command(agent_id: &str, command_type: &str) -> Result<String, TransportError> {
        validate_segment(agent_id, "agent_id")?;
        validate_segment(command_type, "command_type")?;
        Ok(format!("agents.{agent_id}.commands.{command_type}"))
    }

    /// `agents.{agent_id}.status` — agent availability status updates.
    pub fn agent_status(agent_id: &str) -> Result<String, TransportError> {
        validate_segment(agent_id, "agent_id")?;
        Ok(format!("agents.{agent_id}.status"))
    }

    /// `agents.{agent_id}.heartbeat` — periodic heartbeat.
    pub fn agent_heartbeat(agent_id: &str) -> Result<String, TransportError> {
        validate_segment(agent_id, "agent_id")?;
        Ok(format!("agents.{agent_id}.heartbeat"))
    }

    /// `agents.{agent_id}.events.{type}` — agent lifecycle events.
    pub fn agent_event(agent_id: &str, event_type: &str) -> Result<String, TransportError> {
        validate_segment(agent_id, "agent_id")?;
        validate_segment(event_type, "event_type")?;
        Ok(format!("agents.{agent_id}.events.{event_type}"))
    }

    // --- Task subjects ---

    /// `tasks.{task_type}.assignment` — task assignment (queue group eligible).
    pub fn task_assignment(task_type: &str) -> Result<String, TransportError> {
        validate_segment(task_type, "task_type")?;
        Ok(format!("tasks.{task_type}.assignment"))
    }

    /// `tasks.{task_type}.queue.{priority}` — priority-based task queues.
    pub fn task_queue(task_type: &str, priority: &str) -> Result<String, TransportError> {
        validate_segment(task_type, "task_type")?;
        validate_segment(priority, "priority")?;
        Ok(format!("tasks.{task_type}.queue.{priority}"))
    }

    /// `tasks.{task_id}.progress` — task progress updates.
    pub fn task_progress(task_id: &str) -> Result<String, TransportError> {
        validate_segment(task_id, "task_id")?;
        Ok(format!("tasks.{task_id}.progress"))
    }

    /// `tasks.{task_id}.result` — task completion results.
    pub fn task_result(task_id: &str) -> Result<String, TransportError> {
        validate_segment(task_id, "task_id")?;
        Ok(format!("tasks.{task_id}.result"))
    }

    // --- System subjects ---

    /// `system.events.{type}` — system-wide events.
    pub fn system_event(event_type: &str) -> Result<String, TransportError> {
        validate_segment(event_type, "event_type")?;
        Ok(format!("system.events.{event_type}"))
    }

    /// `system.config.{component}` — configuration updates.
    pub fn system_config(component: &str) -> Result<String, TransportError> {
        validate_segment(component, "component")?;
        Ok(format!("system.config.{component}"))
    }

    /// `system.health` — health check signals.
    pub fn system_health() -> String {
        "system.health".to_string()
    }

    // --- Workflow subjects ---

    /// `workflow.{workflow_id}.start` — workflow initiation.
    pub fn workflow_start(workflow_id: &str) -> Result<String, TransportError> {
        validate_segment(workflow_id, "workflow_id")?;
        Ok(format!("workflow.{workflow_id}.start"))
    }

    /// `workflow.{workflow_id}.step.{step_id}` — step completion.
    pub fn workflow_step(workflow_id: &str, step_id: &str) -> Result<String, TransportError> {
        validate_segment(workflow_id, "workflow_id")?;
        validate_segment(step_id, "step_id")?;
        Ok(format!("workflow.{workflow_id}.step.{step_id}"))
    }

    /// `workflow.{workflow_id}.result` — workflow result.
    pub fn workflow_result(workflow_id: &str) -> Result<String, TransportError> {
        validate_segment(workflow_id, "workflow_id")?;
        Ok(format!("workflow.{workflow_id}.result"))
    }

    // --- Wildcard subjects (for subscriptions) ---

    /// `agents.>` — subscribe to all agent subjects.
    pub fn all_agents() -> String {
        "agents.>".to_string()
    }

    /// `tasks.*.assignment` — subscribe to all task assignments.
    pub fn all_task_assignments() -> String {
        "tasks.*.assignment".to_string()
    }

    /// `system.>` — subscribe to all system subjects.
    pub fn all_system() -> String {
        "system.>".to_string()
    }

    /// `workflow.>` — subscribe to all workflow subjects.
    pub fn all_workflows() -> String {
        "workflow.>".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_command_subject() {
        let subject = SubjectTaxonomy::agent_command("agent-1", "execute").unwrap();
        assert_eq!(subject, "agents.agent-1.commands.execute");
    }

    #[test]
    fn agent_status_subject() {
        let subject = SubjectTaxonomy::agent_status("worker-42").unwrap();
        assert_eq!(subject, "agents.worker-42.status");
    }

    #[test]
    fn agent_heartbeat_subject() {
        let subject = SubjectTaxonomy::agent_heartbeat("monitor-1").unwrap();
        assert_eq!(subject, "agents.monitor-1.heartbeat");
    }

    #[test]
    fn agent_event_subject() {
        let subject = SubjectTaxonomy::agent_event("agent-1", "started").unwrap();
        assert_eq!(subject, "agents.agent-1.events.started");
    }

    #[test]
    fn task_assignment_subject() {
        let subject = SubjectTaxonomy::task_assignment("code-review").unwrap();
        assert_eq!(subject, "tasks.code-review.assignment");
    }

    #[test]
    fn task_queue_subject() {
        let subject = SubjectTaxonomy::task_queue("analysis", "high").unwrap();
        assert_eq!(subject, "tasks.analysis.queue.high");
    }

    #[test]
    fn task_progress_subject() {
        let subject = SubjectTaxonomy::task_progress("abc-123").unwrap();
        assert_eq!(subject, "tasks.abc-123.progress");
    }

    #[test]
    fn task_result_subject() {
        let subject = SubjectTaxonomy::task_result("def-456").unwrap();
        assert_eq!(subject, "tasks.def-456.result");
    }

    #[test]
    fn system_event_subject() {
        let subject = SubjectTaxonomy::system_event("config_changed").unwrap();
        assert_eq!(subject, "system.events.config_changed");
    }

    #[test]
    fn system_config_subject() {
        let subject = SubjectTaxonomy::system_config("nats").unwrap();
        assert_eq!(subject, "system.config.nats");
    }

    #[test]
    fn system_health_subject() {
        assert_eq!(SubjectTaxonomy::system_health(), "system.health");
    }

    #[test]
    fn workflow_subjects() {
        let start = SubjectTaxonomy::workflow_start("wf-1").unwrap();
        assert_eq!(start, "workflow.wf-1.start");

        let step = SubjectTaxonomy::workflow_step("wf-1", "step-2").unwrap();
        assert_eq!(step, "workflow.wf-1.step.step-2");

        let result = SubjectTaxonomy::workflow_result("wf-1").unwrap();
        assert_eq!(result, "workflow.wf-1.result");
    }

    #[test]
    fn wildcard_subjects() {
        assert_eq!(SubjectTaxonomy::all_agents(), "agents.>");
        assert_eq!(SubjectTaxonomy::all_task_assignments(), "tasks.*.assignment");
        assert_eq!(SubjectTaxonomy::all_system(), "system.>");
        assert_eq!(SubjectTaxonomy::all_workflows(), "workflow.>");
    }

    #[test]
    fn rejects_empty_segment() {
        assert!(SubjectTaxonomy::agent_command("", "execute").is_err());
        assert!(SubjectTaxonomy::agent_command("agent-1", "").is_err());
        assert!(SubjectTaxonomy::task_assignment("").is_err());
    }

    #[test]
    fn rejects_wildcard_in_segment() {
        assert!(SubjectTaxonomy::agent_command("agent*", "exec").is_err());
        assert!(SubjectTaxonomy::agent_status("agent>").is_err());
    }

    #[test]
    fn rejects_spaces_in_segment() {
        assert!(SubjectTaxonomy::agent_command("agent 1", "exec").is_err());
    }
}
