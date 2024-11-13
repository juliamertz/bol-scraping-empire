use core::sync;
use std::{
    fmt::Display,
    sync::{atomic::Ordering, Arc},
};

#[derive(Debug, Default)]
pub struct AtomicU32(sync::atomic::AtomicU32);

static ORDER: Ordering = Ordering::SeqCst;

impl AtomicU32 {
    pub fn increment(&self) -> u32 {
        self.0.fetch_add(1, ORDER)
    }
    pub fn decrement(&self) -> u32 {
        self.0.fetch_sub(1, ORDER)
    }
    pub fn load(&self) -> u32 {
        self.0.load(ORDER)
    }
}

#[derive(Default, Debug)]
pub struct QueryStatus {
    pub total: AtomicU32,
    pub pending: AtomicU32,
    pub success: AtomicU32,
    pub errored: AtomicU32,
}

impl Display for QueryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "total: {}", self.total.load())?;
        writeln!(f, "pending: {}", self.pending.load())?;
        writeln!(f, "done: {}", self.success.load())?;
        writeln!(f, "errored: {}", self.errored.load())
    }
}

pub type OnUpdate = dyn Fn(&QueryStatus) + Send + Sync;
pub type GlobalStatus = Arc<Status>;
#[derive(Default)]
pub struct Status {
    queries: QueryStatus,
    on_update: Option<Arc<OnUpdate>>,
}

impl Status {
    pub fn new<F>(on_update: F) -> Arc<Self>
    where
        F: Fn(&QueryStatus) + Send + Sync + 'static,
    {
        Arc::new(Self {
            queries: QueryStatus::default(),
            on_update: Some(Arc::new(on_update)),
        })
    }

    fn update(&self) {
        if let Some(callback) = &self.on_update {
            callback(&self.queries);
        }
    }

    pub fn add_pending(&self) {
        self.queries.total.increment();
        self.queries.pending.increment();
        self.update();
    }

    pub fn pending_success(&self) {
        self.queries.pending.decrement();
        self.queries.success.increment();
        self.update();
    }

    pub fn pending_errored(&self) {
        self.queries.pending.decrement();
        self.queries.errored.increment();
        self.update();
    }
}
