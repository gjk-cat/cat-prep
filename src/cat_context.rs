use failure::bail;
use walkdir::WalkDir;
use mdbook::book::{Book, BookItem};
use serde::{Serialize, Deserialize};

use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::read_to_string;

use crate::error::CatError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherCard {
	pub jmeno:    String,
	pub email:    String,
	pub username: String,
	pub bio:      String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleCard {
	pub nazev: String,
	pub tagy:  Vec<String>,
	pub datum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
	pub card:          ArticleCard,
	pub last_modified: String,
	pub modified_by:   String,
	pub author:        String,
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
	pub card:     SubjectCard,
	pub path:     String,
	pub articles: Vec<Article>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teacher {
	pub card:     TeacherCard,
	pub subjects: Subject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatContext {
	pub teacher_cards: Vec<TeacherCard>,
	pub teachers:      Vec<Teacher>,
	pub subject_cards: Vec<SubjectCard>,
	pub subjects:      Vec<Subject>,
	pub article_cards: Vec<ArticleCard>,
	pub articles:      Vec<Article>,
}

pub fn read_teacher_cards() -> Result<Vec<TeacherCard>, CatError> {
	match Path::new("teachers".into()) {
		x if !x.exists() => return Err(CatError::NoTeacherFolder),
		x if x.is_file() => return Err(CatError::TeachersArentFolder),
		_ => (),
	};

	let teachers = WalkDir::new("teachers")
		.into_iter()
		.map(|x| x.expect("failed to walk director - fatal error"))
		.filter_map(|x| {
			if x.file_name().to_string_lossy().ends_with(".toml")
				&& x.file_type().is_file()
			{
				Some(x)
			} else {
				None
			}
		})
		.map(|x| {
			(
				x.file_name().to_string_lossy().to_string(),
				read_to_string(x.path()).expect("failed to open and file"),
			)
		})
		.map(|x| (x.0, toml::de::from_str::<TeacherCard>(&x.1)))
		.collect::<Vec<_>>();

	if let Some((name, res)) = teachers.iter().find(|(_, x)| x.is_err()) {
		return Err(CatError::InvalidTeacherCard {
			name: name.clone(),
			err:  res.clone().unwrap_err(),
		});
	}

	Ok(teachers.into_iter().map(|(_, x)| x.unwrap()).collect::<Vec<TeacherCard>>())
}

impl CatContext {
	pub fn new() -> CatContext {
		CatContext {
			teacher_cards: vec![],
			subject_cards: vec![],
			article_cards: vec![],
			teachers:      vec![],
			subjects:      vec![],
			articles:      vec![],
		}
	}

	pub fn with_book(src: &Book) -> Result<CatContext, CatError> {
		let context = CatContext::new();
		let teacher_cards = read_teacher_cards()?;

		eprintln!("{:?}", teacher_cards);

		let subject_items = src
			.iter()
			.filter_map(|x| if let BookItem::Chapter(c) = x { Some(c) } else { None })
			.filter(|x| x.path.to_str().ends_with("subject.md"))
			.collect::<Vec<_>>();

		eprintln!("{:?}", subject_items);

		//let subject_roots: Vec<

		Ok(context)
	}
}
