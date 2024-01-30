---
title: Rust Patterns for Lifetime Management
slug: rust-patterns-for-lifetime-management
tags: rust, programming, coding, software, engineering
domain: www.sigwait.com
subtitle: A set of patterns for beginner and intermediate Rust programmers to deliberately choose a strategy for ownership and borrowing depending on the goals of the program.
cover: https://cdn.hashnode.com/res/hashnode/image/upload/v1706581315324/T6C-mL9-i.png?auto=format
ignorePost: false
publishAs: 
canonical: 
hideFromHashnodeCommunity: 
seoTitle: 
seoDescription: 
disableComments:
seriesSlug: 
enableToc: true
saveAsDraft: true
---

Lifetime management is a core fundamental skill in becoming proficient in using Rust.

Following is a set of patterns designed to help the programmer select an appropriate strategy for ownership or borrowing depending on the goals of the program.

**Reminder: Lifetimes are a core Rust abstraction that address the complexities of memory management inherent in any computer engineering task. Lifetimes serve as an alternative to automated garbage collection or direct pointer manipulation found in other languages.**

We enumerate the following list of five “lifetime patterns” with pointers on when to consider them appropriate.

## Borrow Everything

Use when:

- performance is critical by default

- there is no specific reason why ownership is required

- data structures are expensive to copy

- only a single stack frame will be accessing an object reference at any given moment

- most or all of the objects originate (and are therefore owned) at the root of the stack

This should likely be your default lifetime strategy in Rust. The reason being that for a linear call chain that is not concurrent, nor performs mutation, data access is temporary and read-only. Functions deeper in the stack can produce unrelated outputs without needing exclusive or permanent control of their inputs.

This forms a natural progression of ownership and lending as execution progresses.

![illustration-1](https://cdn.hashnode.com/res/hashnode/image/upload/v1706585296140/1P68LeilG.png?auto=format)
_As execution winds the stack, later frames borrow the value owned by the earlier frames. As the stack unwinds, the earlier frame remains the owner of the value until program termination and the later frames have gone out of scope, dropping their borrowed references._

The important thing to understand here is that the beginning and end state are the same for a borrowed value because borrowing does not transfer ownership away from the originating caller.

Code Example:

```rust
struct Config {
    path: String
}

fn main() {
    // [`main`] function owns [`Config`] object at beginning of stack.
    let config = Config {
        path: String::from("/etc/nginx/nginx.conf")
    };

    // [`is_valid_config`] is now borrowing [`&Config`] by reference
    match is_valid_config(&config) {
        true => println!("Valid config"),
        false => println!("Invalid config")
    }

    // [`main`] function still owns [`Config`] at end of execution
}

/// Checking to see if [`Config.path`] is/not empty does not
/// require a dedicated copy or exclusive control of the value
/// so borrowing is the best choice here.
fn is_valid_config(config: &Config) -> bool {
    !config.path.is_empty()
}
```

As the comments indicate, this works because the function later in the stack requires neither exclusive control nor a dedicated copy to perform its work on the config object.

If the callee requires neither of those properties nor intends to consume the value (destroy it), then the best solution is likely the immutable shared reference (aka the borrow).

We do not go into mutable borrowing in this guide because the strategies for mutation require additional considerations such as the use of Mutex. We will cover these in an upcoming guide.

## Borrow Most Things, Clone Some Things

use when:

- most objects require only temporary access

- a subset of objects need to be consumed permanently

- a subset of objects originate deeper in the stack, but are returned to ownership nearer to the stack root

- the overhead of Clone’ing is deemed acceptable

A slight variation on Pattern #1, this is where the invariants of immutable borrowing are still mostly satisfied BUT there is an exception where a callee must operate on its own copy of a value that must remain owned by a caller earlier in the stack.

The important criteria here is to establish that the object cannot be moved in addition to not being “borrowable”. This is to say that two parts of the program require their very own copy of the same object.

![illustration-2](https://cdn.hashnode.com/res/hashnode/image/upload/v1706585354753/vt3oCq84F.png?auto=format)
_X is Clone’d while Y is &borrowed. Frame #9 can do whatever it wants with its copy of X (including destroy it) but at the end of execution, Frame #0 still owns the original copy of X as well as the original reference to Y it was lending out._

Note that again, the start and end states are the same with respect to the starting frame. Even though Frame #9 has its own copy of X to use as needed, Frame #0 still retains ownership of the original copy. This could make sense when X is cheap to copy, and Frame #9 is doing something that requires it to have exclusive control of the values.

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
```

_Note: the use of the `#[derive(Clone)]` trait macro to ensure that `Config` is Cloneable._

The main reasons Clone’ing is an appropriate strategy here are because:

The value is cheap to copy (just a small struct of primitives)

The intended goal of the program is to ensure that the copies of the value can be independently modified (or preserved).

Therefore, by definition, we must have multiple copies of the data structure to ensure that their versions can be treated independently.

We would not be able to do this with mutable borrowing or reference counting because any mutation would be seen by all references and therefore undesirable or disallowed. We cannot move the value because then the original value would be lost.

## Borrow Most Things, Move Some Things

Use when:

- most objects require only temporary access

- a subset of objects need to be consumed permanently

- a subset of objects are expensive to Clone

- a subset of objects are only required by a single function or subprocess

Similar to the prior scenario where the callee requires ownership of the value but for some reason Clone’ing is not an option.

This could be for multiple reasons but the most obvious one is if Clone’ing would be too expensive.

In this case, you may still be borrowing most things, but you specifically identify the object that needs to move and pass it by value to the callee such that ownership transfers away from the caller and into the callee.

From there, either:

- The callee eventually passes ownership back to the caller either as the same value or a derivative value

- The callee drops the object after it goes out of scope and it is never seen nor heard from again

In either case, the caller may not reference the object after moving it to the callee. If the callee returns ownership to the caller, the caller may reference the returned value as a new binding. The original binding is no longer valid after a move.

Notably, at the end of the program, Frame #0 is left still owning Y, but no longer owns X because X was moved to Frame #9 and never returned as a new binding. Frame #0 will never know what happened to X :(

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
```

This example shows a situation where the `Config` object is now too expensive to Clone due to the extremely large data structure (giant vector) it contains.

Therefore, when we decide we want to make a versioned representation of it, we allow the versioning function to take exclusive ownership of the value (consume it) and return a new value in its place that we can take new ownership of.

This allows the `save_config_version` function to perform its work, but without the expense of having to provide it with its very own copy of the data structure.

Now obviously if we wanted to make a second version of this `Config` we would face this problem again. The solution to that problem is more sophisticated and beyond the scope of this guide.

## Move All the Things

Use when:

- objects are expensive to Clone

- the inner scope may outlive the outer scope

This covers the case where the program requires that you move all of the values because Clone’ing is too expensive and the scope doing the work may outlive the scope of the caller.

A real example is having two tasks in your program, one that reads data from a socket, and one that processes the data in a longer running computation.

The task reading data from the socket could theoretically pass references to the tasks processing the data, but there is a problem with that. If the task processing the data takes longer to complete than the task reading data from the buffer, the processing task will outlive the buffer reading task.

This is a problem because if the buffer reading task owns the data and it goes out of scope before the processing task, then the processing task is now holding a reference to… nothing. Rust does not allow this. More specifically the borrow checker will detect this possibility and inform you.

This is a situation where you have to move the value from the task that reads the data to the task that processes the data. Often this is expressed as a closure created with the `move` or `async move` keywords.


Here we see the major difference being that ownership is transferred from Task #0 to Task #1 but at the end of execution, Task #0 no longer exists. Task #1 has outlived Task #0 and retains sole ownership of X. Had Task #1 attempted to borrow X, it would be impossible to guarantee the reference because Task #0 would have terminated before Task #1. The borrow-checker will not allow this.

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
    // it was permanently moved into the `spawn`'d closure
}
```

Here we see that the outer scope `main` representing Task #0 will create a `client` and move it to be owned by Task #1. Since Task #1 is async and `main` does not block, the `main` scope will be dropped immediately after starting Task #1.

Task #1 can continue to run until completion as it has taken ownership of the `client` needed to perform its work.

## Reference Count Certain Things

Use when:

- objects are expensive to clone

- multiple threads or processes must access the same references concurrently

- moving and returning ownership is prohibitively complex or precluded by concurrent access requirements

- you are already using locks (Mutex) for atomic mutation

This is the most complex case, where we want to neither move nor implicitly borrow. One can think of this approach as a more explicit form of borrowing where we are tracking each reference holder to an object explicitly using a counter that goes up when new references are taken and down when they are dropped.

This has the effect of giving us a little bit of the best of both worlds, with some complexity as a drawback. We get the efficiency of borrowing in that objects that are expensive to Clone are not actually cloned.

The two main reference counted types in Rust are Rc<T> and Arc<T>. Rc means “reference counted” and Arc means “automatic reference counted”. Calling .clone() on an Rc<T> or Arc<T> does not actually clone the inner value, but rather creates a smart pointer to the inner value that has been allocated on the heap.

From the perspective of the borrow-checker, the receiver of a Clone’d reference counted type has an “owned” copy of the value because the lifetime of the inner value has been moved outside the stack and onto the heap. The value will remain on the heap until the last reference is dropped thereby ensuring that even if earlier references are dropped, later references will still be valid for as long as needed.

Reference counted type Clones are still shared references — the same way a borrow is — but allow the semantics of an owned value without the overhead of actually copying the value.

It may seem like this should always be the best option (best of both worlds right?) — but in practice the extra boilerplate and complexity is only worth it if the more basic starting strategies prove insufficient.

Reserve this approach for when the lifetimes of your stack frames (scopes) are non-linear and moving or cloning bits is not appropriate for control flow or performance reasons.

Note: reference counting is not without pitfalls if done improperly.

Let’s look at when the use of a reference counted type would be the most appropriate solution.

Here we see that Arc::new(X) moves the value from the stack to the heap. Then when clone() is called on the value by subsequent frames, they receive a pointer to the heap location of X and the reference counter goes up or down as references are taken or dropped.

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
```


We will not cover the mutation case here as that will require introducing Mutex. Stay tuned for more in an upcoming article.

**Wrapping Up**

This guide provides you with a practical framework for planning out your lifetime management approach when designing programs in Rust.

**Remember: defaulting to immutable borrowing is usually a good starting point.**

Values that will be **destroyed**, **returned**, or **irreversibly** transformed are probably good candidates for **moving** or **cloning**.

Values that will be used exclusively by a scope that outlives their original scope must be moved into the longer living scope so they can exist after the original scope is dropped.

If performance constraints are bound and concurrent ownership or mutation is required, reference counting is probably the way to go.

A note on lifetime annotations: this guide does not cover lifetime annotations (<‘a>) because they do not provide actual control over the lifetimes of references at runtime. They are hints to the Rust compiler (borrow checker) to disambiguate certain type signatures where the compiler cannot easily infer the intention of the programmer for which references do and do not share lifetimes.

We will do a separate piece on lifetime annotations covering when they are necessary and when you can avoid having to use them altogether.
