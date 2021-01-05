# Rust Geodesy

*Rust Geodesy* (RG), is a platform for experiments with geodetic software and standards. *RG* vaguely resembles the [PROJ](https://proj.org) transformation system, and was built in part to facilitate experiments with alternative data flow models for PROJ. So in order to focus on the data flow, the transformation functionality of *RG* is reduced to the bare minimum.

Hence, seeing *RG* as *"a new PROJ"*, *"another PROJ"*, or *"PROJ [RIIR](https://github.com/ansuz/RIIR)"*, will lead to bad disappointment. At best, you may catch a weak mirage of a *potential* [shape of jazz to come](https://en.wikipedia.org/wiki/The_Shape_of_Jazz_to_Come) for the PROJ internal dataflow.

But the dataflow experimentation is just one of the aims of *RG*, and not the most important - it just happens to be quite visible, since it relates to a hugely important geospatial infrastructure component. Overall, the aims of *RG* are fourfold:

1. Support experimental evolution of [ISO-19111](https://www.iso.org/standard/74039.html), the international standard for *Referencing by Coordinates*.
2. Support development of geodetic transformations.
3. Hence, provide access to a large number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions. Both as a tool for aims (1, 2, 3), and as potential input to the evolution of PROJ.

All four aims are guided by explicitly identified shortcomings in the existing geodetic system landscape.

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
