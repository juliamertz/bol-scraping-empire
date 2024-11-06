use anyhow::Result;
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use std::io::{self};
use std::sync::Arc;
use std::{fmt::Display, sync::atomic::AtomicU32};

#[derive(Default, Debug)]
pub struct QueryStatus {
    pub total: AtomicU32,
    pub pending: AtomicU32,
    pub done: AtomicU32,
    pub errored: AtomicU32,
}

#[derive(Default, Debug, Clone)]
pub struct Status {
    queries: Arc<QueryStatus>,
}

use std::sync::atomic::Ordering;
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

    pub fn incr_pending(&self) {
        self.queries.total.fetch_add(1, Ordering::SeqCst);
        self.queries.pending.fetch_add(1, Ordering::SeqCst);
        self.render(&mut io::stdout()).unwrap();
    }

    pub fn pending_done(&self) {
        self.queries.pending.fetch_sub(1, Ordering::SeqCst);
        self.queries.done.fetch_add(1, Ordering::SeqCst);
        self.render(&mut io::stdout()).unwrap();
    }

    pub fn pending_errored(&self) {
        self.queries.pending.fetch_sub(1, Ordering::SeqCst);
        self.queries.errored.fetch_add(1, Ordering::SeqCst);
        self.render(&mut io::stdout()).unwrap();
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "total: {}", self.queries.total.load(Ordering::SeqCst))?;
        writeln!(
            f,
            "pending: {}",
            self.queries.pending.load(Ordering::SeqCst)
        )?;
        writeln!(f, "done: {}", self.queries.done.load(Ordering::SeqCst))?;
        writeln!(
            f,
            "errored: {}",
            self.queries.errored.load(Ordering::SeqCst)
        )
    }
}
