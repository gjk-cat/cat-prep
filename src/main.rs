extern crate clap;

extern crate serde;
extern crate serde_json;

extern crate mdbook;
extern crate mdbook_cat_prep as cat;

use clap::{App, Arg, ArgMatches, SubCommand};

use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};

use std::io;
use std::process;

use cat::Cat;

pub fn make_app() -> App<'static, 'static> {
	App::new("cat-preprocessor")
		.about("A mdbook for preparing study materials")
		.subcommand(
			SubCommand::with_name("supports")
				.arg(Arg::with_name("renderer").required(true))
				.about("Check whether a renderer is supported by this preprocessor"),
		)
}

fn main() {
	let matches = make_app().get_matches();

	let preprocessor = Cat::new();

	if let Some(sub_args) = matches.subcommand_matches("supports") {
		handle_supports(&preprocessor, sub_args);
	} else if let Err(e) = handle_preprocessing(&preprocessor) {
		eprintln!("[cat prep] {}", e);
		process::exit(1);
	}
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
	let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

	if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
		eprintln!(
			"Warning: The {} plugin was built against version {} of mdbook, \
			 but we're being called from version {}",
			pre.name(),
			mdbook::MDBOOK_VERSION,
			ctx.mdbook_version
		);
	}

	let processed_book = pre.run(&ctx, book)?;
	serde_json::to_writer(io::stdout(), &processed_book)?;

	Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
	let renderer = sub_args.value_of("renderer").expect("Required argument");
	let supported = pre.supports_renderer(&renderer);

	if supported {
		process::exit(0);
	} else {
		process::exit(1);
	}
}
