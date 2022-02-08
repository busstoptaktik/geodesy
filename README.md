# Geodesy

**NOTE:** After a recent dramatic restructuring (and more to come) the *Ruminations* section is not yet in accordance with the actual situation. Take a look in the *Examples* section if anything appears fishy.

*Rust Geodesy* (RG), is a platform for experiments with geodetic software, transformations, and standards. *RG* vaguely resembles the [PROJ](https://proj.org) transformation system, and was built in part on the basis of experiments with alternative data flow models for PROJ. The actual transformation functionality of *RG* is, however, minimal: At time of writing, it includes just a few low level operations, including:

- The three, six, seven, and fourteen-parameter versions of the *Helmert transformation*
- Helmert's companion, the *cartesian/geographic* coordinate conversion
- The full and abridged versions of the *Molodensky transformation*
- Three widely used conformal projections: The *Mercator*, the *Transverse Mercator*, and the *Lambert Conformal Conic* projection
- The *Adapt* operator, which mediates between various conventions for coordinate units and order

While this is sufficient to test the architecture, it is very far from enough to get through a geodesist's daily work.

Hence, viewing *RG* as *another PROJ*, or *PROJ [RiiR](https://acronyms.thefreedictionary.com/RIIR)*, will lead to bad disappointment. At best, you may catch a weak mirage of a *potential* [shape of jazz to come](https://en.wikipedia.org/wiki/The_Shape_of_Jazz_to_Come) for the PROJ internal dataflow.

## Aims
Dataflow experimentation is just one aspect of *RG*. Overall, the aims are fourfold:

1. Support experiments for evolution of geodetic standards.
2. Support development of geodetic transformations.
3. Hence, provide easy access to a number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions. Mostly as a tool for aims (1, 2, 3)

All four aims are guided by a wish to amend explicitly identified shortcomings in the existing geodetic system landscape.

## Documentation

The documentation is currently limited, but take a look at:

- The coordinate operator [documentation](/ruminations/002-rumination.md)
- The [description of `kp`](/ruminations/003-rumination.md), the *Rust Geodesy* coordinate processing program
- This essayistic [rumination](/ruminations/000-rumination.md), outlining the overall philosophy and architecture of *Rust Geodesy*.
- The API documentation at [Docs.rs](https://docs.rs/geodesy)
- The `examples`
- The tests embedded in the [source code](/src/)

## License

*Rust Geodesy*: Copyright 2020, 2021 by Thomas Knudsen <knudsen.thomas@gmail.com>.

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or [here](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or [here](http://opensource.org/licenses/MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
