use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum DiagType {
	// uuid of item
	Item(String),
	Module
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InputNum {
	pub range: (f64, f64),
	pub value: f64,
	pub step: f64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
	Text(String),
	Email(String),
	Phone(String),
	Url(String),
	Number(InputNum),
	Password(String)
}
impl InputType {
	pub fn try_get_str(self) -> Option<String> {
		match self {
			InputType::Text(string) => Some(string),
			InputType::Email(string) => Some(string),
			InputType::Phone(string) => Some(string),
			InputType::Url(string) => Some(string),
			InputType::Password(string) => Some(string),
			_ => None
		}
	}

	pub fn try_get_num(self) -> Option<f64> {
		match self {
			InputType::Number(num) => Some(num.value),
			_ => None
		}
	}

	pub fn get_str(self) -> String {
		self.try_get_str().unwrap()
	}

	pub fn get_num(self) -> f64 {
		self.try_get_num().unwrap()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
	Input(InputType),
	Checkbox(bool),
	// (index, vec)
	Dropdown(usize, Vec<String>),
	List(Vec<String>),
	/// KeyValueLists do not honor ordering. Please don't rely on them
	/// for that.
	KeyValueList(std::collections::HashMap<String, String>)
}
impl FieldType {
	pub fn try_get_input(self) -> Option<InputType> {
		match self {
			FieldType::Input(input) => Some(input),
			_ => None
		}
	}

	pub fn try_get_checkbox(self) -> Option<bool> {
		match self {
			FieldType::Checkbox(toggled) => Some(toggled),
			_ => None
		}
	}

	pub fn try_get_dropdown_index(self) -> Option<usize> {
		match self {
			FieldType::Dropdown(idx, _) => Some(idx),
			_ => None
		}
	}

	pub fn try_get_dropdown_ref(&self) -> Option<&String> {
		match self {
			FieldType::Dropdown(idx, values) => values.get(*idx),
			_ => None
		}
	}

	/// note: Panics if index is out of bounds.
	pub fn try_get_dropdown_value(self) -> Option<String> {
		match self {
			FieldType::Dropdown(idx, mut values) => Some(values.swap_remove(idx)),
			_ => None
		}
	}

	pub fn try_get_list(self) -> Option<Vec<String>> {
		match self {
			FieldType::List(list) => Some(list),
			_ => None
		}
	}

	pub fn try_get_kvlist(self) -> Option<std::collections::HashMap<String, String>> {
		match self {
			FieldType::KeyValueList(kvlist) => Some(kvlist),
			_ => None
		}
	}

	pub fn get_input(self) -> InputType {
		self.try_get_input().unwrap()
	}

	pub fn get_checkbox(self) -> bool {
		self.try_get_checkbox().unwrap()
	}

	pub fn get_dropdown_index(self) -> usize {
		self.try_get_dropdown_index().unwrap()
	}

	pub fn get_dropdown_ref(&self) -> &String {
		self.try_get_dropdown_ref().unwrap()
	}

	pub fn get_dropdown_value(self) -> String {
		self.try_get_dropdown_value().unwrap()
	}

	pub fn get_list(self) -> Vec<String> {
		self.try_get_list().unwrap()
	}

	pub fn get_kvlist(self) -> std::collections::HashMap<String, String> {
		self.try_get_kvlist().unwrap()
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputField {
	pub uuid: String,
	pub title: String,
	pub subtitle: Option<String>,
	pub r#type: FieldType
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreferenceGroup {
	pub title: String,
	pub description: Option<String>,
	pub fields: Vec<InputField>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreferencePane {
	pub title: String,
	pub icon: Option<String>,
	pub groups: Vec<PreferenceGroup>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreferenceDiag {
	pub uuid: String,
	pub panes: Vec<PreferencePane>
}
