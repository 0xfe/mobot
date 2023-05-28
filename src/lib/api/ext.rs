use crate::API;

use crate::api;

use super::Message;

impl API {
    /// Acknowledge a callback query.
    pub async fn acknowledge_callback(
        &self,
        query_id: String,
        text: Option<String>,
    ) -> anyhow::Result<bool> {
        let mut req = api::AnswerCallbackQueryRequest::new(query_id);
        if text.is_some() {
            req = req.with_text(text.unwrap());
        }

        self.answer_callback_query(&req).await
    }

    /// Remove the inline keyboard from a message.
    pub async fn remove_inline_keyboard(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> anyhow::Result<Message> {
        // Remove the inline keyboard.
        self.edit_message_reply_markup(&api::EditMessageReplyMarkupRequest {
            base: api::EditMessageBase::new()
                .with_chat_id(chat_id)
                .with_message_id(message_id)
                .with_reply_markup(api::ReplyMarkup::inline_keyboard_markup(vec![vec![]])),
        })
        .await
    }

    /// Send a "Typing..." chat action.
    pub async fn send_typing(&self, chat_id: i64) -> anyhow::Result<bool> {
        self.send_chat_action(&api::SendChatActionRequest::new(
            chat_id,
            api::ChatAction::Typing,
        ))
        .await
    }
}
