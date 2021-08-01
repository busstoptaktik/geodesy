# Comments on Rust Geodesy

## Issue 000: Overall architecture and philosophy

Thomas Knudsen <knudsen.thomas@gmail.com>

2021-07-31

---

### Prologue

Rust Geodesy, RG, is a geodetic software system, not entirely unlike [PROJ](https://proj.org), but with much more limited transformation functionality. And while PROJ is mature, well supported, well tested, and production ready, RG is neither of these. Partially due to RG being a new born baby, partially due to its aiming at a (much) different set of use cases.

So when I liberally insert comparisons with PROJ in the following, it is for elucidation, not for mocking, neither of PROJ, nor of RG: I have spent much pleasant and instructive time with PROJ, both as a PROJ core developer and as a PROJ user (more about that in an upcomming *Comment on RG*). But I have also spent much pleasant time learning Rust and developing RG, so I feel deeply connected to both PROJ and RG.

PROJ and RG do, however, belong in two different niches of the geodetic software ecosystem: Where PROJ is the production work horse, with the broad community of end users and developers, RG aims at a much more narrow community of geodesists, for geodetic development work - e.g. for development of transformations that may eventually end up in PROJ. As stated in the [README](../README.md)-file, RG aims to:

1. Support experiments for evolution of geodetic standards.
2. Support development of geodetic transformations.
3. Hence, provide easy access to a number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions. Mostly as a tool for aims (1, 2, and 3)

All four aims are guided by a wish to amend explicitly identified shortcomings in the existing geodetic system landscape.

### Getting beefy

Talking architecture and design philosophy out of thin air is at best counterproductive, so let's start with a brief example, demonstrating the Rust Geodesy idiom for converting geographical coordinates to UTM zone 32 coordinates.

```rust
fn main() {
    // [0] Use a brief name for some much used functionality
    use geodesy::CoordinateTuple as Coord;

    // [1] Build some context
    let mut ctx = geodesy::Context::new();

    // [2] Obtain a handle to the utm-operator
    let utm32 = ctx.operator("utm: {zone: 32}").unwrap();

    // [3] Coordinates of some Scandinavian capitals
    let copenhagen = Coord::geo(55., 12., 0., 0.);
    let stockholm  = Coord::geo(59., 18., 0., 0.);

    // [4] We put the coordinates into an array
    let mut data = [copenhagen, stockholm];

    // [5] Then do the forward conversion, i.e. geo -> utm
    ctx.fwd(utm32, &mut data);
    println!({:?}, data);

    // [6] And go back, i.e. utm -> geo
    ctx.inv(utm32, &mut data);
    Coord::geo_all(&mut data);
    println!({:?}, data);
}
```

At comment `[0]`, we start by renaming the library functionality for coordinate handling, from `geodesy::CoordinateTuple` to `Coord`. Since coordinates are at the heart of what we're doing, it should have a brief and clear name. Then why giving it such a long name by design, you may wonder - well, `CoordinateTuple` is the ISO-19111 standard designation of what we colloquially would call *the coordinates*.

---

```rust
// [1] Build some context
let mut ctx = geodesy::Context::new();
```

At comment `[1]` we instantiate a `Context`, which should not come as a surprise if you have been using [PROJ](https:://proj.org) recently. The `Context` provides the interface to the messy world external to RG (files, threads, communication), and in general centralizes all the *mutable state* of the system.

Also, the `Context` is the sole interface between the `RG` transformation functionality and the application program: You may instantiate a transformation object, but the `Context` handles it for you. While you need a separate `Context` for each thread of your program, the `Context` itself is actually designed to eventually do its work in parallel, using several threads.

---

```rust
// [2] Obtain a handle to the utm-operator
let utm32 = ctx.operator("utm: {zone: 32}").unwrap();
```

At comment `[2]`, we use the `operator` method of the `Context` to instantiate an `Operator`. The parametrisation of the operator, i.e. the text `utm: {zone: 32}` is expressed in [YAML](https://en.wikipedia.org/wiki/YAML) using element naming conventions close to those used in PROJ, where the same operator would be described as `proj=utm zone=32`.

So essentially, PROJ and RG uses identical operator parametrisations, but RG, being 40 years younger than PROJ, is able to leverage YAML, an already 20 years old, JSON compatible, generic data representation format. PROJ, on the other hand, was born 20 years prior to YAML, and had to implement its own domain specific format.

Note, however, that contrary to PROJ, when we instantiate an operator in RG, we do not actually get an `Operator` object back, but just a handle to an `Operator`, living its entire life embedded inside the `Context`.
And while the `Context` is mutable, the `Operator`, once created, is *immutable*.

This makes `Operator`s thread-sharable, so the `Context` will eventually (although not yet fully implemented), be able to automatically parallelize large transformation jobs, eliminating much of the need for separate thread handling at the application program level.

---

```rust
// [3] Coordinates of some Scandinavian capitals
let copenhagen = Coord::geo(55., 12., 0., 0.);
let stockholm  = Coord::geo(59., 18., 0., 0.);

// [4] We put the coordinates into an array
let mut data = [copenhagen, stockholm];
```

At comments `[3]` and `[4]` we produce the input data we want to transform. Internally, RG represents angles in radians, and follows the traditional GIS coordinate order of *longitide before latitude*. Externally, however, you may pick-and-choose.

In this case, we choose human readable angles in degrees, and the traditional coordinate order used in geodesy and navigation *latitude before longitude*. The `Coord::geo(...)` function translates that into the internal representation. It has siblings `Coord::gis(...)` and `Coord::raw(...)` which handles GIS coordinate order and raw numbers, respectively. The latter is useful for projected coordinates, cartesian coordinates, and for coordinates with angles in radians. We may also simply give a `CoordinateTuple` as a naked array of four double precision floating point numbers:

```rust
let mumble = Coord([1., 42., 3., 4.]);
```

The `CoordinateTuple` data type does not enforce any special interpretation of what kind of coordinate it stores: That is entirely up to the `Operation` to interpret. A `CoordinateTuple` simply consists of 4 numbers with no other implied interpretation than their relative order, given by the names *first, second, third, and fourth* coordinate, respectively.

RG operators take *arrays of `CoordinateTuples`* as input, rather than individual elements, so at comment `[4]` we put the elements into an array

---

```rust
// [5] Then do the forward conversion, i.e. geo -> utm
ctx.fwd(utm32, &mut data);
println!({:?}, data);
```

At comment `[5]`, we do the actual forward conversion (hence `ctx.fwd(...)`) to utm coordinates. Behind the scenes, `ctx.fwd(...)` splits up the input array into chunks of 1000 elements, for parallel processing in a number of threads (that is: At time of writing, the chunking, but not the thread-parallelism, is implemented).

As the action goes on *in place*, we allow `fwd(..)` to mutate the input data, by using the `&mut`-operator in the method call.

The printout will show the projected data in (easting, northing)-coordinate order.

---

```rust
// [6] And go back, i.e. utm -> geo
ctx.inv(utm32, &mut data);
Coord::geo_all(&mut data);
println!({:?}, data);
```

At comment `[6]`, we roundtrip back to geographical coordinates. Prior to print out, we let `Coord::geo_all(...)` convert from the internal coordinate representation, to the geodetic convention of "latitude before longitude, and angles in degrees".

### Defining your own world

Being intended for authoring of geodetic functionality, customization is very important. RG allows temporal overshadowing of built in functionality by registering user defined macros and operators. This is treated in examples [02 (macros)](../examples/02-02-user_defined_macros.rs) and [03 (operators)](03-user_defined_operators.rs)

### Going ellipsoidal

### Comming attractions

#### Extensions

In [^Knu19] we identified a...

#### Getting physical

#### Logging


### Discussion


### Conclusion


### References

[^Knu19]: Thomas, Kristian og mange andre om PROJ pipelines og nordiske transformationer.

### Notes

In both cases, the use of the GRS80 ellipsoid is implied, but may be expressly stated as  `utm: {zone: 32, ellps: GRS80}` resp. `proj=utm zone=32 ellps=GRS80`


###


*Rust Geodesy* (RG), is a platform for experiments with geodetic software, transformations, and standards. *RG* vaguely resembles the [PROJ](https://proj.org) transformation system, and was built in part to facilitate experiments with alternative data flow models for PROJ. So in order to focus on the data flow, the transformation functionality of *RG* is reduced to the bare minimum.

Hence, viewing *RG* as a *new PROJ*, *another PROJ*, or *PROJ [RIIR](https://github.com/ansuz/RIIR)*, will lead to bad disappointment. At best, you may catch a weak mirage of a *potential* [shape of jazz to come](https://en.wikipedia.org/wiki/The_Shape_of_Jazz_to_Come) for the PROJ internal dataflow.

But the dataflow experimentation is just one of the aims of *RG*, and not the most important - it just happens to be quite visible, since it relates to a hugely important geospatial infrastructure component. Overall, the aims of *RG* are fourfold:

1. Support experimental evolution of [ISO-19111](https://www.iso.org/standard/74039.html), the international standard for *Referencing by Coordinates*.
2. Support development of geodetic transformations.
3. Hence, provide access to a large number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions. Both as a tool for aims (1, 2, 3), and as potential input to the evolution of PROJ.

All four aims are guided by the wish to amend explicitly identified shortcomings in the existing geodetic system landscape.

## Aims

Ironically, the overall rationale for *RG* can be expressed most clearly by giving more in depth descriptions of the aims *in reverse order*. So let's start with aim #4:

### **Support experiments with data flow and alternative abstractions**

As PROJ users will know, PROJ provides a large number of fundamental building bricks for geodetic coordinate operations, and also implements a [pipeline](https://proj.org/operations/pipeline.html) operator for composing these atomic building bricks into more complex transformations.

I designed the *PROJ pipeline operator* and its associated *4D-API* some five years ago, where they were introduced to the PROJ code base in a series of pull requests (See [Plumbing for pipelines](https://github.com/OSGeo/PROJ/pull/453), the follow-up [Horner and Helmert](https://github.com/OSGeo/PROJ/pull/456), [Complex Horner](https://github.com/OSGeo/PROJ/pull/482), the pipeline driver program [cct](https://github.com/OSGeo/PROJ/pull/574), and [the 4D API](https://github.com/OSGeo/PROJ/pull/530)). Getting so far cost me a trip through a code landscape characterized by 15 years of mostly ad-hoc evolution, to which I added my own slew of mess by bolting a 4D data flow on top of a 3D, which in turn had been grafted onto the original 2D data flow architecture a few years earlier.

But it worked... over a time span of a few months, PROJ went from being a mostly-map-projections package, to a complete geodetic transformation package. I did, however, promise myself to get back to the data flow architecture, to implement a streamlined fully 4D data flow, to match the 4D API. Before I got so far, however, Messrs. Butler, Ramsey, Evers and Rouault succesfully conducted the [GDAL barn raising](https://gdalbarn.com/) which, through the herculean efforts of [Even Rouault](https://erouault.blogspot.com/2019/01/srs-barn-raising-8th-report-ready-for.html), led to PROJ support of a.o. OGC-WKT2 and a much improved handling of the EPSG-data base.

His enormous effort also led to a three times larger code base, with many new structural elements, making data flow rearchitecting a much more daunting task. And hence, making it much more attractive to start the data flow architecture experiments as a separate project, entirely outside of PROJ. *RG* is the third in a series of such experiments: The first was written in C, the second in C++ - and now: here comes *RG*, the Rust version.

First and foremost, *RG* provides a reimplementation of the PROJ pipeline operator, and a redesign of the communication data flow between the steps of a pipeline. But changing the data flow architecture of a system, any system, is an extremely intrusive operation. And for a system like PROJ, based on hundreds of cooperating entities, the data flow architecture essentially *is* the system. It is the system in the sense that it is the systematic backbone, enabling some very different components to work in concert. So implementing a new data flow architecture for PROJ is like building a new system, while keeping the existing functional components and API.

Experiments at that scale are better done outside of the PROJ code tree: By starting small, results come in faster, and we can iron out kinks, do extensive testing, and develop relevant abstractions, before attempting a reimplementation within the PROJ code base.

## **Loose notes on the remaining aims**

The aims of being a platform for experiments with geodetic software, and particularly a platform for experiments with the PROJ data flow, is complemented by two other aims:

1. Being a platform for prototyping concepts for future geospatial standards
2. Providing an authoring system for developers of geodetic transformations

While this may seem like aiming in 4 different directions, with a correspondingly diminishing probable hit rate, the aims are in fact complementary and mutually supportive, as will be made clear below.

### A platform for prototyping future geospatial standards

The *platform for experiments towards improved geospatial standards* aspect of PEGS, primarily targets [ISO 19111](https://www.iso.org/standard/74039.html) *Referencing by coordinates*, and [ISO 19162](https://www.iso.org/standard/76496.html) *Well-known text representation of coordinate reference systems*.

Since PROJ provides an implementation (by [Even Rouault](https://github.com/rouault)) of ISO 19111 and its associated WKT descriptor system, PROJ developers should ideally have an opinion about the future development of these standards: It is very likely that some of the problems PROJ users occasionally encounter, really are symptoms of the standards being less than perfect – the world view of the standards are not necessarily geodetically feasible.

So by improving the standards, we may also improve and simplify the implementation and, through better conceptualisations, also simplify its use, by providing a conceptual model that is clearer and better aligned with geodetic realities.

### An authoring system for developers of geodetic transformations

...

## The means to the aims

To support (but not necessarily fulfill) the ambitions, *RG* provides a small number of data types, software components, and conceptual models:

1. An improved pipeline operator
2. A new data type for coordinate tuples, backward compatible with PROJ's `PJ_COORD` at the binary and conceptual level
3. A drastically simplified data flow

...

### The *RG* pipeline operator

Pipelines, in *RG* as in PROJ, are actually implementations of the ISO 19111 concept of "concatenated operations". Concatenated operations are ordered sets of *operations on coordinate tuples* where the output of one operation is fed as input into the next. On the outside, i.e. at the API level, an *RG* pipeline looks and behaves exactly like an atomic operation.

### Well-, somewhat-, and hardly known text formats

**WKT**, the **well known text** format originates from [OGC](opengeospatial.org), and really is a family of formats. One of the family members is used when describing, in human readable (?) form, the coordinate system and coordinate operation concepts established in the international standard [**ISO 19111**](https://www.iso.org/standard/74039.html). WKT itself is defined in [**ISO 19162**](https://www.iso.org/standard/76496.html). These standards are also, and more readily, available in the OGC *abstract specification* series under the catchy aliases [**18-005r4**](http://docs.opengeospatial.org/as/18-005r4/18-005r4.html) and [**18-010r7**](http://docs.opengeospatial.org/is/18-010r7/18-010r7.html).

Since the standardisation organizations OGC and ISO have monopolized the property of a text format to be "well known", any other text format must hence be less known. So in consequence, and for symmetry, we will in the following refer to the (well known) PROJ parametrisation language (i.e. the `proj=utm zone=32 ellps=GRS80` style strings used to instantiate PROJ operations), as **SKT**, the **somewhat known text** format.

### A new data type for coordinate tuples

### A new data type for coordinate sets

### A simplified data flow

CSP C.A.R.Hoare

1. [ISO 19100 series](https://www.iso.org/committee/54904/x/catalogue/)
2. [OGC Abstract specification series](docs.opengeospatial.org)
