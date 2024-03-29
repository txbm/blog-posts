---
title: Rust Patterns for Lifetime Management
slug: rust-patterns-for-lifetime-management
tags: rust, programming, coding, software, engineering
domain: www.sigwait.com
subtitle: A set of patterns for beginner and intermediate Rust programmers to deliberately choose a strategy for ownership and borrowing depending on the program.
cover: https://cdn.hashnode.com/res/hashnode/image/upload/v1706654502273/t2dc6cO35.webp?auto=format
ignorePost: false
publishAs: 
canonical: 
hideFromHashnodeCommunity: 
seoTitle: 
seoDescription: 
disableComments:
seriesSlug: 
enableToc: true
saveAsDraft: false
---

Lifetime management is a core fundamental skill in becoming proficient in using Rust.

Following is a set of patterns designed to help the programmer select an appropriate strategy for ownership or borrowing depending on the program.

**Reminder: Lifetimes are a core Rust abstraction that addresses the complexities of [memory management](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) inherent in any computer engineering task. Lifetimes serve as an alternative to automated garbage collection or direct pointer manipulation found in other languages.**

We enumerate the following list of five "lifetime patterns" with pointers on when to consider them appropriate.

## Borrow Everything

Use when:

- performance is critical by default

- there is no specific reason why ownership is required

- data structures are expensive to copy

- only a single stack frame will be accessing an object reference at any given moment

- most or all of the objects originate (and are therefore owned) at the root of the stack

Immutable borrowing should likely be your [default lifetime strategy](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html). The reason is that data access is temporary and read-only for a linear call chain that is not concurrent nor performs mutation. Functions deeper in the stack can produce unrelated outputs without exclusive or permanent input control.

Immutable borrows form a chain of ownership and lending as execution progresses into the stack.

![illustration-1](https://cdn.hashnode.com/res/hashnode/image/upload/v1706645949909/qXGNgTOLn.png?auto=format)
_As execution winds the stack, later frames borrow the value owned by the earlier frames. As the stack unwinds, the earlier frame remains the value owner until program termination. The later frames have gone out of scope, dropping their borrowed references._

The critical thing to understand here is that the beginning and end states are the same for a borrowed value because borrowing does not transfer ownership away from the originating caller.

Code Example:

```rust
struct Config {
    path: String
}

fn main() {
    // [`main`] function owns [`Config`] object at the beginning of the stack.
    let config = Config {
        path: String::from("/etc/nginx/nginx.conf")
    };

    // [`is_valid_config`] is now borrowing [`&Config`] by reference
    match is_valid_config(&config) {
        true => println!("Valid config"),
        false => println!("Invalid config")
    }

    // [`main`] function still owns [`Config`] at the end of execution
}

/// Checking to see if [`Config.path`] is/not empty does not
/// require a dedicated copy or exclusive control of the value.
/// Borrowing is the best choice here.
fn is_valid_config(config: &Config) -> bool {
    !config.path.is_empty()
}
```

As the comments indicate, this works because the function later in the stack requires neither exclusive control nor a dedicated copy to perform its work on the config object.

If the callee requires neither of those properties nor intends to consume the value (destroy it), then the best solution is likely the immutable shared reference (aka the borrow).

We do not go into mutable borrowing in this guide because the strategies for mutation require additional considerations, such as the use of [Mutex](https://doc.rust-lang.org/book/ch16-03-shared-state.html?search=#using-mutexes-to-allow-access-to-data-from-one-thread-at-a-time). We will cover these in an upcoming guide.

## Borrow Most Things, Clone Some Things

Use when:

- most objects require only temporary access

- a subset of objects will be permanently consumed or altered

- a subset of objects originate later in the stack but are returned for ownership nearer to the stack root

- the overhead of `Clone`’ing is deemed acceptable

A slight variation on Pattern #1, this is where the invariants of immutable borrowing are satisfied with the exception that a callee must [operate on its copy](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#variables-and-data-interacting-with-clone) of a value that must also **remain owned** by a caller earlier in the stack.

The essential constraint here is that we can neither move nor borrow the object. Two parts of the program require separate copies of the same object for valid reasons.

![illustration-2](https://cdn.hashnode.com/res/hashnode/image/upload/v1706646090540/LFnb1EmsL.png?auto=format)
_X is Clone’d while Y is &borrowed. Frame #9 can do whatever it wants with its copy of X (including destroying it). However, at the end of execution, Frame #0 still owns the original copy of X and the original reference to Y it was lending out._

Note that, again, the start and end states are the same. Even though Frame #9 has its copy of X to use as needed, Frame #0 still retains ownership of the original copy. Cloning makes sense when X is cheap to copy, and Frame #9 is doing something that requires exclusive control of the value.

Let’s modify our prior example to see a situation where that could be necessary:

```rust
#[derive(Clone)]
struct Config {
    path: String,
}

struct Versioned<O> {
    version: u32,
    obj: O,
}

fn main() {
    // `main` function owns the `Config` object at the beginning of the stack.
    let config = Config {
        path: String::from("/etc/nginx/nginx.conf"),
    };

    /*
      `config` is `Clone`'d to provide `save_config_version` with
      a copy of `config` so that it can save it into its `Versioned`
      construct. Copying the value is necessary because we must preserve
      the version even if the original copy is changed later.
      Therefore, we store a copy of it within the version struct.
    */
    let version_1 = save_config_version(config.clone());

    // `main` function still owns `config` at the end of execution

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
```

_Note: the use of the `#[derive(Clone)]` trait macro to ensure that `Config` is Cloneable._

The main reasons Clone’ing is an appropriate strategy here are because:

- The value is cheap to copy (a small struct of primitives)
- The program's intended goal is to ensure that the copies of the value can be independently modified (or preserved).

Therefore, we must have multiple copies of the data structure to treat independently.

We could not do this with mutable borrowing or reference counting because all references would see any mutation. We cannot move the value because we will lose its original state after it is mutated.

## Borrow Most Things, Move Some Things

Use when:

- most objects require only temporary access

- we need to consume or alter a subset of objects permanently

- a subset of objects are expensive to `Clone`

- a subset of objects are only required by a single function or subprocess

Similar to the prior scenario where the callee requires ownership of the value, but for some reason, `Clone` is not an option.

There are valid reasons to require ownership. The most obvious is if `Clone` would be too expensive because the value has a large memory footprint.

In this case, you may still be borrowing most things, but you specifically identify the object that [needs to move and pass it by value](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#variables-and-data-interacting-with-move) to the callee such that ownership transfers away from the caller and into the callee.

From there, either:

1. The callee eventually passes ownership back to the caller either as the same value or a derivative value
2. The callee drops the object after it goes out of scope, and it is never seen nor heard from again

In either case, the caller may not reference the object after moving it to the callee. If the callee returns ownership to the caller, the caller may reference the returned value as a new binding. The original binding is no longer valid after a move.

![illustration-3](https://cdn.hashnode.com/res/hashnode/image/upload/v1706646107663/wh1G4izwC.png?auto=format)
_Notably, at the end of the program, Frame #0 is left still owning Y but no longer owns X because X was moved to Frame #9 and never returned as a new binding. Frame #0 will never know what happened to X :(_

We adapt our example again to see a situation where this might occur:

```rust
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
    // `main` function owns the `Config` object at the beginning of the stack.
    let config = Config {
        path: String::from("/etc/nginx/nginx.conf"),
        very_long_vector: Vec::with_capacity(CAPACITY),
    };

    // `config` binding moves into `save_config_version` and
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
```

This example shows a situation where the `Config` object is now too expensive to Clone due to the huge data structure (giant vector) it contains.

Therefore, when we decide to make a versioned representation of it, we allow the versioning function to take exclusive ownership of the value (consume it) and take ownership of the new value returned in its place.

The `save_config_version` function performs its work without the expense of having an owned copy of the data structure.

If we wanted to make a second version of this `Config`, we would face this problem again. The solution to that problem is more sophisticated and beyond the scope of this guide.

## Move All the Things

Use when:

- objects are expensive to Clone

- the inner scope may outlive the outer scope

Moving is necessary when `Clone` is too expensive or the scope needing the value may outlive the scope providing the value.

One example is a program with two tasks. One task reads data from a socket, and the other performs a longer-running computation on the data.

The task reading data from the socket could pass references to the task processing the data, but there is a problem. If the task processing the data takes longer to complete than the task reading data from the buffer, the processing task will outlive the buffer reading task.

If the buffer-reading task goes out of scope before the processing task, the processing task is now holding a reference to nothing. Rust does not allow this. The borrow checker will detect this possibility and report an error.

In this situation, you have to move the value from the task that reads the data to the task that processes the data. Often this is expressed as a closure created with the `move { }` or `async move { }` blocks.

![illustration-4](https://cdn.hashnode.com/res/hashnode/image/upload/v1706646120906/9I2UroF2x.png?auto=format)
_Here, we see a major difference: ownership is transferred from Task #0 to Task #1, but at the end of execution, Task #0 no longer exists. Task #1 has outlived Task #0 and retains sole ownership of X. Task #1 cannot borrow X from Task #0 because Task #0 may terminate before Task #1. The borrow-checker will not allow this._


```rust
use reqwest::Client;

const URL: &str = "https://google.com";

#[tokio::main]
async fn main() {
    let client = Client::new();

    tokio::spawn(async move { client.get(URL).send().await.unwrap() })
        .await
        .unwrap();

    // `client` is no longer a valid reference at this point
    // It permanently moves into the `spawn`'d closure
}
```

Here, we see that the outer scope `main` representing Task #0 will create a `client` and move it to be owned by Task #1. Since Task #1 is async and `main` does not block, the `main` scope will be dropped immediately after starting Task #1.

Task #1 can continue to run until completion as it has taken ownership of the `client` needed to perform its work.

## Reference Count Certain Things

Use when:

- objects are expensive to clone

- multiple threads or processes must access the same references concurrently

- moving and returning ownership is prohibitively complex or precluded by concurrent access requirements

- you are already using locks ([Mutex](https://doc.rust-lang.org/book/ch16-03-shared-state.html?search=#using-mutexes-to-allow-access-to-data-from-one-thread-at-a-time)) for atomic mutation

Reference counting is the most complex technique, where we want to neither move nor implicitly borrow. One can think of this approach as a more explicit form of borrowing where we are [tracking each reference](https://doc.rust-lang.org/book/ch15-04-rc.html#using-rct-to-share-data) holder to an object explicitly using a counter that tracks new and dropped references.

We get the efficiency of borrowing with the semantics of ownership as long as we do not mutate the underlying value.

Rust's two primary reference counted types are `Rc<T>` and `Arc<T>`. Rc means ["reference counted"](https://doc.rust-lang.org/std/rc/struct.Rc.html) and Arc means ["atomically reference counted"](https://doc.rust-lang.org/std/sync/struct.Arc.html). Calling `.clone()` on an `Rc<T>` or `Arc<T>` does not physically copy the inner value but instead creates a smart pointer to the heap-allocated inner value.

From the perspective of the borrow-checker, the receiver of a `Clone`’d reference counted type has an “owned” copy of the value because the lifetime of the inner value has been moved outside the stack and onto the heap. The value will remain on the heap until the last reference drops, ensuring that even if earlier references drop, later references will still be valid for as long as needed.

Reference counted type `Clone`s are still shared references — the same way a borrow is — but allow the semantics of an owned value without the overhead of copying the value.

It may seem like this should always be the best option — but in practice, the extra boilerplate and complexity are only worth it if the more basic lifetime strategies prove insufficient.

Reserve this approach for when the lifetimes of your stack frames (scopes) are non-linear and moving or cloning bits are not appropriate for control flow or performance reasons.

Note: reference counting is not [without pitfalls if done improperly](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html?highlight=Weak).

Let’s look at when using a reference counted type would be an appropriate solution.

![illustration-5](https://cdn.hashnode.com/res/hashnode/image/upload/v1706646132005/bLIbFMURw.png?auto=format)
_Here, we see that `Arc::new(X)` moves the value from the stack to the heap. Then, when `clone()` is called on the value by subsequent frames, they receive a pointer to the heap location of X, and the reference counter goes up or down as references are taken or dropped._

```rust
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

    // We move `config` into the `Arc::new` constructor, which consumes
    // the original `config` value and returns it wrapped in the
    // `Arc<T>` struct.
    // We bind the new Arc<T> wrapped value to a binding of the
    // same name (`config`) as the previous binding is no longer valid.
    let config = Arc::new(config);

    // Here, we use `iter` functions to generate a `Vec` of 100 `Worker`
    // structs.
    // It would be too expensive to Clone `config` 100 times.
    // Thanks to the use of `Arc<Config>`, we are only storing 1 copy
    // of `Config` on the heap and passing a counted reference to each
    // `Worker` by calling `clone()` on the `Arc<Config>` object.
    let workers: Vec<Worker> = iter::repeat(Worker {
        config: config.clone(),
    })
    .take(100)
    .collect();

    assert_eq!(workers[0].config.very_large_vec.capacity(), CAPACITY);
    assert_eq!(workers[0].config, workers[1].config);
}
```


We will not cover the mutation case here as that will require introducing [Mutex](https://doc.rust-lang.org/book/ch16-03-shared-state.html?search=#using-mutexes-to-allow-access-to-data-from-one-thread-at-a-time). Stay tuned for more in an upcoming article.

**Wrapping Up**

This guide provides a practical framework for planning your lifetime management approach when designing programs in Rust.

**Remember: defaulting to immutable borrowing is usually a good starting point.**

Values that will be **destroyed**, **returned**, or **irreversibly** transformed are probably good candidates for **moving** or **cloning**.

Values that will be used exclusively by a scope that outlives their original scope must be moved into the longer-living scope so they can exist after the original scope drops.

If performance constraints are bound and concurrent ownership or mutation is required, reference counting is probably the way to go.

A note on lifetime annotations: this guide does not cover [lifetime annotations](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html) (`<‘a>`) because they do not provide actual control over the lifetimes of references at runtime. They are hints to the Rust compiler (borrow checker) to disambiguate type signatures where the compiler cannot easily infer the programmer's intention for which references do and do not share lifetimes.

We will do a separate piece on lifetime annotations and when they are necessary and useful.

**Follow + Subscribe for More!**
<script async defer src="https://buttons.github.io/buttons.js"></script>

- <a target="_blank" href="https://twitter.com/itsyourcode?ref_src=twsrc%5Etfw" class="twitter-follow-button" data-show-count="false">Follow @itsyourcode</a><script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>
- <a target="_blank" class="github-button" href="https://github.com/txbm" data-color-scheme="no-preference: light; light: light; dark: dark;" data-size="large" aria-label="Follow @txbm on GitHub">Follow @txbm</a>
- <a target="_blank" class="github-button" href="https://github.com/sponsors/txbm" data-color-scheme="no-preference: light; light: light; dark: dark;" data-icon="octicon-heart" data-size="large" aria-label="Sponsor @txbm on GitHub">Sponsor</a>

**Additional Reading**

- [A Journey Through Rust Lifetimes](https://richardanaya.medium.com/a-journey-through-rust-lifetimes-5a08782c7091)
- [Advanced Lifetimes](https://academy.patika.dev/courses/rust-programming/advanced-lifetimes)
- [Understand Lifetimes](https://www.lurklurk.org/effective-rust/lifetimes.html)
- [Rust Doc: Lifetimes](https://doc.rust-lang.org/nomicon/lifetimes.html)
- [Rust Book 1st Edition: Lifetimes](https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/lifetimes.html)
- [A Complete Guide to Ownership and Borrowing](https://earthly.dev/blog/rust-lifetimes-ownership-burrowing/)
- [but what is 'a lifetime](https://www.youtube.com/watch?v=gRAVZv7V91Q&t=5s)
