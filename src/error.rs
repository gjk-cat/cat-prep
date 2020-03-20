//! modul obsahující chybový typ
//! `cat-prepu`
//!
//! `cat-prep` používá na zpracování chyb knihovnu
//! `failure`, které umožňuje jednoduché formátování
//! a propagaci chyb.
//!
//! Bohužel,
//! místo správných chybových jsou v některých
//! variantách [`CatError`] typu uloženy stringy,
//! protože trait `Error` nemá podmínku [`Clone`
//! a některé chyby jsou tudíž neklonovatelné.

use failure::Fail;
use toml::de::Error as TomlError;

use crate::render::RenderType;

/// výčet obsahující možné chyby
#[derive(Debug, Fail, Clone)]
pub enum CatError {
	/// Složka `teachers` neexistuje
	#[fail(display = "teachers folder doesn't exist")]
	NoTeacherFolder,
	/// Soubor `teachers` není složka
	#[fail(display = "file 'teachers' is not a folder")]
	TeachersArentFolder,
	/// Karta učitele nemá správný formát
	#[fail(display = "invalid teacher file: {}: {}", name, err)]
	InvalidTeacherCard { name: String, err: TomlError },
	/// Souboru chybí header, nebo je nesprávně ukončený
	#[fail(display = "the header is either missing or invalid")]
	InvalidOrMissingHeader,
	/// Header souboru není možné naparsovat jako TOML,
	/// nebo neobsahuje všechny povinné hodnoty
	#[fail(display = "the header has invalid format: {}", err)]
	InvalidHeaderFormat { err: TomlError },
	/// Nepodařilo se spustit příkaz v shell,
	/// nšbo došlo k chybě při běhu.
	///
	/// Může implikovat, že některý z následujících nástrojů není dostupný:
	/// - git
	/// - ls
	/// - xargs
	/// - true
	#[fail(
		display = "failed to run command: {} exited with code {} and output '{}'",
		name, status, error
	)]
	CommandFailed { name: String, status: i32, error: String },
	/// `mdBook` neběží v repozitáři.
	/// Pro uživatelské funkce vyžaduje `cat-prep` gitový repozitář
	#[fail(
		display = "mdbook isn't running in a git repository or the repository is bare: {}",
		error
	)]
	NotARepo { error: String },
	/// v šablonovém enginu `tinytemplate` došlo k chybě
	#[fail(display = "tiny template encountered an error: {}", error)]
	TinyError { error: String }, //  TinyError is not Clone :(
	/// některý render zůstal po zavolání funkce `render::execute_renders` nevyužitý
	#[fail(display = "orphan renders: {} at {}", render, site)]
	OrphanRender { site: String, render: RenderType },
}
