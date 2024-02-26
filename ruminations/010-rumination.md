# What's wrong with ISO-19111?

**Thomas Knudsen,** <thokn@sdfi.dk>, 2024-02-26

## Why?

The most recent edition of ISO-19111 "Referencing by coordinates" was published in 2019. Hence, according to ISO's 5 year life cycle of standards, 19111 is up for consideration-of-revision in 2024.

The following is my input to DS/S-276 (the Danish national committee of [ISO Technical Committee 211](https://www.isotc211.org/) "Geographic information/Geomatics"),  for discussion of that subject,

It is initially published here, as a part of the Rust Geodesy [Ruminations](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/README.md), as it is by and large a result of my work with [Rust Geodesy](https://github.com/busstoptaktik/geodesy) as a demonstration platform, outlining a road towards a simpler, leaner ISO-19111.

The text is long, and the subject both sprawling and convoluted. But the gist of it is, that:

- The original conceptual model behind 19111 was mostly in disagreement with the common geodetic world view. But it was simple and sufficient as long as nothing but metre-level absolute accuracy was required
- As accuracy requirements grew, this non-geodetic conceptual model was not feasible anymore, and the 19111 model had to get into closer agreement with modern geodesy
- The 2019 edition comprises an enormous leap in this direction, but there is still more work worth doing
- Also, a number of concepts are too imprecise or too narrowly defined, and hence should be reconsidered

## Introduction

As seen from the perspective of someone who narrowly escaped becoming part of the CEN TC-287 standardisation group at its inception back in the 1990s, and first started participating in ISO TC-211 during the work towards the 2019 version, ISO 19111 appears **to come from** a conceptual world view that had very little to do with geodesy, but made much sense for 1995-era coordinate users.

With steadily increasing accuracy requirements, and with the ubiquity of GNSS, the conceptual world view of that era has long ago stopped being generally feasible. And with 19111(2019), the standard took a huge leap towards a more geodetically realistic, while still end user applicable, conceptual world view.

In my humble opinion, it is, however, still possible to take further steps in this direction, so it is my hope that an upcoming revision of 19111 will take some of these steps, in addition to the obvious task of repairing bugs, limitations and/or inaccuracies.

Also, as will hopefully beome clear in the following, such steps may lead towards great conceptual simplification, by not having to paper over differences between the conceptual world view and the geodetic realities. Perhaps, we may deprecate, and even (in a later revision) entirely eliminate these aspects.

axiomatic empirical
opinion

Not trying to provide solutions - only pointing out aspects that could use improvement.

## Item 1: The concept of "coordinate transformations" is *way* too underexposed

In section 3.1 "Terms and definitions", the two notes to item 3.1.12 "coordinate transformation" comprises the entire geodetic justification for 19111

**3.1.12 coordinate transformation:**
coordinate operation that changes coordinates in a source coordinate reference system to coordinates in a target coordinate reference system in which the source and target coordinate reference systems are based on different datums

- Note 1 to entry: A coordinate transformation uses parameters which are derived empirically. Any error in those coordinates will be embedded in the coordinate transformation and when the coordinate transformation is applied the embedded errors are transmitted to output coordinates.

- Note 2 to entry: A coordinate transformation is colloquially sometimes referred to as a 'datum transformation'. This is erroneous. A coordinate transformation changes coordinate values. It does not change the definition of the datum. In this document coordinates are referenced to a coordinate reference system. A coordinate transformation operates between two coordinate reference systems, not between two datums.

## Item 2: `CoordinateSet` is not sufficiently useful

In 19111, `CoordinateSet` is the fundamental interface to actual data (cf. fig 5, sect. 7.4).

The `CoordinateSet` interface models something that, in practical implementations would be e.g. *a stack* or *an array* of `CoordinateTuple`s combined with a link to the relevant `CoordinateMetadata`.

The `CoordinateMetadata` consists of either a `CRSid` or a `CRS`, and (if the CRS is dynamic) a `coordinateEpoch`.

Now, what's wrong with that? Quite a bit, actually...

**First:** The `coordinateTuple` element is, with a reference to 19107, defined as an ordered set [1..*] of `DirectPosition`s. In other words, *an empty set of coordinate tuples is not allowed*.

For practial use cases, this is unfortunate, since one must start somewhere, and for observational time series (or for iteratively computed, derived data sets), you start without anything: The data structure, with pointers to metadata and backing memory is instantiated **prior to** the first observation!

Hence, `[1..*]` should be `[0..*]`.

**Second:** Since the data item `coordinateTuple` is really an ordered set of coordinate tuples, the plural form `coordinateTuples` would be more enlightening.

**Third:** the 19107 `DirectPosition` device, which (as seen from this observer's vantage point), is rather obscure, at least is clear enough to allow one to conclude, that it refers to something entirely and exclusively *spatial*.

But anything derived one way or another from GNSS-observations, is inherently *spatio-temporal*. This should obviously be supported directly by the `CoordinateSet` interface.

**Fourth:** The `CoordinateSet` interface repairs on this missing property by pushing the chronoreference to the metadata-interface, where at most one epoch is allowed.

I have a very hard time trying to construct a practical use case for this. GNSS-time series consist of observations at epoch `(t_0, t_1,.., t_n)`, and each observation is referred to the dynamic reference frame *at the observation epoch*.

**Hence:** No one in their right mind would ever transform their observations to a common epoch, and throw away the time component (making it impossible to transform back to the actual observation).

*Nevertheless, this is the use case `CoordinateSet` is built for!*

**Proposal:** Could we cut the ties to 19107, and let it drift its unmoored way out over the horizon? In my opinion, 19111 is the anchor, that ties the entire 19100 series to the physical reality - it is *not* "turtles all the way down", so we do not need to build `CoordinateTuples` on top of 19107-`DirectPosition`s.

If anyone cares about 19107, they could revise it to make it the other way round: `CoordinateTuples` can perfectly well be of any dimension, including temporal, so 19107-ish `DirectPosition`s could be their restriction to the spatial domain.

**In continuation: Do we actually have a way of expressing CRS `foo` to the observation epoch?**

To me, it doesn't look like. Please convince me that it can be done.

The entire case looks a bit like say, ETRS89, which by definition coincides with ITRS (or rather, the corresponding frames) at the 1989.0 epoch, but in that case, we're talking of two different reference systems, and the epoch is an implementation detail.


## Item 3: The 'S' in CRS is misleading

Geodetically, a System is an abstraction, "the recipe for constructing a reference frame". It is *not* possible to refer a coordinate to an abstraction. One needs a realization of the abstraction, i.e. a reference **frame**.

All geodesists know this, but most geodesy users do not. So the 'S is for System' in CRS subverts communication.

The concept of a CRS (as a brief way of referring to a potentially huge hodge-podge of conventional as well as empirical parameters and operations) is, however, quite useful: EPSG ids are way more *communicable* than the full story.

But since a CRS is not a system, could we find a reasonable alternative expansion of the CRS acronym, replacing "Coordinate Reference System"?

**Proposal:** *Coordinate Reference **Specifier*** seems reasonable to me.

## Item 4: The concept of a CRS leads to unnecessary complication

According to 19111, a CRS has a "definition"
The typical CRS today, consists of a reference frame plus some kind of coordinate operation


## Item 5: `DatumEnsemble` is too narrowly defined

A datum ensemble is used when the accuracy requirements are sufficiently lax to allow mixing coordinates from any of a range of reference frames (e.g. any realization of WGS84, or any of the individual national realizations of the European system ETRS89).

Typically we name the ensemble-of-reference-frames after a common abstract reference **system** behind the individual realizations, i.e. for the examples just given, WGS84 and ETRS89, respectively.

But systems come before realizations, and the first realization of  a (planned) series is also part of the series. So the current definition of DatumEnsemble as `Datum [2..*]` should be either `[1..*]` or `[0..*]`.

## Item 6: Support pipelines of operations

## Item 7: Operations are underspecified, and the definitions given are potentially misleading

Coordinate operations (and their parameters) are more thoroughly described in 19157 (WKT) and in EPSG Guidance Note 7-2. Especially the latter is a wonderfully accessible resource, for understanding and implementing coordinate operators.

That level of detail and specificity is not appropriate for 19111. But it is probably possible to give more precise, and better articulated definitions of the three interrelated concepts of "coordinate transformation", "coordinate conversion" and "coordinate operation".

Especially, it is not sufficiently clear that the discrimination between transformations and conversions are related to whether the parameters of the operation are *formally defined* (conversion) or *empirically derived* (transformation). In other words: Any operation may implement either a conversion or a transformation, depending on the lineage of their parameters.

Additional value could be provided by describing the cases of reversible and irreversible operations

## Further reading

### Geodesy ruminations

(Start with Rumination 009)

- [Rumination 000](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/000-rumination.md): Overall architecture and philosophy
- [Rumination 001](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/001-rumination.md): A few words about an often-seen pipeline
- [Rumination 002](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/002-rumination.md): The missing manual
- [Rumination 003](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/003-rumination.md): kp - the RG Coordinate Processing program
- [Rumination 004](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/004-rumination.md): Why Rust Geodesy - some background
- [Rumination 005](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/005-rumination.md): Divided by a common language
- [Rumination 006](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/006-rumination.md): Still confused, but at a higher level
- [Rumination 007](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/007-rumination.md): Operator parameter introspection
- [Rumination 008](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/008-rumination.md): Geodesy from a PROJ perspective
- [Rumination 009](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/009-rumination.md): Teach yourself Geodesy in less than 900 seconds (of arc)
