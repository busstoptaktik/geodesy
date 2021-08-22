# Geodesy

*Rust Geodesy* (RG), is a platform for experiments with geodetic software, transformations, and standards. *RG* vaguely resembles the [PROJ](https://proj.org) transformation system, and was built in part on the basis of experiments with alternative data flow models for PROJ. The actual transformation functionality of *RG* is, however, minimal: At time of writing, it includes just 4 low level operations:

1. The three, six, seven, and fourteen-parameter versions of the *Helmert transformation*
2. Helmert's companion, the *cartesian/geographic* coordinate conversion
3. The full and abridged versions of the *Molodensky transformation*
4. The *transverse mercator* projection

While this is sufficient to test the architecture, it is very far from enough to get through a geodesist's daily work.

Hence, viewing *RG* as *another PROJ*, or *PROJ [RiiR](https://acronyms.thefreedictionary.com/RIIR)*, will lead to bad disappointment. At best, you may catch a weak mirage of a *potential* [shape of jazz to come](https://en.wikipedia.org/wiki/The_Shape_of_Jazz_to_Come) for the PROJ internal dataflow.

But dataflow experimentation is just one aspect of *RG*. Overall, the aims are fourfold:

1. Support experiments for evolution of geodetic standards.
2. Support development of geodetic transformations.
3. Hence, provide easy access to a number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions. Mostly as a tool for aims (1, 2, 3)

All four aims are guided by a wish to amend explicitly identified shortcomings in the existing geodetic system landscape.

## Documentation

The documentation is currently very limited, but take a look at:

- The coordinate operator [documentation](/comments/002-comment.md)
- The API documentation at [Docs.rs](https://docs.rs/geodesy)
- The [examples](examples)
- The tests embedded in the [source code](/src/)
- This essayistic [comment](/comments/000-comment.md), outlining the overall philosophy and architecture of *Rust Geodesy*.

## License

*Rust Geodesy*: Copyright 2020, 2021 by Thomas Knudsen <knudsen.thomas@gmail.com>.

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or [here](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or [here](http://opensource.org/licenses/MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
