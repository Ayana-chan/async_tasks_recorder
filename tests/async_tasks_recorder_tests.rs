//! Passed in CPU i7-11800H (8 core 16 thread).

#[allow(unused_imports)]
use async_tasks_recorder::*;
use async_tasks_recorder_tests_repo::*;
use serial_test::{serial, parallel};

mod async_tasks_recorder_tests_repo;

#[test]
#[parallel]
fn test_once_multi() {
    do_async_test(
        RuntimeType::MultiThread,
        test_once(),
    );
}

#[test]
#[parallel]
fn test_once_single() {
    do_async_test(
        RuntimeType::CurrentThread,
        test_once(),
    );
}

#[test]
#[parallel]
#[should_panic(expected = "Timeout before success")]
fn test_once_fail_multi() {
    do_async_test(
        RuntimeType::MultiThread,
        test_once_fail(),
    );
}

#[test]
#[parallel]
#[should_panic(expected = "Timeout before success")]
fn test_once_fail_single() {
    do_async_test(
        RuntimeType::CurrentThread,
        test_once_fail(),
    );
}

#[test]
#[parallel]
fn test_basic_multi() {
    do_async_test(
        RuntimeType::MultiThread,
        test_basic(30, 0..60, None, 200),
    );
}

#[test]
#[parallel]
fn test_basic_single() {
    do_async_test(
        RuntimeType::CurrentThread,
        test_basic(5, 0..60, Some(1500), 100),
    );
}

#[test]
#[parallel]
fn test_redo_multi() {
    do_async_test(
        RuntimeType::MultiThread,
        test_once_redo(),
    );
}

#[test]
#[parallel]
fn test_redo_single() {
    do_async_test(
        RuntimeType::CurrentThread,
        test_once_redo(),
    );
}

#[test]
#[parallel]
fn test_random_multi() {
    do_async_test(
        RuntimeType::MultiThread,
        test_random(30, 2..15,
                    3, 400,
                    200, 13,
                    60),
    );
}

#[test]
#[parallel]
fn test_random_single() {
    do_async_test(
        RuntimeType::CurrentThread,
        test_random(8, 2..15,
                    3, 600,
                    200, 13,
                    60),
    );
}

#[test]
#[parallel]
fn test_interleave_multi() {
    do_async_test(
        RuntimeType::MultiThread,
        test_interleave(30, 2, 5,
                        2..15,
                        3, 600,
                        200, 13,
                        60),
    );
}

#[test]
#[parallel]
fn test_interleave_single() {
    do_async_test(
        RuntimeType::CurrentThread,
        test_interleave(5, 2, 5,
                        2..15,
                        3, 600,
                        200, 13,
                        60),
    );
}

#[test]
#[serial]
fn test_stress_large_group_num() {
    do_async_test(
        RuntimeType::MultiThread,
        test_interleave(300, 2, 5,
                        2..15,
                        1, 3000,
                        500, 13,
                        60),
    );
}

#[test]
#[serial]
fn test_stress_large_group_size_range() {
    do_async_test(
        RuntimeType::MultiThread,
        test_interleave(50, 1, 50,
                        2..15,
                        1, 3000,
                        500, 13,
                        60),
    );
}

#[test]
#[serial]
fn test_stress_large_task_latency_range() {
    do_async_test(
        RuntimeType::MultiThread,
        test_interleave(50, 5, 10,
                        0..50,
                        1, 3000,
                        500, 13,
                        60),
    );
}

#[test]
#[parallel]
fn test_simple_launch_check_multi() {
    do_async_test(
        RuntimeType::MultiThread,
        test_simple_launch_check(20000),
    );
}

#[test]
#[parallel]
fn test_simple_launch_check_single() {
    do_async_test(
        RuntimeType::CurrentThread,
        test_simple_launch_check(5000),
    );
}

#[test]
#[parallel]
fn test_simple_launch_check_revoke_multi() {
    do_async_test(
        RuntimeType::MultiThread,
        test_simple_launch_check_revoke_loop(20000, 1),
    );
}

#[test]
#[parallel]
fn test_simple_launch_check_revoke_single() {
    do_async_test(
        RuntimeType::CurrentThread,
        test_simple_launch_check_revoke_loop(5000, 1),
    );
}

#[test]
#[parallel]
fn test_simple_launch_check_revoke_loop_multi() {
    do_async_test(
        RuntimeType::MultiThread,
        test_simple_launch_check_revoke_loop(2000, 30),
    );
}

#[test]
#[parallel]
fn test_simple_launch_check_revoke_loop_single() {
    do_async_test(
        RuntimeType::CurrentThread,
        test_simple_launch_check_revoke_loop(500, 30),
    );
}
