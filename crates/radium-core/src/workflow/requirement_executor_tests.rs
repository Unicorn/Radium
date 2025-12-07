#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::braingrid_client::{BraingridTask, TaskStatus as BgTaskStatus};

    #[test]
    fn test_task_dependency_filtering() {
        // Create test tasks matching REQ-178 structure
        let tasks = vec![
            BraingridTask {
                id: "task-4-id".to_string(),
                short_id: Some("TASK-4".to_string()),
                number: "4".to_string(),
                title: "Task 4".to_string(),
                description: None,
                status: BgTaskStatus::Planned,
                assigned_to: None,
                dependencies: vec!["3".to_string()], // blocked by task 3
            },
            BraingridTask {
                id: "task-1-id".to_string(),
                short_id: Some("TASK-1".to_string()),
                number: "1".to_string(),
                title: "Task 1".to_string(),
                description: None,
                status: BgTaskStatus::Completed, // Already done
                assigned_to: None,
                dependencies: vec![], // No dependencies
            },
            BraingridTask {
                id: "task-3-id".to_string(),
                short_id: Some("TASK-3".to_string()),
                number: "3".to_string(),
                title: "Task 3".to_string(),
                description: None,
                status: BgTaskStatus::Planned,
                assigned_to: None,
                dependencies: vec!["2".to_string()], // blocked by task 2
            },
            BraingridTask {
                id: "task-2-id".to_string(),
                short_id: Some("TASK-2".to_string()),
                number: "2".to_string(),
                title: "Task 2".to_string(),
                description: None,
                status: BgTaskStatus::Planned,
                assigned_to: None,
                dependencies: vec!["1".to_string()], // blocked by task 1 (which is completed)
            },
        ];

        // Filter out completed tasks
        let pending: Vec<_> = tasks.iter()
            .filter(|t| t.status != BgTaskStatus::Completed)
            .collect();

        // Should have 3 pending tasks (2, 3, 4)
        assert_eq!(pending.len(), 3);

        // Build a status map
        let status_map: std::collections::HashMap<String, BgTaskStatus> = tasks.iter()
            .map(|t| (t.number.clone(), t.status.clone()))
            .collect();

        // Find tasks whose dependencies are all completed
        let ready_tasks: Vec<_> = pending.iter()
            .filter(|task| {
                task.dependencies.iter().all(|dep_num| {
                    status_map.get(dep_num)
                        .map(|status| *status == BgTaskStatus::Completed)
                        .unwrap_or(false)
                })
            })
            .collect();

        // Only TASK-2 should be ready (its dependency TASK-1 is completed)
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].number, "2");
    }

    #[test]
    fn test_task_id_construction() {
        // Test with short_id present
        let task_with_short_id = BraingridTask {
            id: "uuid".to_string(),
            short_id: Some("TASK-5".to_string()),
            number: "5".to_string(),
            title: "Test".to_string(),
            description: None,
            status: BgTaskStatus::Planned,
            assigned_to: None,
            dependencies: vec![],
        };
        assert_eq!(task_with_short_id.task_id(), "TASK-5");

        // Test with short_id missing (constructed from number)
        let task_without_short_id = BraingridTask {
            id: "uuid".to_string(),
            short_id: None,
            number: "3".to_string(),
            title: "Test".to_string(),
            description: None,
            status: BgTaskStatus::Planned,
            assigned_to: None,
            dependencies: vec![],
        };
        assert_eq!(task_without_short_id.task_id(), "TASK-3");
    }
}
