# reaclib-rs

A parsing library for the [reaclib] data format.

The data is represented by `Set`, and the parsing is mostly done by `Iter`.
The data can be collected into a type that implements `FromIterator`, such as `Vec`.
A convenience function `to_hash_map` is provided for the case that you want a `Vec` of all
`Set`s for each reaction.

[reaclib]: https://reaclib.jinaweb.org/

## Format

The format is documented on the [reaclib format help page][reaclib_format].
There are two formats, both supported by this library.
`Format` is used to indicate which version to expect.

[reaclib_format]: https://reaclib.jinaweb.org/help.php?topic=reaclib_format

## Examples

```rust
use reaclib::{Format, Iter, Set};
use std::{fs::File, io::BufReader};

let file = File::open("reaclib")?;
let file = BufReader::new(file);
let iter = Iter::new(file, Format::Reaclib1);
let data: Vec<Set> = iter.collect::<Result<_, _>>()?;
```

```rust
use reaclib::{Format, Reaction, Set, to_hash_map};
use std::{collections::HashMap, io::stdin};

let input = stdin().lock();
let data: HashMap<Reaction, Vec<Set>> = to_hash_map(input, Format::Reaclib2)?;
```

## Features

* `serde`: Provide `Serialize` and `Deserialize` implementations for [serde](https://serde.rs).

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
