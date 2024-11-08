# Geodesy

## Abstract

**Rust Geodesy** is - unsurprisingly - a geodesy library written in the Rust programming language.

Rust Geodesy provides a number of **features** to support a number of **objectives**.

The most important **features** are

- a set of more than 30 geodetic **transformation primitives**
- a set of more than 40 primitives for **operations on the ellipsoid**
- a means for **composing** these primitives into more complex operations.

The most important **objectives** are

- to support new, and hopefully better, abstractions,
- to use these abstractions to build better, simpler, and more tractable,
  geospatial **standards, transformations, and software**.

If any of this resonates with you, read on after this minimal usage example...

## Usage

Initialize a new project, using Geodesy:

```console
$ cargo new foo
     Created binary (application) `foo` package

$ cd foo
$ cargo add geodesy
```

Then overwrite the contents of the `foo/src/main.rs` file with this:
A minimal example, computing the UTM coordinates of some Scandinavian capitals

```rust
use geodesy::prelude::*;

fn main() -> Result<(), Box<Error>> {
    let mut context = Minimal::new();
    let utm33 = context.op("utm zone=33")?;

    let cph = Coor2D::geo(55., 12.); // Copenhagen
    let sth = Coor2D::geo(59., 18.); // Stockholm
    let mut data = [cph, sth];

    context.apply(utm33, Fwd, &mut data)?;
    println!("{:?}", data);
    Ok(())
}
```

and try it out:

```console
$ cargo r
    Finished dev [unoptimized + debuginfo] target(s) in 0.11s
     Running `C:\FLOW\AD\RG\foo\target\debug\foo.exe`

[Coor2D([308124.36786033923, 6098907.825005002]), Coor2D([672319.9640879404, 6543920.334127973])]
```

## Concrete

*Rust Geodesy* (RG), is a platform for experiments with geodetic software, transformations, and standards. *RG* vaguely resembles the [PROJ](https://proj.org) transformation system, and was built in part on the basis of experiments with *alternative data flow models for PROJ*. The fundamental **transformation** functionality of *RG* is fairly complete (i.e. on par with the datum shift/reference frame transformation capability of PROJ), while the number of **projections** supported is a far cry from PROJ's enormous gamut. It does, however, support a suite of the most important ones:

- Transverse Mercator
- Universal Transverse Mercator (UTM)
- Web Mercator
- Mercator
- Oblique Mercator
- Lambert Conformal Conic
- Lambert Azimuthal Equal Area

But fundamentally, *RG* is born as a *geodesy*, rather than
a *cartography* library. And while PROJ benefits from four
decades of *reality hardening*, RG, being a platform for experiments,
does not have operational robustness as a main focus.
Hence, viewing *RG* as *another PROJ*, or
*PROJ [RiiR](https://acronyms.thefreedictionary.com/RIIR)*,
will lead to bad disappointment.
At best, you may catch a weak mirage of a *potential*
[shape of jazz to come](https://en.wikipedia.org/wiki/The_Shape_of_Jazz_to_Come)
for the PROJ internal dataflow.

That said, being written in Rust, with all the memory safety guarantees Rust provides,
*RG* by design avoids a number of pitfalls that are explicitly worked
around in the PROJ code base. So the miniscule size of *RG* compared to
PROJ is not just a matter of functional pruning. It is also a matter of
development using a tool wonderfully suited for the task at hand.

Also, having the advantage of learning from PROJ experience, both from
a user's and a developer's perspective, *RG* is designed to be
significantly more extensible than PROJ. So perhaps for a number of
applications, and despite its limitations, RG may be sufficient, and
perhaps even useful.

## Aims

Dataflow experimentation is just one aspect of *RG*. Overall, the aims are (at least) fourfold:

1. Support experiments for evolution of geodetic standards.
2. Support development of geodetic transformations.
3. Hence, provide easy access to a number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions. Mostly as a tool for aims (1, 2, 3)

All four aims are guided by a wish to amend explicitly identified
shortcomings in the existing geodetic system landscape.

## Documentation

The documentation is currently limited, but take a look at:

- The coordinate operator [documentation](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/002-rumination.md)
- The [description of `kp`](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/003-rumination.md), the *Rust Geodesy* coordinate processing program
- This essayistic [rumination](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/000-rumination.md), outlining the overall philosophy and architecture of *Rust Geodesy*, and [this related](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/008-rumination.md) comparison between PROJ and RG
- The API documentation at [Docs.rs](https://docs.rs/geodesy)
- The [`examples`](https://github.com/busstoptaktik/geodesy/tree/main/examples)
- The tests embedded in the [source code](https://github.com/busstoptaktik/geodesy/tree/main/src)
- [This](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/006-rumination.md) rather concrete and [this](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/005-rumination.md) more philosophical description of the main discrepancy between geodesy and geomatics, *RG* tries to elucidate and amend.

## License

*Rust Geodesy*: Copyright 2020, 2021, 2022, 2023, 2024 by
Thomas Knudsen <knudsen.thomas@gmail.com> and contributors.

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or [here](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or [here](http://opensource.org/licenses/MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
