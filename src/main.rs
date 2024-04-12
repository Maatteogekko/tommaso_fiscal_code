use std::io::{stdin, stdout, Write};

use tommaso_fiscal_code::{info, validate_or_error};

fn main() {
    loop {
        print!("Insert code to validate: ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap_or_else(|err| {
            eprintln!("Error reading input: {}", err);
            std::process::exit(1);
        });

        let result = validate_or_error(&input);
        match result {
            Ok(_) => {
                println!("Code is valid");

                let info = info(&input).unwrap();
                println!("Info:");
                println!("\tBorn on: {}", info.born_on);
                println!("\tGender: {}", info.gender);
                println!("\t{}", info.place_of_birth);
            }
            Err(e) => println!("Code is invalid: {}", e),
        }
        stdout().flush().unwrap();
    }
}
