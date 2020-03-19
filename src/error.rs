use failure::Fail;
use toml::de::Error as TomlError;
//use std::io::Error as IoError;

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
	#[fail(
		display = "failed to run command: {} exited with code {} and output '{}'",
		name, status, error
	)]
	CommandFailed { name: String, status: i32, error: String },
	#[fail(display = "failed to change directory: {}", error)]
	CantChdir { error: String }, // because IoError is not Clone, fml
}