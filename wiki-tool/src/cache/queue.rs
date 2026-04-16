use serde::{Deserialize, Serialize};
use std::path::Path;

/// Task status in the ingest queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Processing,
    Done,
    Failed,
}

/// A single ingest task in the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestTask {
    /// Unique task identifier.
    pub id: String,
    /// Path to source file.
    pub source_path: String,
    /// Current status.
    pub status: TaskStatus,
    /// Number of retry attempts.
    pub retry_count: u32,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Persistent ingest queue with crash recovery.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IngestQueue {
    pub tasks: Vec<IngestTask>,
}

const MAX_RETRIES: u32 = 3;

impl IngestQueue {
    /// Load queue from JSON file.
    pub fn load(state_dir: &Path) -> crate::Result<Self> {
        let queue_path = state_dir.join("ingest-queue.json");
        if !queue_path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&queue_path)?;
        let queue: Self = serde_json::from_str(&content)?;
        Ok(queue)
    }

    /// Save queue to JSON file.
    pub fn save(&self, state_dir: &Path) -> crate::Result<()> {
        std::fs::create_dir_all(state_dir)?;
        let queue_path = state_dir.join("ingest-queue.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(queue_path, content)?;
        Ok(())
    }

    /// Add a new task to the queue.
    pub fn add_task(&mut self, source_path: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.tasks.push(IngestTask {
            id: id.clone(),
            source_path: source_path.to_string(),
            status: TaskStatus::Pending,
            retry_count: 0,
            error: None,
        });
        id
    }

    /// Get the next pending task.
    pub fn next_pending(&mut self) -> Option<&mut IngestTask> {
        self.tasks
            .iter_mut()
            .find(|t| t.status == TaskStatus::Pending)
    }

    /// Mark a task as processing.
    pub fn mark_processing(&mut self, id: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.status = TaskStatus::Processing;
        }
    }

    /// Mark a task as done.
    pub fn mark_done(&mut self, id: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.status = TaskStatus::Done;
        }
    }

    /// Mark a task as failed, with retry if under limit.
    pub fn mark_failed(&mut self, id: &str, error: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.retry_count += 1;
            task.error = Some(error.to_string());
            if task.retry_count < MAX_RETRIES {
                task.status = TaskStatus::Pending; // Retry
            } else {
                task.status = TaskStatus::Failed;
            }
        }
    }

    /// Resume crashed tasks (Processing → Pending).
    pub fn resume_crashed(&mut self) {
        for task in &mut self.tasks {
            if task.status == TaskStatus::Processing {
                task.status = TaskStatus::Pending;
            }
        }
    }

    /// Remove completed tasks from the queue.
    pub fn cleanup(&mut self) {
        self.tasks.retain(|t| t.status != TaskStatus::Done);
    }
}
