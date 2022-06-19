# Ruminations on Rust Geodesy

## Rumination 004: Why Rust Geodesy - some background (DRAFT)

Thomas Knudsen <knudsen.thomas@gmail.com>

2022-05-17. Last [revision](#document-history) 2022-06-19

### Abstract

```sh
$ echo 55 12 | kp "geo:in | utm zone=32"
> 12 55 0 0
```

---

### Prologue

As already outlined in the [README](../README.md)-file and in rumination no. [000](./000-rumination.md), the primary aims for RG is to

1. Support experiments for the evolution of geodetic standards.
2. Support development of geodetic transformations.
3. Hence, provide easy access to a number of basic geodetic operations, not limited to coordinate operations, and...
4. ...support experiments with data flow and alternative abstractions.

Essentially, the last two of these aims are means for realizing the first two. In other words, we have a set of two *goals*, and a set of two *means*, where the first ("provide easy access...") relates to the *scope* of RG, while the last ("support experiments...") relates to the *architecture*.

In this rumination, I will concentrate on the *means*, and specifically on the *architecture* aspect ("aim #4") of the *means*. For the *scope* part, cf. rumination no. [002](./002-rumination.md), for coordinate operations, and the upcomming rumination no. [007](./007-rumination.md), for the more general geodetic operations.

The overall architecture of RG was outlined in rumination no. [000](./000-rumination.md). Below, I will go into more detail with the implementation aspect, which has been simmering since I introduced the [Plumbing for Pipelines](https://github.com/OSGeo/PROJ/pull/453) functionality in [PROJ](https://proj.org).

During the work with Plumbing for Pipelines, it became clear that the PROJ internal dataflow badly needed an overhaul. But before I found the time to do this, [GDALbarn](https://gdalbarn.com/), the *GDAL Coordinate System Barn Raising* got funded. This had ground-shattering effects on the PROJ code base which, in less than a year, more than doubled in number of code lines, and grew additional functionality (and complexity) never before dreamt of.

With all the fantastic effects of the realisation of GDALbarn also came the less desirable side effect, that experiments with the internals became much harder to carry out. Hence, the plan to build a much smaller system, with just enough functionality to do some rudimentary geodetic work, and consequently much more freedom to shuffle around the internals. This in the hope of reaching solutions that, come time and means, could serve as inspiration for architectural remodeling of PROJ.

That's how Rust Geodesy (RG) was born. But first after a number of false starts, that contributed to the shaping of ideas, although they never really took off. All in all, the current incarnation of RG is the sixth in a series, after two attempts in C, two in C++, and one previous version in Rust, since essentially the RG architecture was redone from scratch after version 0.7.1.

But let's take a look at some of the PROJ problems that has fueled the efforts for finding different solutions for RG.

### Problem #1: The data flow

The major problem in the PROJ dataflow is related to PROJ's historical background as a strictly 2-D projection library, where parts of the processing (e.g. the handling of center meridians, false northing, and false easting), is carried out in the data flow layer, rather than in the individual projection implementations.

This made (kind of) sense as long as the PROJ operations consisted of nothing but projections, where these factors are common and have well defined meanings. Once datum transformations entered the scene, that was no longer the case, and it should have been eliminated immediately. With 3-D and 4-D data flow channels bolted onto the original 2-D channel (an aspect for which I'm personally to blame), things got correspondingly worse, as recently seen in the [Implement Vertical Offset and slope transformation method](https://github.com/OSGeo/PROJ/pull/3200) PROJ pull request, where @rouault says (with my emphasis):

> I was not sure how to map the latitude and longitude of the "evaluation point" of the method, which is the 'origin' of the inclinated plane, to PROJ string parameters. I've used the classic lon_0 and lat_0 parameters, but not completely sure it is appropriate **(I have to re-add lon_0 in the forward method, since generic code in PROJ, substracts it, as it is what is convenient for map projections)**

### Solution #1: The data flow

RG solves most of the data flow problems simply by dictating that all data flow channels are 4-D. Since modern GNSS-based geodesy is inherently 4-D, it is the proper thing to do anyway.

Also, the handling of generic cartographical constants is removed from the data flow level. While this requires a few extra lines of trivial code for each projection implementation, it also reduces complexity at the data flow level, where all operations (projections or not) can be treated identically.

### Problem #2: The OS interface abstraction

PROJ features the `PJ_CONTEXT` type, interfacing between the OS and the transformation system, in the sense that it abstracts the file-io subsystem, and centralizes handling of material that is sensitive to multithreading. Hence in PROJ, a `PJ_CONTEXT` is expected to be locked to a single thread.

Context style data types are annoying, but experience from work on both PROJ and TRLIB, the former transformation system of SDFI (the Danish NMA), shows that they are hard to get rid of. So let us at least try to make them less annoying...

### Solution #2: The OS interface abstraction

In Rust Geodesy, the OS interface is handled by a structure called "the provider". To make the provider less annoying than a PROJ context, it has been given much more functionality, so it's not just a bolted on afterthought, *but the actual API* for accessing the system functionality.

To (potentially) reduce the annoying aspect even further, there is not just one provider implementation, but two - `Minimal` and `Plain`, respectively. `Minimal` is much in the style of "classic" PROJ, while `Plain` in principle provides access to externally defined transformations from e.g. the EPSG registry (i.e. "resolver" functionality). Unlike PROJ, however, the definitions used by `Plain` are to be represented in user provided plain text files, not in a SQLite file database.

Hence, Rust Geodesy does not depend on external database functionality. The provider functionality is, however, represented by a Trait, not by fixed data structures, so users wishing to support PROJ's SQLite representation of the EPSG registry, can implement their own provider, supporting that.

In a sense, this reflects the modularity of the transformation operator implementation onto the provider implementation: You can use one of the built ins, or you can provide your own. This allows for a very slim core functionality, that can be expanded as needed in actual use cases. In PROJ, the introduction of SQLite as a dependency resulted in much whining: The unreasonable expectations/unfounded entitlement is smoke thick over at <https://github.com/OSGeo/PROJ/issues/1552>. But since RG has the good fortune of being able to learn from PROJ,  why build a monolithic provider/resolver-architecture, when we don't have to? The operator architecture is modular, after all.

### Problem #3: The geodetic registry interface ("the resolver")

In PROJ, much automation effort is spent trying to guess the most appropriate transformation betwwen any two CRS. Let's call this effort "resolving" and the contraption implementing that "the resolver".

PROJ is built on the idea that the users should know what system (e.g. represented as an EPSG CRS code) their data are comming from, and which system they want them transformed to. RG on the other hand is built on the much simpler idea, that the users should know which transformation they want to apply.

The crux of it is that it is questionable whether the idea of constructing a resolver really is plausible? At least the resolver's task is not always empirically unique, but the selection of the proper transformation must be supported by heuristics. This has lead PROJ users to trouble forcing the proper transformation to be used (supported, but not highlighted by PROJ), as seen e.g. [here](https://lists.osgeo.org/pipermail/proj/2021-October/010369.html):
> I know PROJ7 knows about time-dependent transformations, I just can't seem
to figure out how to use that to get my GCP coordinates in today's MGA2020
grid

[here](https://github.com/OSGeo/PROJ/issues/2885):

> I would like to see the difference between NAVD88 and EGM96 at a particular point (...) The NOAA's vertical datum transformation tool can provide that information (...) However, as you can see from the output, the value returned by proj is 1.41 while the vdatum tool gives me -0.033. What am I doing wrong here?

[here](https://lists.osgeo.org/pipermail/proj/2021-November/010440.html):

> We're trying to adjust proj to use a specific transformation between two
CRSes *by default*

and [here](https://github.com/OSGeo/PROJ/issues/2318) where, despite his herculean efforts of making the heuristics as perfect as possible, even Even Rouault himself must describe the situation as "complex":

> This area is complex and I must confess I generally have troubles to predict how the code I've written always behave in all situations

One might object that resolving to building on user selection of the exact transformation, rather than a set of input- and output CRS is to ignore the troubles of the non-expert users, who are left to ask their friendly geodesist colleague which transformation to select. I will, however, insist that this is not terribly different from having to ask the same person about "what CRS am I supposed to use for input and output here?". The real difference is that the friendly geodesist colleague actually gets a chance to provide the most useful advice for the task at hand.

### Solution #3: The geodetic registry interface ("the resolver")

Where PROJ supports both "find the proper transformation given input and output CRS", and "use this transformation - do not ask any questions", RG deems the concept of a resolver implausible in the general case, and leaves the implementation of one up the end user, by whom it may be implemented as part of a `Provider` data structure.


### Document History

Major revisions and additions:

- 2022-05-17: Initial version
- 2022-05-22: Draft level additions
- 2022-06-19: A few more additions. Still draft level
