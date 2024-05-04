use crate::{api, Event};

/// Represent the current state of the progressbar.
enum ProgressState<'a> {
    /// The task is still in progress.
    Working,

    // The task completed successfully -- show the str.
    Done(&'a str),

    /// The task failed -- show the str.
    Failed(&'a str),
}

/// This method generates the progress bar string out of unicode block characters.
fn progress_str(i: i64, state: ProgressState) -> String {
    // Set of horizontal block characters of increasing size.
    let blocks = [
        '\u{258F}', '\u{258E}', '\u{258D}', '\u{258C}', '\u{258B}', '\u{258A}', '\u{2589}',
    ];

    let num_full_blocks = i / blocks.len() as i64;

    let bar = blocks[blocks.len() - 1]
        .to_string()
        .repeat(num_full_blocks as usize);

    let partial_bar = blocks[(i % blocks.len() as i64) as usize];

    // If the state is Done or Failed, then show an additional message. Defaults to
    // a unicode checkmark or cross.
    match state {
        ProgressState::Working => format!("{}{}", bar, partial_bar),
        ProgressState::Done(c) => format!("{}{} {}", bar, partial_bar, c),
        ProgressState::Failed(c) => format!("{}{} {}", bar, partial_bar, c),
    }
}

/// A ProgressBar that can be rendered in a telegram message. This calls a long running async
/// task and shows a progress bar while the task is running. The progress bar is updated every
/// `update_interval` seconds. If the task completes before the `timeout` then the progress bar
/// is replaced with a checkmark. If the task fails, then the progress bar is replaced with a
/// cross.
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// The timeout for the progress bar. If the task completes before this timeout, then the
    /// progress bar is replaced with a checkmark.
    pub timeout: std::time::Duration,

    /// The update interval for the progress bar. The progress bar is updated every
    /// `update_interval` seconds.
    pub update_interval: std::time::Duration,

    /// The string to show when the task fails.
    pub failed_str: String,

    /// The string to show when the task completes successfully.
    pub done_str: String,

    /// If true, then show the result of the task after the progress bar.
    pub show_result: bool,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self {
            timeout: std::time::Duration::from_secs(60),
            update_interval: std::time::Duration::from_millis(500),
            failed_str: '\u{2718}'.into(),
            done_str: '\u{2714}'.into(),
            show_result: false,
        }
    }
}

impl ProgressBar {
    /// Create a new progress bar.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the timeout for the progress bar. If the task completes before this timeout, then the
    /// progress bar is replaced with a checkmark.
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the update interval for the progress bar. The progress bar is updated every
    /// `update_interval` seconds.
    pub fn with_update_interval(mut self, update_interval: std::time::Duration) -> Self {
        self.update_interval = update_interval;
        self
    }

    /// Start the progress bar. This calls the async function `f` and shows a progress bar while
    /// the task is running. The progress bar is updated every `update_interval` seconds. If the
    /// task completes before the `timeout` then the progress bar is replaced with a checkmark.
    /// If the task fails, then the progress bar is replaced with a cross.
    ///
    /// If `show_result` is true, then the result of the task is shown after the progress bar.
    ///
    /// Returns the result of the task.
    pub async fn start<F, R>(&self, e: &Event, f: F) -> anyhow::Result<R>
    where
        F: futures::Future<Output = anyhow::Result<R>> + Send + 'static,
        R: Default + Send + Sync + 'static,
    {
        // Send an empty message to get a message id for the progress bar.
        let mut message = e.send_message("...").await?;

        let (completed_tx, mut completed_rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async {
            _ = completed_tx.send(f.await);
        });

        let mut count = 0;
        let mut done = false;
        let mut result: R = R::default();

        while !done {
            tokio::select! {
                // Update the progress bar.
                _ = tokio::time::sleep(self.update_interval) => {
                    count += 1;
                    message = e.edit_message(message.message_id, progress_str(count, ProgressState::Working)).await?;
                    e.send_chat_action(api::ChatAction::Typing).await?;
                }

                // Timeout.
                _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                    done = true;
                    message = e.edit_message(message.message_id,
                        format!("{} {}", progress_str(count, ProgressState::Failed(self.failed_str.as_str())),
                            "Something's wrong!")).await?;
                }

                // The future has completed.
                v = &mut completed_rx => {
                    done = true;
                    result = v??;
                    message = e.edit_message(message.message_id,
                        progress_str(count, ProgressState::Done(self.done_str.as_str()))).await?;
                    e.delete_message(message.message_id).await?;

                }
            }
        }

        Ok(result)
    }
}
