use anyhow::Result;
use core::sync;
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use std::{
    fmt::Display,
    io::{self},
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

#[derive(Default, Debug, Clone)]
pub struct Status {
    queries: Arc<QueryStatus>,
}

impl Status {
    pub fn new() -> Self {
        Self {
            queries: Arc::new(QueryStatus::default()),
        }
    }

    pub fn render(&self, w: &mut impl std::io::Write) -> Result<()> {
        w.queue(cursor::MoveTo(0, 0))?;
        w.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
        write!(w, "{self}")?;
        Ok(())
    }

    pub fn add_pending(&self) {
        self.queries.total.increment();
        self.queries.pending.increment();
        self.render(&mut io::stdout()).unwrap();
    }

    pub fn pending_success(&self) {
        self.queries.pending.decrement();
        self.queries.success.increment();
        self.render(&mut io::stdout()).unwrap();
    }

    pub fn pending_errored(&self) {
        self.queries.pending.decrement();
        self.queries.errored.increment();
        self.render(&mut io::stdout()).unwrap();
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "total: {}", self.queries.total.load())?;
        writeln!(f, "pending: {}", self.queries.pending.load())?;
        writeln!(f, "done: {}", self.queries.success.load())?;
        writeln!(f, "errored: {}", self.queries.errored.load())
    }
}
