use lazy_static::lazy_static;

lazy_static! {
    static ref TASK_EXEC_RECODER_CHECKER: TaskExecCountChecker = TaskExecCountChecker::new();
}

/// Record every success task.
#[derive(Default)]
struct TaskExecCountChecker {
    task_exec_recorder: scc::HashSet<String>,
}

impl TaskExecCountChecker {
    pub fn new() -> Self {
        TaskExecCountChecker::default()
    }

    pub async fn check_async(&self, task_id: &str) {
        if self.task_exec_recorder.contains_async(task_id).await {
            panic!("{:?} | Task {} executed after success!", std::time::Instant::now(), task_id);
        }
    }

    pub async fn check_and_record_async(&self, task_id: String) {
        if self.task_exec_recorder.contains_async(&task_id).await {
            panic!("{:?} | Task {} success more than once!", std::time::Instant::now(), task_id);
        }
        let _ = self.task_exec_recorder.insert_async(task_id).await;
    }
}

/// A task always return ok. \
/// Latency is caused by `std::thread::sleep`, which won't stop when panic in main thread occurs.
pub async fn success_task(latency_ms: u64, task_id: String) -> Result<(), ()> {
    // must use `sleep` of std
    std::thread::sleep(std::time::Duration::from_millis(latency_ms));
    println!("{:?} | ---->finish {}, task latency: {}", std::time::Instant::now(), task_id, latency_ms);
    TASK_EXEC_RECODER_CHECKER.check_and_record_async(task_id).await;
    Ok(())
}

/// A task always return err.
pub async fn fail_task(latency_ms: u64, task_id: String) -> Result<(), ()> {
    std::thread::sleep(std::time::Duration::from_millis(latency_ms));
    println!("{:?} | ---->fail {}, task latency: {}", std::time::Instant::now(), task_id, latency_ms);
    Err(())
}

/// A task possibly return err. \
/// `success_probability`: The percentage probability of success. Supposed to be \[0, 100\].
pub async fn random_task(latency_ms: u64, success_probability: u8, task_id: String) -> Result<(), ()> {
    TASK_EXEC_RECODER_CHECKER.check_async(&task_id).await;
    std::thread::sleep(std::time::Duration::from_millis(latency_ms));
    let rand_point = fastrand::u8(0..100);
    if rand_point < success_probability {
        println!("{:?} | ---->rand Ok {}, point: {} < {}, task latency: {}", std::time::Instant::now(), task_id, rand_point, success_probability, latency_ms);
        TASK_EXEC_RECODER_CHECKER.check_and_record_async(task_id).await;
        Ok(())
    } else {
        println!("{:?} | rand Err {}, point: {} >= {}, task latency: {}", std::time::Instant::now(), task_id, rand_point, success_probability, latency_ms);
        Err(())
    }
}