use docopt::Docopt;
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;

use libhsbc::category::{Category, UNKNOWN_CATEGORY};
use libhsbc::parser::Parser;
use libhsbc::transaction::Transaction;

const USAGE: &str = "
HSBC Statement Parser

Usage:
  hsbc statement <pdf> [--category-file=<category-file>]
  hsbc add-categories <pdf> <category-file>
  hsbc (-h | --help)
  hsbc --version

Options:
  -h --help                         Show this screen.
  --version                         Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_statement: bool,
    cmd_add_categories: bool,
    flag_category_file: String,
    arg_pdf: String,
    arg_category_file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.cmd_statement {
        statement(&args.arg_pdf)?;
    }

    if args.cmd_add_categories {
        add_categories(&args.arg_pdf, &args.arg_category_file)?;
    }

    Ok(())
}

fn statement(pdf_filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pdf_file = File::open(pdf_filename).unwrap();
    let mut parser = Parser::new();
    let statement = parser.parse(pdf_file)?;
    let j = serde_json::to_string_pretty(&statement)?;
    println!("{}", j);
    Ok(())
}

fn add_categories(
    pdf_filename: &str,
    categories_filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let pdf_file = File::open(pdf_filename)?;
    let mut categories_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .append(false)
        .open(categories_filename)?;

    let mut categories: Vec<Category> =
        serde_json::from_reader(&categories_file).unwrap_or_default();
    let mut parser = Parser::new();
    let mut statement = parser.parse(pdf_file)?;
    let mut transactions: Vec<Transaction> = Vec::new();
    transactions.append(&mut statement.debits);
    transactions.append(&mut statement.credits);

    for transaction in transactions {
        let matches: Vec<&Category> = categories
            .iter()
            .filter(|c| c.is_match(&transaction.details))
            .collect();

        if !matches.is_empty() {
            // This transaction is already in the categories file
            continue;
        }

        match categories
            .iter_mut()
            .find(|ref c| c.name == UNKNOWN_CATEGORY)
        {
            Some(category) => {
                // We already have a category called 'Unknown'
                // Add the merchant if it doesn't exist in the patterns already
                if !category.is_match(&transaction.details) {
                    category.patterns.push(transaction.details);
                }
            }
            None => {
                // We don't have a category for Unknown, so create it
                categories.push(Category {
                    name: UNKNOWN_CATEGORY.to_string(),
                    patterns: vec![transaction.details],
                })
            }
        }
    }

    categories.sort_by(|a, b| a.name.cmp(&b.name));
    let j = serde_json::to_string_pretty(&categories)?;
    categories_file.seek(std::io::SeekFrom::Start(0))?;
    categories_file.write_all(j.as_bytes())?;

    Ok(())
}
