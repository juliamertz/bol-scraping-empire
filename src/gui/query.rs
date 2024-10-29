use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream::try_channel;
use iced::Subscription;

use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

// Just a little utility function
pub fn file<I: 'static + Hash + Copy + Send + Sync, T: ToString>(
    id: I,
    url: T,
) -> iced::Subscription<(I, Result<Progress, Error>)> {
    Subscription::run_with_id(
        id,
        download(url.to_string()).map(move |progress| (id, progress)),
    )
}

fn download(url: String) -> impl Stream<Item = Result<Progress, Error>> {
    try_channel(1, move |mut output| async move {
        let _ = output.send(Progress::Downloading { percent: 0.0 }).await;

        std::thread::sleep(Duration::from_secs(2));

        let _ = output.send(Progress::Finished).await;

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub enum Progress {
    Downloading { percent: f32 },
    Finished,
}

#[derive(Debug, Clone)]
pub enum Error {
    RequestFailed(Arc<reqwest::Error>),
    NoContentLength,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::RequestFailed(Arc::new(error))
    }
}
