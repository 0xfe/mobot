use crate::{
    api::{self, API},
    Text,
};
use std::sync::Arc;

/// `Event` represents an event sent to a chat handler.
#[derive(Clone)]
pub struct Event {
    pub api: Arc<API>,
    pub update: crate::Update,
}

impl Event {
    pub fn new(api: Arc<API>, update: crate::Update) -> Self {
        Self { api, update }
    }

    /// Acknowledge a callback query.
    pub async fn acknowledge_callback(&self, text: Option<String>) -> anyhow::Result<bool> {
        let query_id = self.update.query_id()?.to_string();

        let mut req = api::AnswerCallbackQueryRequest::new(query_id);
        if text.is_some() {
            req = req.with_text(text.unwrap());
        }

        self.api.answer_callback_query(&req).await
    }

    /// Remove the inline keyboard from a message.
    pub async fn remove_inline_keyboard(&self) -> anyhow::Result<api::Message> {
        let chat_id = self.update.chat_id()?;
        let message_id = self.update.message_id()?;

        // Remove the inline keyboard.
        self.api
            .edit_message_reply_markup(&api::EditMessageReplyMarkupRequest {
                base: api::EditMessageBase::new()
                    .with_chat_id(chat_id)
                    .with_message_id(message_id)
                    .with_reply_markup(api::ReplyMarkup::inline_keyboard_markup(vec![vec![]])),
            })
            .await
    }

    /// Send a chat action.
    pub async fn send_chat_action(&self, action: api::ChatAction) -> anyhow::Result<bool> {
        self.api
            .send_chat_action(&api::SendChatActionRequest::new(
                self.update.chat_id()?,
                action,
            ))
            .await
    }

    /// Send a message to the chat.
    pub async fn send_message(&self, text: impl Into<Text>) -> anyhow::Result<api::Message> {
        let text = text.into();

        self.api
            .send_message(
                &api::SendMessageRequest::new(self.update.chat_id()?, text.clone())
                    .with_parse_mode(text.into()),
            )
            .await
    }

    /// Edit the message with the given text (uses the parsemode of the message)
    pub async fn edit_last_message(&self, text: impl Into<String>) -> anyhow::Result<api::Message> {
        self.edit_message(self.update.message_id()?, text).await
    }

    /// Edit the message with the given text (uses the parsemode of the message)
    pub async fn edit_message(
        &self,
        message_id: i64,
        text: impl Into<String>,
    ) -> anyhow::Result<api::Message> {
        let chat_id = self.update.chat_id()?;

        self.api
            .edit_message_text(&api::EditMessageTextRequest {
                base: api::EditMessageBase::new()
                    .with_chat_id(chat_id)
                    .with_message_id(message_id),
                text: text.into(),
            })
            .await
    }

    // Delete the last message
    pub async fn delete_last_message(&self) -> anyhow::Result<bool> {
        let chat_id = self.update.chat_id()?;
        let message_id = self.update.message_id()?;

        self.api
            .delete_message(&api::DeleteMessageRequest::new(chat_id, message_id))
            .await
    }

    // Delete a specific message
    pub async fn delete_message(&self, message_id: i64) -> anyhow::Result<bool> {
        let chat_id = self.update.chat_id()?;

        self.api
            .delete_message(&api::DeleteMessageRequest::new(chat_id, message_id))
            .await
    }

    pub async fn send_menu(
        &self,
        text: impl Into<Text>,
        menu: Vec<String>,
    ) -> anyhow::Result<api::Message> {
        let text = text.into();
        let chat_id = self.update.chat_id()?;

        self.api
            .send_message(
                &api::SendMessageRequest::new(chat_id, text.clone())
                    .with_parse_mode(text.into())
                    .with_reply_markup(api::ReplyMarkup::inline_keyboard_markup(vec![menu
                        .iter()
                        .map(|item| api::InlineKeyboardButton::from(item).with_callback_data(item))
                        .collect()])),
            )
            .await
    }

    /// Send a sticker to the chat.
    pub async fn send_sticker(&self, sticker: impl Into<String>) -> anyhow::Result<api::Message> {
        self.api
            .send_sticker(&api::SendStickerRequest::new(
                self.update.chat_id()?,
                sticker.into(),
            ))
            .await
    }
}
