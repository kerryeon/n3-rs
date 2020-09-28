#![feature(is_sorted)]

mod n3_std;

mod cache;
mod code;
mod compact;
mod context;
mod error;
mod execs;
mod externs;
mod graph;
mod nodes;
mod seed;
mod tensor;
mod variable;

pub use self::ast::{Out, Outs};
pub use self::error::{Error, Result};
pub use self::execs::ExecRoot;
pub use self::graph::Table;

use n3_parser::{ast, Parser};

#[cfg(test)]
mod tests_recon {
    use std::fs;

    fn recon(source: &str) {
        let parser = super::Parser::new();

        let source_recon1 = format!("{:?}", parser.parse_file(source).unwrap());
        println!("{}", &source_recon1);
        let source_recon2 = format!("{:?}", parser.parse_file(&source_recon1).unwrap());

        assert_eq!(source_recon1, source_recon2);
    }

    #[test]
    fn test_dummy() {
        let source = fs::read_to_string("tests/data/nodes/__user__/sample/dummy.n3").unwrap();

        recon(&source);
    }

    #[test]
    fn test_all_externs() {
        for source in super::n3_std::get_sources().values() {
            recon(&source);
        }
    }
}