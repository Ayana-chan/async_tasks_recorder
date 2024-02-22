use std::future::Future;
use crate::async_tasks_recorder_tests_repo::RuntimeType;

pub fn do_async_test<Fut>(runtime_type: RuntimeType, test_future: Fut)
    where Fut: Future {
    let runtime;
    match runtime_type {
        RuntimeType::CurrentThread => {
            runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build().unwrap();
        }
        RuntimeType::CustomThread(thread_num) => {
            runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(thread_num)
                .enable_all()
                .build().unwrap();
        }
        RuntimeType::MultiThread => {
            runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build().unwrap();
        }
    }

    runtime.block_on(test_future);
}

/// return a closure to generate incremental `task_id` string.
pub fn get_task_id_generator() -> impl FnMut() -> String {
    let rand_str: String = std::iter::repeat_with(fastrand::alphabetic).take(6).collect();
    let mut count: usize = 0;
    move || {
        let ans = format!("task_id_{}_{:06}", rand_str, count);
        count += 1;
        ans
    }
}

/// Generate `task_id` vec with a new generator from `get_task_id_generator`
pub fn generate_task_id_vec(task_num: usize) -> Vec<String> {
    let mut task_id_generator = get_task_id_generator();
    let mut task_id_vec: Vec<String> = Vec::new();
    task_id_vec.resize(task_num, Default::default());
    for i in 0..task_num {
        task_id_vec[i] = task_id_generator();
    }
    task_id_vec
}

/// get a shuffled `vec[0, 1, 2, ...]` to map index
pub fn get_shuffled_index_map(length: usize) -> Vec<usize> {
    let mut map: Vec<usize> = (0..length).collect();
    fastrand::shuffle(&mut map);
    map
}

pub fn get_arithmetic_sequence(num: usize, min: usize, mut max: usize) -> Vec<usize> {
    if num == 0 {
        return vec![];
    }
    if max < min {
        max = min;
    }

    let mut res: Vec<usize> = vec![];
    res.resize(num, 0);

    let delta = max - min;
    unsafe {
        // res[i] = i * (max - min);
        *res.get_unchecked_mut(0) = 0;
        for i in 1..num {
            *res.get_unchecked_mut(i) = res.get_unchecked(i - 1) + delta;
        }
    }

    // res[i] = i * (max - min) / num + min
    for item in &mut res {
        *item = *item / num + min;
    }

    res
}
