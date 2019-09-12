use docopt::Docopt;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::str::FromStr;

use libhsbc::category::{Category, UNKNOWN_CATEGORY};
use libhsbc::parser::Parser;
use libhsbc::transaction::Transaction;

const USAGE: &str = "
HSBC Statement Parser

Usage:
  hsbc statement <pdf> [--category-file=<category-file>]
  hsbc overview <pdf> [--category-file=<category-file>]
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
    cmd_overview: bool,
    flag_category_file: String,
    arg_pdf: String,
    arg_category_file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.cmd_statement {
        show_statement(&args.arg_pdf, &args.flag_category_file)?;
    }

    if args.cmd_add_categories {
        add_categories(&args.arg_pdf, &args.arg_category_file)?;
    }

    if args.cmd_overview {
        show_overview(&args.arg_pdf, &args.flag_category_file)?;
    }

    Ok(())
}

fn show_overview(
    pdf_filename: &str,
    category_filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let categories = if category_filename.is_empty() {
        Vec::new()
    } else {
        let category_file = File::open(category_filename)?;
        serde_json::from_reader(&category_file).unwrap_or_default()
    };

    let pdf_file = File::open(pdf_filename)?;
    let mut parser = Parser::new(categories);
    let mut statement = parser.parse(pdf_file)?;
    statement.calculate_category_overview();

    println!("\nHSBC Creditcard Statement\n");
    println!(
        "Transactions:          {}",
        statement.credits.len() + statement.debits.len()
    );
    println!("Total Debits:     {}", statement.total_debits);
    println!("Total Credits:    {}\n", statement.total_credits);
    statement.categories.sort_by(|a, b| a.name.cmp(&b.name));
    for category in &statement.categories {
        if category.name == UNKNOWN_CATEGORY {
            continue;
        }
        println!(
            "Spent {} AED or {:.2} GBP on {}",
            category.debits - category.credits,
            (category.debits - category.credits) * Decimal::from_str("0.220260").unwrap(),
            category.name.to_lowercase(),
        );
    }

    for category in &statement.categories {
        let mut percentage =
            (category.debits - category.credits) / statement.total_debits * Decimal::new(100, 0);

        if percentage < Decimal::new(0, 0) {
            percentage = Decimal::new(0, 0);
        }

        println!("\n{} ({:.0}% of all spend)", category.name, percentage);
        let mut transactions = statement.get_debits_for_category(&category.name);
        transactions.sort_by(|a, b| b.amount.cmp(&a.amount));
        for transaction in transactions {
            println!("{}    {}", &transaction.amount, &transaction.details);
        }
    }

    Ok(())
}

fn show_statement(
    pdf_filename: &str,
    category_filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let categories = if category_filename.is_empty() {
        Vec::new()
    } else {
        let category_file = File::open(category_filename)?;
        serde_json::from_reader(&category_file).unwrap_or_default()
    };

    let pdf_file = File::open(pdf_filename)?;
    let mut parser = Parser::new(categories);
    let mut statement = parser.parse(pdf_file)?;
    statement.calculate_category_overview();
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
    let mut parser = Parser::new(Vec::new());
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
