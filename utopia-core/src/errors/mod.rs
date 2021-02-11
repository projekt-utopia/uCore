use std::{error::Error, fmt::{self, Display, Formatter, Debug}};

#[derive(Debug)]
pub struct ModuleNotAvailableError {
    name: &'static str
}
impl ModuleNotAvailableError {
    pub fn new(name: &'static str) -> Self {
        ModuleNotAvailableError { name }
    }
}
impl Error for ModuleNotAvailableError {}
impl Display for ModuleNotAvailableError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Attemt to find unloaded module: {}", self.name)
    }
}

#[derive(Debug)]
pub struct LibraryItemNotAvailableError {
    name: String
}
impl LibraryItemNotAvailableError {
    pub fn new(name: String) -> Self {
        LibraryItemNotAvailableError { name }
    }
}
impl Error for LibraryItemNotAvailableError {}
impl Display for LibraryItemNotAvailableError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Attemt to find {} in library was unsuccessful", self.name)
    }
}

#[derive(Debug)]
pub struct UnkownUtopiaError<T: Debug> {
    msg: &'static str,
    custom: T
}
impl<T: Debug> UnkownUtopiaError<T> {
    #[allow(dead_code)] // It's better if this will not be implemented :)
    pub fn new(msg: &'static str, custom: T) -> Self {
        UnkownUtopiaError { msg, custom }
    }
}
impl<T: Debug> Error for UnkownUtopiaError<T> {}
impl<T: Debug> Display for UnkownUtopiaError<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "An error occured: {} - Debug info: {:?}", self.msg, self.custom)
    }
}
