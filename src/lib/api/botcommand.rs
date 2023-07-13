use mobot_derive::BotRequest;
use serde::{Deserialize, Serialize};

use super::API;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BotCommand {
    /// Text of the command, 1-32 characters. Can contain only lowercase English
    /// letters, digits and underscores.
    pub command: String,

    /// Description of the command, 3-256 characters.
    pub description: String,
}

#[derive(Debug, Serialize, Clone, BotRequest)]
/// This strcut represents the scope type for BotCommandScope.
pub enum BotCommnandScopeType {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "all_private_chats")]
    AllPrivateChats,
    #[serde(rename = "all_group_chats")]
    AllGroupChats,
    #[serde(rename = "all_chat_administrators")]
    AllChatAdministrators,
    #[serde(rename = "chat")]
    Chat,
    #[serde(rename = "chat_administrators")]
    ChatAdministrators,
    #[serde(rename = "chat_member")]
    ChatMember,
}

#[derive(Debug, Serialize, Clone, BotRequest)]
pub struct BotCommandScope {
    #[serde(rename = "type")]
    pub type_: BotCommnandScopeType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
}

#[derive(Default, Debug, Serialize, Clone, BotRequest)]
pub struct SetMyCommandsRequest {
    /// At most 100 commands can be specified.
    pub commands: Vec<BotCommand>,

    /// A JSON-serialized object, describing scope of users for which the commands are
    /// relevant. Defaults to BotCommandScopeDefault.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<BotCommandScope>,

    /// A two-letter ISO 639-1 language code. If empty, commands will be applied to all
    /// users from the given scope, for whose language there are no dedicated commands
    /// For example, en-GB will apply to all users within the scope who use English
    /// (United Kingdom) as their language. Defaults to en-US if omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Debug, Serialize, Clone, BotRequest)]
pub struct DeleteMyCommandsRequest {
    /// A JSON-serialized object, describing scope of users for which the commands are
    /// relevant. Defaults to BotCommandScopeDefault.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<BotCommandScope>,

    /// Two-letter ISO 639-1 language code. See above.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

type GetMyCommandsRequest = DeleteMyCommandsRequest;

impl API {
    pub async fn get_my_commands(
        &self,
        req: &GetMyCommandsRequest,
    ) -> anyhow::Result<Vec<BotCommand>> {
        self.client.post("getMyCommands", req).await
    }

    pub async fn set_my_commands(&self, req: &SetMyCommandsRequest) -> anyhow::Result<bool> {
        self.client.post("setMyCommands", req).await
    }

    pub async fn delete_my_commands(&self, req: &DeleteMyCommandsRequest) -> anyhow::Result<bool> {
        self.client.post("deleteMyCommands", req).await
    }
}
