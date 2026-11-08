//! 摸底实验 T-1：napi 的普通 Rust 替身。
//!
//! audio-engine 的 `src/bindings/` 是写给 Node.js 用的。这里用普通 Rust 实现它用到的那一小部分接口，
//! 让引擎源码一个字不改就能当普通 Rust 库用：错误是字符串，回调是闭包，异步任务跑在 tokio 上。

pub mod bindgen_prelude {
    use std::fmt;
    use std::sync::Arc;

    pub use crate::threadsafe_function::ThreadsafeFunction;

    /// 对应 `napi::Error`：只保留错误文案，引擎靠 `reason` 的前缀判断错误类别。
    #[derive(Debug, Clone)]
    pub struct Error {
        pub reason: String,
    }

    impl Error {
        pub fn from_reason(reason: impl Into<String>) -> Self {
            Self {
                reason: reason.into(),
            }
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.reason)
        }
    }

    impl std::error::Error for Error {}

    pub type Result<T, E = Error> = std::result::Result<T, E>;

    /// 对应 JS 的 Buffer：在 Rust 里就是普通的 Vec。
    pub type Buffer = Vec<u8>;

    /// 对应 JS 的 Float32Array：包一层 Vec，保留 napi 的 `new(Vec<f32>)` 写法。
    #[derive(Debug, Clone, Default)]
    pub struct Float32Array(Vec<f32>);

    impl Float32Array {
        pub fn new(data: Vec<f32>) -> Self {
            Self(data)
        }
    }

    impl std::ops::Deref for Float32Array {
        type Target = [f32];
        fn deref(&self) -> &[f32] {
            &self.0
        }
    }

    /// 对应 `napi::Env`：普通 Rust 里没有 JS 环境，只是个占位。
    pub struct Env;

    /// 对应 `napi::Task`：`compute` 在阻塞线程里跑，`resolve` 把结果交回调用方。
    pub trait Task: Send + Sized + 'static {
        type Output: Send + 'static;
        type JsValue;
        fn compute(&mut self) -> Result<Self::Output>;
        fn resolve(&mut self, env: Env, output: Self::Output) -> Result<Self::JsValue>;
    }

    /// 对应 `napi::AsyncTask`：用 `run().await` 拿结果。
    pub struct AsyncTask<T: Task>(T);

    impl<T: Task> AsyncTask<T> {
        pub fn new(task: T) -> Self {
            Self(task)
        }

        pub async fn run(self) -> Result<T::JsValue> {
            let mut task = self.0;
            let (mut task, output) = tokio::task::spawn_blocking(move || {
                let output = task.compute();
                (task, output)
            })
            .await
            .map_err(|e| Error::from_reason(format!("async task join error: {e}")))?;
            task.resolve(Env, output?)
        }
    }

    /// 对应 JS 回调函数：在 Rust 里就是一个能跨线程调用的闭包。
    pub struct Function<A, R = ()> {
        f: Arc<dyn Fn(A) -> R + Send + Sync>,
    }

    impl<A, R> Function<A, R> {
        pub fn new(f: impl Fn(A) -> R + Send + Sync + 'static) -> Self {
            Self { f: Arc::new(f) }
        }

        pub fn build_threadsafe_function(&self) -> ThreadsafeFunctionBuilder<A, R> {
            ThreadsafeFunctionBuilder {
                f: Arc::clone(&self.f),
            }
        }
    }

    pub struct ThreadsafeFunctionBuilder<A, R> {
        f: Arc<dyn Fn(A) -> R + Send + Sync>,
    }

    impl<A, R> ThreadsafeFunctionBuilder<A, R> {
        pub fn build(self) -> Result<ThreadsafeFunction<A, R>> {
            Ok(ThreadsafeFunction::new(self.f))
        }
    }
}

pub mod threadsafe_function {
    use std::sync::Arc;

    /// 对应 napi 的两种调用方式：普通闭包里都是直接调用。
    #[derive(Debug, Clone, Copy)]
    pub enum ThreadsafeFunctionCallMode {
        Blocking,
        NonBlocking,
    }

    pub struct ThreadsafeFunction<A, R = ()> {
        f: Arc<dyn Fn(A) -> R + Send + Sync>,
    }

    impl<A, R> ThreadsafeFunction<A, R> {
        pub(crate) fn new(f: Arc<dyn Fn(A) -> R + Send + Sync>) -> Self {
            Self { f }
        }

        pub fn call(&self, value: A, _mode: ThreadsafeFunctionCallMode) -> R {
            (self.f)(value)
        }
    }
}

pub use bindgen_prelude::{Error, Result};
