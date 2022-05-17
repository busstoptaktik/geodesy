# Ruminations on Rust Geodesy

## Rumination 004: Some background

Thomas Knudsen <knudsen.thomas@gmail.com>

2022-05-17. Last [revision](#document-history) 2022-05-17

### Abstract

```sh
$ echo 55 12 | kp "geo:in | utm zone=32"
> 691875.6321 6098907.8250 0.0000 0.0000
```

---

### Prologue

As already oulined in the [readme](../README.md)-file and in rumination no. [000](./000-rumination.md), the primary aims for RG were to

1. Support experiments for evolution of geodetic standards.
2. Support development of geodetic transformations.
3. Hence, provide easy access to a number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions.

In this rumination, I will concentrate on the last of these aims, which has been simmering since I introduced the [Plumbing for Pipelines](https://github.com/OSGeo/PROJ/pull/453) functionality in [PROJ](https://proj.org). During the work with Plumbing for Pipelines (PfP below), it became clear that the PROJ internal dataflow badly needed an overhaul. But before I found the time to do this, [GDALbarn](https://gdalbarn.com/), the *GDAL Coordinate System Barn Raising* got funded, which had ground-shattering effects on the PROJ code base which, in less than a year, more than doubled in number of code lines, and grew additional functionality never before dreamt of.

With all the fantastic effects of the realisation of GDALbarn also came the less desirable side effect, that experiments with the internals became much harder to carry out. Hence, the plan to build a much smaller system, with just enough functionality to do some rudimentary geodetic work, and consequently much more freedom to shuffle around the internals.

That's how Rust Geodesy (RG) was born. But first after a number of false starts, that contributed to the shaping of ideas, but never really materialized. All in all, the current incarnation of RG is the sixth in a series, after two attempts in C, two in C++, and one previous version in Rust, since essentially the RG code base was rewritten from scratch after version 0.7.1.

### Problem #1: The data flow

####

The major problem in PROJ dataflow is the fact that PROJ started as a strictly 2-D projection library, where parts of the processing (most notably the handling of center meridians, false northing, and false easting), is carried out in the data flow layer, rather than in the individual projection implementations.

This made (kind of) sense as long as the PROJ operations consisted of nothing but projections, where these factors are common and have well defined meanings. Once datum transformations entered the scene, that was no more the case, and should have been eliminated immediately. With 3-D and 4-D data flow channels bolted onto the original 2-D channel, things got correspondingly worse (trust me: I'm the one to blame), as recently seen in the [Implement Vertical Offset and slope transformation method](https://github.com/OSGeo/PROJ/pull/3200) PROJ pull request, where @rouault says (with my emphasis):

> I was not sure how to map the latitude and longitude of the "evaluation point" of the method, which is the 'origin' of the inclinated plane, to PROJ string parameters. I've used the classic lon_0 and lat_0 parameters, but not completely sure it is appropriate **(I have to re-add lon_0 in the forward method, since generic code in PROJ, substracts it, as it is what is convenient for map projections)**

### Solution #1: The data flow

RG solves (most of) the data flow problems by dictating that all data flow channels are 4-D, since modern GNSS-based geodesy is inherently 4-D.
Also, generic cartographical constants are not handled at the data flow level, but must be handled in each projection implementation individually. While this adds a few lines of code to each projection implementation, it also reduces complexity at the data flow level, since all operations can be treated equally.


What are the problems

How can we solve them

How can we implement the solutions

TBC!

### Document History

Major revisions and additions:

- 2022-05-17: Initial version
