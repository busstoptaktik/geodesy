# Ruminations on Rust Geodesy

## Rumination 004: Why Rust Geodesy - some background

Thomas Knudsen <knudsen.thomas@gmail.com>

2022-05-17. Last [revision](#document-history) 2022-05-17

### Abstract

```sh
$ echo 55 12 | kp "geo:in | utm zone=32"
> 691875.6321 6098907.8250 0.0000 0.0000
```

---

### Prologue

As already outlined in the [readme](../README.md)-file and in rumination no. [000](./000-rumination.md), the primary aims for RG is to

1. Support experiments for evolution of geodetic standards.
2. Support development of geodetic transformations.
3. Hence, provide easy access to a number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions.

In this rumination, I will concentrate on the latter of these aims, which has been simmering since I introduced the [Plumbing for Pipelines](https://github.com/OSGeo/PROJ/pull/453) functionality in [PROJ](https://proj.org). During the work with Plumbing for Pipelines, it became clear that the PROJ internal dataflow badly needed an overhaul. But before I found the time to do this, [GDALbarn](https://gdalbarn.com/), the *GDAL Coordinate System Barn Raising* got funded. This had ground-shattering effects on the PROJ code base which, in less than a year, more than doubled in number of code lines, and grew additional functionality never before dreamt of.

With all the fantastic effects of the realisation of GDALbarn also came the less desirable side effect, that experiments with the internals became much harder to carry out. Hence, the plan to build a much smaller system, with just enough functionality to do some rudimentary geodetic work, and consequently much more freedom to shuffle around the internals. This in the hope of reaching solutions that, come time and means, could serve as inspiration for architectural remodeling of PROJ.

That's how Rust Geodesy (RG) was born. But first after a number of false starts, that contributed to the shaping of ideas, but never really took off. All in all, the current incarnation of RG is the sixth in a series, after two attempts in C, two in C++, and one previous version in Rust, since essentially the RG architecture was redone from scratch after version 0.7.1.

But let's take a look at some of the PROJ problems that has fueled the efforts for finding different solutions for RG.

(TBC: Starting with the data flow, as it is simple. The treatment of the abstractions is split up in the parts dealing with improving directly on PROJ's architecture, and the parts which deals with an (at least originally) generally unfortunate OGC/ISO foundational model. A model that has induced correspondingly unfortunate user expectations concerning what can realistically be expected from a transformation system (a subject that will be elaborated in a later Rumination)
Note: Provider vs. resolver vs. "is a resolver really plausible")

### Problem #1: The data flow

The major problem in the PROJ dataflow is related to PROJ's historical background as a strictly 2-D projection library, where parts of the processing (e.g. the handling of center meridians, false northing, and false easting), is carried out in the data flow layer, rather than in the individual projection implementations.

This made (kind of) sense as long as the PROJ operations consisted of nothing but projections, where these factors are common and have well defined meanings. Once datum transformations entered the scene, that was no more the case, and it should have been eliminated immediately. With 3-D and 4-D data flow channels bolted onto the original 2-D channel (an aspect for which I'm personally to blame), things got correspondingly worse, as recently seen in the [Implement Vertical Offset and slope transformation method](https://github.com/OSGeo/PROJ/pull/3200) PROJ pull request, where @rouault says (with my emphasis):

> I was not sure how to map the latitude and longitude of the "evaluation point" of the method, which is the 'origin' of the inclinated plane, to PROJ string parameters. I've used the classic lon_0 and lat_0 parameters, but not completely sure it is appropriate **(I have to re-add lon_0 in the forward method, since generic code in PROJ, substracts it, as it is what is convenient for map projections)**

### Solution #1: The data flow

RG solves most of the data flow problems simply by dictating that all data flow channels are 4-D. Since modern GNSS-based geodesy is inherently 4-D, it is the proper thing to do anyway.

Also, the handling of generic cartographical constants is removed from the data flow level. While this requires a few extra lines of trivial code for each projection implementation, it also reduces complexity at the data flow level, where all operations (projections or not) can be treated identically.

### Problem #2: The architecture

### Problem #3: The abstractions

What are the problems

How can we solve them

How can we implement the solutions

(TBC! - The unreasonable expectations are plentiful over at <https://github.com/OSGeo/PROJ/issues/1552>, but anyways: The operator architecture is modular, so why bulid a monolithic provider/resolver-arkitecture?)

### Document History

Major revisions and additions:

- 2022-05-17: Initial version
- 2022-05-22: Draft level additions
