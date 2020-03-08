extern crate clap;
extern crate mdbook;

use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

///	A no-op	preprocessor.
pub	struct Nop;

impl Nop {
	pub	fn new() ->	Nop	{
		Nop
	}
}

impl Preprocessor for Nop {
	fn name(&self) -> &str {
		"nop-preprocessor"
	}

	fn run(&self, ctx: &PreprocessorContext, book: Book) ->	Result<Book, Error>	{
		// In testing we want to tell the preprocessor to blow up by setting a
		// particular config value
		if let Some(nop_cfg) = ctx.config.get_preprocessor(self.name())	{
			if nop_cfg.contains_key("blow-up") {
				return Err("Boom!!1!".into());
			}
		}

		// we *are*	a no-op	preprocessor after all
		Ok(book)
	}

	fn supports_renderer(&self,	renderer: &str)	-> bool	{
		renderer !=	"not-supported"
	}
}
