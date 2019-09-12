use docopt::Docopt;
use serde::Deserialize;

use libhsbc::parser::Parser;

const USAGE: &str = "
HSBC Statement Parser

Usage:
  hsbc statement <pdf> [--category-file=<category.json>]
  hsbc generate-categories <pdf>
  hsbc (-h | --help)
  hsbc --version

Options:
  -h --help                         Show this screen.
  --version                         Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_statement: bool,
    cmd_generate_categories: bool,
    flag_category_file: String,
    arg_pdf: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.cmd_statement {
        let file = std::fs::File::open(&args.arg_pdf).unwrap();
        let mut parser = Parser::new();
        match parser.parse(file) {
            Ok(statement) => {
                let j = serde_json::to_string_pretty(&statement).unwrap();
                println!("{}", j);
            }

            Err(e) => println!("Error: {:#?}", e),
        }
    }

    if args.cmd_generate_categories {
        let file = std::fs::File::open(&args.arg_pdf).unwrap();
        let mut parser = Parser::new();
        match parser.parse(file) {
            Ok(statement) => {
                let j = serde_json::to_string_pretty(&statement).unwrap();
                println!("{}", j);
            }

            Err(e) => println!("Error: {:#?}", e),
        }
    }
}
