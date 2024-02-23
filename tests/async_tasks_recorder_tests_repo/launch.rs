use std::ops::RangeBounds;
use std::sync::Arc;
use async_tasks_recorder::AsyncTasksRecorder;
use crate::async_tasks_recorder_tests_repo::task::random_task;
use crate::async_tasks_recorder_tests_repo::tools::get_shuffled_index_map;

/// Launch success task by `task_id_vec`.
/// The latency of each task is randomly selected within `latency_range`.
pub async fn launch_vec<Range>(manager: &AsyncTasksRecorder<String>, task_id_vec: &Arc<Vec<String>>,
                               latency_range: Range, task_success_probability: u8)
    where Range: RangeBounds<u64> + Clone {
    let task_num = task_id_vec.len();
    let shuffled_map = get_shuffled_index_map(task_num);
    for i in 0..task_num {
        let manager_backup = manager.clone();
        let task_id = task_id_vec[shuffled_map[i]].clone();
        let latency = fastrand::u64(latency_range.clone());
        // println!("spawn launch: {} latency: {}", mapped_index, latency);
        let task = random_task(latency, task_success_probability, task_id.clone());
        let fut = async move {
            let _ = manager_backup.launch(task_id,
                                  task).await;
        };
        tokio::spawn(fut);
    }
}
