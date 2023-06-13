use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct KeyboardButton {
    /// Text of the button. If none of the optional fields are used, it will be sent as a message when the button is pressed
    pub text: String,
    // Optional fields omitted
}

impl<T: Into<String>> From<T> for KeyboardButton {
    fn from(text: T) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct InlineKeyboardButton {
    /// Label text on the button
    pub text: String,

    /// HTTP or tg:// url to be opened when button is pressed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Callback data to be sent in a callback query to the bot when button is pressed, 1-64 bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_data: Option<String>,
}

impl<T: Into<String>> From<T> for InlineKeyboardButton {
    fn from(text: T) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }
}

impl InlineKeyboardButton {
    pub fn with_callback_data<T: Into<String>>(mut self, callback_data: T) -> Self {
        self.callback_data = Some(callback_data.into());
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ReplyMarkup {
    InlineKeyboardMarkup {
        /// Array of button rows, each represented by an Array of KeyboardButton objects
        inline_keyboard: Vec<Vec<InlineKeyboardButton>>,

        /// Requests clients to resize the keyboard vertically for optimal fit
        resize_keyboard: bool,

        /// Requests clients to hide the keyboard as soon as it's been used
        one_time_keyboard: bool,

        /// Use this parameter if you want to show the keyboard to specific users only
        selective: bool,

        /// The placeholder to be shown in the input field when the keyboard is active; 1-64 characters
        #[serde(skip_serializing_if = "Option::is_none")]
        input_field_placeholder: Option<String>,

        /// Requests clients to always show the keyboard in the chat (users may not otherwise see the keyboard)
        is_persistent: bool,
    },
    ReplyKeyboardMarkup {
        /// Array of button rows, each represented by an Array of KeyboardButton objects
        keyboard: Vec<Vec<KeyboardButton>>,

        /// Requests clients to resize the keyboard vertically for optimal fit
        resize_keyboard: bool,

        /// Requests clients to hide the keyboard as soon as it's been used
        one_time_keyboard: bool,

        /// Use this parameter if you want to show the keyboard to specific users only
        selective: bool,

        /// The placeholder to be shown in the input field when the keyboard is active; 1-64 characters
        #[serde(skip_serializing_if = "Option::is_none")]
        input_field_placeholder: Option<String>,

        /// Requests clients to always show the keyboard in the chat (users may not otherwise see the keyboard)
        is_persistent: bool,
    },
    ReplyKeyboardRemove {
        /// Requests clients to remove the custom keyboard (user will not be
        /// able to summon this keyboard; if you want to hide the keyboard from
        /// sight but keep it accessible, use one_time_keyboard in ReplyKeyboardMarkup)
        remove_keyboard: bool,

        /// Use this parameter if you want to remove the keyboard for specific users only
        selective: bool,
    },
    ForceReply {
        /// Shows reply interface to the user, as if they manually selected the bot's message and tapped 'Reply'
        force_reply: bool,

        /// The placeholder to be shown in the input field when the keyboard is active; 1-64 characters
        #[serde(skip_serializing_if = "Option::is_none")]
        input_field_placeholder: Option<String>,

        /// Use this parameter if you want to force reply from specific users only
        selective: bool,
    },
}

impl ReplyMarkup {
    pub fn inline_keyboard_markup(inline_keyboard: Vec<Vec<InlineKeyboardButton>>) -> ReplyMarkup {
        ReplyMarkup::InlineKeyboardMarkup {
            inline_keyboard,
            resize_keyboard: false,
            one_time_keyboard: false,
            selective: false,
            input_field_placeholder: None,
            is_persistent: false,
        }
    }

    pub fn reply_keyboard_markup(keyboard: Vec<Vec<KeyboardButton>>) -> ReplyMarkup {
        ReplyMarkup::ReplyKeyboardMarkup {
            keyboard,
            resize_keyboard: false,
            one_time_keyboard: true,
            selective: false,
            input_field_placeholder: None,
            is_persistent: false,
        }
    }

    pub fn reply_keyboard_remove() -> ReplyMarkup {
        ReplyMarkup::ReplyKeyboardRemove {
            remove_keyboard: true,
            selective: false,
        }
    }

    pub fn force_reply() -> ReplyMarkup {
        ReplyMarkup::ForceReply {
            force_reply: true,
            input_field_placeholder: None,
            selective: false,
        }
    }
}

impl<T: Into<String>> From<T> for ReplyMarkup {
    fn from(text: T) -> Self {
        serde_json::from_str(&text.into()).unwrap()
    }
}
