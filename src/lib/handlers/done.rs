use crate::{Action, Event};

pub async fn done_handler<S>(_: Event, _: S) -> Result<Action, anyhow::Error> {
    Ok(Action::Done)
}
