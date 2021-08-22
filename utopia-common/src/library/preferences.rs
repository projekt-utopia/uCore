use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum DiagType {
	// uuid of item
	Item(String),
	Module
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InputType {
	Text(String),
	Email(String),
	Password(String)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FieldType {
	Input(InputType),
	Checkbox(bool),
	Dropdown(Vec<String>),
	List(Vec<String>),
	KeyValueList(std::collections::HashMap<String, String>)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputField {
	pub uuid: String,
	pub title: String,
	pub r#type: FieldType
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreferencePane {
	pub fields: Vec<InputField>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreferenceDiag {
	pub uuid: String,
	pub panes: std::collections::HashMap<String, PreferencePane>
}
