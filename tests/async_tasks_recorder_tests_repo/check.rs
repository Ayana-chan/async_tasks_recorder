use lazy_static::lazy_static;
use std::sync::Arc;
use async_tasks_recorder::{AsyncTasksRecorder, TaskState};
use crate::async_tasks_recorder_tests_repo::{DEFAULT_CHECK_INTERVAL_MS, DEFAULT_CHECK_TIMEOUT_MS};
use crate::async_tasks_recorder_tests_repo::task::random_task;
use crate::async_tasks_recorder_tests_repo::tools::get_shuffled_index_map;

lazy_static! {
    static ref TASK_STATE_CHECKER: TaskStateChecker = TaskStateChecker::new();
}

/// Record launched tasks' state
#[derive(Default)]
struct TaskStateChecker {
    task_state_recorder: scc::HashMap<String, TaskState>,
}

impl TaskStateChecker {
    pub fn new() -> Self {
        TaskStateChecker::default()
    }

    pub async fn check_async(&self, task_id: &str, new_state: &TaskState) {
        let pre_state = self.get_status_async(&task_id).await;
        // check
        let ans = check_state_transition(&pre_state, &new_state);
        println!("{:?} | Check State Trans {}: {:?} -> {:?}", std::time::Instant::now(), task_id, pre_state, new_state);
        if !ans {
            panic!("{:?} | Failed Checking State Trans {}: {:?} -> {:?}", std::time::Instant::now(), task_id, pre_state, new_state);
        }
        // update
        self.task_state_recorder.entry_async(task_id.to_string()).await
            .and_modify(|v| *v = new_state.clone())
            .or_insert(new_state.clone());
    }

    async fn get_status_async(&self, task_id: &str) -> TaskState {
        self.task_state_recorder.get_async(task_id).await
            .map(|oe| oe.get().clone())
            .unwrap_or(TaskState::NotFound)
    }
}

fn check_state_transition(pre: &TaskState, now: &TaskState) -> bool {
    match pre {
        TaskState::Success => *now == TaskState::Success,
        TaskState::Failed => *now != TaskState::NotFound,
        TaskState::Working => *now != TaskState::NotFound,
        TaskState::NotFound => true,
    }
}

/// If `redo` is `Some(t)`, it will be redo after a delay after the task is found to have failed.
/// Check per `interval_ms`. \
/// Err when the time consumption reaches `timeout_ms`.
pub async fn check_success(manager: &AsyncTasksRecorder<String>, task_id: &str,
                           interval_ms: Option<u64>, timeout_ms: Option<u128>,
                           suffix_query_time: u128) {
    let interval_ms = interval_ms.unwrap_or(DEFAULT_CHECK_INTERVAL_MS.clone());
    let timeout_ms = timeout_ms.unwrap_or(DEFAULT_CHECK_TIMEOUT_MS.clone());
    let start_time = std::time::Instant::now();

    let mut task_status;

    loop {
        // println!("{:?} | check {}, interval: {}", std::time::Instant::now(), task_id, interval_ms);
        // timeout
        if start_time.elapsed().as_millis() >= timeout_ms {
            panic!("{:?} | Timeout before success. task_id: {:?}", std::time::Instant::now(), task_id);
        }

        task_status = manager.query_task_state(task_id).await;
        TASK_STATE_CHECKER.check_async(task_id, &task_status).await;

        if task_status == TaskState::Success {
            println!("{:?} | success {}, used time: {}", std::time::Instant::now(), task_id, start_time.elapsed().as_millis());
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms.clone())).await;
    }

    // suffix check, to check whether success change
    let first_success_time_start_ms = start_time.elapsed().as_millis();
    let mut time_used_ms;
    loop {
        time_used_ms = start_time.elapsed().as_millis() - first_success_time_start_ms;
        // timeout
        if time_used_ms >= suffix_query_time {
            println!("{:?} | suffer check {} finish", std::time::Instant::now(), task_id);
            break;
        }

        task_status = manager.query_task_state(task_id).await;
        if task_status != TaskState::Success {
            panic!("{:?} | Task {} change from success to {:?}", std::time::Instant::now(), task_id, task_status);
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms.clone())).await;
    }
}

/// A task will redo after it is found to have failed.
/// Check per `interval_ms`. \
/// Err when the time consumption reaches `timeout_ms`.
pub async fn check_success_auto_redo(manager: &AsyncTasksRecorder<String>, task_id: &str,
                                     interval_ms: Option<u64>, timeout_ms: Option<u128>,
                                     suffix_query_time: u128, redo_task_latency: u64,
                                     redo_task_success_probability: u8) {
    let interval_ms = interval_ms.unwrap_or(DEFAULT_CHECK_INTERVAL_MS.clone());
    let timeout_ms = timeout_ms.unwrap_or(DEFAULT_CHECK_TIMEOUT_MS.clone());
    let start_time = std::time::Instant::now();

    let mut task_status;

    loop {
        // println!("{:?} | check {}, interval: {}", std::time::Instant::now(), task_id, interval_ms);
        // timeout
        if start_time.elapsed().as_millis() >= timeout_ms {
            panic!("{:?} | Timeout before success. task_id: {:?}", std::time::Instant::now(), task_id);
        }

        task_status = manager.query_task_state(task_id).await;
        TASK_STATE_CHECKER.check_async(task_id, &task_status).await;

        match task_status {
            TaskState::Success => {
                println!("{:?} | success {}, used time: {}", std::time::Instant::now(), task_id, start_time.elapsed().as_millis());
                break;
            }
            TaskState::Failed => {
                // redo
                println!("{:?} | redo {}", std::time::Instant::now(), task_id);
                let task = random_task(redo_task_latency, redo_task_success_probability, task_id.to_string());
                let _ = manager.launch(task_id.to_string(), task).await;
            }
            _ => {}
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms.clone())).await;
    }

    // suffix check, to check whether success change
    let first_success_time_start_ms = start_time.elapsed().as_millis();
    let mut time_used_ms;
    loop {
        time_used_ms = start_time.elapsed().as_millis() - first_success_time_start_ms;
        // timeout
        if time_used_ms >= suffix_query_time {
            println!("{:?} | suffer check {} finish", std::time::Instant::now(), task_id);
            break;
        }

        task_status = manager.query_task_state(task_id).await;
        if task_status != TaskState::Success {
            panic!("{:?} | Task {} change from success to {:?}", std::time::Instant::now(), task_id, task_status);
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms.clone())).await;
    }
}

/// Check all task_id in a vec randomly and parallelly.
/// Return after all checks finish.
pub async fn check_success_vec(manager: &AsyncTasksRecorder<String>, task_id_vec: &Arc<Vec<String>>,
                               interval_ms: Option<u64>, timeout_ms: Option<u128>,
                               suffix_query_time: u128) {
    let task_num = task_id_vec.len();
    let shuffled_map = get_shuffled_index_map(task_num);
    let mut join_set = tokio::task::JoinSet::new();
    for i in 0..task_num {
        let manager_backup = manager.clone();
        let task_id_vec = task_id_vec.clone();
        let mapped_index = shuffled_map[i];
        // println!("{:?} | spawn check {}", std::time::Instant::now(), mapped_index);
        let fut = async move {
            check_success(&manager_backup, &task_id_vec[mapped_index],
                          interval_ms.clone(), timeout_ms.clone(),
                          suffix_query_time).await;
        };
        join_set.spawn(fut);
    }

    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            if e.is_panic() {
                std::panic::resume_unwind(e.into_panic());
            }
        }
    }
}

/// Check all task_id in a vec randomly and parallelly.
/// Return after all checks finish.
pub async fn check_success_vec_auto_redo(manager: &AsyncTasksRecorder<String>, task_id_vec: &Arc<Vec<String>>,
                                         interval_ms: Option<u64>, timeout_ms: Option<u128>,
                                         suffix_query_time: u128, redo_task_latency: u64,
                                         redo_task_success_probability: u8) {
    let task_num = task_id_vec.len();
    let shuffled_map = get_shuffled_index_map(task_num);
    let mut join_set = tokio::task::JoinSet::new();
    for i in 0..task_num {
        let manager_backup = manager.clone();
        let task_id_vec = task_id_vec.clone();
        let mapped_index = shuffled_map[i];
        // println!("{:?} | spawn check {}", std::time::Instant::now(), mapped_index);
        let fut = async move {
            check_success_auto_redo(&manager_backup, &task_id_vec[mapped_index],
                                    interval_ms.clone(), timeout_ms.clone(),
                                    suffix_query_time, redo_task_latency,
                                    redo_task_success_probability).await;
        };
        join_set.spawn(fut);
    }

    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            if e.is_panic() {
                std::panic::resume_unwind(e.into_panic());
            }
        }
    }
}


