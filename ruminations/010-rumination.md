<!-- markdownlint-disable MD013 -->

# What's wrong with ISO-19111?

**Thomas Knudsen,** <thokn@sdfi.dk>, 2024-02-26

## Motivation

The most recent edition of ISO-19111 "Referencing by coordinates" was published in 2019. Hence, according to ISO's 5 year life cycle of standards, 19111 is up for consideration-of-revision in 2024. The following is my input for these considerations to [DS/S-276](https://www.ds.dk/da/udvalg/kategorier/it/geografisk-information), the Danish national committee of [ISO Technical Committee 211](https://www.isotc211.org/) and [CEN TC 287](https://standards.cencenelec.eu/dyn/www/f?p=205:7:0::::FSP_ORG_ID:6268&cs=1D5368A4F6E101B66AD14AB12AC0FC914).

The material is initially published here, as a part of the Rust Geodesy [Ruminations](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/README.md), since it is by and large a result of my work with [Rust Geodesy](https://github.com/busstoptaktik/geodesy) as a demonstration platform, outlining a road towards a simpler, leaner ISO-19111.

The text is long, and the subject both sprawling and convoluted. But the gist of it is, that:

- The original conceptual model leading to 19111 was mostly in disagreement with the common geodetic world view. But it was simple and sufficient as long as metre-level absolute accuracy was acceptable
- As accuracy requirements grew, this non-geodetic conceptual model was not feasible anymore, and the model had to get into closer agreement with modern geodesy
- The 2019 edition of 19111 has come a long way, but there is still more work worth doing
- Also, a number of concepts are still either too vaguely or too restrictively defined, and hence should be revised

**Also note that** while some of the changes proposed may seem extensive at first glance, they are actually rather clarifications than substantial changes. The aim is to support communication with end users and developers, through better alignment between geomatics and geodesy. The changes should require minor-to-no changes to software implementations of the standard.

## Introduction

### Point-of-view

With the personal luck of (narrowly) escaping becoming part of the geospatial standardization efforts at their inception back in the 1990's, I first started participating in the work around ISO-19111 "Referencing by Coordinates" when its 2019 revision was well under way.

Hence, my impression of the conceptual world view behind early geospatial standardization is based on anecdotal evidence - although largely supported by excavation of archaeological traces still visible in 19111.

With only a slight dose of exaggeration, that world view can be described in brief as follows:

> Geodetic coordinate systems, like their mathematical namesakes, are built on an axiomatic foundation, an eternal, immutable ether called WGS84. And **ANY** coordinate system can be strictly defined as a Helmert transformation from WGS84.

While superficially nonsensical, this world view is actually quite reasonable: It is simple to implement and sufficiently accurate if the expected georeference accuracy is at the metre level, as it was in the 1990's.

But with steadily increasing accuracy requirements, and with the ubiquity of GNSS, the conceptual world view of that era has long ago ceased being generally feasible. And with 19111(2019), the standard represents a geodetically fairly realistic, while still end user applicable, conceptual world view.

### Directions

In my humble opinion, it is, however, still possible to take further steps toward geodetic realism, so it is my hope that an upcoming revision of 19111 will take some of these steps, in addition to the obvious tasks of repairing bugs, relaxing constraints, and clarifying ambiguities.

Also, as will hopefully become clear in the following, such steps may lead toward great conceptual simplification, by not having to paper over differences between the conceptual world view and the geodetic realities. Perhaps, we may deprecate, and even (in a later revision) entirely eliminate these aspects.

Below, I try to identify a number of conceptual problems, some needing much discussion, some more immediately actionable. So except for a few cases, I will not present ready-baked solutions, rather try to inspire discussion. Not only discussion of the specific matters, but also the overall problem that ISO 19111 is way too careful in its language.

### Divided by a common language

As 19111 (along with 19161) describes the relation between coordinates (i.e. numbers), and locations (i.e. the physical world), 19111 should speak in geodetic and hence empirical terms. As elaborated under Item 0 below, there is no axiomatic highway towards the georeference. The georeference is fundamentally geodetic and empirical, so 19111 needs to bridge the gap between geodesy and geomatics - in other words, 19111 must "speak geodesy".

But geodesists communicate about the physical world, so they tend to get away with being linguistically much more sloppy than geomaticians, since physical reality and human conception are magnificent disambiguators.

Geomaticians on the other hand, must be conceptually and linguistically more strict, since they concern themselves with feeding the bit-crunching monsters, which posess neither imagination, nor reason.

Bridging the gap between contextual sloppiness, and context free rigor is no simple feat. Reaching a common understanding may very well take yet another few decades. But while that understanding materializes, at least we can try to maintain, trim and focus 19111, making sure it doesn't buckle under its own load *en route*.

## Item 0: Empirical contraptions vs. axiomatic idealizations

It is still overly underexposed in 19111, that geodetic reference frames are **empirical contraptions**, while geometric coordinate systems are **axiomatic idealizations** and that the only way to establish a connection between the abstract coordinate tuples, and the concrete physical world, is through a reference frame squarely embedded in that physical world.

So to remedy this, 19111 should stop talking about coordinates referenced to metadata (as in [Figure 3](https://docs.ogc.org/as/18-005r4/18-005r4.html#figure_3)), but rather try to make it clear that e.g:

- Coordinates are surveyed and adjusted *according* to rules given in reference **system** definitions, but *referenced* to reference **frames**.
- The georeference does not change when a transformation is applied. But through the transformation, the data referenced to reference frame **A** may be made somewhat more interoperable ("aligned") with those referenced to frame **B**.
- Transformations are *empirical predictions*, not magic wands conjuring up new georeferences without having to do the surveys.
- Geodetic reference frames are given as coordinate- and velocity lists (or, equivalently in the satellite navigation case: as ephemerides), not as orthogonal unit vectors in an idealized vector space.

19111 ties coordinates to the physical reality, and should not be ashamed of that.

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

The `CoordinateSet` interface models something that, in practical implementations would be e.g. *a stack, list, or array* of `CoordinateTuple`s combined with a link to the relevant `CoordinateMetadata`.

The `CoordinateMetadata` consists of either a `CRSid` or a `CRS`, and (if the CRS is dynamic) a `coordinateEpoch`.

Now, what's wrong with that? Quite a bit, actually...

**First:** `CoordinateSet` is an interface for accessing `coordinateTuple` elements which, with a reference to 19107, are defined as an ordered set [1..*] of `DirectPosition`s. In other words, *an empty set of coordinate tuples is not allowed*.

For practical use cases, this is unfortunate, since one must start somewhere, and for observational time series (or for iteratively computed, derived data sets), we start without anything: The data structure, with pointers to metadata and backing memory is instantiated **prior to** the registration of the first observation!

Hence, `[1..*]` should be `[0..*]`.

**Second:** Additionally, the data type should probably be a sequence, not an ordered set: The reasonable intention is to model an *array*-like item, i.e. something that can be read and handled in indexed order. An ordered *set* implies that the material is ordered with respect to some **intrinsic property** of the elements of the set - which is simple for numerical data in one dimension, but not in two or more (where lexical ordering by dimension may stop the gap, but not in any terribly useful way: For continuous data it is effectively identical to sorting along the first dimension)

**Third:** the 19107 `DirectPosition` device, which (as seen from this observer's vantage point), is rather obscure, at least is clear enough to allow one to conclude, that it refers to something entirely and exclusively *spatial*.

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

According to 19111, a CRS has a "definition".
The typical CRS today, consists of a reference frame plus some kind of coordinate operation

[Figure 3](https://docs.ogc.org/as/18-005r4/18-005r4.html#figure_3) illustrates some of this.

TODO

<!--
Refererer til metadata, men geodæsi handler om at referere til virkeligheden. Det er 19111's mission - i modsætning til 19107. Og georeferencen er til en referenceramme, ikke til et sæt metadata.

En transformation er empirisk, og flytter ikke georeferencen til en anden ramme. Den implementerer en prædiktion ("hvilken koordinat X2 ville vi have opnået i system B, givet at vi har X1 i system A)

Derfor er figur 3 misvisende: Det sammensatte datasæt er ikke refereret til CM3 - men CS1 og CS2 er blevet gjort "noget interoperable" ved hjælp af dels en empirisk prædiktion (CS1), dels en aksiomatisk konvertering (CS2)

It is important that 19111 reflects how geodesy *actually* works. And "geodetic coordinate systems are not coordinate systems"
-->

## Item 5: `DatumEnsemble` is too narrowly defined

A datum ensemble is used when the accuracy requirements are sufficiently lax to allow mixing coordinates from any of a range of reference frames (e.g. any realization of WGS84, or any of the individual national realizations of the European system ETRS89).

Typically we name the ensemble-of-reference-frames after a common abstract reference **system** behind the individual realizations, i.e. for the examples just given, WGS84 and ETRS89, respectively.

But systems come before realizations, and the first realization of  a (planned) series is also part of the series. So the current definition of DatumEnsemble as `Datum [2..*]` should be either `[1..*]` or (preferably) `[0..*]`.

## Item 6: Support pipelines of operations

Coordinate operations can be used to align datasets from different referece frames. But often a series of commonly-implemented operations (e.g. operations from the gamut of EPSG Guidance Note 7-2), is needed to implement the alignment between two CRS. Also, what comprises an operation is by-and-large arbitrary: A projection operator including false easting and northing yields identical results to a false-origin-free version followed by a linear transformation.

While ISO-19111 allows operation concatenation, it does so only in cases where intermediate CRS exist for every step. This is quite impractical, and we should try to establish a way of more directly supporting operation pipelines.

## Item 7: Operations are underspecified, and the definitions given are potentially misleading

Coordinate operations (and their parameters) are more thoroughly described in 19157 (WKT) and in EPSG Guidance Note 7-2. Especially the latter is a wonderfully accessible resource, for understanding and implementing coordinate operators.

That level of detail and specificity is not appropriate for 19111. But it is likely possible to give more precise, and better articulated definitions of the three interrelated concepts of "coordinate transformation", "coordinate conversion" and "coordinate operation".

Especially, it is not sufficiently clear that the discrimination between transformations and conversions are related to whether the parameters of the operation are *formally defined* (conversion) or *empirically derived* (transformation). In other words: Any operation may implement either a conversion or a transformation, depending on the lineage of their parameters.

As an example, the Transverse Mercator operation is usually considered a conversion, while the Helmert operation is usually considered a transformation (implementing rotation, translation, and scaling, through empirically derived parameters).

The rotation, translation, and scaling can however, also be implemented by empirical manipulation of the center meridian, false origin, and scale parameters of the Transverse Mercator operator.

Hence, the difference between conversions and transformations is not in their algorithmic definition, but in the *lineage of their associated parameters*. This is actually hinted at in the (informative, not normative) Annex C.5, and in a rather indirect way, as mentioned in [Item 1](#item-1-the-concept-of-coordinate-transformations-is-way-too-underexposed), in a note to sub-section 3.1.12 "coordinate transformation" in the *Terms and definitions* chapter.

Rather than being relegated to a remark in an annex, and a footnote in a sub-section, this distinction should be elaborated on at chapter or at least section level.

It also raises the question of whether it makes sense to discern formally between conversions and transformations. Fundamentally the difference is beween whether the formal error when applying the operation is 0 or non-0.

What we colloquially call a datum shift is a prediction between two fundamentally **empirical** reference frames, and neither the provided ("source")  nor the predicted ("target") coordinate is known exactly - unless, of course, the provided coordinate is one of the defining stations of the source reference frame. In this special case, the predicted coordinate will swim in a lake of prediction-variance only.

In the general case, however, the predicted coordinate will posess uncertainty from a combination of the original variance and the prediction-variance. Neither of which are easy to formalize in operational terms.

But the conceptualization of operations, conversions and transformations does not make it easier in any way to start this discussion, which becomes increasingly important as the demand for positional accuracy increases.

Hence, we could simply talk about operations, as a first step towards tackling the problem of operationalizing the representation of variance propagation in geodetic ~~transformations~~ conversions:

- We **convert** coordinates to **align** them with a different reference frame (or a different CRS based on the same reference frame, in which case the alignment will be perfect).
- We do so by applying an **operator** (which may be implemented as a pipeline of more fundamental operators)
- The operator, parameterized by its parameters (which may be defined or derived) implements the **operation** applied to the source coordinates.
- The target coordinates still have exactly the same variance with respect to the **source** reference frame (since we may take the operator parameters to be given by definition, and transform back to the source coordinates).
- With respect to the **target reference frame,** however, the variance of the target coordinates has increased.

**Additional value** could be provided by more clearly describing the relation between reversible operations and their inverses. In the current state of affairs, the conversion from A to B and that from B to A are just two unrelated operators.

## Conclusion?

No - but perhaps we should vote for revision!

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
