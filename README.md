> A big bug has just been fixed, so the document may not be complete, but don't worry too much

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
- Don't want tasks with the same `task_id` to succeed more then once.
- Want to record and query all succeeded tasks and failed tasks.
- Want to handling every task in the same state (not just focus on one state).
- Need linearizable query.
- Want to revoke a task.

[Example](https://github.com/Ayana-chan/ipfs_storage_cruster/tree/master/crates/async_tasks_recorder/examples).

A recorder can only use one `task_id` type. The type of `task_id` should be:
- `Eq + Hash + Clone + Send + Sync + 'static`
- Cheap to clone (sometimes can use `Arc`).


**For more usage, nature and proofs, please refer to Document.**

