use crate::refcounted::{RefCountedPtr, Wrapper};
use cef_sys::{
    _cef_task_runner_t, _cef_task_t, cef_currently_on, cef_post_delayed_task, cef_post_task,
    cef_task_runner_get_for_current_thread, cef_task_runner_get_for_thread, cef_thread_id_t,
};
use parking_lot::Mutex;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ThreadId {
    UI = cef_thread_id_t::TID_UI as isize,
    FileBackground = cef_thread_id_t::TID_FILE_BACKGROUND as isize,
    FileUserVisible = cef_thread_id_t::TID_FILE_USER_VISIBLE as isize,
    FileUserBlocking = cef_thread_id_t::TID_FILE_USER_BLOCKING as isize,
    ProcessLauncher = cef_thread_id_t::TID_PROCESS_LAUNCHER as isize,
    IO = cef_thread_id_t::TID_IO as isize,
    Renderer = cef_thread_id_t::TID_RENDERER as isize,
}

ref_counted_ptr! {
    /// Structure that asynchronously executes tasks on the associated thread. It is
    /// safe to call the functions of this structure on any thread.
    ///
    /// CEF maintains multiple internal threads that are used for handling different
    /// types of tasks in different processes. The [ThreadId] definitions
    /// list the common CEF threads. TaskWrapper runners are also available for
    /// other CEF threads as appropriate (for example, V8 WebWorker threads).
    pub struct TaskRunner(*mut _cef_task_runner_t);
}

impl TaskRunner {
    /// Returns the task runner for the current thread. Only CEF threads will have
    /// task runners. None will be returned if this function is called
    /// on an invalid thread.
    pub fn get_for_current_thread() -> Option<Self> {
        unsafe { Self::from_ptr(cef_task_runner_get_for_current_thread()) }
    }
    /// Returns the task runner for the specified CEF thread.
    pub fn get_for_thread(thread_id: ThreadId) -> Option<Self> {
        unsafe { Self::from_ptr(cef_task_runner_get_for_thread(thread_id as i32)) }
    }
    /// Returns true if called on the specified thread. Equivalent to using
    /// `TaskRunner::get_for_thread(thread_id).belongs_to_current_thread()`.
    pub fn currently_on(thread_id: ThreadId) -> bool {
        unsafe { cef_currently_on(thread_id as cef_thread_id_t::Type) != 0 }
    }
    /// Post a task for execution on the specified thread. Equivalent to using
    /// `TaskRunner::get_for_thread(thread_id).post_task(task)`.
    pub fn post_task_on(thread_id: ThreadId, task: impl FnOnce() + Send + 'static) -> bool {
        unsafe {
            cef_post_task(
                thread_id as cef_thread_id_t::Type,
                TaskWrapper::new(task).wrap().into_raw(),
            ) != 0
        }
    }
    /// Post a task for delayed execution on the specified thread. Equivalent to
    /// using `TaskRunner::get_for_thread(thread_id)->post_delayed_task(task, delay_ms)`.
    pub fn post_delayed_task_on(
        thread_id: ThreadId,
        task: impl FnOnce() + Send + 'static,
        delay_ms: i64,
    ) -> bool {
        unsafe {
            cef_post_delayed_task(
                thread_id as cef_thread_id_t::Type,
                TaskWrapper::new(task).wrap().into_raw(),
                delay_ms,
            ) != 0
        }
    }

    /// Returns true if this object is pointing to the same task runner as
    /// `that` object.
    pub fn is_same(&self, that: &Self) -> bool {
        self.0
            .is_same
            .map(|is_same| unsafe { is_same(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if this task runner belongs to the current thread.
    pub fn belongs_to_current_thread(&self) -> bool {
        self.0
            .belongs_to_current_thread
            .map(|belongs_to_current_thread| unsafe {
                belongs_to_current_thread(self.as_ptr()) != 0
            })
            .unwrap_or(false)
    }
    /// Returns true if this task runner is for the specified CEF thread.
    pub fn belongs_to_thread(&self, thread_id: ThreadId) -> bool {
        self.0
            .belongs_to_thread
            .map(|belongs_to_thread| unsafe {
                belongs_to_thread(self.as_ptr(), thread_id as cef_thread_id_t::Type) != 0
            })
            .unwrap_or(false)
    }
    /// Post a task for execution on the thread associated with this task runner.
    /// Execution will occur asynchronously.
    pub fn post_task(&self, task: impl FnOnce() + Send + 'static) -> bool {
        self.0
            .post_task
            .map(|post_task| unsafe {
                post_task(self.as_ptr(), TaskWrapper::new(task).wrap().into_raw()) != 0
            })
            .unwrap_or(false)
    }
    /// Post a task for delayed execution on the thread associated with this task
    /// runner. Execution will occur asynchronously. Delayed tasks are not
    /// supported on V8 WebWorker threads and will be executed without the
    /// specified delay.
    pub fn post_delayed_task(
        &self,
        task: impl FnOnce() + Send + 'static,
        delay_ms: i64,
    ) -> bool {
        self.0
            .post_delayed_task
            .map(|post_delayed_task| unsafe {
                post_delayed_task(self.as_ptr(), TaskWrapper::new(task).wrap().into_raw(), delay_ms) != 0
            })
            .unwrap_or(false)
    }
}

pub struct TaskWrapper(Mutex<Option<Box<dyn FnOnce() + Send + 'static>>>);

ref_counted_ptr!{
    struct TaskPtr(*mut _cef_task_t);
}

impl TaskWrapper {
    pub(crate) fn new(task: impl FnOnce() + Send + 'static) -> Self {
        TaskWrapper(Mutex::new(Some(Box::new(task))))
    }
}

impl Wrapper for TaskWrapper {
    type Cef = _cef_task_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            _cef_task_t {
                base: unsafe { std::mem::zeroed() },
                execute: Some(Self::execute),
            },
            self,
        )
    }
}

cef_callback_impl! {
    impl for TaskWrapper: _cef_task_t {
        fn execute(&self) {
            if let Some(task) = self.0.lock().take() {
                task();
            }
        }
    }
}
