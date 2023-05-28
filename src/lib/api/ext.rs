use crate::API;

use crate::api;

impl API {
    pub async fn answer_callback_text(
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
}
