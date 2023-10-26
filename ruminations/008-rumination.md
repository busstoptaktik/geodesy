# Ruminations on Rust Geodesy

## Rumination 008: Geodesy from a PROJ perspective

Thomas Knudsen <thokn@sdfi.dk>

2023-08-31. Last [revision](#document-history) 2023-08-31

### Abstract

```sh
$ echo 12 55 | kp "proj:in | geodesy:out | geodesy:in | proj:out"
> 12 55 0 0
```

---

### Prologue

The original aim with the work eventually leading to Rust Geodesy (RG) may, in few words, be paraphrased as:

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

**But PROJ** is a package of geodetic software composed from an *ad hoc* zoo of geodetic paraphernalia bolted onto of a package of cartographic software. An unholy mess for which, unfortunately, I'm to a large extent, the one to blame.

**While RG**, having the good fortune of being able to learn and draw inspiration from PROJ, implements fundamental geodetic concepts in a more structured way, and adds a small number of cartographic projections, to the extent they are needed in order to complete common geodetic tasks.

---

### Historical differences

**PROJ** was originally written in the early 1980's, by Gerald Ian (Jerry) Evenden (GIE, 1935-2016), at the USGS Woods Hole Coastal and Marine Service Center. It appears to have been conceptualized as a companion to the **MAPGEN** cartographic system [1].

The code library, `libproj.a`, and its accompanying command line interface, the `proj` executable, were focused strictly on providing map projection functionality, and by design excluded any support for geodesy in general, and datum shifts in particular.

Entirely ignoring these subjects was, however, not feasible at a time where the transition from the NAD27 datum, to NAD83 was imminent. Hence, the PROJ package included both the [nad2nad](https://github.com/OSGeo/PROJ/blob/4.7/src/nad2nad.c) tool for shifting between these two North American datums, and the [geod](https://github.com/OSGeo/PROJ/blob/4.7/src/geod.c) tool for computation of geodesics on the ellipsoid.

So even from the beginning (or at least since the long lasting version 4 of the package), PROJ has included some geodetic functionality, although in an implementation orthogonal and somewhat separate from the cartographic main functionality.

When Frank Warmerdam took over PROJ maintenance in 1999, further geodetic functionality was soon added in the form of the [cs2cs](https://github.com/OSGeo/PROJ/blob/4.7/src/cs2cs.c) filter. From a geodetic viewpoint, however, PROJ was still severely limited by its cartographic, and hence inherently 2D, architecture.

This changed in October 2015, where [Piyush Agram](https://www.linkedin.com/in/piyush-shanker-agram-78a76b2/) introduced the [3D API Extension](https://github.com/OSGeo/PROJ/commit/757a2c8f946faccf9d094d76cb79e6ebe0006564), in order to support the [SCH](https://github.com/OSGeo/PROJ/blob/5.0/src/PJ_sch.c) "spherical cross-track, height" radar sensor system.

In 2016, Kristian Evers and I further extended this to 4D, and introduced the "pipeline" operator in the RFC [Transformation Pipelines](https://github.com/OSGeo/PROJ/pull/388), finally merged as [Plumbing for Pipelines](https://github.com/OSGeo/PROJ/pull/453) followed by [Pipeline plus API](https://github.com/OSGeo/PROJ/pull/445) in November 2016. So in slightly more than a year, PROJ went from strictly 2D, to fully 4D, had gained a number of new geodetic operators, and on top of that, the pipeline operator, for composition of elementary operators into more complex geodetic transformations.

**Rust Geodesy** was born through a series of experiments with alternative data flow architectures for PROJ: While PROJ's data flow made much sense for a library being strictly 2-D and strictly supporting projection style operators, it became unnecessarily cluttered for the data flow of generic operators operating in a 2/3/4-D space.

I'll elaborate on the details below. For now, let me just state that it required quite a few false starts to actually arrive at something feasible. Essentially RG is the sixth in a series of experiments, preceded by two versions in C, two in C++, and one in Rust.

So while RG is the result of much experimentation, it is by no means mature to the degree PROJ is: PROJ is reality hardened through 40 years of real-world usage, while RG is still architecturally fluid, and probably not even feature complete.

This does not necessarily imply that PROJ cannot benefit from RG ideas: Most of RG's architectural scaffolding can be projected directly onto the PROJ code base. And while this will require a hearty dose of elbow grease, it will almost certainly result in PROJ becoming faster, more comprehensible, and easier to maintain.

RG's current main *raison d'etre*, is to be a platform for experimentation towards a leaner, cleaner, and geodetically more viable, set of ISO geospatial standards. This involves the surmounting of galactic scale inerty, so visual progress is not expected immediately. At a decadal time scale it may, however, lead to the potential for additional clean ups in the PROJ business logic.

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

#### The context-bound implementation of ISO 19111

---

### Implementation differences

---

### Ergonomical differences

#### The syntax

#### The data flow (loops in fwd/inv versus loops in the ops)

---

### The relation to ISO/OGC geomatics standards

Parameters: Defined (conversion) vs. determined (transformation)

---

### Bibliography

[1] [MAPGEN](https://www.usgs.gov/publications/mapgen-cartographic-system)

### Document History

Major revisions and additions:

- 2023-08-31: Outline
