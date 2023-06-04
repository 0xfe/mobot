use crate::{api, chat};

pub enum ProgressState<'a> {
    Working,
    Done(&'a str),
    Failed(&'a str),
}

fn progress_str(i: i64, state: ProgressState) -> String {
    let blocks = vec![
        '\u{258F}', '\u{258E}', '\u{258D}', '\u{258C}', '\u{258B}', '\u{258A}', '\u{2589}',
    ];

    let num_full_blocks = i / blocks.len() as i64;

    let bar = blocks[blocks.len() - 1]
        .to_string()
        .repeat(num_full_blocks as usize);

    let partial_bar = blocks[(i % blocks.len() as i64) as usize];

    match state {
        ProgressState::Working => format!("{}{}", bar, partial_bar),
        ProgressState::Done(c) => format!("{}{} {}", bar, partial_bar, c),
        ProgressState::Failed(c) => format!("{}{} {}", bar, partial_bar, c),
    }
}

#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub timeout: std::time::Duration,
    pub update_interval: std::time::Duration,
    pub failed_str: String,
    pub done_str: String,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_update_interval(mut self, update_interval: std::time::Duration) -> Self {
        self.update_interval = update_interval;
        self
    }

    pub async fn start<F>(&self, e: &chat::Event, f: F) -> anyhow::Result<String>
    where
        F: futures::Future<Output = anyhow::Result<String>> + Send + 'static,
    {
        // Send an empty message to get a message id for the progress bar.
        let mut message = e.send_text("...").await?;

        let (completed_tx, mut completed_rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async {
            _ = completed_tx.send(f.await);
        });

        let mut count = 0;
        let mut done = false;
        let mut result: String = "".into();

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
                    if self.show_result {
                        e.edit_message(message.message_id, result.clone()).await?;
                    } else {
                        e.delete_message(message.message_id).await?;
                    }

                }
            }
        }

        Ok(result)
    }
}
