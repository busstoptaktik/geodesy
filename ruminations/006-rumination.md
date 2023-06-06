# Ruminations on Rust Geodesy

## Rumination 006: Still confused, but at a higher level

Thomas Knudsen <thokn@sdfi.dk>

2021-05-06. Last [revision](#document-history) 2022-06-19

### Abstract

```sh
$ echo 55 12 | kp "geodesy:in | geomatics:out"
> 12 55 0 0
```

---

### Prologue

This is a reply to a fellow mailing list participant's recent utterance of:

> I am still confused as to what a CRS datum is

### What *is* a CRS? What *is* a datum?

You probably are *confused for a good reason,* since the geoinformatics standardisation process has turned a number of extremely simple geodetic concepts into something needlessly complicated, based on the (likely wrong) assumption that they can be *defined* on an axiomatic foundation.

On the other hand, geodetic nomenclature has always been horribly sloppy, so the basically simple concepts of geodesy have been needlessly complicated to discuss. So no wonder the unification of geodesy and geoinformatics has resulted in an awful (although useful) set of standards: Linguistically strict, but geodetically dubious.

So an important mission for the comming years will be to enrich the geodetic community with the linguistic strictness of geoinformatics, while putting the coordinate reference standardisation more in line with actual geodesy (well, actually it already mostly is, it's more the common usage, i.e. the "broadly commonly perceived interpretation", rather than "the actual current standard's content" that is off and needs mending).

But back to the subject: What is a CRS datum?

Ignoring all formal definitions (in the sacred name of traditional geodetic sloppiness), a datum is a CRS which is **particularly useful** as the foundation for any number of other CRS, which are then said to be "based on that datum". Modern 3D datums are often, although not entirely correct, known as "reference frames" (RF). In its most tangible form, a RF consists of a list of geodetic physical markers ("stations") with corresponding position- and velocity vectors.

The positions ("coordinates") and velocities ("deformations") are computed from geodetic observations by following a set of guidance rules. The set of rules is called a "reference system". So a reference frame (datum) is the materialization (formally called a "realization") of a reference system, which itself is just a general idea of "a useful way of arranging our world view" - a publication of some sort.

If you perform a measurement and turn your observation into coordinates by leveraging (in the way prescribed by the reference system publication) coordinates from the list of stations of your selected RF, then your resulting coordinate will be given with respect to that RF ("in that datum").

If your measurement is carried out by GPS, then the RF will not be given by the coordinates and deformations of geodetic stations on the surface of the earth, but by the ephemerides of the constellation of satellites - and your resulting coordinate will, by black box magic, be given with respect to the most recent realization of WGS84.

Of course in the strict nomenclature of geoinformatics, this is nonsense: A datum flies at a higher abstract level than a CRS and is first useful for representing terrestrial positions once it is combined with a coordinate system (CS). The datum determines the orientation of the CS, and together they comprise the CRS.

In the sloppy nomenclature of geodesy, you would hardly discern between geocentric 3D cartesian coordinates (which is the "natural coordinate system" for GPS observations), or the same location given as a (longitude, latitude, height)-tuple, or as map coordinates (e.g. UTM) given as (easting, northing, height). A geodesist wouldn't care, because the three variants convey the same information to anyone having the contextual information of "which *kind* of coordinates" are at play. In other words: When looking at a CRS, the geodesist cares about the RF/datum, but not about the CS.

Obviously, in geoinformatics we do not have the luxury of relying on contextual information - we need a strictly defined CRS to make sure that geoinformatics software handles the coordinates correctly.

The CRS, however, is also contextual information. And while a geodesist would consider this kind of contextual information simply as a label (e.g. an URI), in geoinformatics the CRS posesses internal state, making it possible to derive a transformation between two CRS' from their internal state.

In geodesy, on the other hand, the two CRS-labels of the CRS' would be used as indices into a table of transformations, in order to locate the proper recipe for interoperability between the two.

Essentially, the amount of information to store is identical, no matter whether represented as internal states or external transformations. But in order to represent a CRS as a formal definition, you need an axiomatic foundation - and most likely such a foundation does not exist, since a CRS depend on a datum, and datums are geophysical items *and hence empirical.*

Transformations, on the other hand, are just recipes: Do this, then that, then the-other, independent of any assumptions regarding what they transform between.

A transformation bridging two datums, A and B, is fundamentally empirical, and must be based on reference material consisting of coordinates from a number of stations having coordinates *in both datums*. **Subjecting a coordinate tuple *a* in datum A to the transformation does not magically turn it into a coordinate tuple in datum B - it merely makes it somewhat interoperable with coordinate tuples given in datum B:** The transformation implements a *prediction* of what would have been measured at station *a* if it had been measured according to the rules of datum B, it does not (and fundamentally can not) turn datum A coordinates into datum B coordinates.

Also, the transformation does not constitute a *definition* of datum B with respect to datum A. Actually, all modern datums share the same *definition*: A right-handed, 3D cartesian system with the meter as scale, origin at the center of mass of the earth, Z-axis direction as determined by the conventional international origin (CIO), X-axis orthogonal to the Z-axis and pointing towards the (conventional) Greenwich meridian, and Y-axis orthogonal to the Z- and X-axes.

So since the definition is identical, all transformations between modern datums should be the identity operator. They are, however, not, because datums are empirical contraptions, not idealized mathematical objects. Hence, datums are only perfect to the degree they are able to materialize ("realize") the platonic ideals of their maternal reference *system*. Imperfect due to the stochastical nature of measurement, due to unaccounted-for planetary dynamics and presumably also due to occasional blunders.

So there is no such thing as the *definition of a datum* (although you can have a definition of the reference *system* behind the datum-realization), and hence no such thing as *the definition of a CRS* - so the standards for referencing by coordinates are way less rigorous than they seem. Hence, it is very likely that we could obtain a simpler and stronger foundation for coordinate reference by building it on the basis of transformations *between* CRS, rather than *definitions of* CRS. At least such a foundation would be much better aligned with the geodetic science building the actual reference frames.

### Document History

Major revisions and additions:

- 2022-05-06: Mailing list reply
- 2022-06-19: Rebuild as rumination
- 2023-06-06: Minor (mostly markup) corrections
