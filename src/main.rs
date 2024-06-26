use the_nuker::{Cleaner, read_arg};
use inquire::Confirm;
use std::process::exit;


fn main() {
    let arg = read_arg();

    let confirmation = Confirm::new("Are you sure you want to delete this thing?")
        .with_help_message("It'll be gone forever")
        .with_placeholder("y|n")
        .prompt();

    match confirmation {
        Ok(true) => {
            println!("OK then, cleaning {:?}...", arg);
            Cleaner::new(arg).clean();
        }

        Ok(false) => {
            println!("Action canceled");
            exit(1);
        }

        Err(e) => {
            println!("Don't know what you did but the program threw an error: \x1b[1m\x1b[33m{e}\x1b[0m");
            exit(1);
        }
    }
}