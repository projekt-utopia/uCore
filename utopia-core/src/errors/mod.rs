use std::{error::Error, fmt::{self, Display, Formatter, Debug}};

#[derive(Debug)]
pub struct ModuleABIError {
    name: &'static str,
    version: &'static str,
    expected: &'static str
}
impl ModuleABIError {
    pub fn new(name: &'static str, version: &'static str, expected: &'static str) -> Self {
        ModuleABIError { name, version, expected }
    }
}
impl Error for ModuleABIError {}
impl Display for ModuleABIError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "The module {} implements ABI version {}. µCore is built against {}", self.name, self.version, self.expected)
    }
}

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
pub struct FrontendNotAvailableError {
    name: String
}
impl FrontendNotAvailableError {
    pub fn new(name: String) -> Self {
        FrontendNotAvailableError { name }
    }
}
impl Error for FrontendNotAvailableError {}
impl Display for FrontendNotAvailableError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Attemt to find Frontend {} was unsuccessful", self.name)
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
