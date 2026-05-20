use anyhow::{bail, Result};

use super::task::{Task, TaskStatus};

pub fn can_transition(from: TaskStatus, to: TaskStatus) -> bool {
    match (from, to) {
        // Pending can move to InProgress, Blocked, or Cancelled
        (TaskStatus::Pending, TaskStatus::InProgress) => true,
        (TaskStatus::Pending, TaskStatus::Blocked) => true,
        (TaskStatus::Pending, TaskStatus::Cancelled) => true,

        // InProgress can move to Complete, Failed, Blocked, or Cancelled
        (TaskStatus::InProgress, TaskStatus::Complete) => true,
        (TaskStatus::InProgress, TaskStatus::Failed) => true,
        (TaskStatus::InProgress, TaskStatus::Blocked) => true,
        (TaskStatus::InProgress, TaskStatus::Cancelled) => true,

        // Failed can retry to InProgress or be Cancelled
        (TaskStatus::Failed, TaskStatus::InProgress) => true,
        (TaskStatus::Failed, TaskStatus::Cancelled) => true,

        // Blocked can resume to InProgress or be Cancelled
        (TaskStatus::Blocked, TaskStatus::InProgress) => true,
        (TaskStatus::Blocked, TaskStatus::Cancelled) => true,

        // Terminal states have no outgoing transitions
        (TaskStatus::Complete, _) => false,
        (TaskStatus::Cancelled, _) => false,

        // Same state is always allowed (idempotent)
        (a, b) if a == b => true,

        // All other transitions are invalid
        _ => false,
    }
}

pub fn transition(task: &mut Task, to: TaskStatus) -> Result<()> {
    if can_transition(task.status, to) {
        if to == TaskStatus::Failed {
            task.failure_count += 1;
        }
        task.status = to;
        Ok(())
    } else {
        bail!(
            "Invalid status transition from {:?} to {:?}",
            task.status,
            to
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::TaskType;

    #[test]
    fn test_pending_to_in_progress() {
        let mut task = Task::new("T1", "Test", TaskType::Implementation);
        assert!(can_transition(TaskStatus::Pending, TaskStatus::InProgress));
        transition(&mut task, TaskStatus::InProgress).unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_in_progress_to_complete() {
        let mut task = Task::new("T1", "Test", TaskType::Implementation);
        task.status = TaskStatus::InProgress;
        assert!(can_transition(TaskStatus::InProgress, TaskStatus::Complete));
        transition(&mut task, TaskStatus::Complete).unwrap();
        assert_eq!(task.status, TaskStatus::Complete);
    }

    #[test]
    fn test_complete_cannot_transition() {
        let mut task = Task::new("T1", "Test", TaskType::Implementation);
        task.status = TaskStatus::Complete;
        assert!(!can_transition(
            TaskStatus::Complete,
            TaskStatus::InProgress
        ));
        assert!(transition(&mut task, TaskStatus::InProgress).is_err());
    }

    #[test]
    fn test_failed_increments_failure_count() {
        let mut task = Task::new("T1", "Test", TaskType::Implementation);
        task.status = TaskStatus::InProgress;
        transition(&mut task, TaskStatus::Failed).unwrap();
        assert_eq!(task.failure_count, 1);
        transition(&mut task, TaskStatus::InProgress).unwrap();
        transition(&mut task, TaskStatus::Failed).unwrap();
        assert_eq!(task.failure_count, 2);
    }

    #[test]
    fn test_invalid_transition_pending_to_complete() {
        let mut task = Task::new("T1", "Test", TaskType::Implementation);
        assert!(!can_transition(TaskStatus::Pending, TaskStatus::Complete));
        assert!(transition(&mut task, TaskStatus::Complete).is_err());
    }

    #[test]
    fn test_blocked_to_in_progress() {
        let mut task = Task::new("T1", "Test", TaskType::Implementation);
        task.status = TaskStatus::Blocked;
        assert!(can_transition(TaskStatus::Blocked, TaskStatus::InProgress));
        transition(&mut task, TaskStatus::InProgress).unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
    }
}
