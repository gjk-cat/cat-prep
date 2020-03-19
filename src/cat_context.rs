use walkdir::WalkDir;
use mdbook::book::{Book, BookItem};
use serde::{Serialize, Deserialize};

use std::env;
use std::path::{Path, PathBuf};
use std::fs::read_to_string;
use std::collections::HashMap;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teacher {
	pub card:          TeacherCard,
	pub subjects:      Vec<Subject>,
	pub files_created: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatContext {
	pub teacher_cards: Vec<TeacherCard>,
	pub teachers:      Vec<Teacher>,
	pub subject_cards: Vec<SubjectCard>,
	pub subjects:      Vec<Subject>,
	pub article_cards: Vec<ArticleCard>,
	pub articles:      Vec<Article>,
	pub tags:          HashMap<String, Vec<ArticleCard>>,
}

pub fn extract_header(src: &str) -> Result<(String, String), CatError> {
	let header = src
		.lines()
		.take_while(|x| *x != "+++")
		.map(|x| x.to_string())
		.collect::<Vec<String>>()
		.join("\n");

	if header == src {
		Err(CatError::InvalidOrMissingHeader)?;
	}

	let body = src
		.lines()
		.skip_while(|x| *x != "+++")
		.skip(1)
		.map(|x| x.to_string())
		.collect::<Vec<String>>()
		.join("\n");

	Ok((header, body))
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
			tags:          vec![],
		}
	}

	pub fn with_book(src: &mut Book) -> Result<CatContext, CatError> {
		let context = CatContext::new();
		let teacher_cards = read_teacher_cards()?;
		let mut errors: Vec<_> = vec![];

		eprintln!("{:?}", teacher_cards);

		let mut teachers = teacher_cards
			.iter()
			.filter_map(|x| {
    			let (status, files_created, error) = sh!(
	    			"git whatchanged --author=\"{}\\|{}\\|{}\" --diff-filter=A --no-commit-id --name-only  | ( xargs ls -d || true ) | xargs -n 1 realpath --relative-to=src", x.jmeno,
	    			x.email, x.username);

    			if status != 0 {
        			errors.push(CatError::CommandFailed { status, error, name: "git".into() });
        			return None;
    			}

    			Some(Teacher {
	    			card: x.clone(),
	    			subjects: vec![],
	    			files_created: files_created
	    				.lines()
	    				.map(PathBuf::from)
	    				.collect::<Vec<_>>()
    			})
			})
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

			return Err(errors[0].clone());
		}

		let mut subject_items = src
			.iter()
			.filter_map(|x| if let BookItem::Chapter(c) = x { Some(c) } else { None })
			.filter(|x| x.path.to_str().unwrap().ends_with("subject.md"))
			.cloned()
			.collect::<Vec<_>>();

		let mut subject_cards = vec![];

		src.for_each_mut(|x| {
			if let BookItem::Chapter(c) = x {
				if subject_items.contains(c) {
					let (header, body) = match extract_header(&c.content) {
						Ok(hb) => hb,
						Err(e) => {
							errors.push(e);
							return;
						}
					};
					c.content = body;

					let mut card: SubjectCard = match toml::de::from_str(&header) {
						Ok(c) => c,
						Err(e) => {
							errors.push(CatError::InvalidHeaderFormat { err: e });
							return;
						}
					};

					card._resolved_path = Some(c.path.clone());

					subject_cards.push(card);
				}
			}
		});

		if !errors.is_empty() {
			errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

			return Err(errors[0].clone());
		}

		let mut subjects = subject_cards
			.iter()
			.map(|x| Subject {
				path:      x._resolved_path.clone().unwrap(),
				path_root: x
					._resolved_path
					.clone()
					.unwrap()
					.parent()
					.unwrap()
					.to_path_buf(),
				card:      x.clone(),
				articles:  vec![],
			})
			.collect::<Vec<_>>();

		let mut article_cards = vec![];

		src.for_each_mut(|x| {
			if let BookItem::Chapter(c) = x {
				if subjects.iter().any(|y| {
					c.path.starts_with(&y.path_root)
						&& c.path.file_name().map(|x| x.to_str().unwrap())
							!= Some("subject.md")
				}) {
					let (header, body) = match extract_header(&c.content) {
						Ok(hb) => hb,
						Err(e) => {
							errors.push(e);
							return;
						}
					};
					c.content = body;

					let mut card: ArticleCard = match toml::de::from_str(&header) {
						Ok(c) => c,
						Err(e) => {
							errors.push(CatError::InvalidHeaderFormat { err: e });
							return;
						}
					};

					card._resolved_path = Some(c.path.clone());

					article_cards.push(card);
				}
			}
		});

		if !errors.is_empty() {
			errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

			return Err(errors[0].clone());
		}

		let articles = article_cards
			.iter()
			.filter_map(|x| {
				let (status, last_modified, error) = sh!(
					"{}",
					&format!(
						"git log -1 --pretty=\"format:%ci\" -- src/'{}'",
						x._resolved_path.clone().unwrap().display()
					)
				);

				if status != 0 {
					errors.push(CatError::CommandFailed {
						status,
						error,
						name: "git".into(),
					});
					return None;
				}

				let (status, modified_by, error) = sh!(
					"{}",
					&format!(
						"git log -s -n1 --pretty='format:%an' -- src/'{}'",
						x._resolved_path.clone().unwrap().display()
					)
				);

				if status != 0 {
					errors.push(CatError::CommandFailed {
						status,
						error,
						name: "git".into(),
					});
					return None;
				}

				let a = Article {
					card: x.clone(),
					author: teachers
						.iter()
						.find(|y| {
							y.files_created.contains(&x._resolved_path.clone().unwrap())
						})
						.map(|y| y.card.jmeno.clone())
						.unwrap_or("Neznámý".into()),
					modified_by,
					last_modified,
					path: x._resolved_path.clone().unwrap(),
				};

				subjects
					.iter_mut()
					.find(|y| x._resolved_path.clone().unwrap().starts_with(&y.path_root))
					.map(|y| y.articles.push(a.clone()));

				Some(a)
			})
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

			return Err(errors[0].clone());
		}

		subjects.iter().for_each(|x| {
			if teachers
				.iter_mut()
				.find(|y| y.files_created.contains(&x.path))
				.map(|y| y.subjects.push(x.clone()))
				.is_some()
			{
				return;
			}

			for a in &x.articles {
				if teachers
					.iter_mut()
					.find(|y| y.files_created.contains(&a.path))
					.map(|y| y.subjects.push(x.clone()))
					.is_some()
				{
					return;
				}
			}
		});

		eprintln!("{:#?}", src);
		eprintln!("{:#?}", subject_cards);
		eprintln!("{:#?}", subjects);
		eprintln!("{:#?}", article_cards);
		eprintln!("{:#?}", articles);
		eprintln!("{:#?}", teachers);

		Ok(context)
	}
}
