use the_nuker::{Cleaner, read_arg};
use dialoguer::Confirm;
use std::process::exit;

fn main() {
    pretty_env_logger::init();

    let arg = read_arg().unwrap_or_else(|err| {
        println!("\x1b[1m\x1b[33mhelp\x1b[0m: {err}");
        exit(1);
    });

    let confirmation = Confirm::new()
                                    .with_prompt("Do you wish to confirm your action? You're about to nuke that directory you just typed lol")
                                    .interact()
                                    .unwrap_or_else(|e| {
                                        println!("\x1b[31mERROR\x1b[0m: {e}");
                                        exit(1);
                                    });


    if confirmation {
        println!("OK then, cleaning {:?}...", arg.dir);
        Cleaner::new(arg.dir).clean();
    } else {
        println!("Action canceled");
    }
}