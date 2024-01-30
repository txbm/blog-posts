struct Config {
    path: String,
}

fn main() {
    // [`main`] function owns [`Config`] object at beginning of stack.
    let config = Config {
        path: String::from("/etc/nginx/nginx.conf"),
    };

    // [`is_valid_config`] is now borrowing [`&Config`] by reference
    match is_valid_config(&config) {
        true => println!("Valid config"),
        false => println!("Invalid config"),
    }

    // [`main`] function still owns [`Config`] at end of execution
}

/// Checking to see if [`Config.path`] is/not empty does not
/// require a dedicated copy or exclusive control of the value
/// so borrowing is the best choice here.
fn is_valid_config(config: &Config) -> bool {
    !config.path.is_empty()
}
