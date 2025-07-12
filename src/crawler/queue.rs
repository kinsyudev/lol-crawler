use crate::models::database::{SummonerPriority, SummonerTask};
use std::collections::VecDeque;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct SummonerQueue {
    high_priority: RwLock<VecDeque<SummonerTask>>,
    medium_priority: RwLock<VecDeque<SummonerTask>>,
    low_priority: RwLock<VecDeque<SummonerTask>>,
}

impl SummonerQueue {
    pub fn new() -> Self {
        Self {
            high_priority: RwLock::new(VecDeque::new()),
            medium_priority: RwLock::new(VecDeque::new()),
            low_priority: RwLock::new(VecDeque::new()),
        }
    }

    pub async fn push(&self, task: SummonerTask) {
        match task.priority {
            SummonerPriority::High => {
                let mut queue = self.high_priority.write().await;
                queue.push_back(task);
            }
            SummonerPriority::Medium => {
                let mut queue = self.medium_priority.write().await;
                queue.push_back(task);
            }
            SummonerPriority::Low => {
                let mut queue = self.low_priority.write().await;
                queue.push_back(task);
            }
        }
    }

    pub async fn push_batch(&self, tasks: Vec<SummonerTask>) {
        let mut high_tasks = Vec::new();
        let mut medium_tasks = Vec::new();
        let mut low_tasks = Vec::new();

        for task in tasks {
            match task.priority {
                SummonerPriority::High => high_tasks.push(task),
                SummonerPriority::Medium => medium_tasks.push(task),
                SummonerPriority::Low => low_tasks.push(task),
            }
        }

        if !high_tasks.is_empty() {
            let mut queue = self.high_priority.write().await;
            for task in high_tasks {
                queue.push_back(task);
            }
        }

        if !medium_tasks.is_empty() {
            let mut queue = self.medium_priority.write().await;
            for task in medium_tasks {
                queue.push_back(task);
            }
        }

        if !low_tasks.is_empty() {
            let mut queue = self.low_priority.write().await;
            for task in low_tasks {
                queue.push_back(task);
            }
        }
    }

    pub async fn pop(&self) -> Option<SummonerTask> {
        // Try high priority first
        {
            let mut queue = self.high_priority.write().await;
            if let Some(task) = queue.pop_front() {
                return Some(task);
            }
        }

        // Then medium priority
        {
            let mut queue = self.medium_priority.write().await;
            if let Some(task) = queue.pop_front() {
                return Some(task);
            }
        }

        // Finally low priority
        {
            let mut queue = self.low_priority.write().await;
            queue.pop_front()
        }
    }

    pub async fn size(&self) -> (usize, usize, usize) {
        let high_size = self.high_priority.read().await.len();
        let medium_size = self.medium_priority.read().await.len();
        let low_size = self.low_priority.read().await.len();
        (high_size, medium_size, low_size)
    }

    pub async fn total_size(&self) -> usize {
        let (high, medium, low) = self.size().await;
        high + medium + low
    }

    pub async fn is_empty(&self) -> bool {
        self.total_size().await == 0
    }

    pub async fn clear(&self) {
        let mut high = self.high_priority.write().await;
        let mut medium = self.medium_priority.write().await;
        let mut low = self.low_priority.write().await;
        
        high.clear();
        medium.clear();
        low.clear();
    }

    pub async fn peek_next(&self) -> Option<SummonerPriority> {
        {
            let queue = self.high_priority.read().await;
            if !queue.is_empty() {
                return Some(SummonerPriority::High);
            }
        }

        {
            let queue = self.medium_priority.read().await;
            if !queue.is_empty() {
                return Some(SummonerPriority::Medium);
            }
        }

        {
            let queue = self.low_priority.read().await;
            if !queue.is_empty() {
                return Some(SummonerPriority::Low);
            }
        }

        None
    }

    pub async fn remove_duplicates(&self) {
        // This is a simplified implementation - in production you might want
        // to use a more efficient approach with sets
        self.remove_duplicates_from_queue(&self.high_priority).await;
        self.remove_duplicates_from_queue(&self.medium_priority).await;
        self.remove_duplicates_from_queue(&self.low_priority).await;
    }

    async fn remove_duplicates_from_queue(&self, queue: &RwLock<VecDeque<SummonerTask>>) {
        let mut queue_guard = queue.write().await;
        let mut seen = std::collections::HashSet::new();
        let mut new_queue = VecDeque::new();

        while let Some(task) = queue_guard.pop_front() {
            if seen.insert(task.puuid.clone()) {
                new_queue.push_back(task);
            }
        }

        *queue_guard = new_queue;
    }
}

impl Default for SummonerQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_task(puuid: &str, priority: SummonerPriority) -> SummonerTask {
        SummonerTask {
            puuid: puuid.to_string(),
            summoner_name: format!("Player{}", puuid),
            region: "na1".to_string(),
            priority,
            added_at: Utc::now(),
            retries: 0,
        }
    }

    #[tokio::test]
    async fn test_queue_priority_order() {
        let queue = SummonerQueue::new();

        // Add tasks in reverse priority order
        queue.push(create_test_task("low", SummonerPriority::Low)).await;
        queue.push(create_test_task("high", SummonerPriority::High)).await;
        queue.push(create_test_task("medium", SummonerPriority::Medium)).await;

        // Should pop in priority order
        assert_eq!(queue.pop().await.unwrap().puuid, "high");
        assert_eq!(queue.pop().await.unwrap().puuid, "medium");
        assert_eq!(queue.pop().await.unwrap().puuid, "low");
        assert!(queue.pop().await.is_none());
    }

    #[tokio::test]
    async fn test_queue_sizes() {
        let queue = SummonerQueue::new();

        queue.push(create_test_task("1", SummonerPriority::High)).await;
        queue.push(create_test_task("2", SummonerPriority::High)).await;
        queue.push(create_test_task("3", SummonerPriority::Medium)).await;
        queue.push(create_test_task("4", SummonerPriority::Low)).await;

        let (high, medium, low) = queue.size().await;
        assert_eq!(high, 2);
        assert_eq!(medium, 1);
        assert_eq!(low, 1);
        assert_eq!(queue.total_size().await, 4);
    }

    #[tokio::test]
    async fn test_batch_push() {
        let queue = SummonerQueue::new();

        let tasks = vec![
            create_test_task("1", SummonerPriority::High),
            create_test_task("2", SummonerPriority::Medium),
            create_test_task("3", SummonerPriority::Low),
        ];

        queue.push_batch(tasks).await;

        let (high, medium, low) = queue.size().await;
        assert_eq!(high, 1);
        assert_eq!(medium, 1);
        assert_eq!(low, 1);
    }
}