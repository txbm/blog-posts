#[derive(Clone)]
struct Config {
    path: String,
}

struct Versioned<O> {
    version: u32,
    obj: O,
}

fn main() {
    // `main` function owns `Config` object at beginning of stack.
    let config = Config {
        path: String::from("/etc/nginx/nginx.conf"),
    };

    /*
      `config` is `clone`'d to provide `save_config_version` with its
      own copy of `config` so that it can save it into its `Versioned`
      construct. This is necessary because a version must be preserved
      even if the original copy is changed later on. Therefore, we must
      store a copy of it as the original may be modified.
    */
    let version_1 = save_config_version(config.clone());

    // `main` function still owns `config` at end of execution

    // `save_config_version` owned the copy of `Config` while it was
    // creating its owned `Versioned` object. Then it dropped
    // its ownership of `config` and `Versioned` by returning them both
    // to `main`, bound as `version_1`.

    assert_eq!(version_1.version, 1);
    assert_eq!(version_1.obj.path, "/etc/nginx/nginx.conf");
}

fn save_config_version(config: Config) -> Versioned<Config> {
    Versioned {
        version: 1,
        obj: config,
    }
}
