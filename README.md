# HSBC Statement PDF Analyzer

This repository contains a library (libhsbc) and a CLI program (hsbc) for analyzing HSBC credit card statements.

It works with statements issued in the United Arab Emirates, and has not been tested with any other formats.

## Usage 

```
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
```

## Examples (all values redacted with 0.00)

### Basic overview without categories

The `hsbc overview <pdf>` command output will analye a statement, and output an overview of the total debits/credits, as well as a list of all transactions. 

This is useful, but this CLI tool can be a lot smarter and categorize spends if you provide a JSON file with a list of categories, and the text patterns that should be used to match a transaction to a category. See the examples below for more...

```sh
$ hsbc overview September2019.pdf 

HSBC Creditcard Statement

Transactions:        182
Total Debits:       0.00
Total Credits:      0.00

Unknown (100% of all spend)
0.00 AED - INSTASHOP DMCC
0.00 AED - DUBAI DUTY FREE
0.00 AED - DUBAI DUTY FREE- C2 DU DUBAI
0.00 AED - INSTASHOP DMCC
0.00 AED - WOLFIS BIKE SHOP
0.00 AED - FRIENDS AVENUE CATERIN DUBAI
... all other transactions ...
```

### Training new categories

In order to get richer analysis of spending, this CLI tool uses a JSON file which maps text patterns.

There is a basic categories.json file in this repository that can be used as a starting point. 

You can use `hsbc add-categories <pdf> <category-file>` to read all transactions from a statement PDF you have, and insert them into a category JSON file. They will be added to the 'Unknown' category, but you can then cut/paste them into your own categories.

If the category file you specify does not exist, it will be created.
If it does exist already, then any new previously uncategorized transactions will be added (in the Unknown category).

The patterns inside the categories JSON file are fuzzy, so for example `STARBUCKS` will match any transaction found with `STARBUCKS DUBAI MARINA` or `starbucks #259`. The matching is done in priority order - the first match found will be the chosen category.

```
$ hsbc add-categories September2019.pdf categories.json
$ head -n20 categories.json
[
  {
    "name": "DIY",
    "patterns": [
      "Speedex International",
      "ACE-"
    ]
  },
  {
    "name": "Dining (in)",
    "patterns": [
      "ZOMATO ORDER",
      "UBER EATS"
    ]
  },
  ...
```

### Categorised Overview

Now if we re-run the overview command, but pass in our categories file, we get much richer information.

```
$ hsbc overview September2019.pdf --category-file categories.json

HSBC Creditcard Statement

Transactions:          182
Total Debits:         0.00 AED
Total Credits:        0.00 AED

Spent 0.00 AED on utilities
Spent 0.00 AED on dining (out)
Spent 0.00 AED on transportation
Spent 0.00 AED on shopping (food & drink)
Spent 0.00 AED on travel
Spent 0.00 AED on recreation
Spent 0.00 AED on shopping (misc)
Spent 0.00 AED on dining (in)
Spent 0.00 AED on diy
Spent 0.00 AED on health

Utilities (31% of all spend)
0.00 AED - Emicool Plus Cooling C Dubai
0.00 AED - SMART DUBAI GOVERNMENT DUBAI
0.00 AED - SMART DUBAI GOVERNMENT DUBAI
0.00 AED - SMARTDXBGOV ETISALAT
0.00 AED - VIRGIN MOBILE UAE\(EITC DUBAI
0.00 AED - SMART DUBAI GOVERNMENT DUBAI
0.00 AED - Smart Dubai Government Dubai
0.00 AED - SMART DUBAI GOVERNMENT DUBAI

Dining (out) (17% of all spend)
0.00 AED - COURTYARD BY MARRIOTT
0.00 AED - P.F CHANGS -CC 4324
0.00 AED - GOURMET WOK
0.00 AED - OSSEGG Pivovary s.r.o. Praha 2
0.00 AED - COURTYARD BY MARRIOTT
0.00 AED - COURTYARD BY MARRIOTT
0.00 AED - FOREFRONT FACILITIES M DUBAI

... (this continues for all categories/transactions)
```

### JSON Output

The CLI also supports exporting all information in JSON

```
$ hsbc statement September2019.pdf --category-file categories.json
{
  "total_credits": "0.00",
  "total_debits": "0.00",
  "credits": [
    {
      "id": "1d57b881d9b66159af777dc43f394203d58a52d98a699c080ebf52da865677a4",
      "date": 1565740800,
      "details": "REFUND CAREEM",
      "amount": "0.00",
      "category": "Transportation"
    },
    ...
  ],
  "debits": [
    {
      "id": "1d57b881d9b66159af777dc43f394203d58a52d98a699c080ebf52da865677a4",
      "date": 1565740800,
      "details": "CAREEM",
      "amount": "0.00",
      "category": "Transportation"
    },
    ...
  ],
  "categories": [
    {
      "name": "Unknown",
      "count": 0,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "Transportation",
      "count": 16,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "Recreation",
      "count": 8,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "Dining (in)",
      "count": 3,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "Dining (out)",
      "count": 42,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "Travel",
      "count": 6,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "Shopping (misc)",
      "count": 8,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "Shopping (food & drink)",
      "count": 17,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "Utilities",
      "count": 8,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "DIY",
      "count": 2,
      "credits": "0.00",
      "debits": "0.00"
    },
    {
      "name": "Health",
      "count": 1,
      "credits": "0.00",
      "debits": "0.00"
    }
  ]
}
```