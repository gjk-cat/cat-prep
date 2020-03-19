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
		}
	}

	pub fn with_book(src: &mut Book) -> Result<CatContext, CatError> {
		let context = CatContext::new();
		let teacher_cards = read_teacher_cards()?;

		eprintln!("{:?}", teacher_cards);

		let mut subject_items = src
			.iter()
			.filter_map(|x| if let BookItem::Chapter(c) = x { Some(c) } else { None })
			.filter(|x| x.path.to_str().unwrap().ends_with("subject.md"))
			.cloned()
			.collect::<Vec<_>>();

		let mut subject_cards = vec![];
		let mut subject_roots: Vec<PathBuf> = vec![];

		let mut errors: Vec<_> = vec![];
		src.for_each_mut(|x| if let BookItem::Chapter(c) = x {
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
                		errors.push(CatError::InvalidHeaderFormat{ err: e });
                		return;
            		}
        		};

        		card._resolved_path = Some(c.path.clone());

        		subject_cards.push(card);
        		subject_roots.push(c.path.parent().unwrap().to_path_buf())
    		}
		});

		if !errors.is_empty() {
    		errors.iter()
    			.for_each(|x| eprintln!("[cat-prep] {}", x));

    		return Err(errors[0].clone());
		}


		eprintln!("{:#?}", src);
		eprintln!("{:#?}", subject_cards);
		eprintln!("{:#?}", subject_roots);


		Ok(context)
	}
}
