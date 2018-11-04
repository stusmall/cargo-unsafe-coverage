#![allow(unused_variables, dead_code)]

extern crate failure;
extern crate syn;

use std::fs::File;
use std::io::Read;
use std::ops::Add;
use std::path::Path;

use failure::Error;

use syn::{Expr, Item, Stmt};

#[derive(Debug, Default, PartialEq)]
struct SafenessSummary {
    safe_statements: u64,
    unsafe_statements: u64,
}

impl SafenessSummary {
    fn new_leaf(is_unsafe: bool) -> Self {
        SafenessSummary {
            safe_statements: if is_unsafe { 0 } else { 1 },
            unsafe_statements: if is_unsafe { 1 } else { 0 },
        }
    }
}

impl Add for SafenessSummary {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SafenessSummary {
            safe_statements: self.safe_statements + other.safe_statements,
            unsafe_statements: self.unsafe_statements + other.unsafe_statements,
        }
    }
}

fn main() {
    println!("Hello, world!");
}

fn parse_source_file(file: &Path) -> Result<SafenessSummary, Error> {
    let mut file = File::open(file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    process_string(&content)
}

fn process_string(src: &str) -> Result<SafenessSummary, Error> {
    let ast = syn::parse_file(src).map_err(|err| failure::err_msg(err.to_string()))?;
    Ok(ast
        .items
        .iter()
        .map(|x| process_item(x, false))
        .fold(SafenessSummary::default(), |acu, summary| acu + summary))
}

fn process_expression(expr: &Expr, already_unsafe: bool) -> SafenessSummary {
    match expr {
        Expr::Call(call) => call
            .args
            .iter()
            .map(|expr| process_expression(&expr, already_unsafe))
            .fold(SafenessSummary::new_leaf(already_unsafe), |acc, summary| {
                acc + summary
            }),

        Expr::Unsafe(unsafe_block) => unsafe_block
            .block
            .stmts
            .iter()
            .map(|stmt| process_stmt(&stmt, true))
            .fold(SafenessSummary::default(), |acc, summary| acc + summary),

        Expr::Macro(_) => {
            //TODO:  Can(should) we expand this?
            SafenessSummary::new_leaf(already_unsafe)
        }
        _ => {
            unimplemented!();
        }
    }
}

fn process_stmt(stmt: &Stmt, already_unsafe: bool) -> SafenessSummary {
    match stmt {
        Stmt::Item(item) => process_item(&item, already_unsafe),
        Stmt::Expr(expr) => process_expression(&expr, already_unsafe),
        Stmt::Local(_) => {
            unimplemented!();
        }
        Stmt::Semi(expr, _) => process_expression(&expr, already_unsafe),
    }
}

fn process_item(item: &Item, already_unsafe: bool) -> SafenessSummary {
    match item {
        Item::ExternCrate(_) => unimplemented!(),
        Item::Use(_) => unimplemented!(),
        Item::Static(_) => unimplemented!(),
        Item::Const(_) => unimplemented!(),
        Item::Fn(function) => function //TODO: check safeness of function
            .block
            .stmts
            .iter()
            .map(|stmt| process_stmt(&stmt, already_unsafe || function.unsafety.is_some()))
            .fold(
                SafenessSummary::new_leaf(already_unsafe || function.unsafety.is_some()),
                |acc, summary| acc + summary,
            ),
        Item::Mod(m) => SafenessSummary::new_leaf(already_unsafe),
        Item::ForeignMod(_) => unimplemented!(),
        Item::Type(_) => unimplemented!(),
        Item::Existential(_) => unimplemented!(),
        Item::Struct(_) => unimplemented!(),
        Item::Enum(_) => unimplemented!(),
        Item::Union(_) => unimplemented!(),
        Item::Trait(_) => unimplemented!(),
        Item::TraitAlias(_) => unimplemented!(),
        Item::Impl(_) => unimplemented!(),
        Item::Macro(_) => unimplemented!(),
        Item::Macro2(_) => unimplemented!(),
        Item::Verbatim(_) => unimplemented!(),
    }
}

#[test]
fn hello_world() {
    let source = "
    fn main() {
        println!(\"hello world!\");
    }
    ";

    assert_eq!(
        process_string(source).unwrap(),
        SafenessSummary {
            safe_statements: 2,
            unsafe_statements: 0
        }
    );
}

#[test]
fn simple_unsafe_block() {
    let source = "
    fn main() {
        unsafe {
            unimplemented!();
        }
    }
    ";

    assert_eq!(
        process_string(source).unwrap(),
        SafenessSummary {
            safe_statements: 1,
            unsafe_statements: 1
        }
    );
}

#[test]
fn simple_unsafe_func() {
    let source = "\
unsafe fn foreign() {
    unimplemented!();
}

fn main() {

}
";

    assert_eq!(
        process_string(source).unwrap(),
        SafenessSummary {
            safe_statements: 1,
            unsafe_statements: 2
        }
    );
}

#[test]
fn compare_split_unsafe_blocks() {
    let source1 = "
fn main() {
  unsafe {
    foreign();
    foreign();
    foreign();
  }
}
";

    let source2 = "
fn main() {
  unsafe {
    foreign();
  }
  unsafe {
    foreign();
  }
  unsafe{
    foreign();
  }
}
";

    assert_eq!(
        process_string(source1).unwrap(),
        process_string(source2).unwrap()
    );
}
