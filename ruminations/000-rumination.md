# Ruminations on Rust Geodesy

## Rumination 000: Overall architecture and philosophy

Thomas Knudsen <knudsen.thomas@gmail.com>

2021-07-31. Last [revision](#document-history) 2022-03-24

### Abstract

```rust
let utm32 = ctx.op("utm zone=32").unwrap();
ctx.apply(utm32, Fwd, &mut data);
println!({:?}, data);
ctx.apply(utm32, Inv, &mut data);
println!({:?}, data);
```

---

### Prologue

#### What is Rust Geodesy?

Rust Geodesy, RG, is a geodetic software system, not entirely unlike [PROJ](https://proj.org), but with much more limited transformation functionality: While PROJ is mature, well supported, well tested, and production ready, RG is neither of these. This is partially due to RG being a new born baby, partially due to its aiming at a (much) different set of use cases.

So when I liberally insert comparisons with PROJ in the following, it is for elucidation, not for mocking - neither of PROJ, nor of RG: I have spent much pleasant and instructive time with PROJ, both as a PROJ core developer and as a PROJ user (more about that in an upcomming *Rumination on RG*). But I have also spent much pleasant time learning Rust and developing RG, so I feel deeply connected to both PROJ and RG.

PROJ and RG do, however, belong in two different niches of the geodetic software ecosystem: Where PROJ is the production work horse, with the broad community of end users and developers, RG aims at a much more narrow community of geodesists, for geodetic development work - e.g. for development of transformations that may eventually end up in PROJ. As stated in the [README](/README.md)-file, RG aims to:

1. Support experiments for evolution of geodetic standards.
2. Support development of geodetic transformations.
3. Hence, provide easy access to a number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions. Mostly as a tool for aims {1, 2, 3}

#### Why Rust Geodesy?

The motivation for these aims, i.e. the **why** of the project, is the **wish to amend explicitly identified shortcommings** in the existing landscape of geodetic software and standards.

#### How will it emerge?

The development work driven by this motivation is supported by a few basic design principles, the **how** of the project:

- An architectural scaffolding of four dimensional data flow paths, enabling the constrution of complex operations from simpler elements
- A design philosophy of keeping things flexible by not overspecifying
- An architectural focus on run-time extensibility and adaptability
- A geodetic focus on transformations, i.e. relations *between* systems, rather than definition *of* systems

or in fewer words: *Don't overdo it*.

### A deep dive

Talking architecture and design philosophy out of thin air is at best counterproductive, so let's start with a brief example, demonstrating the RG idiom for converting geographical coordinates to UTM zone 32 coordinates (for the corresponding operation using the RG coordinate processing command line program `kp`, see [Rumination 003](/ruminations/003-rumination.md)).

```rust
fn main() {
    // [0] Conventional shorthand for accessing the major functionality
    use geodesy::preamble::*;

    // [1] Build some context
    let mut ctx = Minimal::default();

    // [2] Obtain a handle to the utm-operator
    let utm32 = ctx.op("utm zone=32").unwrap();

    // [3] Coordinates of some Scandinavian capitals
    let copenhagen = Coord::geo(55., 12., 0., 0.);
    let stockholm  = Coord::geo(59., 18., 0., 0.);

    // [4] Put the coordinates into an array
    let mut data = [copenhagen, stockholm];

    // [5] Then do the forward conversion, i.e. geo -> utm
    ctx.apply(utm32, Fwd, &mut data);
    for coord in data {
        println!({:?}, coord);
    }

    // [6] And go back, i.e. utm -> geo
    ctx.apply(utm32, Inv, &mut data);
    for coord in data {
        println!({:?}, Coord::to_geo(coord));
    }
}
```

At comment `[0]`, we obtain access to the most important data types and functionality of Rust Geodesy.

---

```rust
// [1] Build some context
let mut ctx = Minimal::default();
```

At comment `[1]` we instantiate a context `Provider` which, to [PROJ](https:://proj.org) users, will look much like the PROJ `PJ_CONTEXT`object: The `Provider` provides the interface to the messy world external to RG (files, threads, communication, geodetic constants and transformation definitions), and in general centralizes all the *mutable state* of the system.

Also, and contrary to the PROJ `PJ_CONTEXT` case, the `Provider` is in general the sole interface between the `RG` transformation functionality and the application program: You may instantiate a transformation object, but the `Provider` handles it for you. And while you need a separate `Provider` for each thread of your program, the `Provider` itself is designed to eventually do its work in parallel, using several threads.

Note that although RG provides `Minimal`, and a few other built-in `Provider`s, a `Provider` really is anything implementing the [`Provider`](/src/provider/mod.rs) trait, i.e. not necessarily an integral part of RG.

The intention with this is to make it possible for an application program to supply a `Provider`, providing access to external resources **in precisely the form they are available in the execution environment of the application program**.

So forget about discussions on whether transformation definitions should be read from a local SQLite file database, a conection to an external database, or from a local text file: These discussions can be laid to rest simply by providing a `Provider` accessing resources in whichever form is most convenient for the case at hand.

---

```rust
// [2] Obtain a handle to the utm-operator
let utm32 = ctx.op("utm zone=32").unwrap();
```

At comment `[2]`, we use the `op`(eration) method of the `Provider` to instantiate an `Op`erator (closely corresponding to the `PJ` object in PROJ).

The parametrisation of the operator, i.e. the text `utm zone=32` uses parameter naming conventions closely corresponding to those used in PROJ, where the same operator would be described as `proj=utm zone=32`
(see also [ellps implied](#note-ellps-implied) in the Notes section).

PROJ has a *de facto* convention that **the first** element in a definition string should identify the name of the operator being instantiated. RG *formalizes* this convention, and hence does not need the `proj=` part, since by definition (and not just by convention) the first element *is* the operator identification.

Through the evolution of PROJ from a projection library to a generic transformation library, the `proj=` part has become slightly confusing, since it is used to identify not just `proj`ections, but any kind of geodetic operations. RG, being born as a generic geodetic library, eliminates this potential confusion.

Note, however, that contrary to PROJ, when instantiating an operator in RG, you do not actually get an `Op`erator object back, but just a handle to an `Op`erator - an `OpHandle` (which is actually just a weakly disguised UUID). The `Op` itself lives its entire life embedded inside the `Provider`. And while the `Provider` is mutable, the `Operator`, once created, is *immutable*.

This makes `Operator`s thread-sharable, so the `Provider` will eventually (although still not implemented), be able to automatically parallelize large transformation jobs, eliminating some of the need for separate thread handling at the application program level.

**A note on naming:** The method for instantiating an `Op`erator is called `Provider::op(...)`. This eliminates the need to discern between operators and operations: Conceptually, an **operation** is an *instantiation of an operator*, i.e. an operator with parameters fixed, and ready for work. An **operator** on the other hand, is formally a datatype, i.e. just a description of a memory layout of the parameters. At the API level we don't care, and hence use the abbreviation `op(...)`, which returns a handle to an **operation**, which can be used to **operate** on a set of **operands**. It's op...s all the way down!

---

```rust
// [3] Coordinates of some Scandinavian capitals
let copenhagen = Coord::geo(55., 12., 0., 0.);
let stockholm  = Coord::geo(59., 18., 0., 0.);

// [4] We put the coordinates into an array
let mut data = [copenhagen, stockholm];
```

At comments `[3]` and `[4]` we produce the input data we want to transform. The RG coordinate type is called `Coord`, and covers more or less everything called a `CoordinateTuple` according to the ISO-19111 standard. In other words, anything we would colloquially call *the coordinates* of something.

Internally, RG represents angular coordinates in radians, and follows the traditional GIS coordinate order of *longitude before latitude*. Externally, however, you may pick-and-choose. In this case, we choose human readable angles in degrees, and the traditional coordinate order used in geodesy and navigation: *latitude before longitude*. The `Coord::geo(...)` function translates that into the internal representation. It has siblings `Coord::gis(...)` and `Coord::raw(...)` which handles GIS coordinate order and raw numbers, respectively. The latter is useful for projected coordinates, cartesian coordinates, and for coordinates with angles in radians. We may also simply give a `Coord` as a naked array of four double precision floating point numbers:

```rust
let somewhere = Coord([1., 42., 3., 4.]);
```

The `Coord` data type does not enforce any special interpretation of what kind of coordinate it stores: That is entirely up to the `Op`erator to interpret. A `Coord` simply consists of 4 numbers with no other implied interpretation than their relative order, given by the names *first, second, third, and fourth*, respectively.

RG `Op`erators take *arrays of `Coord`* as input, rather than individual elements, so at comment `[4]` we collect the elements into an array.

---

```rust
// [5] Then do the forward conversion, i.e. geo -> utm
ctx.apply(utm32, Fwd, &mut data);
println!({:?}, data);
```

At comment `[5]`, we do the actual forward conversion to utm coordinates. Behind the scenes, `ctx.apply(...)` is free to split up the input array into chunks, for parallel processing in a number of threads, although in practice this is not yet actually implemented.

As the action goes on *in place*, we allow `apply(..)` to mutate the input data, by using the `&mut`-operator in the method call.

The printout will show the projected data in (easting, northing)-coordinate order:

```rust
Coord([ 691875.6321403517, 6098907.825001632, 0.0, 0.0])
Coord([1016066.6135867655, 6574904.395327058, 0.0, 0.0])
```

---

```rust
// [6] And go back, i.e. utm -> geo
ctx.apply(utm32, Inv, &mut data);
println!({:?}, Coord::to_geo(data));
```

At comment `[6]`, we roundtrip back to geographical coordinates. During print out, we let `Coord::to_geo(...)` convert from the internal coordinate representation, to the geodetic convention of "latitude before longitude, and angles in degrees".

### Simplified pipeline syntax

In PROJ, the syntax for pipelines was bolted on to an existing specification syntax, utilizing whichever left-over holes in the syntax-wall that would accomodate its bolts (take my word for it: I am the one to blame). That led to a somewhat verbose syntax (more nuts than bolts, perhaps), as seen from this example, sandwiching a Helmert shift between conversions from geographical to cartesian coordinates, and back:

```js
proj=pipeline
    step proj=cart ellps=intl
    step proj=helmert x=-87 y=-96 z=-120
    step proj=cart inv ellps=GRS80
```

In RG, knowing up front how useful operator pipelines are, we let them play a much more prominent role: In RG, everything is a pipeline, even if there is only a single step in that pipeline. So we leave out the `proj=pipeline` part of the syntax: When everything really is a pipeline, we do not need to state that fact explicitly in every single definition.

Also, since pipelines are obviously akin to Unix style shell pipes, we use the Unix shell syntax, i.e. the vertical bar character `|`, to separate the steps of the pipeline. All in all this results in a compact, and very readable pipeline definition syntax:

```js
cart ellps=intl | helmert x=-87 y=-96 z=-120 | cart inv ellps=GRS80
```

### Redefining the world

Being intended for authoring of geodetic functionality, customization is a very important aspect of the RG design. Hence, RG allows temporal overshadowing of built in functionality by registering user defined macros and operators. This is treated in detail in examples [02 (macros)](/examples/02-user_defined_macros.rs) and [03 (operators)](/examples/03-user_defined_operators.rs). Here, let's just take a minimal look at the workflow, which can be described briefly as *define, register, instantiate, and use:*

First a macro:

```rust
// Define a macro, using "look-up" notation (^) for the macro parameters
let macro_text = "cart ellps=^ellps_0 | helmert: x=^x y=^y z=^z |  cart inv ellps=^ellps_1";

// Register the macro, under the name "geo:helmert"
ctx.register_macro("geo:helmert", macro_text);

// Instantiate the geo:helmert macro with replacement values
// for the parameters left, right, x, y, z
ed50_wgs84 = ctx.op("geo:helmert ellps_0=intl ellps_1=GRS80 x=-87 y=-96 z=-120").unwrap();
// Note that the syntax is identical to that used for built-in operators

// ... and use:
ctx.apply(ed50_wgs84, Fwd, data);
```

Then a user defined operator:

```rust
use geodesy::operator_construction::*;

// See examples/03-user-defined-operators.rs for implementation details
pub struct MyNewOperator {
    args: OperatorArgs,
    foo: f64,
    ...
}

// Register
ctx.register_op("my_new_operator", MyNewOperator::operator);

// Instantiate
let my_new_operator_with_foo_as_42 = ctx.operation(
    "my_new_operator foo=42"
).unwrap();

// ... and use:
ctx.apply(my_new_operator_with_foo_as_42, Fwd, data);
```

Essentially, once they are registered, macros and user defined operators work exactly like the built-ins.

Also, the user defined operators overshadow the built-in names for any subsequent instantiations. So testing alternative implementations of built-in operators is as easy as registering a new operator with the same name as a built-in.

By design, however, macros cannot overshadow built-ins: To trigger the macro argument expansion mechanism, macros need to indicate their macro-identity by including a `:` in their name, in contrast to the built-in names.

### Going ellipsoidal

Much functionality related to geometrical geodesy can be associated with the ellipsoid model in use, and hence, in a software context, be modelled as methods on the ellipsoid object.

In RG, ellipsoids are represented by the `Ellipsoid` data type:

```rust
pub struct Ellipsoid {
    a: f64,
    ay: f64,
    f: f64,
}
```

In most cases, the ellipsoid in use will be rotationally symmetrical, but RG anticipates the use of triaxial ellipsoids. As can be seen, the `Ellipsoid` data type is highly restricted, containing only the bare essentials for defining the ellipsoidal size and shape. All other items are implemented as methods:

```rust
let GRS80 = geodesy::Ellipsoid::named("GRS80");

let E = GRS80.linear_eccentricity();
let b = GRS80.semiminor_axis();
let c = GRS80.polar_radius_of_curvature();
let n = GRS80.third_flattening();
let es = GRS80.eccentricity_squared();
```

The functionality also includes ancillary latitudes, and computation of geodesics on the ellipsoid - see [example 01](../examples/01-geometrical-geodesy.rs) for details.

### Comming attractions

RG is in early-stage development, so a number of additions are planned.

#### Geometric geodesy

In `[Knudsen et al, 2019]` we identified a small number of operations collectively considered the "bare minimum requirements for a geodetic transformation system":

1. Geodetic-to-Cartesian coordinate conversion, and its inverse.
2. Helmert transformations of various kinds (2D, 3D, 4D or, equivalently: 4 parameter, 3/7 parameter and 14/15 parameter).
3. The Molodensky transformation.
4. Horizontal grid shift (“NADCON-transformation”).
5. Vertical grid shift (ellipsoidal-to-orthometric height transformation).

Of these only the three first are fully implemented in RG, while the grid shift operations are in various stages of completion. These are **need to do** elements for near future work.

Also, a number of additional projections are in the pipeline.

#### Physical geodesy

Plans for invading the domain of physical geodesy are limited, although the `Ellipsoid` data type will probably very soon be extended with entries for the *International Gravity Formula, 1930* and the *GRS80 gravity formula*.

#### Coordinate descriptors

Combining the generic `Coord`s with `CoordinateDescriptor`s will make it possible to sanity check pipelines, and automate coordinate order and unit conversions.

#### Logging

The Rust ecosystem includes excellent logging facilities, utilized, but still under-utilized in RG.

### Discussion

From the detailed walkthrough of the example above, we can summarize "the philosophy of RG" as:

- **Be flexible:** User defined macros and operators are first class citizens in the RG ecosystem - they are treated exactly as the built-ins, and hence, can be used as vehicles for implementation of new built-in functionality. Even the `Provider`, i.e. the interface between the RG internals and external resources, can be swapped out with a user supplied/application specific one.
- **Don't overspecify:** For example, the `Coord` object is just an ordered set of four numbers, with no specific interpretation implied. It works as a shuttle, ferrying the operand between the steps of a pipeline of `Op`erators: The meaning of the operand is entirely up to the `Op`erator.
- **Transformations are important. Systems not so much:** RG does not anywhere refer explicitly to input or output system names. Although it can be used to construct transformations between specific reference frames (as in the "ED50 to WGS84" case, in the *user defined macro* example), it doesn't really attribute any meaning to these internally.
- **Coordinates and data flow pathways are four dimensional:** From end to end, data runs through RG along 4D pathways. Since all geodata capture today is either directly or indirectly based on GNSS, the coordinates are inherently four dimensional. And while much application software ignores this fact, embracing it is the only way to obtain even decimeter accuracy over time scales of just a few years. Contemporary coordinate handling software should never ignore this.
- **Draw inspiration from good role models, but not zealously:** PROJ and the ISO-19100 series of geospatial standards are important models for the design of RG, but on the other hand, RG is also built to address and investigate some perceived shortcomings in the role models.

... and, although only sparsely touched upon above:

- **Operator pipelines are awesome:** Perhaps not a surprising stance, since I invented the concept and implemented it in PROJ five years ago, through the [Plumbing for Pipelines](https://github.com/OSGeo/PROJ/pull/453) pull request.

While operator pipelines superficically look like the ISO-19100 series concept of *concatenated operations*, they are more general and as we pointed out in `[Knudsen et al, 2019]`, also very powerful as a system of bricks and mortar for the construction of new conceptual buildings. Use more pipelines!

### Conclusion

Rust Geodesy is a new, still functionally limited, system for experimentation with, and authoring of, new geodetic transformations, concepts, algorithms and standards. Go get it while it's hot!

### References

**Reference:** `[Knudsen et al, 2019]`

Thomas Knudsen, Kristian Evers, Geir Arne Hjelle, Guðmundur Valsson, Martin Lidberg and Pasi Häkli: *The Bricks and Mortar for Contemporary Reimplementation of Legacy Nordic Transformations*. Geophysica (2019), 54(1), 107–116.

### Notes

#### **Note:** ellps implied

In both cases, the use of the GRS80 ellipsoid is implied, but may be expressly stated as  `utm zone=32 ellps=GRS80}` resp. `proj=utm zone=32 ellps=GRS80`

#### **Note:** Idiomatic Rust

In production, we would check the return of `ctx.op(...)`, rather than just `unwrap()`ping:

```rust
if let Some(utm32) = ctx.op("utm: {zone: 32}") {
    let copenhagen = C::geo(55., 12., 0., 0.);
    let stockholm = C::geo(59., 18., 0., 0.);
    ...
}
```

In C, using PROJ, the demo program would resemble this (untested) snippet:

```CPP
#include <proj.h>

#int main() {
    PJ_CONTEXT *C = proj_context_create();
    PJ *P = proj_create(C, "proj=utm zone=32");

    PJ_COORD copenhagen = proj_coord(12, 55, 0, 0);
    PJ_COORD stockholm = proj_coord(18, 59, 0, 0);

    /* Forward */
    copenhagen = proj_trans(P, PJ_FWD, copenhagen);
    stockholm = proj_trans(P, PJ_FWD, stockholm);

    /* ... and back */
    copenhagen = proj_trans(P, PJ_INV, copenhagen);
    stockholm = proj_trans(P, PJ_INV, stockholm);

    proj_destroy(P);
    proj_context_destroy(C);
}
```

### Document History

Major revisions and additions:

- 2021-08-08: Added a section briefly describing GYS
- 2021-08-26: Extended prologue
- 2022-03-24: Total rewrite
