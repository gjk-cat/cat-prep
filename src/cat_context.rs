use walkdir::WalkDir;
use mdbook::book::{Book, BookItem};

use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::error::CatError;
use crate::models::*;

/// funkce, která vykrojí header daného stringu
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

/// přečte karty učitelů
/// bohužel, čtení ostatních karet je již
/// více provázané, což znesnadňuje
/// jejich oddělení do vlastních funkcí
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

/// Cat kontext
///
/// Typ obsahující kompletní kotext `cat-prepu`.
/// Jednotlivé struktury obsahují spoustu redundance,
/// za účelem jednoduchého vyhledávání potřebných informací.
/// Nedoporučuje se tedy tento instance tohoto typu
/// po vytvoření mutovat, protože redudantní kopie jednotlivých
/// objektů si mohou přestat vzájemně odpovídat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatContext {
	/// obsahuje karty jednotlivých kantorů
	pub teacher_cards: Vec<TeacherCard>,
	/// profily vyučujících
	pub teachers:      Vec<Teacher>,
	/// karty předmětů
	pub subject_cards: Vec<SubjectCard>,
	/// předměty
	pub subjects:      Vec<Subject>,
	/// karty článků
	pub article_cards: Vec<ArticleCard>,
	/// články
	pub articles:      Vec<Article>,
	/// obsahuje hashmapu tagů
	///
	/// tagy jsou sesbírány z jednotlivých
	/// článků, jako hodnoty pak figurují články,
	/// které mají daný tag přidělený
	///
	/// při renderování je tato hashmapa zkonvertována
	/// na typ `TagContext`, který je prakticky newtype
	/// pattern na typu `Vec<(String, Vec<ArticleCard>)>`.
	///
	/// `TagContext` je následně využit jako šablonový
	/// kontext pro generování stránky s tagy.
	pub tags:          HashMap<String, Vec<ArticleCard>>,
}

impl CatContext {
	/// vygeneruje prázdný [`CatContext`].
	/// Užitečné pro generování umělého kontextu
	pub fn new() -> CatContext {
		CatContext {
			teacher_cards: vec![],
			subject_cards: vec![],
			article_cards: vec![],
			teachers:      vec![],
			subjects:      vec![],
			articles:      vec![],
			tags:          HashMap::new(),
		}
	}

	/// vygeneruje kontext dle knihy.
	/// Tato funkce knihuju mutuje, protože odděluje headery
	/// od obsahu jednotlivých souborů
	pub fn with_book(src: &mut Book) -> Result<CatContext, CatError> {
		let (status, is_inside, error) = sh!("git rev-parse --is-inside-work-tree");

		if status != 0 || !is_inside.trim().parse().unwrap_or(false) {
			return Err(CatError::NotARepo { error });
		}

		let mut teacher_cards = read_teacher_cards()?;
		teacher_cards.sort_by(|a, b| a.jmeno.cmp(&b.jmeno));
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
	    				.collect::<Vec<_>>(),
					articles:  vec![],
    			})
			})
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

			return Err(errors[0].clone());
		}

		let subject_items = src
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
		subject_cards.sort_by(|a, b| a.nazev.cmp(&b.nazev));

		let mut subjects = subject_cards
			.iter()
			.map(|x| Subject {
				path:            x._resolved_path.clone().unwrap(),
				path_root:       x
					._resolved_path
					.clone()
					.unwrap()
					.parent()
					.unwrap()
					.to_path_buf(),
				card:            x.clone(),
				articles:        vec![],
				resolved_author: None,
			})
			.collect::<Vec<_>>();
		subjects.sort_by(|a, b| a.card.nazev.cmp(&b.card.nazev));

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
		article_cards.sort_by(|a, b| a.nazev.cmp(&b.nazev));

		if !errors.is_empty() {
			errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

			return Err(errors[0].clone());
		}

		let mut articles = article_cards
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
					modified_resolved: None,
					resolved_author: None,
					subject_card: None,
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

		subjects.iter_mut().for_each(|x| {
			if let Some(t) = teachers.iter().find(|t| {
				x.card.zodpovedna_osoba == t.card.username
					|| x.card.zodpovedna_osoba == t.card.jmeno
					|| x.card.zodpovedna_osoba == t.card.email
			}) {
				x.resolved_author = Some(t.card.clone());
			}
		});

		articles.iter().for_each(|x| {
			let _ = teachers
				.iter_mut()
				.find(|y| y.files_created.contains(&x.path))
				.map(|y| y.articles.push(x.clone()));
		});

		articles.iter_mut().for_each(|x| {
			if let Some(t) = teachers.iter().find(|t| {
				x.modified_by == t.card.username
					|| x.modified_by == t.card.jmeno
					|| x.modified_by == t.card.email
			}) {
				x.modified_resolved = Some(t.card.clone());
			}
			if let Some(t) = teachers.iter().find(|t| {
				x.author == t.card.username
					|| x.author == t.card.jmeno
					|| x.author == t.card.email
			}) {
				x.resolved_author = Some(t.card.clone());
			}

			x.subject_card = subjects
				.iter()
				.find(|y| x.path.starts_with(&y.path_root))
				.map(|y| y.card.clone());
			//↑ should never be None because to be considered
			// an article, there needs to be a subject prefix
		});

		Ok(CatContext {
			teacher_cards,
			teachers,
			subject_cards,
			subjects,
			article_cards: article_cards.clone(),
			articles,
			tags: article_cards.iter().fold(HashMap::new(), {
				|mut acc, x| {
					x.tagy.iter().for_each(|y| {
						acc.entry(y.into()).or_insert(vec![]).push(x.clone())
					});

					acc
				}
			}),
		})
	}
}
