use failure::Fail;
use toml::de::Error as TomlError;

#[derive(Debug, Fail, Clone)]
pub enum CatError {
	#[fail(display = "teachers folder doesn't exist")]
	NoTeacherFolder,
	#[fail(display = "file 'teachers' is not a folder")]
	TeachersArentFolder,
	#[fail(display = "invalid teacher file: {}: {}", name, err)]
	InvalidTeacherCard { name: String, err: TomlError },
	#[fail(display = "the header is either missing or invalid")]
	InvalidOrMissingHeader,
	#[fail(display = "the header has invalid format: {}", err)]
	InvalidHeaderFormat { err: TomlError },
}
