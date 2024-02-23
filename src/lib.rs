//! # Introduction
//!
//! A struct for recording execution status of async tasks with lock-free and async methods.
//!
//! Functions:
//! - Able to host `Future`s and query whether they are **not found**, **successful**, **failed**, or **running**.
//! - Able to host `Future`s to revoke the succeeded `Future`s and make them **not found**.
//!
//! Dependency:
//! - Depend on `tokio` with feature `rt`, so cannot use other async runtimes.
//! - Depend on [scc](https://crates.io/crates/scc) for lock-free and async `HashSet`.
//!
//! Use this crate if:
//! - Easy to generate an **unique** `task_id` (not necessarily `String`) for a future (task).
//! - Don't want tasks with the same `task_id` to succeed more then once.
//! - Want to record and query all succeeded tasks and failed tasks.
//! - Want to handling every task in the same state (not just focus on one state).
//! - Need linearizable query.
//! - Want to revoke a task, and don't want the revoking to succeed more then once.
//!
//! [Example](https://github.com/Ayana-chan/ipfs_storage_cruster/tree/master/crates/async_tasks_recorder/examples).
//!
//! A recorder can only use one `task_id` type. The type of `task_id` should be:
//! - `Eq + Hash + Clone + Send + Sync + 'static`
//! - Cheap to clone (sometimes can use `Arc`).
//!
//! # Usage
//!
//! Launch a task with a **unique** `task_id` and a `Future` by [launch](AsyncTasksRecoder::launch).
//!
//! Query the state of the task with its `task_id`
//! by [query_task_state](AsyncTasksRecoder::query_task_state) or [query_task_state_quick](AsyncTasksRecoder::query_task_state_quick).
//!
//! Revoke a task with its `task_id` and a `Future` for revoking by [revoke_task_block](crate::AsyncTasksRecoder::revoke_task_block).
//!
//! ## Skills
//!
//! Remember that you can add **anything** in the `Future` to achieve the functionality you want.
//! For example:
//! - Handle your `Result` in `Future`, and then return empty result `Result<(),()>`.
//! - Send a message to a one shot channel at the end of the `Future` to notify upper level that "This task is done".
//! Don't forget to consider using `tokio::spawn` when the channel may not complete sending immediately.
//! - Set other callback functions.
//!
//! It's still efficient to store metadata of tasks at external `scc::HashMap` (`task_id` \-\> metadata).
//!
//! > It is recommended to directly look at the source code (about 150 line) if there is any confusion.
//!
//! ## When Shouldn't Use This Crate
//!
//! The consumption of all operations in this crate and cloning times
//! is about two to three times that of the implementation using `scc::Hashmap`.
//!
//! This crate use **three `HashSet`** to make it easy to operate all tasks in the same state,
//! And two more `HashSet` for linearizability of query and supporting revoking operation.
//!
//! Note that `scc`'s containers have less contention in **single** access when it grows larger.
//!
//! Therefore, if you don't need operating every task in the same state,
//! then just use `scc::HashMap` (`task_id` \-\> `task_status`) to build a simpler implementation,
//! which might have less contention and cloning, but more expansive to iterate.
//! And the `scc::HashMap::update_async` could be a powerful tool for atomic operations.
//!
//! You should also avoid using this crate if you just want to handle every tasks in only one state.
//! For example, if you just want to manage the failed tasks,
//! then you should use `scc::HashMap` to record tasks' states,
//! and insert the failed tasks into a external `Arc<scc::HashSet>` in `Future`.
//!
//! # Theory & Design
//!
//! ## Abstract Model
//! Here is the three-level structure for thinking about tasks' status:
//! - Level 0: `real_not_found`, `real_failed`, `real_working`, `real_success` : **Exact status** of the tasks in the CPU (seen by God).
//! - Level 1: `failed_tasks`, `working_tasks`, `success_tasks` : **Containers** to store `task_id`s (a `task_id` can be stored in 0 to 2 containers simultaneously).
//! - Level 2: `NotFound`, `Failed`, `Working`, `Success` : **States** of the task that could be obtained by `query_task_state`.
//!
//! ## State Transition Diagram
//! - `NotFound` \-\-\-\-\> `Working` (first launch)
//! - `Working` \-\-\-\-\> `Failed` (task failed)
//! - `Failed` \-\-\-\-\> `Working` (first launch after failed)
//! - `Working` \-\-\-\-\> `Success` (task success)
//! - `Success` \-\-\-\-\> `NotFound` (revoke)
//!
//! If you equivalent `NotFound` to `Failed`, and ignore `revoke`, then:
//!
//! `Failed` \<\-\-\-\> `Working` \-\-\-\-\> `Success`
//!
//! ## Nature
//! ### About Task
//! 1. A task is **launched** by passing a `Future<Output=Result<R, E>>` with unique `task_id`.
//! 2. A task is `real_success` when return `Ok(R)`, and `real_failed` when return `Err(E)`.
//! 3. Different future with **the same `task_id`** is considered **the same task**.
//! 4. The same task **can only `real_success` once**, e.g. a purchase process would never succeed more then once by launching with unique process id as `task_id`.
//! 5. A succeeded task can only be **revoked** successfully **once**.
//!
//! ### About Task State
//! 1. If a task's state is `Success`, it must be `real_success`, i.e. $\text{Success}(id) \rightarrow \text{real\_success}(id)$.
//! 2. If a task's state is `Failed`, it may be in any status, but mostly `real_failed`.
//! 3. If a task's state is `Working`, it may be in any status, but mostly `real_working`.
//! 4. If a task's state is `NotFound`, it may be in any status, but mostly `real_not_found`.
//!
//! ### About Task State Transition
//! 1. Any task's state can be **queried** at any time.
//! 2. The initial state of the task is `NotFound`.
//! 3. Task's state won't change immediately after `launch` called. But if you query after `launch().await`, you will get changed result.
//! 4. Always, when a task whose state is `Failed` or `NotFound` is launched, it will be `Working` at some future moment.
//! 5. Always, when a task is `Working`, it would eventually be `Fail` or `Success`.
//! 6. Always, when a task is `Success`, it would keep `Success` until the revoking succeed, and then become `NotFound`.
//!
//! # Other
//! Further propositions and proofs at [AsyncTasksRecoder](AsyncTasksRecoder).
//!
//! Relationship between states and containers at [query_task_state](AsyncTasksRecoder::query_task_state).
//!
//! Use [query_task_state_quick](AsyncTasksRecoder::query_task_state_quick) for less contention.
//!

use std::borrow::Borrow;
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;

pub use scc;

/// Thread-safe.
///
/// Everything is public, so hash functions and initial capacity can be customized.
#[derive(Default, Debug)]
pub struct TaskManager<T>
    where T: Eq + Hash {
    /// All tasks that have launched
    pub all_tasks: scc::HashSet<T>,
    /// Tasks on execution. Usually more contention.
    pub working_tasks: scc::HashSet<T>,
    /// Succeeded tasks.
    pub success_tasks: scc::HashSet<T>,
    /// Failed tasks.
    pub failed_tasks: scc::HashSet<T>,
    /// Tasks that is going to be revoked. Just for [revoke_task_block](crate::AsyncTasksRecoder::revoke_task_block).
    pub revoking_tasks: scc::HashSet<T>,
}

impl<T> TaskManager<T>
    where T: Eq + Hash {
    /// Create default and empty `TaskManager`
    pub fn new() -> Self {
        TaskManager {
            all_tasks: scc::HashSet::new(),
            working_tasks: scc::HashSet::new(),
            success_tasks: scc::HashSet::new(),
            failed_tasks: scc::HashSet::new(),
            revoking_tasks: scc::HashSet::new(),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum TaskState {
    /// running or pending
    Working,
    Success,
    Failed,
    NotFound,
}

/// Arc was used internally, so after `clone`, the same `TaskManager` was used,
/// which means you can share `AsyncTasksRecoder` by clone.
///
/// # Usage
///
/// Launch a task with a **unique** `task_id` and a `Future` by [launch](AsyncTasksRecoder::launch).
///
/// Query the state of the task with its `task_id`
/// by [query_task_state](AsyncTasksRecoder::query_task_state) or [query_task_state_quick](AsyncTasksRecoder::query_task_state_quick).
///
/// # Further Propositions & Proofs
///
/// ## P01
/// **A task (or tasks with the same `task_id`) wouldn't be executed again after first success.**
///
/// When a task fail, it wouldn't break anything. Failure just means the task could be launched again,
/// so if this proposition (**P01**) is true, there is almost nothing to worry about.
/// For further discussion, please refer to **P02**.
///
/// From now on, only consider the situation of success.
///
/// `working_tasks` play the role of lock,
/// which allow tasks with the same `task_id` to execute remaining codes (after `insert` & before `remove`) only once.
/// And before `remove` from `working_tasks`, the succeeded `task_id` has been in `success_tasks`.
///
/// An equivalent pseudocode can be obtained.
/// - `working_tasks` become a **mutex** (maybe a **spin lock**) for one `task_id`.
/// - `success_tasks` become an atomic boolean, which can only change from false to true.
/// - An execution of a task becomes adding on an atomic int (`count`).
///
/// Therefore, if the `count` is never greater than 1, it means that the task will only be called once.
///
/// ```not_rust
/// let working_task_id = mutex::new();
/// let success_task_id = atomic(false);
/// let count = atomic(0);
/// launch_multi_thread {
///     working_task_id.lock();
///     if success_task_id.get() {
///         exit();
///     }
///     count.add(1);
///     success_task_id.set(true);
///     working_task_id.unlock();
/// }
/// assert_eq!(count.get(), 1);
/// ```
///
/// Obviously, `success_tasks.set(true)` can only be executed once.
/// After that, `exit()` is always called.
/// This results in `count.add(1)` being called only once, too. Q.E.D.
///
/// ## P02
/// **Task failure is not harmful for recorder.**
///
/// Considering the situation of failure, the pseudocode becomes like this:
///
/// ```not_rust
/// let working_task_id = mutex::new();
/// let success_task_id = atomic(false);
/// let failed_task_id = atomic(false); // Initially not in `failed_tasks`, but not important
/// let count = atomic(0);
/// launch_multi_thread {
///     working_task_id.lock();
///     if success_task_id.get() {
///         exit();
///     }
///     // Here should be `real_working`
///     failed_task_id.set(false); // So it shouldn't be `Failed`, just remove from `failed_tasks`
///     count.add(1);
///     if real_success {
///         success_task_id.set(true);
///     } else {
///         // `real_failed`
///         failed_task_id.set(true); // become `Failed`
///     }
///     working_task_id.unlock();
/// }
/// assert_eq!(count.get(), 1);
/// ```
///
/// In a launch (critical section by `working_tasks`), the initial value of failed is ignored.
/// Therefore, it's not important whether `failed_tasks` changes are atomic for launches.
///
/// From the perspective of `query_task_state`,
/// `failed_tasks` is only meaningful when `task_id` is in it.
///
/// `task_id` is in `failed_tasks` only when it become `real_failed` and before redo (next `real_working`).
/// Very good.
///
/// ## P03
/// **No state would turn back to `NotFound`.**
///
/// From the pseudocode in **P02**:
///
/// ```not_rust
/// // entry `working_task_id`
/// if real_success {
///     success_task_id.set(true);
/// } else {
///     // `real_failed`
///     failed_task_id.set(true); // Become `Failed`
/// }
/// // leave `working_task_id`
/// ```
///
/// It can be found that as long as the task has entered `working_tasks` once,
/// when exiting `working_tasks`,
/// the task must already be in one of the `failed_tasks` or `success_tasks` options.
///
/// So after first `Working`, the task must be in one of `tasks`,
/// then it won't be `NotFound` again. Q.E.D.
///
/// ## P04
/// **If you query after `launch().await`, you will get changed result.**
///
/// `launch()` finishes just before `Future.await`.
/// So before `launch()` finishes, all `tasks` has been changed,
/// which means you won't get outdated `Failed` or `NotFound` after `launch().await`.
///
/// ## P05
/// **Query is linearizable.**
///
/// Query logic:
/// 1. check `all_task` -> `NotFound`
/// 2. check `working_tasks` -> `Working`
/// 3. check `success_tasks` -> `Success`
/// 4. check `failed_tasks` -> `Failed`
/// 5. check `all_tasks` -> `Working` if contained, otherwise `NotFound`
///
/// Linearizability: An event can be considered to occur at a moment
/// during the time period between a request and a response.
/// If all events can be sorted on a linear timeline,
/// then the model is linearizable.
///
/// For example, task might still be in `working_tasks` when `real_failed` or `real_success`,
/// but it is still linearizable to return `Working` at this moment.
/// Because as soon as the task is removed from `working_tasks`,
/// any query later won't return outdated `Working`.
///
/// The same reasoning can be used to prove situations 1, 2, 3 and 4.
///
/// We seem to be able to draw this conclusion:
/// If I mark a path in state transition diagram,
/// and label each step with number `1, 2, .., n` ,
/// then it is linearizable when
/// the number corresponding to the query result **never** decreases.
///
/// Don't worry, the delay is usually none or occasionally very low
/// (This can be proven, but it's too difficult to explain clearly. I'll skip it here).
///
/// What is the situation when we cannot find the target in 4 tasks (situation 5)?
///
/// This is like a turtle and rabbit race.
/// Because the `all_tasks` is checked first,
/// so let's assume that the task begins at `Working`.
/// If its actual path is:
/// 1. `Fail` (from `Working`)
/// 2. `Working`
/// 3. `Fail`
/// 4. `Working`
/// 5. `Success`
/// 6. `NotFound`
/// 7. `Working`
/// 8. ...
///
/// At step 1, the task has left `working_tasks` and has been in `failed_tasks`
/// when we are querying `working_tasks`.
/// And at step 2, it has come back to `working_tasks`
/// when we are querying `failed_tasks`.
/// Then it maybe continue to jump several steps.
/// In this case, it is still linearizable when `Working` (step 2) returned,
/// because every query later won't return earlier steps.
/// Always returning `Failed` (step 1) in this case also keeps it linearizable, but not good.
///
/// Note that the task can jump any steps, so it might have been `Working` 10 times already between your query.
///
/// So we only need to select one of `Working` or `NotFound` to return:
/// - If succeeded once, and it was just not in `success_tasks`,
/// then it might be `NotFound`, `Working` or steps ahead of `Working` (maybe next `NotFound`).
/// - If failed once, and it was just not in `failed_tasks`,
/// then it might be `Working` or steps ahead of `Working` (maybe next `NotFound`).
/// - If it is `NotFound`, return `NotFound`, otherwise return `Working`.
///
/// And a proposition will hold in other situations:
/// A task is `NotFound` when it is **not** in `all_tasks`.
/// Because:
/// 1. `all_tasks` has high priority in query logic.
/// 2. When revoke a task, it is removed from `all_tasks` before removing from `success_tasks`.
/// 3. When launch a task, it is insert into `working_tasks` before inserting into `all_tasks`.
///
/// Just follow this proposition, we get:
/// If the task is not in `all_tasks`, return `NotFound`, otherwise return `Working`.
///
#[derive(Default, Debug, Clone)]
pub struct AsyncTasksRecoder<T>
    where T: Eq + Hash + Clone + Send + 'static {
    task_manager: Arc<TaskManager<T>>,
}

impl<T> AsyncTasksRecoder<T>
    where T: Eq + Hash + Clone + Send + Sync + 'static {
    /// Create a completely new `AsyncTasksRecoder`.
    pub fn new() -> Self {
        AsyncTasksRecoder {
            task_manager: TaskManager::new().into(),
        }
    }

    /// Create by `TaskManager`
    pub fn new_with_task_manager(task_manager: TaskManager<T>) -> Self {
        AsyncTasksRecoder {
            task_manager: task_manager.into(),
        }
    }

    /// Create by `Arc` of `TaskManager`
    pub fn new_with_task_manager_arc(task_manager: Arc<TaskManager<T>>) -> Self {
        AsyncTasksRecoder {
            task_manager,
        }
    }

    /// Launch a task that returns `Result<R, E>`.
    ///
    /// - `task_id`: Uniquely mark a task. Different `Future` with **the same `task_id`** is considered **the same task**.
    /// - `task`: A `Future` to be executed automatically.
    ///
    /// The returned result can just be ignored.
    ///
    /// - Return `Ok(())` if this launch effectively gets the task into working.
    /// - Return `Err((TaskState, Fut))` if launch canceled because the task is at `TaskState` state.
    /// `Fut` is the unused `task`. `TaskState` in `Err` can only be `Working` or `Success`.
    ///
    /// The return value of task is ignored, so please use other methods to handle the return value,
    /// such as channel or shared variable.
    ///
    /// If you query after `launch().await`, you will get changed result (**P04** at [AsyncTasksRecoder](crate::AsyncTasksRecoder)).
    pub async fn launch<Fut, R, E>(&self, task_id: T, task: Fut) -> Result<(), (TaskState, Fut)>
        where Fut: Future<Output=Result<R, E>> + Send + 'static,
              R: Send,
              E: Send {
        // insert working -> insert all -> check success -> remove failed -> work -> insert success/failed -> remove working
        // `working_tasks` play the role of lock
        let res = self.task_manager.working_tasks.insert_async(task_id.clone()).await;
        if res.is_err() {
            // on working
            return Err((TaskState::Working, task));
        }

        let _ = self.task_manager.all_tasks.insert_async(task_id.clone()).await;

        // check succeeded
        if self.task_manager.success_tasks.contains_async(&task_id).await {
            return Err((TaskState::Success, task));
        }
        // remove from `failed_tasks` if contained
        self.task_manager.failed_tasks.remove_async(&task_id).await;

        // adjust args
        let task_manager = self.task_manager.clone();

        // start
        let _task = tokio::spawn(async move {
            let add_pin_res = task.await;
            if add_pin_res.is_ok() {
                let _ = task_manager.success_tasks.insert_async(task_id.clone()).await;
                task_manager.working_tasks.remove_async(&task_id).await;
            } else {
                let _ = task_manager.failed_tasks.insert_async(task_id.clone()).await;
                task_manager.working_tasks.remove_async(&task_id).await;
            }
        });

        Ok(())
    }

    /// Block until task finishes ("block" means it will keep `await` until finishing).
    ///
    /// [launch](crate::AsyncTasksRecoder::launch) for more detail.
    ///
    /// Can be mixed with launch.
    ///
    /// If you only need `launch_block`, then you probably don't need this crate.
    pub async fn launch_block<Fut, R, E>(&self, task_id: T, task: Fut) -> Result<(), (TaskState, Fut)>
        where Fut: Future<Output=Result<R, E>> + Send + 'static,
              R: Send,
              E: Send {
        let res = self.task_manager.working_tasks.insert_async(task_id.clone()).await;
        if res.is_err() {
            return Err((TaskState::Working, task));
        }

        let _ = self.task_manager.all_tasks.insert_async(task_id.clone()).await;

        if self.task_manager.success_tasks.contains_async(&task_id).await {
            return Err((TaskState::Success, task));
        }
        self.task_manager.failed_tasks.remove_async(&task_id).await;

        // start (block)
        let add_pin_res = task.await;
        if add_pin_res.is_ok() {
            let _ = self.task_manager.success_tasks.insert_async(task_id.clone()).await;
            self.task_manager.working_tasks.remove_async(&task_id).await;
        } else {
            let _ = self.task_manager.failed_tasks.insert_async(task_id.clone()).await;
            self.task_manager.working_tasks.remove_async(&task_id).await;
        }

        Ok(())
    }

    /// Query the state of a task by `task_id`.
    ///
    /// Linearizability guaranteed (**P05** at [AsyncTasksRecoder](crate::AsyncTasksRecoder)).
    ///
    /// **NOTE**: `working_tasks` usually has more contention.
    pub async fn query_task_state<Q>(&self, task_id: &Q) -> TaskState
        where T: Borrow<Q>,
              Q: Hash + Eq + ?Sized {

        // quick check `NotFound`
        if !self.task_manager.all_tasks.contains_async(task_id).await {
            return TaskState::NotFound;
        }

        if self.task_manager.working_tasks.contains_async(task_id).await {
            return TaskState::Working;
        }

        if self.task_manager.success_tasks.contains_async(task_id).await {
            return TaskState::Success;
        }

        if self.task_manager.failed_tasks.contains_async(task_id).await {
            return TaskState::Failed;
        }

        // maybe has become the next state of `Success` or `Failed`
        if self.task_manager.all_tasks.contains_async(task_id).await {
            TaskState::Working // next of `Failed`
        } else {
            TaskState::NotFound // next of `Success`
        }
    }

    /// Return `Working` if not in either `success_tasks` or `failed_tasks`.
    ///
    /// Less Query and much less contention.
    ///
    /// Even when the `task_id`'s launch have not occurred, return `Working`.
    ///
    /// Use this if you are certain that the task's launch must occur at some point in the past or future,
    /// and don't care about when the launch occurs
    /// (because first launch always turns into `Working` at some point).
    pub async fn query_task_state_quick<Q>(&self, task_id: &Q) -> TaskState
        where T: Borrow<Q>,
              Q: Hash + Eq + ?Sized {
        if self.task_manager.success_tasks.contains_async(task_id).await {
            return TaskState::Success;
        }

        if self.task_manager.failed_tasks.contains_async(task_id).await {
            return TaskState::Failed;
        }

        TaskState::Working
    }

    /// Revoke succeeded task.
    /// Block until `revoke_task` finishes ("block" means it will keep `await` until finishing).
    /// Linearizability guaranteed.
    ///
    /// - `target_task_id`: The `task_id` of the target task that you want to revoke.
    /// - `revoke_task`: The `Future` to revoke the task (e.g. Delete a picture which is downloaded before).
    ///
    /// Ignoring the returned result can also keep linearizability.
    ///
    /// - Return `Ok(R)` if succeed (R is result of `revoke_task`). In this case, the `target_task_id` is removed from `success_tasks`.
    /// - Return `Err(RevokeFailReason<Fut, E>)` if `revoke_task` canceled or `revoke_task` returns `Err(E)`.
    ///
    /// If you want to revoke asynchronously, you could (all deadlock-free):
    /// 1. Use another `AsyncTasksRecoder`, and launch a `Future` that call `revoke_task_block` here.
    /// 2. Create a new unique `task_id` for this `revoke_task`, and launch it in this `AsyncTasksRecoder`.
    ///
    /// check not revoking and set revoking -> check succeeded -> do revoke_task -> set not launched -> set not succeeded -> set not revoking
    pub async fn revoke_task_block<Fut, R, E>(&self, target_task_id: T, revoke_task: Fut) -> Result<R, RevokeFailReason<Fut, E>>
        where Fut: Future<Output=Result<R, E>> + Send + 'static,
              R: Send,
              E: Send {
        // should not revoking
        let res = self.task_manager.revoking_tasks.insert_async(target_task_id.clone()).await;
        if res.is_err() {
            return Err(RevokeFailReason::Revoking(revoke_task));
        }
        // should in success
        if !self.task_manager.success_tasks.contains_async(&target_task_id).await {
            return Err(RevokeFailReason::NotSuccess(revoke_task));
        }

        // start (block)
        let revoke_task_res = revoke_task.await;

        match revoke_task_res {
            // revoke task is err
            Err(e) => {
                // set not revoke
                self.task_manager.revoking_tasks.remove_async(&target_task_id).await;

                Err(RevokeFailReason::RevokeTaskError(e))
            }
            // success
            Ok(res) => {
                // set not ever launched
                self.task_manager.all_tasks.remove_async(&target_task_id).await;
                // set not success
                self.task_manager.success_tasks.remove_async(&target_task_id).await;
                // set not revoke
                self.task_manager.revoking_tasks.remove_async(&target_task_id).await;

                Ok(res)
            }
        }
    }

    /// Get a cloned `Arc` of `task_manager`.
    /// Then you can do anything you want (Every containers are public).
    pub fn get_task_manager_arc(&self) -> Arc<TaskManager<T>> {
        self.task_manager.clone()
    }

    /// Get a reference of `success_tasks`.
    pub fn get_success_tasks_ref(&self) -> &scc::HashSet<T> {
        &self.task_manager.success_tasks
    }

    /// Get a reference of `working_tasks`.
    pub fn get_working_tasks_ref(&self) -> &scc::HashSet<T> {
        &self.task_manager.working_tasks
    }

    /// Get a reference of `failed_tasks`.
    pub fn get_failed_tasks_ref(&self) -> &scc::HashSet<T> {
        &self.task_manager.failed_tasks
    }

    /// Get a reference of `revoking_tasks`. Not commonly used.
    pub fn get_revoking_tasks_ref(&self) -> &scc::HashSet<T> {
        &self.task_manager.revoking_tasks
    }

    /// Get a reference of `all_tasks`. Not commonly used.
    pub fn get_all_tasks_ref(&self) -> &scc::HashSet<T> {
        &self.task_manager.all_tasks
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum RevokeFailReason<Fut, E>
    where Fut: Send,
          E: Send {
    NotSuccess(Fut),
    Revoking(Fut),
    RevokeTaskError(E),
}

