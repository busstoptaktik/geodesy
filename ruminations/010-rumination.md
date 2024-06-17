<!-- markdownlint-disable MD013 -->

# What's wrong with ISO-19111?

**Thomas Knudsen,** <thokn@sdfi.dk>, 2024-02-26/05-07

## Motivation

The most recent edition of ISO-19111 "Referencing by coordinates" was published in 2019. Hence, according to ISO's 5 year life cycle of standards, 19111 is up for consideration-of-revision in 2024. The following is my input for these considerations to [DS/S-276](https://www.ds.dk/da/udvalg/kategorier/it/geografisk-information), the Danish national committee of [ISO Technical Committee 211](https://www.isotc211.org/) and [CEN TC 287](https://standards.cencenelec.eu/dyn/www/f?p=205:7:0::::FSP_ORG_ID:6268&cs=1D5368A4F6E101B66AD14AB12AC0FC914).

The material is initially published here, as a part of the Rust Geodesy [Ruminations](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/README.md), since it is by and large a result of my work with [Rust Geodesy](https://github.com/busstoptaktik/geodesy) as a demonstration platform, outlining a road towards a simpler, leaner ISO-19111.

The text is long, and the subject both sprawling and convoluted. But the gist of it is, that:

- The original conceptual model prior to 19111 was mostly in disagreement with the common geodetic world view. But it was simple and sufficient as long as meter-level absolute accuracy was acceptable
- As accuracy requirements grew, this non-geodetic conceptual model was not feasible anymore, and the model, in its 19111 incarnation, is in closer agreement with common geodetic world views
- Hence, 19111 is not terrible (there's an entire, successful industry based on it, after all), but there is still more work worth doing
- Also, a number of concepts are still either too vaguely or too restrictively defined, and hence should be revised

**Note that** while some of the changes suggested below, may seem both extensive and disruptive at first glance, they are actually rather clarifications than substantial changes. The aim is to support communication with end users and developers, through better alignment between geomatics and geodesy, and to future-proof the terminology for a time where millimeter-scale positional accuracy will be a common expectation. The changes suggested should require minor-to-no changes to software implementations of the standard.

## Introduction

### Point-of-view

With the personal luck of (narrowly) escaping becoming part of the geospatial standardization efforts at their inception back in the 1990's, I first started participating in the work around ISO-19111 "Referencing by Coordinates" when its 2019 revision was well under way.

Hence, my impression of the conceptual world view behind early (and especially pre-19111) geospatial standardization is based on anecdotal evidence - although largely supported by excavation of archaeological traces still visible in 19111.

With only a slight dose of exaggeration, that world view can be described in brief as follows:

> Geodetic coordinate systems, like their mathematical namesakes, are built on an axiomatic foundation. For geodetic coordinate systems, this foundation is an *eternal, immutable ether* called WGS84. And **ANY** coordinate system can be strictly **defined** as a Helmert transformation from WGS84.

While superficially nonsensical, there was a time, when this world view was quite reasonable: It is simple to implement and sufficiently accurate if the expected georeference accuracy is at the meter level, as it often was in the 1990's.

But with steadily increasing accuracy requirements, and with the ubiquity of GNSS, the conceptual world view of that era has long ago ceased being generally feasible. And with 19111, we have a geodetically more realistic, while still end user applicable, conceptual world view.

### Directions

In my humble opinion, it is, however, still possible to take further steps toward geodetic realism, so it is my hope that an upcoming revision of 19111 will take some of these steps, in addition to the obvious tasks of repairing bugs, relaxing constraints, and clarifying ambiguities.

Also, as will hopefully become clear in the following, such steps may lead toward great conceptual simplification, by not having to paper over differences between the conceptual world view and the geodetic realities. Perhaps, we may deprecate, and even (in a later revision) entirely eliminate these aspects.

Below, I try to identify a number of conceptual problems, some needing much discussion, some more immediately actionable. So except for a few cases, I will not present ready-baked solutions, rather try to inspire discussion. Not only discussion of the specific matters, but also the overall problem that ISO 19111 is way too careful in its language.

### Divided by a common language

As 19111 (along with 19161) describes the relation between coordinates (i.e. numbers), and locations (i.e. the physical world), 19111 should speak in geodetic and hence empirical terms. As elaborated under Item 0 below, there is no axiomatic highway towards the georeference. The georeference is fundamentally geodetic and empirical, so 19111 needs to bridge the gap between geodesy and geomatics - in other words, 19111 must "speak geodesy".

But when we communicate as **geodesists**, we communicate about the physical world, so we tend to get away with being linguistically sloppy, since physical reality and human conception are magnificent disambiguators.

When we communicate as **geomaticians**, on the other hand, we must be conceptually and linguistically more strict, since we concern ourselves with feeding the bit-crunching monsters, which possess neither imagination, nor reason.

Bridging the gap between contextual sloppiness, and context free rigor is no simple feat. Building a sufficiently rich terminology and understanding may very well take yet another few decades. But while that materializes, at least we can try to maintain, trim and focus 19111, making sure it doesn't buckle under its own load *en route*.

## Item 0: Empirical contraptions vs. axiomatic idealizations

It is still overly underexposed in 19111, that geodetic reference frames are **empirical contraptions**, while geometric coordinate systems are **axiomatic idealizations**. Hence, the only way to establish a connection from the abstract coordinate tuples, to the concrete physical world, is through a reference frame squarely embedded in that physical world.

So to more clearly express this, 19111 should stop talking about coordinates referenced to metadata (as in [Figure 3](https://docs.ogc.org/as/18-005r4/18-005r4.html#figure_3)), but rather make it clear that e.g:

- Coordinates are surveyed and adjusted *according* to rules given in reference **system** definitions, but *referenced* to reference **frames**.
- The georeference does not change when a transformation is applied. But through the transformation, the data referenced to reference frame **A** may be made somewhat more interoperable ("aligned") with those referenced to frame **B**.
- Transformations are *empirical predictions*, not magic wands conjuring up new georeferences without having to do the surveys.
- Geodetic reference frames are given as coordinate- and velocity lists (or, equivalently in the satellite navigation case: as ephemerides), not as orthogonal unit vectors in an idealized vector space.

**19111 ties coordinates to the physical reality,** and hence marks the point where geomatics standardization must transcend the abstractions, and tie into the messy, empirical real world. This is the entire *raison d'etre* for 19111, and we should not be ashamed of that.

## Item 1: The concept of "coordinate transformations" is *way* too underexposed

In section 3.1 "Terms and definitions", the two notes to entry 3.1.12 "coordinate transformation" comprises most of the geodetic justification for 19111

> **3.1.12 coordinate transformation:**
> coordinate operation that changes coordinates in a source coordinate reference system to coordinates in a target coordinate reference system in which the source and target coordinate reference systems are based on different datums
>
> - Note 1 to entry: A coordinate transformation uses parameters which are derived empirically. Any error in those coordinates will be embedded in the coordinate transformation and when the coordinate transformation is applied the embedded errors are transmitted to output coordinates.
>
> - Note 2 to entry: A coordinate transformation is colloquially sometimes referred to as a 'datum transformation'. This is erroneous. A coordinate transformation changes coordinate values. It does not change the definition of the datum. In this document coordinates are referenced to a coordinate reference system. A coordinate transformation operates between two coordinate reference systems, not between two datums.

Let's dig deeper into this under item 7 below, but in the meantime, let's look at a few easier-to-handle insufficiencies of 19111(2019):

## Item 2: `CoordinateSet` is vaguely defined, and not sufficiently useful

In 19111, `CoordinateSet` is the fundamental interface to actual data (cf. [figure 5](https://docs.ogc.org/as/18-005r4/18-005r4.html#figure_5), sect. 7.4).

The `CoordinateSet` interface models something that, in practical implementations, would be e.g. *a stack, list, or array* of `CoordinateTuple`s combined with a link to the relevant `CoordinateMetadata`.

The `CoordinateMetadata` consists of either a `CRSid` or a `CRS`, and (if the CRS is dynamic) a `coordinateEpoch`.

Now, what's wrong with that? Quite a bit, actually...

**First:** `CoordinateSet` is an interface for accessing `coordinateTuple` elements which, with a reference to 19107, are defined as an ordered set [1..*] of `DirectPosition`s. In other words, *an empty set of coordinate tuples is not allowed*.

For practical use cases, this is unfortunate, since one must start somewhere, and for observational time series (or for iteratively computed, derived data sets), we start without anything: The data structure, with pointers to metadata and backing memory is instantiated **prior to** the registration of the first observation!

Hence, `[1..*]` should be `[0..*]`.

**Second:** Additionally, the data type should probably be a sequence, not an ordered set: The reasonable intention is to model an *array*-like item, i.e. something that can be read and handled in indexed order. An ordered *set* implies that the material is ordered with respect to some **intrinsic property** of the elements of the set - which is simple for numerical data in one dimension, but not in two or more (where lexical ordering by dimension may stop the gap, but not in any terribly useful way: For continuous data it is effectively identical to sorting along the first dimension)

**Third:** the 19107 `DirectPosition` device, which (as seen from this observer's vantage point), is rather obscure, at least is clear enough to allow one to conclude that it refers to something entirely and exclusively *spatial*.

But anything derived one way or another from GNSS-observations, is inherently *spatio-temporal*. So this should obviously be supported directly by the `CoordinateSet` interface.

**Fourth:** The `CoordinateSet` interface repairs on this missing property by pushing the chronoreference to the metadata-interface, *where at most one epoch is allowed!*

I have a very hard time trying to construct a practical use case for this. GNSS-time series consist of observations at epoch `(t_0, t_1,.., t_n)`, and each observation is referenced to the dynamic reference frame *at the observation epoch*.

**Hence:** No one in their right mind would ever transform their observations to a common epoch, and throw away the time component (making it impossible to transform back to the actual observation).

*Nevertheless, this is the use case `CoordinateSet` is built for!*

**Proposal:** Could we cut the ties to 19107, and let it drift its unmoored way out over the horizon? In my opinion, 19111 is the anchor, that ties the entire 19100 series to the physical reality - it is *not* "turtles all the way down", so we do not need to build `CoordinateTuples` on top of 19107-`DirectPosition`s: Geodesy (and hence 19111) is the foundation that ties the abstract coordinates to the physical reality.

So if anyone actually cares about 19107, let them revise it to make it the other way round: `CoordinateTuples` can perfectly well be of any dimension, including temporal, so 19107-ish `DirectPosition`s could be their restriction to the spatial domain.

**In continuation: Do we actually have a way of expressing CRS `foo` to the observation epoch?**

Apparently this is impossible. If true, this is clearly a missing feature. The entire case looks a bit like say, ETRS89, which by definition coincides with ITRS (or rather, their corresponding frames do) at the 1989.0 epoch, but in that case, we're talking of two different reference systems, and the epoch is an implementation detail.

## Item 3: The 'S' in CRS is misleading

Geodetically, a System is an abstraction, "the recipe for constructing a reference frame". It is *not* possible to refer a coordinate to an abstraction. One needs a realization of the abstraction, i.e. a reference **frame**.

All geodesists know this, but most geodesy users do not. So the 'S is for System' in CRS subverts communication.

The concept of a CRS (as a brief way of referring to a potentially huge hodge-podge of conventional as well as empirical parameters and operations) is, however, quite useful: EPSG ids are way more *communicable* than the full story.

But since a CRS *is not a system,* could we find a reasonable alternative expansion of the CRS acronym, replacing "Coordinate Reference System"?

**Proposal:** *Coordinate Reference **Specifier*** or *Coordinate Reference **Selector*** both seem reasonable to me, but I'm sure native English speakers can come up with better alternatives.

## Item 4: The CRS concept leads to unnecessary complication

According to 19111, a CRS has a "definition". But at the bottom of any CRS is a reference frame. And as argued above, a reference frame is empirical, hence irreducible and non-definable.

So the concept that "a CRS has a definition, and from the definition, we can infer transformations to other CRS" is highly limited: It works as long as we stay within the same reference frame, but no longer than that.

But modulo the reference frame ("Base CRS"), the "definition of a CRS" is just the operation going from the CRS back to the reference frame. For this, we introduce an entire class of new concepts (perhaps most of [chapter 9](https://docs.ogc.org/as/18-005r4/18-005r4.html#27)), essentially covering the same ground as if just associating an operation with the CRS.

This was covered more extensively in [this 2021 discussion](https://github.com/OSGeo/PROJ/issues/2854) where I a.o. opined that:

> The confusion of these matters is encouraged by the mistaken foundation of the ISO/OGC geospatial standards series, which somehow asserts that a CRS is definable in absolute terms. Being able to define a CRS in absolute terms would be nice, since once you have an absolute definition of two CRS', you would be able to determine an infallible transformation between the two.
>
> That's possible in mathematics, where coordinate systems are Platonic ideals. In geodesy, coordinate systems are much more messy: You can only **define** a reference system, by writing a book describing how to **realize** that system on the physical Earth.
>
> The **reference system** *is the book*. The realization is the corresponding **reference *frame***. And the reference frame (i.e. a collection of physical points with associated coordinate and velocity information) is what you can measure point coordinates with respect to.
>
> So the definition (i.e. the book) may guide us toward constructing a transformation involving a given CRS/reference frame. But we cannot determine any coordinates of physical features with respect to the **system** - only with respect to the **frame**.
>
> The projected CRS `EPSG:3395` is related to the geographical base CRS `EPSG:4326` by the coordinate operation described by `proj=merc ellps=WGS84` (or actually its inverse form). So the closest you can get to a "definition" of `EPSG:3395` is to say that `EPSG:3395` is the CRS for which coordinates gets related to `EPSG:4326` by subjecting them to the coordinate operation given by `inv proj=merc ellps=WGS84`. Or in other words: *The definition of a CRS is the coordinate operation which brings us to its base CRS*. Once we arrive at the base CRS, that's the end of the definition in absolute terms: You have arrived from your trip from the platonic gardens of mathematics to the messy moors of geodesy.
>
> To continue the journey from there and onto another base CRS, you will have to rely on empirically determined transformations - you are in the waste lands of approximations, where a meter is not a meter, a radian not a radian, and the distance between two points is not the same as the difference between their coordinates. Welcome to geodesy!
>
> In one sense, however, things are much simpler in geodesy: A CRS is really just a label, without any internal state. While that sounds strange comming from the ISO/OGC world, it really simplifies a lot of things, since that label is the key to looking up the transformation to any other CRS in **the little black book all geodesists are secretly given upon graduation** (or, having lost the book: Looking it up at the [EPSG](https://epsg.org) website).

## Item 5: `DatumEnsemble` is too narrowly defined

A datum ensemble is used when the accuracy requirements are sufficiently lax to allow mixing coordinates from any of a range of reference frames (e.g. any realization of WGS84, or any of the individual national realizations of the European system ETRS89).

Typically we name the ensemble-of-reference-frames after a common abstract reference **system** behind the individual realizations, i.e. for the examples just given, WGS84 and ETRS89, respectively.

But systems come before realizations, and the first realization of  a (planned) series is also part of the series. So the current definition of DatumEnsemble as `Datum [2..*]` should be either `[1..*]` or (preferably) `[0..*]`.

## Item 6: Support pipelines of operations

Coordinate operations can be used to align datasets from different reference frames. But often a series of commonly-implemented operations (e.g. operations from the gamut of EPSG Guidance Note 7-2), is needed to implement the alignment between two CRS. Also, what comprises an operation is by-and-large arbitrary: A projection operator including false easting and northing yields identical results to a false-origin-free version followed by a linear transformation.

While ISO-19111 allows operation concatenation, it does so only in cases where intermediate CRS exist for every step. This is quite impractical, and we should try to establish a way of more directly supporting operation pipelines.

## Item 7: Operations are underspecified, and the definitions given are potentially misleading

Coordinate operations (and their parameters) are more thoroughly described in 19162 (WKT) and in EPSG Guidance Note 7-2. Especially the latter is a wonderfully accessible resource, for understanding and implementing coordinate operators.

That level of detail and specificity is not appropriate for 19111. But it is likely possible to give more precise, and better articulated definitions of the three interrelated concepts of "coordinate transformation", "coordinate conversion" and "coordinate operation".

In particular, it is not sufficiently clear that the discrimination between transformations and conversions are related to whether the parameters of the operation are *formally defined* (conversion) or *empirically derived* (transformation). In other words: Any operation may implement either a conversion or a transformation, depending on the lineage of their parameters.

**As an example,** the Transverse Mercator operation is usually considered a conversion, while the Helmert operation is usually considered a transformation (implementing rotation, translation, and scaling, through empirically derived parameters).

The rotation, translation, and scaling can however, also be implemented by empirical manipulation of the center meridian, false origin, and scale parameters of the Transverse Mercator operator.

Hence, the difference between conversions and transformations is not in their algorithmic definition, but in the *lineage of their associated parameters*. This is actually hinted at in the (informative, not normative) [Annex C.5](https://docs.ogc.org/as/18-005r4/18-005r4.html#98), and in a rather indirect way, as mentioned in [Item 1](#item-1-the-concept-of-coordinate-transformations-is-way-too-underexposed), in a note to sub-section **3.1.12 coordinate transformation** in the [*Terms and definitions*](https://docs.ogc.org/as/18-005r4/18-005r4.html#4) section.

Rather than being relegated to a remark in an annex, and a note in a sub-section, this distinction should be elaborated on at chapter or at least section level.

This also raises the question of whether it makes sense to discern formally between conversions and transformations. Fundamentally the difference is between whether the formal error when applying the operation is 0 or non-0.

What we colloquially call a datum shift is a prediction between two fundamentally **empirical** reference frames, and neither the provided ("source")  nor the predicted ("target") coordinate is known exactly - unless, of course, the provided coordinate is one of the defining stations of the source reference frame.

In the general case, however, the predicted coordinate will possess uncertainty from a combination of the original variance and the prediction-variance. Neither of which are easy to formalize in operational 19111 terms.

But the conceptualization of operations, conversions and transformations does not make it easier in any way to start this discussion, which becomes increasingly important as the demand for positional accuracy increases.

Hence, we could simply talk about operations, as a first step towards tackling the problem of operationalizing the representation of variance propagation in geodetic ~~transformations~~ conversions. A **potential terminology** could be:

- We **convert** coordinates to **align** them with a different reference frame (or a different CRS based on the same reference frame, in which case the alignment will be perfect).
- We do so by applying an **operator** (which may be implemented as a pipeline of more fundamental operators)
- The operator, parameterized by its parameters (which may be defined or derived) implements the **operation** applied to the source coordinates.
- The target coordinates still have exactly the same accuracy with respect to the **source** reference frame (since, knowing the lineage of the coordinates, we may take the operator parameters to be given *by definition*, and transform back to the source coordinates).
- With respect to the **target reference frame,** however, the coordinates have a worse accuracy, due to the combined variance effects of observation and prediction.

**Finally,** note that an enhanced specification of operations could provide **additional value** by more clearly describing the relation between reversible operations and their inverses. In the current state of affairs, the conversion from A to B and that from B to A are just two unrelated operators.

## Conclusion?

No - but perhaps we should vote for revision!

## Et cetera

### Regarding 19107

As noted by [Martin Desruisseaux](https://github.com/desruisseaux) in a remark on the [discussion page](https://github.com/busstoptaktik/geodesy/discussions/116) for this text, the 19111 references to 19107 should be to the 2003 edition. This is actually also the edition referred in the bibliography chapter of 19111, but not in the main text, where an unqualified 19107 (and hence implied "latest edition") is used. This should probably be amended.

## Further reading (on Rust Geodesy)

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

<!--
## Endnotes

**In the light of that world view,** when the apparent center of mass, related to the ED50 datum differs by approximately 200 m from that of WGS84, then it's because the Wise Fathers of ED50 had figured *"wouldn't it be nice with a coordinate system somewhat offset from the earth's centre-of-mass?".*

So they equipped an expedition, and went underground to locate the earth's centre-of-mass. Once found, they surveyed an exactly defined differential distance from there, drove a stake into the earth's inner core at exactly that position, and declared with celebration: **"From here, we will survey our continent".**

problemet med kommunikation når standarden ikke svarer til geodæsien, og kommunikationen bliver stedse vigtigere når nøjagtighedskravene bliver større

De praktiske ændringer i implementeringer vil være minimale, men kommunikationen med slutbrugere vil være simplere
-->
