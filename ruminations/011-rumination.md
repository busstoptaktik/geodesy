<!-- markdownlint-disable MD013 -->

# Some potential elements of an introduction to a revised version of ISO 19111/OGC topic 2

**Thomas Knudsen,** <thokn@kds.dk>, 2024-11-06

As indicated in the essay [What's wrong with ISO-19111?](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/010-rumination.md), quite a few things could be improved in the current version of ISO-19111 "Referencing by coordinates".

The following is an informal suggestion for some elements of an introduction to an updated version of 19111, providing an overview of a few of the most important prerequisites for a simpler, more straightforwardly geodetic, standard for "referencing by coordinates".

Feel free to discuss the contents [here](https://github.com/busstoptaktik/geodesy/discussions/116), or by e-mailing me directly.

## Introduction

"Referencing by coordinates", the title and subject of this standard, is the term used for the art of establishing the correspondence between tuples of numbers (colloquially called "coordinates"), and physical positions in, on, or near a celestial body.

"Georeferencing" is the term used in the important special case where that celestial body is the earth.

In the following we will mostly use earth-based terms and expositions. The applicability to other celestial bodies will in general be implied. Earth-based terms are chosen due to the practical importance of georeferencing and due to the earth-native vocabulary and body-of-experience of the earth-dwelling primary audience of the standard.

### Coordinate systems and reference frames

In abstract terms, the relation between coordinates and positions is based on mathematial coordinate systems. The earth, however, is not an abstract mathematical object, but a concrete physical body. Hence, in order to relate coordinates to positions we also need *physical realizations* of the coordinate systems.

Historically such physical realizations (called "reference frames") consist of collections of physical markers in the landscape, with corresponding coordinates, computed from observations and adopted by convention. In modern cases, the coordinates may be supplemented with velocity vectors, and the physical markers and their coordinates may be represented by satellites and their ephemerides.

### Mathematical abstractions and empirical contraptions

Establishing reference frames is out-of-scope for this standard: it is a subject of the science and art of geodesy. This standard is limited to providing guidance to how the field of geomatics may utilize the results of geodesy. To do this, some limited understanding of geodesy is important. First and foremost understanding the fact that geodetic reference frames are *empirical contraptions* rather than mathematical abstractions.

Modern reference frames are typically realizations of the International Terrestrial Reference System (ITRS), as described in ISO 19161-1. In abstract terms, ITRS is a 3D cartesian system, with origin at the earth's center-of-mass, and oriented along the earth's axis of rotation. Hence, in abstract terms, i.e. if geodetic reference frames were ideal mathematical coordinate systems, the transformation between any pair of ITRS realizations would be the identity operation, as the frames would be exactly identical. In reality, however, the ITRS realizations differ, especially due to longer observation time series, better observation coverage, and improved processing strategies as the field matures.

As realizations of ITRS are not exactly identical, the realizations in the International Terrestrial Reference **Frame** (ITRF) series come with associated *transformations* between consecutive versions. But such transformations do *not* establish exact mathematical relations between the frames. Since the frames are empirical, so are the transformations: Rather than *defining* one frame in terms of another, they implement (mostly very reliable) *predictions* of "given coordinates surveyed with respect to frame A, what coordinates would have been obtained if instead surveyed with respect to frame B".

### Frames are dynamic - transformations are empirical

Also, the frames are not only empirical. As the earth is dynamic, so are the frames: The immediate coordinate of a physical marker is given by a reference coordinate at a fixed time, the "epoch" of the system, offset by the corresponding velocity vector, multiplied by the duration since the epoch. Correspondingly, the transformations are dynamic, and the above-mentioned frames "A" and "B" might as well be the same frame at different instants of time.

Hence, **transforming coordinates does not change the georeference**: The connection beween the coordinates and the physical earth is still given through the frame employed at time of data capture.

Transforming coordinates between frames (or times) does, however, to a larger or smaller degree, *align* coordinates in one frame with coordinates in another, enabling the integrated use of heterogenous sets of coordinates, in cases where the accuracy of the transformation is sufficient.

In the ITRF cases, this is often the case, as the transformations typically replicate re-surveys to better than typical survey accuracy. This is, however, seldom the case if transforming between modern frames and older national or regional frames, such as ED50 or NAD27, pre-dating the satellite geodesy era.

### Geographical coordinates

The above-mentioned 3D cartesian system is seldom useful near the surface of the earth, where we instead introduce the well known geographical grid of parallels and meridians, enabling the use of latitude/longitude/height-pairs for the georeference. Like the cartesian system, the geographical grid has its origin in the center-of-mass of the earth, although mediated by a reference ellipsoid with that origin. Contrary to the transformation between different reference frames, the transformation between cartesian and geographical coordinates is exact: Cartesian and geographical+height coordinates carry the same information, related to the same reference frame.

### Projected coordinates

For visualization purposes, neither 3D cartesian, nor geographical coordinates are particularly useful. Visualization typically takes place  via a 2D medium: A screen or a sheet of paper. To this end, *projected* coordinates are employed. Multitudes of projections exist, but common to them all is the problem that a double-curved surface as the earth's cannot be mapped perfectly to a plane: One may obtain proper mapping of either angles (by employing *conformal* projections), or areas (by employing equal area projections), but not of both at once. Hence, one must select a projection fit for the task at hand: Conformal projections for topographical mapping, equal-area projections for thematical mapping.

Most projections, however, are invertible: When mapping from geographical coordinates to projected coordinates, we may also map the other way round. Hence, as in the cartesian-to-geographic case, we do not lose any information, and do not change reference frame, by converting coordinates in this way, as long as we take care of keeping the height (and time) information unchanged in the process.

### Linear or angular, but not euclidean

Hence, as long as we stay within a given reference frame, it does not matter whether our coordinates are represented as cartesian, geographic, or projected: We keep the same mathematically identical information, and the same accuracy. Once transforming between frames, however, we will always lose accuracy, due to the empirical nature of reference frames.

For the same reason, while the coordinates are given in either angular (geographic) or linear (cartesian, projected) units, they are, however, not represented in a truly euclidean space: Due to the difference ("tensions") between the abstract coordinate systems, and the empirical reference frames, the apparent distance or angle between points, as derived from frame coordinates, will never exactly (but in most cases very closely) match the distance or angle observed in the field.

For projected coordinates, due to the fundamental mathematical limitations in mapping, additional deviations occur, which (contrary to the tensions) can be accounted for, due to the exact mathematical correspondence between geographical and projected coordinates.

### Non-linear heights

#### Gravity related heights

Prior to the satellite geodesy era, levelling was the primary way of measuring heights. But rather than actual heights, levelling fundamentally provides potential differences in the gravity field, between the target point and a geophysical equipotential reference surface called the geoid.

The difference between levelling based heights and geometric heights is non-trivial, but to excellent accuracy, the levelled heights can be approximated from geometric heights, by subtracting the separation between the geoid and the reference ellipsoid. This separation is typically obtained from interpolation in a gridded model.

Contrary to geometrical heights, the levelling based heights (and their derived approximations) always reflect the direction of water flow from higher to lower values, by incorporating local variations in the gravity field into the numerical value of the height.

#### Pressure related heights

In meteorology and oceanography, pressure levels are often useful for conceptual as well as dynamical modelling. Like the above-mentioned gravity related heights, pressure levels are mostly monotonic, i.e. the value decreases as one ascends through the model levels. Contrary to the levelling case, the pressure levels are typically measured in highly vivid media, making the direct conversion into linear units even more non-trivial.

In general these kinds of non-linear height units are known as *parametric* heights, and their proper handling requires specific domain knowledge.

### Summary

In summary, the georeference is fundamentally empirical, and height and time are as important as planar coordinates. Coordinates are given in linear, angular, or parametric units. But although the linear and angular units may seem to be euclidean, due to their relation to an empirical reference frame, they are not entirely so.

Transformations between reference frames, like the reference frames themselves, are fundamentally empirical. They implement a prediction of what one would have obtained working in one reference frame, given results obtained working in another. While expressed in mathematical terms, they do not represent an exact mathematical *relation* between the given reference frames. Hence, transformations may align one set of coordinates to another, but the transformed set will lose spatial accuracy, and fundamentally the georeference does not change.
