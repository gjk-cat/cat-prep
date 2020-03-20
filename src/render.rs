use std::path::PathBuf;
use mdbook::book::Book;
use tinytemplate::TinyTemplate;

use crate::cat_context::CatContext;
use crate::error::CatError;
use crate::models::*;

#[derive(Debug, Clone)]
pub enum RenderType {
	Top(String),
	Both(String, String),
	Append(String),
	EntirePage(String),
}

use RenderType::*;

#[derive(Debug, Clone)]
pub struct RenderSite {
	site:   PathBuf,
	render: RenderType,
}

impl RenderSite {
	pub fn new(site: PathBuf, render: RenderType) -> Self {
		RenderSite { site, render }
	}
}

pub trait Render {
	fn render(&self, context: &CatContext) -> Result<RenderSite, CatError>;
}

static TEACHER_TEMPLATE: &'static str = r#"
<h2 id="{card.username}">{card.jmeno}</h2>

- email: {card.email}
- username: {card.username}

### Bio
{card.bio}

### Předměty
{{ for p in subjects }} - [{p.card.nazev}]({p.path})
{{ endfor }}

### Materiály
{{ for a in articles }} - [{a.card.nazev}]({a.path})
{{ endfor }}
"#;

impl Render for Teacher {
	fn render(&self, context: &CatContext) -> Result<RenderSite, CatError> {
		let render_site = PathBuf::from("teachers.md");
		let mut tt = TinyTemplate::new();

		tt.add_template("teacher", TEACHER_TEMPLATE)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;
		let res = tt
			.render("teacher", &self)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		eprintln!("{}", res);

		Ok(RenderSite::new(render_site, Append(res)))
	}
}

static SUBJECT_PRE_TEMPLATE: &'static str = r#"
| Název | { card.nazev } |
| ----- | -------------- |
{{ if resolved_author }}| Zodpovědná osoba |  [{resolved_author.jmeno}](teachers.md#{resolved_author.username}) | {{ else }}| Zodpovědná osoba | {card.zodpovedna_osoba} | {{ endif }}
| Popis | { card.bio }   |
"#;

static SUBJECT_POST_TEMPLATE: &'static str = r#"
### Seznam materiálů
{{ for a in articles }} - [{a.card.nazev}]({a.path})
{{ endfor }}
"#;

impl Render for Subject {
	fn render(&self, context: &CatContext) -> Result<RenderSite, CatError> {
		let render_site = self.path.clone();
		let mut tt = TinyTemplate::new();

		tt.add_template("subject_pre", SUBJECT_PRE_TEMPLATE)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;
		tt.add_template("subject_post", SUBJECT_POST_TEMPLATE)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		let pre = tt
			.render("subject_pre", &self)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		let post = tt
			.render("subject_post", &self)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		eprintln!("{}\n{}", pre, post);

		Ok(RenderSite::new(render_site, Both(pre, post)))
	}
}

static ARTICLE_PRE_TEMPLATE: &'static str = r#"
| Název | {card.nazev} |
| ----- | ------------ |
{{ if resolved_author }}| Autor |  [{resolved_author.jmeno}](teachers.md#{resolved_author.username}) | {{ else }}| Autor | {author} | {{ endif }}
{{ if modified_resolved }}| Naposledy upravil |  [{modified_resolved.jmeno}](teachers.md#{modified_resolved.username}) | {{ else }}| Naposledy upravil | {card.zodpovedna_osoba} | {{ endif }}
| Poslední změna | {last_modified} |
| Předmět | [{subject_card.nazev}]({subject_card._resolved_path}) |
{{ if card.datum }}| Datum | {card.datum} |{{endif}}
"#;

static ARTICLE_POST_TEMPLATE: &'static str = r#"
#### Tagy
{{ for tag in card.tagy}} [{tag}](tags.md#{tag}) {{ endfor }}
"#;

impl Render for Article {
	fn render(&self, context: &CatContext) -> Result<RenderSite, CatError> {
		let render_site = self.path.clone();
		let mut tt = TinyTemplate::new();

		tt.add_template("article_pre", ARTICLE_PRE_TEMPLATE)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;
		tt.add_template("article_post", ARTICLE_POST_TEMPLATE)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		let pre = tt
			.render("article_pre", &self)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		let post = tt
			.render("article_post", &self)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		eprintln!("{}\n{}", pre, post);

		Ok(RenderSite::new(render_site, Both(pre, post)))
	}
}

pub fn render(context: &CatContext, book: &mut Book) -> Result<(), CatError> {
	let mut pending_renders: Vec<RenderSite> = vec![];
	let mut errors: Vec<CatError> = vec![];

	context.teachers.iter().for_each(|t| match t.render(context) {
		Ok(r) => pending_renders.push(r),
		Err(e) => errors.push(e),
	});

	if !errors.is_empty() {
		errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

		return Err(errors[0].clone());
	}

	context.subjects.iter().for_each(|t| match t.render(context) {
		Ok(r) => pending_renders.push(r),
		Err(e) => errors.push(e),
	});

	if !errors.is_empty() {
		errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

		return Err(errors[0].clone());
	}

	context.articles.iter().for_each(|t| match t.render(context) {
		Ok(r) => pending_renders.push(r),
		Err(e) => errors.push(e),
	});

	if !errors.is_empty() {
		errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

		return Err(errors[0].clone());
	}

	Ok(())
}
