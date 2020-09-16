use std::collections::BTreeSet;

use n3_parser::error::ParseError;

use crate::ast;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseError(ParseError),
    BuildError(BuildError),
    ExternalError(ExternalError),
}

#[derive(Debug, PartialEq)]
pub enum BuildError {
    TensorNodeError(TensorNodeError),
    GraphError(GraphError),
    GraphNodeError(GraphNodeError),
    GraphCallError(GraphCallError),
}

#[derive(Debug, PartialEq)]
pub enum TensorNodeError {
    NoSuchNode { name: String },
    MismatchedName { expected: String, given: String },
}

#[derive(Debug, PartialEq)]
pub enum GraphError {
    NoSuchVariable {
        name: String,
        candidates: BTreeSet<String>,
    },
    DuplicatedVariable {
        name: String,
    },
    CycledVariables {
        names: BTreeSet<String>,
    },
}

#[derive(Debug, PartialEq)]
pub enum GraphNodeError {
    MismatchedId {
        expected: u64,
        given: u64,
    },
    MismatchedSize {
        expected: &'static [&'static str],
        given: usize,
    },
    MismatchedShapesExistence {
        expected: bool,
        given: bool,
    },
}

#[derive(Debug, PartialEq)]
pub enum GraphCallError {
    MismatchedName {
        expected: Vec<&'static str>,
        given: String,
    },
    MismatchedSize {
        expected: Vec<&'static str>,
        given: usize,
    },
    MismatchedInputs {
        expected: ast::GraphInputsType,
        given: ast::GraphInputsType,
    },
    MismatchedRepeat {
        expected: bool,
        given: bool,
    },
    MismatchedArgs {
        expected: &'static [&'static str],
        given: Vec<String>,
    },
}

#[derive(Debug)]
pub enum ExternalError {
    IOError(std::io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ParseError(a), Self::ParseError(b)) => a.eq(b),
            (Self::BuildError(a), Self::BuildError(b)) => a.eq(b),
            (Self::ExternalError(a), Self::ExternalError(b)) => a.eq(b),
            _ => false,
        }
    }
}

impl PartialEq for ExternalError {
    fn eq(&self, other: &Self) -> bool {
        // note: only test the types
        match (self, other) {
            (Self::IOError(_), Self::IOError(_)) => true,
            _ => false,
        }
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Self::ParseError(error)
    }
}

impl From<BuildError> for Error {
    fn from(error: BuildError) -> Self {
        Self::BuildError(error)
    }
}

macro_rules! impl_into_error(
    ($t:ident) => {
        impl From<$t> for BuildError {
            fn from(error: $t) -> Self {
                Self::$t(error)
            }
        }

        impl From<$t> for Error {
            fn from(error: $t) -> Self {
                Self::BuildError(error.into())
            }
        }

        impl<T> From<$t> for Result<T> {
            fn from(error: $t) -> Self {
                Err(Error::from(error))
            }
        }
    }
);

impl_into_error!(TensorNodeError);
impl_into_error!(GraphError);
impl_into_error!(GraphNodeError);
impl_into_error!(GraphCallError);

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::ExternalError(ExternalError::IOError(error))
    }
}
