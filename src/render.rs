//! modul obsahující renderovací funkce tohoto preprocessoru
//!
//! V první fázi dojde ke zpracování cat kontextu na seznam
//! typu [`Vec<RenderSite>`], který obsahuje renderovací příkazy
//! a jejich cíle.
//!
//! Následně je seznam renderů aplikován na knihu.
//! Vzhledem k tomu, že samotné vytváření renderů
//! přidává do knihy několik článků, tak obě operace
//! vyžadují mutabilní přístup ke knize.
//!
//! Tvorba renderů je zprostředkována pomocí traity
//! [`Render`]. Všechny výchozí rendery využívají `tinytemplate`
//! šablony. Pro `tinytemplate` je nejlepší, když
//! je šablona statický strig, proto jsou zde všechny
//! šablony prozatím 'nahardcodované' jako immutabilní
//! globální stringy.
//!
//! V budoucnu by bylo možné využít makra `include_str!()`
//! k extrakci těchto šablon do vnějších souborů.

use std::fmt;
use std::path::PathBuf;
use std::convert::From;
use std::collections::HashMap;

use mdbook::{
	BookItem,
	book::{Book, Chapter},
};
use tinytemplate::TinyTemplate;
use serde::{Serialize, Deserialize};

use crate::cat_context::CatContext;
use crate::error::CatError;
use crate::models::*;

/// typ daného renderu (a jeho obsah).
/// Určuje chování, jakým bude zacházeno
/// z obsahem článku
#[derive(Debug, Clone)]
pub enum RenderType {
	/// Přidá vyrenderovaný obsah na začátek
	Prepend(String),
	/// Přidá vyrenderovaný obsah na konec
	Append(String),
	/// Přidá něco na začátek a něco na konec
	Both(String, String),
	/// Přepíše stárnku
	EntirePage(String),
}

use RenderType::*;

impl fmt::Display for RenderType {
	/// je zapotřebí, aby nám vypisování RenderTypu v
	/// chybách nekazilo výstup. Tato implementace
	/// tedy usekne asociovaná data každé varianty
	/// a vypíše pouze její název.
	///
	/// Pro vypsání nejen názvu, ale i obsahu použijte
	/// debug formátování ("{:?}" nebo "{:#?}").
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Prepend(_) => write!(f, "Prepend"),
			Append(_) => write!(f, "Append"),
			Both(_, _) => write!(f, "Both"),
			EntirePage(_) => write!(f, "EntirePage"),
		}
	}
}

/// Uchovává informace o daném renderu
#[derive(Debug, Clone)]
pub struct RenderSite {
	/// soubor, ve kterým má být render proveden
	pub site:   PathBuf,
	/// typ a obsah renderu
	pub render: RenderType,
}

impl RenderSite {
	/// vytvoří nový render
	pub fn new(site: PathBuf, render: RenderType) -> Self {
		RenderSite { site, render }
	}
}

/// Trait umožňující renderování struktury
/// jako Markdown/HT?ML
pub trait Render {
	/// metoda pro renderování daného typu.
	///
	/// V případě, že renderování selže by měla
	/// implementace vracet správný chybový typ
	fn render(&self, context: &CatContext) -> Result<RenderSite, CatError>;
}

/// šablona karty učitele
pub static TEACHER_TEMPLATE: &'static str = r#"
<h2 id="{card.username}">{card.jmeno}</h2>

- email: <a href="mailto:{card.email}">{card.email}</a>
- username: {card.username}

### Bio
{card.bio}

### Předměty
{{ for p in subjects }} - [{p.card.nazev}](/{p.path})
{{ endfor }}

### Materiály
{{ for a in articles }} - [{a.card.nazev}](/{a.path})
{{ endfor }}
<hr>
"#;

impl Render for Teacher {
	fn render(&self, _: &CatContext) -> Result<RenderSite, CatError> {
		let render_site = PathBuf::from("teachers.md");
		let mut tt = TinyTemplate::new();

		tt.add_template("teacher", TEACHER_TEMPLATE)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;
		let res = tt
			.render("teacher", &self)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		dbg!("{}", &res);

		Ok(RenderSite::new(render_site, Append(res)))
	}
}

/// šablona pro seznam učitelů
pub static TEACHER_LIST_TEMPLATE: &'static str = r#"
{{ for t in list }} [{t.jmeno}](#{t.username}) {{ endfor }}
"#;

/// tato struktura existuje jako způsob obcházení limitací `tinytemplate`
#[derive(Debug, Serialize, Clone)]
pub struct TeacherList {
    /// karty všech učitelů
    pub list: Vec<TeacherCard>,
}

impl Render for TeacherList {
	fn render(&self, _: &CatContext) -> Result<RenderSite, CatError> {
		let render_site = PathBuf::from("teachers.md");
		let mut tt = TinyTemplate::new();

		tt.add_template("teacher", TEACHER_LIST_TEMPLATE)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;
		let res = tt
			.render("teacher", &self)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		dbg!("{}", &res);

		Ok(RenderSite::new(render_site, Append(res)))
	}
}

/// šablona karty předmětu (část před obsahem)
pub static SUBJECT_PRE_TEMPLATE: &'static str = r#"
| Název | { card.nazev } |
| ----- | -------------- |
{{ if resolved_author }}| Zodpovědná osoba |  [{resolved_author.jmeno}](/teachers.md#{resolved_author.username}) | {{ else }}| Zodpovědná osoba | {card.zodpovedna_osoba} | {{ endif }}
| Popis | { card.bio }   |
"#;

/// šablona seznamu materiálů v daném předmětu (část za obsahem)
pub static SUBJECT_POST_TEMPLATE: &'static str = r#"
### Seznam materiálů
{{ for a in articles }} - [{a.card.nazev}](/{a.path})
{{ endfor }}
"#;

impl Render for Subject {
	fn render(&self, _: &CatContext) -> Result<RenderSite, CatError> {
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

		dbg!("{}\n{}", &pre, &post);

		Ok(RenderSite::new(render_site, Both(pre, post)))
	}
}

/// šablona karty článku (část před obsahem)
pub static ARTICLE_PRE_TEMPLATE: &'static str = r#"
| Název | {card.nazev} |
| ----- | ------------ |
{{ if resolved_author }}| Autor |  [{resolved_author.jmeno}](/teachers.md#{resolved_author.username}) | {{ else }}| Autor | {author} | {{ endif }}
{{ if modified_resolved }}| Naposledy upravil |  [{modified_resolved.jmeno}](/teachers.md#{modified_resolved.username}) | {{ else }}| Naposledy upravil | {modified_by} | {{ endif }}
| Poslední změna | {last_modified} |
| Předmět | [{subject_card.nazev}](/{subject_card._resolved_path}) |
{{ if card.datum }}| Datum | {card.datum} |{{endif}}
"#;

/// čablona seznamu tagů u článku (část za obsahem)
///
/// tato šablona také embedduje Disqus za účelem zprostředkování
/// komentářů.
pub static ARTICLE_POST_TEMPLATE: &'static str = r#"
#### Tagy
{{ for tag in card.tagy}} [{tag}](/tags.md#{tag}) {{ endfor }}

<div id="disqus_thread"></div>
<script>var disqus_config = function () \{ this.page.url = window.location.href; this.page.identifier = window.location.href; }; (function() \{ var d = document, s = d.createElement('script'); s.src = 'https://gjk-cat.disqus.com/embed.js'; s.setAttribute('data-timestamp', +new Date()); (d.head || d.body).appendChild(s); })(); </script>
<noscript>Please enable JavaScript to view the <a href="https://disqus.com/?ref_noscript">comments powered by Disqus.</a></noscript>
"#;

impl Render for Article {
	fn render(&self, _: &CatContext) -> Result<RenderSite, CatError> {
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

		dbg!("{}\n{}", &pre, &post);

		Ok(RenderSite::new(render_site, Both(pre, post)))
	}
}

/// struktura obsahující pár tag - články
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
	/// samotný tag jako string
	pub name:     String,
	/// seznam článků s tímto tagem
	pub articles: Vec<ArticleCard>,
}

/// tagový kontext pro `tinytemplate` šablonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagContext {
	/// vektor obsahující prvky typu [`Tag`]
	pub tags: Vec<Tag>,
}

/// konverze z tagové hasmapy na šablonový kontext
impl From<&HashMap<String, Vec<ArticleCard>>> for TagContext {
	fn from(src: &HashMap<String, Vec<ArticleCard>>) -> Self {
		let mut tags =
			src.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>();
		tags.sort_by(|a, b| a.0.cmp(&b.0));

		Self {
			tags: tags
				.into_iter()
				.map(|(k, v)| Tag { name: k, articles: v })
				.collect::<Vec<_>>(),
		}
	}
}

/// šablona pro stránku se seznamem tagů a asociovaných článků
pub static TAGS_TEMPLATE: &'static str = r#"
# Tagy
{{ for tag in tags }} [{tag.name}](#{tag.name}) {{ endfor }}

{{ for tag in tags }}
<h3 id="{tag.name}">{tag.name}</h3>
{{ for a in tag.articles }}
 - [{a.nazev}](/{a._resolved_path}){{ endfor }}
{{ endfor }}
"#;

impl Render for TagContext {
	fn render(&self, _: &CatContext) -> Result<RenderSite, CatError> {
		let render_site = PathBuf::from("tags.md");
		let mut tt = TinyTemplate::new();

		tt.add_template("tags", TAGS_TEMPLATE)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;
		let res = tt
			.render("tags", &self)
			.map_err(|e| CatError::TinyError { error: e.to_string() })?;

		dbg!("{}", &res);

		Ok(RenderSite::new(render_site, EntirePage(res)))
	}
}

/// vytvoří rendery z objektů
///
/// zároveň založí stránky `teachers.md`
/// a `tags.md`.
pub fn create_renders(
	context: &CatContext,
	book: &mut Book,
) -> Result<Vec<RenderSite>, CatError> {
	let mut pending_renders: Vec<RenderSite> = vec![];
	let mut errors: Vec<CatError> = vec![];

	match (TeacherList { list: context.teacher_cards.clone() }).render(context) {
    	Ok(r) => pending_renders.push(r),
    	Err(e) => return Err(e),
	}

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

	match TagContext::from(&context.tags).render(context) {
		Ok(r) => pending_renders.push(r),
		Err(e) => return Err(e),
	}


	if !context.teacher_cards.is_empty() {
    	book.push_item(BookItem::Chapter(Chapter::new(
    		"Vyučující",
    		"# Vyučující\n".to_string(),
    		"teachers.md".to_string(),
    		vec![],
    	)));
	}

	if !context.tags.is_empty() {
    	book.push_item(BookItem::Chapter(Chapter::new(
    		"Tagy",
    		"".to_string(),
    		"tags.md".to_string(),
    		vec![],
    	)));
	}

	dbg!("[cat prep] prerender: {:#?}", &book);

	Ok(pending_renders)
}

/// spustí dané Rendery na knize
///
/// jelikož nevyužitý render pravdepodobně znamená chybnou syntaxi,
/// vrací chybu v případě nevyužitých renderů.
pub fn execute_renders(
	mut pending_renders: Vec<RenderSite>,
	book: &mut Book,
) -> Result<(), CatError> {
	book.for_each_mut(|c| {
		if let BookItem::Chapter(c) = c {
			let path = c.path.clone();

			pending_renders.iter().filter(|x| x.site == path).for_each(|x| {
				match &x.render {
					Prepend(s) => c.content = format!("{}\n{}", c.content, s),
					Both(pre, post) =>
						c.content = format!("{}\n{}\n{}", pre, c.content, post),
					Append(s) => c.content = format!("{}\n{}", c.content, s),
					EntirePage(s) => c.content = s.clone(),
				}
			});

			pending_renders.retain(|x| x.site != c.path);
		}
	});

	if !pending_renders.is_empty() {
		for RenderSite { site, render } in &pending_renders {
			println!("[cat-prep] error: oprhan render: {} at {}", render, site.display());
		}
		return Err(CatError::OrphanRender {
			site:   pending_renders[0].site.display().to_string(),
			render: pending_renders[0].render.clone(),
		});
	}

	Ok(())
}
