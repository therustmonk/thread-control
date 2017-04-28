# Thread-control library

Missing Rust execution control tools for **[threads](https://doc.rust-lang.org/std/thread/)**
and **[futures](https://github.com/alexcrichton/futures-rs)**.

## Threads example

```rust
use std::thread;
use thread_control::*;

fn main() {
    let (flag, control) = make_pair();
    let handle = thread::spawn(move || {
        while flag.alive() {
        }
    });
    assert_eq!(control.is_done(), false);
    control.stop(); // Also you can `control.interrupt()` it
    handle.join();
    assert_eq!(control.is_interrupted(), false);
    assert_eq!(control.is_done(), true);
}
```

## Futures example

```rust
let (flag, control) = thread_control::make_pair();

let duration = Duration::from_secs(5);
let alive_checker = Interval::new(duration, &handle).unwrap()
    .and_then(move |value| {
        if flag.is_alive() {
            Ok(value)
        } else {
            Err(other("stream was interrupted by thread control!"))
        }
    });

thread::spawn(move || {
    // Any routine with `control.is_done()` checking
});

let managed_routine = alive_checker.select(mystream).for_each(|_| Ok(()));
core.run(managed_routine).unwrap();
```

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
