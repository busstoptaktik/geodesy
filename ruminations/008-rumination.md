# Ruminations on Rust Geodesy

## Rumination 008: Geodesy from a PROJ perspective

Thomas Knudsen <thokn@sdfi.dk>

2023-08-31. Last [revision](#document-history) 2023-11-08

### Abstract

```sh
$ echo 12 55 | kp "proj:in | geodesy:out | geodesy:in | proj:out"
> 12 55 0 0
```

---

### Prologue

The original (2017) aim with the work eventually leading to Rust Geodesy (RG) may, in few words, be paraphrased as:

> To demonstrate, in a compact and accessible sandbox style, that PROJ can benefit from a number of
>
> - architectural simplifications,
> - extensions, and
> - general improvements
>
> Not through far reaching reimplementation, but through generalisation of already existing PROJ idioms.

To elucidate that, this rumination compares PROJ and RG in terms of **ontology** (i.e. *what is it?*), **history** (*where did it come from?*), **architecture** (*how is it built?*), **implementation** (*what is it built of?*), and **ergonomics** (*how is it used?*).

But while born as a playground for potential PROJ improvements, today the main focus of RG is different. So for completeness' sake we round off with an overview of the relation of both PROJ and RG, to the ISO/OGC series of geospatial standards. A series of standards, which PROJ implements and innovates/improves upon, and which RG partially implements, in the quest for a path towards simplification and conceptual unification of geodesy and geomatics.

---

### Ontological differences

**PROJ** is a package of geodetic software, while **RG** on the other hand, is a package of geodetic software. So not much of a difference there.

**But PROJ** is a package of geodetic software composed from an *ad hoc* zoo of geodetic paraphernalia bolted onto of a package of cartographic software. An unholy mess for which, unfortunately, I'm at least to some extent, the one to blame.

**While RG**, having the good fortune of being able to learn and draw inspiration from PROJ, implements fundamental geodetic concepts in a more structured way, and adds a small number of cartographic projections, to the extent they are needed in order to complete common geodetic tasks.

---

### Historical differences

**PROJ** was originally written in the early 1980's, by Gerald Ian (Jerry) Evenden (GIE, 1935-2016), at the USGS Woods Hole Coastal and Marine Service Center. It appears to have been conceptualized as a companion to the [MAPGEN][1] cartographic system.

The code library, `libproj.a`, and its accompanying command line interface, the `proj` executable, were focused strictly on providing map projection functionality, and by design excluded any support for geodesy in general, and datum shifts in particular.

Entirely ignoring these subjects was, however, not feasible, especially not in the US, at a time where the transition from the NAD27 datum, to NAD83 was imminent. Hence, the PROJ package included both the [nad2nad](https://github.com/OSGeo/PROJ/blob/4.7/src/nad2nad.c) tool for shifting between these two North American datums, and the [geod](https://github.com/OSGeo/PROJ/blob/4.7/src/geod.c) tool for computations involving geodesics on the ellipsoid.

So even from the beginning (or at least since the long lasting version 4 of the package), PROJ has included some geodetic functionality, although in an implementation orthogonal and somewhat decoupled from the cartographic main functionality.

When Frank Warmerdam took over PROJ maintenance in 1999, further geodetic functionality was soon added in the form of the [cs2cs](https://github.com/OSGeo/PROJ/blob/4.7/src/cs2cs.c) filter. From a geodetic viewpoint, however, PROJ was still severely limited by its cartographic, and hence inherently 2D, architecture.

This changed in October 2015, where [Piyush Agram](https://www.linkedin.com/in/piyush-shanker-agram-78a76b2/) introduced the [3D API Extension](https://github.com/OSGeo/PROJ/commit/757a2c8f946faccf9d094d76cb79e6ebe0006564), in order to support the [SCH](https://github.com/OSGeo/PROJ/blob/5.0/src/PJ_sch.c) "spherical cross-track, height" radar sensor system.

In 2016, Kristian Evers and I further extended this to 4D, and introduced the "pipeline" operator in the RFC [Transformation Pipelines](https://github.com/OSGeo/PROJ/pull/388), finally merged as [Plumbing for Pipelines](https://github.com/OSGeo/PROJ/pull/453) followed by [Pipeline plus API](https://github.com/OSGeo/PROJ/pull/445) in November 2016. So in slightly more than a year, PROJ went from strictly 2D, to fully 4D, had gained a number of new geodetic operators, and on top of that, the pipeline operator, for composition of elementary operators into more complex geodetic transformations.

During the work with Plumbing for Pipelines, it became clear that the PROJ internal dataflow badly needed an overhaul. But before I found the time to do this, [GDALbarn](https://gdalbarn.com/), the *GDAL Coordinate System Barn Raising* got funded. This had ground-shattering effects on the PROJ code base which, in less than a year, through [Even Rouault](https://github.com/rouault)'s herculean efforts, more than doubled in number of code lines, and grew additional functionality (and complexity) never before dreamt of.

**Rust Geodesy** was born through a series of experiments with alternative data flow architectures for PROJ: While PROJ's data flow made much sense for a library being strictly 2-D and strictly supporting projection style operators, it became unnecessarily cluttered for the data flow of generic operators operating in a 2/3/4-D space.

But with all the fantastic effects of the realisation of GDALbarn also came the less desirable side effect, that experiments with the internals became much harder to carry out. Hence, the plan to build a much smaller system, with just enough functionality to do some rudimentary geodetic work, and consequently much more freedom to shuffle around the internals. This, in the hope of reaching solutions that (come time and means) could serve as inspiration for architectural remodeling of PROJ.

I'll elaborate on the details below. For now, let me just state that it required quite a few false starts to actually arrive at something feasible. Essentially RG is the sixth in a series of experiments, preceded by two versions in C, two in C++, and one in Rust.

So while RG is the result of much experimentation, it is by no means mature to the degree PROJ is: PROJ is reality hardened through 40 years of real-world usage, while RG is still architecturally and API-wise fluid, and probably not even feature complete.

This does not necessarily imply that PROJ cannot benefit from RG ideas: Most of RG's architectural scaffolding can be projected directly onto the PROJ code base. And while this will require a hearty dose of elbow grease, it will almost certainly result in PROJ becoming faster, more comprehensible, and easier to maintain.

This RG-fication of PROJ is, however, not anything I see happening in the near future, so RG's current main *raison d'etre*, is to be a platform for experimentation towards a leaner, cleaner, and geodetically more viable, set of ISO geospatial standards. This involves the surmounting of galactic scale inerty, so visual progress is not expected immediately. At a decadal time scale it may, however, lead to the potential for additional clean ups in the PROJ business logic.

### Architectural differences

**PROJ** was designed to centralize the handling of anything that the class of cartographic projection operators has in common, by integrating that into the data flow architecture. Most notably this regards the size (but not the shape) of the ellipsoid used, the false easting, the false northing, and the central meridian of the projection.

*Hence, all PROJ projection code is designed to work on an ellipsoid of size 1, with central meridian 0, and false (northing, easting) of (0, 0).*

This eliminates a lot of boilerplate in the implementation of the individual projections. But once introducing non-projection, non-2-D operators, we instead had to introduce a **metric shitload** of *preparation*, *finalization*, and *selection* boilerplate into PROJ's [fwd](https://github.com/OSGeo/PROJ/blob/master/src/fwd.cpp) and [inv](https://github.com/OSGeo/PROJ/blob/master/src/inv.cpp) operator invocation logic.

Also, the PROJ operators are designed to work on a single coordinate at a time, hence invoking all the ceremonial boilerplate for *every single coordinate tuple* processed.

**Rust Geodesy** eliminates these elements by:

1. Leaving the handling of units and offsets to the individual operator implementations
2. Having the operators operate on *collections of* coordinate tuples
3. Supporting arbitrary user data types and coordinate dimensionalities through the support of the `CoordinateSet` trait

#### The operator building blocks

RG, like PROJ, is based on the architectural core principle of individually developed and maintained operators, typically encapsulating all code for a given geodetic operator in a single source file.

Unlike PROJ, however, RG supports the run-time integration of user defined operators and macros. This is a consequence of the very different approaches taken to the implementation and ergonomics of the *context* concept.

#### The context interface

In PROJ, the context type, `PJ_CONTEXT` was originally designed to enable multithreaded use of error messaging, and overloading of the `stdio` file access interface. It was bolted onto the existing API, and designed in a way making it "as invisible as possible", i.e. entirely invisible for singlethreaded programs, but imposing additional ceremony for multithreaded.

Knowing from experience with both [PROJ](https://proj.org) and [trlib](https://github.com/busstoptaktik/trlib) that some kind of system interface context is unavoidable, RG makes a virtue out of necessity, by assigning the context the leading role in the API design. We cannot avoid it - so make it carry its own weight.

#### An open and extensible architecture

RG is intended as a platform for experiments, so openness and extensibility is extremely important. One of the places where this is particularly evident is in the Context handling.

The RG Context is modelled as a trait, so while RG comes with ready made Context objects, users may implement their own Contexts at application compile time, having them work seamlessly with the RG library code.

In the same way, additional operators may be supplied by the user applications, for direct inclusion alongside the built in ones. And if this compile time extension facility is not enough, RG also supports some limited *run time* user supplied extensions, through a simple macro programming facility.

#### The PROJ implementation of ISO 19111

(Left to the reader as an exercise 😊)

---

### Implementation differences

#### Programming Language

PROJ and RG differ in a large spectrum of implementation details. Most visibly through their programming language of choice. **PROJ** was originally a pure C project, but with the work towards implementation of the ISO-19111 model and WKT, switched to C++. **RG** has always been written in pure Rust.

#### The data flow ("Plumbing")

With its background in attempts to re-rationalise the PROJ data flow, RG obviously differs from PROJ in the plumbing aspect. Most evidently by moving scaling and offset factors from the data flow level into the individual operator implementations.

On one hand, this increases the amount of **(trivial)** boilerplate necessary for implementing projection-like operators, while on the other hand it decreases the amount of **(non-trivial)** workaround kludges necessary for implementing non-projection operators.

So the centralized handling makes much sense for a projection library ("classic PROJ"), but not really for a more general transformation library (modern PROJ, RG, trlib).

The foundational data unit is another design decision deeply influencing the data flow. **PROJ** operators are designed to handle *individual coordinates* (`CoordinateTuple`s in ISO-19111 lingo), while **RG** takes a step further into the ISO wilderness, and forces the operators to handle *ordered sets of coordinate tuples,* i.e. the ISO-19111 `CoordinateSet` interface.

This implies that **PROJ** handles sets of coordinate tuples at the data flow level, handing out one coordinate tuple after the other to the low level operators, while **RG** hands entire sets directly to the operators.

This comes with the (modest) penalty of a loop construct in every operator implementation. But it also significantly increases the potential for compiler provided optimizations, e.g. parallelization, unrolling, and offloading to the GPU. Also, it facilitates the simplification of the background data structures, by amortizing recomputations of ancillary factors over the entire coordinate set, rather than on each individual coordinate tuple.

This eliminates the need for individual *opaque objects* for every operator: RG has only one data structure for all parameter handling, rather than PROJ's individual opaque object types for almost every operator implemented.


---

### Ergonomical differences

From a developer's point of view, the ergonomical differences between PROJ and RG are largely a matter of Rust's opinionated take on the build process: Using RG in a Rust based project is as easy as adding an extra line in the `[Dependencies]` section of the project's `Cargo.toml` file: The `cargo` build tool will handle all direct and transitive dependencies for both native and cross compilation; [crates.io](https://crates.io/crates/geodesy) and [lib.rs](https://lib.rs/geodesy) will provide the general view of how RG relates to the Rust library landscape in general, and the [docs.rs](https://docs.rs/geodesy/) service will provide the API documentation.

While a.o. the ubiquity of `cmake` has simplified C++ multi-platform library handling significantly, its great flexibility also comes with increased responsibility for the developer. And while this is surmountable for experienced developers, introducing PROJ in their workflow, and able to amortize the initial effort over thousands of builds, it may very well be prohibitive for a beginner, otherwise perfectly able to jump directly in and using RG. This despite the stellar quality of the [PROJ](https://proj.org) documentation, compared to the somewhat mediocre RG counterpart.

#### The syntax

Developers aside. For end users of applications built with RG or PROJ, the operator syntax is probably the most important ergonomical aspect. And while there are great similarities in the foundational syntaxen, the RG mechanism for expressing operator pipelines is more compact and a bit different from PROJ's.

In PROJ, we instantiate a projection object for zone 32 of the UTM system as

```js
proj=utm zone=32
```

whereas in RG we obtain the same result using this slightly simpler incantation

```js
utm zone=32
```

PROJ has a *de facto* convention that **the first** element in a definition string should identify the name of the operator being instantiated. RG *formalizes* this convention, and hence does not need the `proj=` part, since by definition (and not just by convention) the first element *is* the operator identification.

Through the evolution of PROJ from a projection library to a generic transformation library, the `proj=` part has become slightly confusing, since it is used to identify not just `proj`ections, but any kind of geodetic operations. RG, being born as a generic geodetic library, eliminates this potential confusion.

**But for pipelines** RG simplifies the syntax even more...

In PROJ, the syntax for pipelines was bolted on to an existing specification syntax, utilizing whichever left-over holes in the syntax-wall that would accomodate its bolts (take my word for it: I am the one to blame). That led to a somewhat verbose syntax (more nuts than bolts, perhaps), as seen from this example, sandwiching a Helmert shift between conversions from geographical to cartesian coordinates, and back:

```js
proj=pipeline
    step proj=cart ellps=intl
    step proj=helmert x=-87 y=-96 z=-120
    step proj=cart inv ellps=GRS80
```

In RG, knowing up front how useful operator pipelines are, we let them play a much more prominent role: In RG, *everything* is a pipeline, even if there is only a single step in that pipeline. So we leave out the `proj=pipeline` part of the syntax: When everything really is a pipeline, we do not need to state that fact explicitly in every single definition.

Also, since pipelines are obviously akin to Unix style shell pipes, we use the Unix shell syntax, i.e. the vertical bar character `|`, to separate the steps of the pipeline. All in all this results in a compact, and very readable pipeline definition syntax:

```js
cart ellps=intl | helmert x=-87 y=-96 z=-120 | cart inv ellps=GRS80
```

While this is easier to both read and write, realizing the ubiquity of PROJ strings, RG also includes a converter, turning PROJ syntax into RG syntax (the inverse direction is trivial).

---

### The relation to ISO/OGC geomatics standards

Since Even Rouault's gargantuan effort in implementing the ISO-19111 model, and its WKT2-based CRS expression, as part of the already mentioned [GDALbarn](https://gdalbarn.com/) project, **PROJ** has been a major implementation of the standard. Furthermore, Even Rouault has made major efforts towards a more contemporary implementation, through his work on [PROJJSON](https://proj.org/en/9.3/specifications/projjson.html), *"a better WKT than WKT,"* and on [Geodetic TIFF Grids](https://proj.org/en/9.3/specifications/geodetictiffgrids.html) *"a better grid format than all other grid formats"*.

**RG** on the other hand has no focus on implementing the WKT2 CRS-representations. It is rather built on the idea that a CRS is simply a label, useful for looking up transformations in a relevant register. This approach is already a part of the ISO-19100 model, but RG takes the more radical view that a CRS is fundamentally empirical, and hence the "definition" (i.e. the WKT2) aspect of CRS' should be abandoned entirely. These aspects are further elaborated on in [Rumination 4](004-rumination.md), [Rumination 5](005-rumination.md) and [Rumination 6](006-rumination.md).

So all in all, and to various degrees, *both PROJ and RG embraces and supports the main ideas expressed in the ISO/OGC geomatics standards series.*

Both projects also strive to drive the standards forward. **PROJ** by providing improved solutions to existing elements, **RG** by highlighting how much we can simplify the standards, by eliminating elaborate abstractions over non-existing geodetic concretions.

---

### Bibliography

[1]: https://www.usgs.gov/publications/mapgen-cartographic-system

G.I.Evenden, 1986: [MAPGEN][1] Cartographic System. Proceedings - 1986 Working Symposium on Oceanographic Data Systems, pp. 239-245

### Document History

Major revisions and additions:

- 2023-08-31: Outline
- 2023-11-08: First public version
