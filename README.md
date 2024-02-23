# Introduction

A struct for recording execution status of async tasks with lock-free and async methods.

Functions:
- Able to host `Future`s and query whether they are **not found**, **successful**, **failed**, or **running**.
- Able to host `Future`s to revoke the succeeded `Future`s and make them **not found**.

Dependency:
- Depend on `tokio` with feature `rt`, so cannot use other async runtimes.
- Depend on [scc](https://crates.io/crates/scc) for lock-free and async `HashSet`.

Use this crate if:
- Easy to generate an **unique** `task_id` (not necessarily `String`) for a future (task).
- Don't want tasks with the same `task_id` to succeed more than once.
- Want to record and query all succeeded tasks and failed tasks.
- Want to handle every task in the same state (not just focus on one state).
- Need linearizable query.
- Want to revoke a task, and don't want the revoking to succeed more than once.

[Example](https://github.com/Ayana-chan/ipfs_storage_cruster/tree/master/crates/async_tasks_recorder/examples).

A recorder can only use one `task_id` type. The type of `task_id` should be:
- `Eq + Hash + Clone + Send + Sync + 'static`
- Cheap to clone (sometimes can use `Arc`).

# Usage

Launch a task with a **unique** `task_id` and a `Future` by `launch`.

Query the state of the task with its `task_id`
by `query_task_state` or `query_task_state_quick`.

Revoke a task with its `task_id` and a `Future` for revoking by `revoke_task_block`.

## Skills

Remember that you can add **anything** in the `Future` to achieve the functionality you want.
For example:
- Handle your `Result` in `Future`, and then return empty result `Result<(),()>`.
- Send a message to a one shot channel at the end of the `Future` to notify upper level that "this task done".
  Don't forget to consider using `tokio::spawn` when the channel may not complete sending immediately.
- Set other callback functions.

It's still efficient to store metadata of tasks at external `scc::HashMap` (`task_id` \-\> metadata).

> It is recommended to directly look at the source code (about 150 line) if there is any confusion.

## When Shouldn't Use This Crate

The consumption of all operations in this crate and cloning times
is about two to three times that of the implementation using `scc::Hashmap`.

This crate use **three `HashSet`** to make it easy to operate all tasks in the same state,
And two more `HashSet` for linearizability of query and supporting revoking operation.

Note that `scc`'s containers have less contention in **single** access when it grows larger.

Therefore, if you don't need operating every task in the same state,
then just use `scc::HashMap` (`task_id` \-\> `task_status`) to build a simpler implementation,
which might have less contention and cloning, but more expansive to iterate.
And the `scc::HashMap::update_async` could be a powerful tool for atomic operations.

You should also avoid using this crate if you just want to handle every task in only one state.
For example, if you just want to manage the failed tasks,
then you should use `scc::HashMap` to record tasks' states,
and insert the failed tasks into an external `Arc<scc::HashSet>` in `Future`.

**Version less than 1.1.1 has bugs.**

**For more usage, nature and proofs, please refer to Document.**

