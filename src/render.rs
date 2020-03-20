use std::path::PathBuf;
use mdbook::book::Book;
use tinytemplate::TinyTemplate;

use crate::cat_context::CatContext;
use crate::error::CatError;
use crate::models::*;

#[derive(Debug, Clone)]
pub enum RenderType {
    Top(String),
    Both(String),
    Append(String),
    EntirePage(String),
}

use RenderType::*;

#[derive(Debug, Clone)]
pub struct RenderSite {
	site: PathBuf,
	render: RenderType,
}

impl RenderSite {
    pub fn new(site: PathBuf,  render: RenderType) -> Self {
        RenderSite {
            site,
            render,
        }
    }
}

pub trait Render {
	fn render(&self, context: &CatContext) -> Result<RenderSite, CatError>;
}


static TEACHER_TEMPLATE: &'static str =
r#"
<h2 id="{card.username}">{card.jmeno}</h2>
- email: {card.email}
- username: {card.username}
### Bio
{card.bio}
### Předměty
{{ for p in subjects }}
- [{p.card.nazev}]({p.path})
{{ endfor }}
### Materiály
{{ for a in articles }}
- [{a.card.nazev}]({a.path})
{{ endfor }}
"#;

impl Render for Teacher {
    fn render(&self, context: &CatContext) -> Result<RenderSite, CatError> {
        let render_site = PathBuf::from("teachers.md");
		let mut tt = TinyTemplate::new();

		tt.add_template("teacher", TEACHER_TEMPLATE).map_err(|e| CatError::TinyError{ error: e.to_string() })?;
		let res = tt.render("teacher", &self).map_err(|e| CatError::TinyError{ error: e.to_string() })?;

		eprintln!("{}", res);

		Ok(RenderSite::new(render_site, Append(res)))
    }
}


pub fn render(context: &CatContext, book: &mut Book) -> Result<(), CatError> {
	let mut pending_renders: Vec<RenderSite> = vec![];
	let mut errors: Vec<CatError> = vec![];

	context.teachers
		.iter()
		.for_each(|t| match t.render(context) {
    		Ok(r) => pending_renders.push(r),
    		Err(e) => errors.push(e),
		});

	if !errors.is_empty() {
		errors.iter().for_each(|x| eprintln!("[cat-prep] {}", x));

		return Err(errors[0].clone());
	}

    Ok(())
}
