struct Config {
    path: String,
    very_long_vector: Vec<String>,
}

struct Versioned<O> {
    version: u32,
    obj: O,
}

const CAPACITY: usize = usize::MAX / 10000000;

fn main() {
    // `main` function owns `Config` object at beginning of stack.
    let config = Config {
        path: String::from("/etc/nginx/nginx.conf"),
        very_long_vector: Vec::with_capacity(CAPACITY),
    };

    // `config` binding is moved into `save_config_version` and
    // dropped from this scope. The new `Versioned` representation of
    // `config` is returned and stored in the new binding called
    // `versioned_config`.
    let versioned_config = save_config_version(config);

    // `config` is no longer a valid binding at this point

    assert_eq!(versioned_config.version, 1);
    assert_eq!(versioned_config.obj.path, "/etc/nginx/nginx.conf");
    assert_eq!(versioned_config.obj.very_long_vector.capacity(), CAPACITY);
}

fn save_config_version(config: Config) -> Versioned<Config> {
    Versioned {
        version: 1,
        obj: config,
    }
}
