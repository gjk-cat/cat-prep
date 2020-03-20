use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherCard {
	pub jmeno:    String,
	pub email:    String,
	pub username: String,
	pub bio:      String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleCard {
	pub nazev:          String,
	pub tagy:           Vec<String>,
	pub datum:          Option<String>,
	pub _resolved_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
	pub card:          ArticleCard,
	pub last_modified: String,
	pub modified_by:   String,
	pub author:        String,
	pub path:          PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectCard {
	pub nazev:            String,
	pub zodpovedna_osoba: String,
	pub bio:              String,
	pub _resolved_path:   Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
	pub card:      SubjectCard,
	pub path:      PathBuf,
	pub path_root: PathBuf,
	pub articles:  Vec<Article>,
	pub resolved_author: Option<TeacherCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teacher {
	pub card:          TeacherCard,
	pub subjects:      Vec<Subject>,
	pub articles:      Vec<Article>,
	pub files_created: Vec<PathBuf>,
}
