use std::iter;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
struct Config {
    very_large_vec: Vec<String>,
}

#[derive(Clone)]
struct Worker {
    config: Arc<Config>,
}

const CAPACITY: usize = usize::MAX / 10000000;

fn main() {
    // Our `config` object is very large and too expensive to copy.
    let config = Config {
        very_large_vec: Vec::with_capacity(CAPACITY),
    };

    // We move `config` into the `Arc::new` constructor which consumes
    // the original `config` value and returns it wrapped in the
    // `Arc<T>` struct.
    // We bind the new Arc<T> wrapped value to a binding of the
    // same name (`config`) as the previous binding is no longer valid.
    let config = Arc::new(config);

    // Here we use `iter` functions to generate a `Vec` of 100 `Worker`
    // structs.
    // It would be too expensive to Clone `config` 100 times.
    // Thanks to the use of `Arc<Config>`, we are only storing 1 copy
    // of `Config` on the heap, and passing a counted reference to each
    // `Worker` by calling `clone()` on the `Arc<Config>` object.
    let workers: Vec<Worker> = iter::repeat(Worker {
        config: config.clone(),
    })
    .take(100)
    .collect();

    assert_eq!(workers[0].config.very_large_vec.capacity(), CAPACITY);
    assert_eq!(workers[0].config, workers[1].config);
}
