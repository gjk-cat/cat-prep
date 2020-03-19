extern crate clap;
extern crate toml;
extern crate serde;
extern crate mdbook;
extern crate walkdir;
extern crate failure;

use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

pub mod error;
pub mod cat_context;

use cat_context::CatContext;

/// A no-op preprocessor.
pub struct Cat;

impl Cat {
	pub fn new() -> Cat {
		Cat
	}
}

impl Preprocessor for Cat {
	fn name(&self) -> &str {
		"cat-preprocessor"
	}

	fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
		if let Some(cat_cfg) = ctx.config.get_preprocessor(self.name()) {
			if cat_cfg.contains_key("blow-up") {
				return Err("Boom!!1!".into());
			}
		}

		let context = match CatContext::with_book(&mut book) {
			Ok(c) => c,
			Err(e) => {
				eprintln!("[cat prep] failed to create cat context: {}", e);
				return Err(e.to_string().into());
			}
		};

		Ok(book)
	}

	fn supports_renderer(&self, renderer: &str) -> bool {
		renderer != "not-supported"
	}
}
