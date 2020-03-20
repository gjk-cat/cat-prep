//! ## cat-prep
//!
//! vstupní bod knihovny,
//! obsahuje většinu funkcionality
//! tohoto preprocesoru.
//!
//! Za zmínku stojí zejména moduly
//! [`cat_context`] a [`render`].
#![deny(missing_docs)]

extern crate clap;
extern crate toml;
extern crate serde;
extern crate mdbook;
extern crate walkdir;
extern crate failure;
extern crate tinytemplate;

#[macro_use]
extern crate shells;

use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

pub mod error;
pub mod models;
pub mod render;
pub mod cat_context;

use cat_context::CatContext;

/// Samotný preprocesor.
/// .
/// Tento preprocesor nepotřebuje žádný state,
/// mimo [`CatContext`], který je ale hned využit.
/// Proto si postačí s jednotkovou strukturou.
pub struct Cat;

impl Cat {
	/// Vytvoří novou "instanci" preprocesoru
	///
	/// Přestože je tato funkce relativně zbytečná,
	/// je považována za standardní API preprocesorů
	pub fn new() -> Cat {
		Cat
	}
}

impl Preprocessor for Cat {
	/// Název preprocesoru,
	/// pro použití mdbookem
	fn name(&self) -> &str {
		"cat-preprocessor"
	}

	/// spustí preprocesor i s jeho kontextem.
	///
	/// `cat-prep` mdbookový kontext nepotřebuje.
	/// Tato funkce nejdříve vygeneruje [`CatContext`] potřebný
	/// pro renderování knihy, a posléze ji vyrenderuje.
	///
	/// Je nutno dodat, že už i generování kontextu knihu mutuje
	/// -> dochází k oddělování headerů od obsahu stránky
	fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
		let context = match CatContext::with_book(&mut book) {
			Ok(c) => c,
			Err(e) => {
				eprintln!("[cat prep] failed to create cat context: {}", e);
				return Err(e.to_string().into());
			}
		};

		let renders = match render::create_renders(&context, &mut book) {
			Ok(rs) => rs,
			Err(e) => {
				eprintln!("[cat prep] failed to prepare renders of cat content: {}", e);
				return Err(e.to_string().into());
			}
		};

		if let Err(e) = render::execute_renders(renders, &mut book) {
			eprintln!("[cat prep] failed to prepare renders of cat content: {}", e);
			return Err(e.to_string().into());
		}

		dbg!("{:#?}", context);

		Ok(book)
	}

	/// určuje kompatibilitu s různymi renderery.
	/// Kromě HTML (a tedy i PDF) je však většina rendererů
	/// teprve v plenkách a není používaná,
	/// proto zde není omezena kompatibilita
	/// - každý renderer, který podporuje plnou škálu markdownu
	/// a základní HTML je schopen použít `cat-prep`
	fn supports_renderer(&self, renderer: &str) -> bool {
		renderer != "not-supported"
	}
}
