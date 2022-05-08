# Ruminations on Rust Geodesy

## Rumination 002: The missing manual

Thomas Knudsen <knudsen.thomas@gmail.com>

2021-08-20. Last [revision](#document-history) 2022-05-08

### Abstract

```sh
$ echo 553036. -124509 | kp "dms | geo inv"
> 55.51  -12.7525 0 0
```

---

### Contents

- [Prologue](#prologue)
- [A brief `kp` HOWTO](#a-brief-kp-howto)
- [`adapt`](#operator-adapt): The order-and-unit adaptor
- [`cart`](#operator-cart): The geographical-to-cartesian converter
- [`dm`](#operator-nmea-dm-nmeass-and-dms): DDMM.mmm encoding, sub-entry under `nmea`
- [`dms`](#operator-nmea-dm-nmeass-and-dms): DDMMSS.sss encoding, sub-entry under `nmea`
- [`helmert`](#operator-helmert): The Helmert (similarity) transformation
- [`lcc`](#operator-lcc): The Lambert Conformal Conic projection
- [`merc`](#operator-merc): The Mercator projection
- [`molodensky`](#operator-molodensky): The full and abridged Molodensky transformations
- [`nmea`](#operator-nmea-dm-nmeass-and-dms): degree/minutes encoding with obvious extension to seconds.
- [`nmeass`](#operator-nmea-dm-nmeass-and-dms): DDMMSS.sss encoding, sub-entry under `nmea`
- [`noop`](#operator-noop): The no-operation
- [`tmerc`](#operator-tmerc): The transverse Mercator projection
- [`utm`](#operator-utm): The UTM projection

### Prologue

Architecturally, the operators in Rust Geodesy (`cart`, `tmerc`, `helmert` etc.) live below the API surface. This means they are not (and should not be) described in the API documentation over at [docs.rs](https://docs.rs/geodesy). Rather, their use should be documented in a separate *Rust Geodesy User's Guide*, a book which may materialize some day, as time permits, interest demands, and RG has matured and stabilized sufficiently. Until then, this *Rumination* will serve as stop gap for operator documentation.

A *Rust Geodesy Programmer's Guide* would probably also be useful, and wil definitely materialize before the next week with ten fridays. Until then, the [API documentation](https://docs.rs/geodesy), the [code examples](/examples), and the [architectural overview](/ruminations/000-rumination.md) may be useful. The RG transformation program `kp` is described in [RG Rumination 003](/ruminations/003-rumination.md). Its [source code](/src/bin/kp.rs) may also be of interest as  study material for programmers. But since it is particularly useful for practical experimentation with RG operators, let's start with a *very* brief description of `kp`.

### A brief `kp` HOWTO

The `kp` command line syntax is

```sh
kp "operation" file1 file2 ...
```

or, with input from `stdin`:

```sh
echo coordinate |  kp "operation"
```

**Example:**
Convert the geographical coordinate tuple (55 N, 12 E) to utm, zone 32 coordinates:

```sh
echo 55 12 0 0 | kp "geo:in | utm zone=32"
> 691875.6321 6098907.8250 0.0000 0.0000
```

While RG coordinates are always 4D, `kp` will provide zero-values for any left-out postfix dimensions:

```sh
echo 55 12 | kp "geo:in | utm zone:32"
> 691875.6321 6098907.8250 0.0000 0.0000
```

In the examples in the operator descriptions below, we will just give the operator representation, and imply the `echo ... | kp ...` part.

If in doubt, use `kp --help` or read [Rumination 003: `kp` - the RG Coordinate Processing program](/ruminations/003-rumination.md).

---

### Operator `adapt`

**Purpose:** Adapt source coordinate order and angular units to target ditto, using a declarative approach.

**Description:** Let us first introduce the **coordinate traits** *eastish, northish, upish, timish*, and their geometrical inverses *westish, southish, downish, reversed-timeish*, with mostly evident meaning:

A coordinate is

- **eastish** if you would typically draw it along an abscissa (e.g. longitude or easting),
- **northish** if you would typically draw it along an ordinate (e.g. latitude or northing),
- **upish** if you would need to draw it out of the paper (e.g. height or elevation), and
- **timeish** if it represents ordinary, forward evolving time (e.g. time or time interval).

*Westish, southish, downish*, and *reversed-timeish* are the axis-reverted versions of the former four. These 8 spatio-temporal directional designations have convenient short forms,
`e, n, u, t` and `w, s, d, r`, respectively.

Also, we introduce the 3 common angular representations *degrees, gradians, radians*, conventionally abbreviated as `deg`, `gon` and `rad`.

The Rust Geodesy internal format of a four dimensional coordinate tuple is `e, n, u, t`, and the internal unit of measure for angular coordinates is radians. In `adapt`, terms, this is described as `enut_rad`.

`adapt` covers much of the same ground as the `PROJ` operators [`axisswap`](https://proj.org/operations/conversions/axisswap.html) and [`unitconvert`](https://proj.org/operations/conversions/unitconvert.html), but using a declarative, rather than imperative, approach: You never tell `adapt` how you want things done, only what kind of result you want. You tell it where you want to go `from`, and where you want to go `to` (and in most cases actually only one of those). Then `adapt` figures out how to fulfill that wish.

**Example:** Read data in degrees, (latitude, longitude, height, time)-order, write homologous data in radians, (longitude, latitude, height, time)-order, i.e. latitude and longitude swapped.

```js
adapt from=neut_deg  to=enut_rad
```

But since the target format is identical to the default internal format, it can be left out, and the operation be written simply as:

```js
adapt from=neut_deg
```

(end of example)

**Usage:** Typically, `adapt` is used in one or both ends of a pipeline, to match data between the RG internal representation and the requirements of the embedding system:

```sh
adapt from=neut_deg | cart ... | helmert ... | cart inv ... | adapt to=neut_deg
```

Note that `adapt to=...` and `adapt inv from=...` are equivalent. The latter form is sometimes useful: It is a.o. used behind the scenes when using RG's predefined macros, `geo` (latitude, longitude) and `gis` (longitude, latitude), as in:

```sh
geo:in | cart ... | helmert ... | cart inv ... | geo:out
```

where `geo:out` is defined as `geo:in inv`.

---

### Operator `cart`

**Purpose:** Convert from geographic coordinates + ellipsoidal height to geocentric cartesian coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | Inverse operation: cartesian-to-geographic |
| `ellps: name` | Use ellipsoid `name` for the conversion|

**Example**:

```sh
geo:in | cart ellps=intl | helmert x=-87 y=-96 z=-120 | cart inv ellps=GRS80 | gis:out
```

cf. [Rumination no. 001](/ruminations/001-rumination.md) for details about this perennial pipeline.

---

### Operator `helmert`

**Purpose:**
Datum shift using a 3, 6, 7 or 14 parameter similarity transformation.

**Description:**
In strictly mathematical terms, the Helmert (or *similarity*) transformation transforms coordinates from their original coordinate system, *the source basis,* to a different system, *the target basis.* The target basis may be translated, rotated and/or scaled with respect to the source basis. The inter-axis angles are, however, fixed (hence, the *similarity* moniker).

So mathematically we may think of this as "*transforming* the coordinates from one well defined basis to another". But geodetically, it is more correct to think of the operation as *aligning* rather than *transforming,* since geodetic reference frames are very far from the absolute platonic ideals implied in the mathematical idea of bases.

Rather, geodetic reference frames are empirical constructions, realised using datum specific rules for survey and adjustment. Hence, coordinate tuples subjected to a given similarity transform, do not magically become realised using the survey rules of the target datum. But they gain a degree of interoperability with coordinate tuples from the target: The transformed (aligned) values represent our best knowledge about *what coordinates we would obtain,* if we re-surveyed the same physical point, using the survey rules of the target datum.

**Warning:**
Two different conventions are common in Helmert transformations involving rotations. In some cases the rotations define a rotation of the reference frame. This is called the "coordinate frame" convention (EPSG methods 1032 and 9607). In other cases, the rotations define a rotation of the vector from the origin to the position indicated by the coordinate tuple. This is called the "position vector" convention (EPSG methods 1033 and 9606).

Both conventions are common, and trivially converted between as they differ by sign only. To reduce this great source of confusion, the `convention` parameter must be set to either `position vector` or `coordinate_frame` whenever the operation involved rotations. In all other cases, all parameters are optional.

| Parameter | Description |
|-----------|-------------|
| `inv` | Inverse operation: output-to-input datum. Mathematically, a sign reversion of all parameters. |
| `x`  | offset along the first axis  |
| `y`  | offset along the second axis |
| `z`  | offset along the third axis  |
| `rx` | rotation around the first axis  |
| `ry` | rotation around the second axis |
| `rz` | rotation around the third axis  |
| `s`  | scaling factor given in parts-per-million |
| `dx`  | rate-of-change for offset along the first axis  |
| `dy`  | rate-of-change for offset along the second axis |
| `dz`  | rate-of-change for offset along the third axis  |
| `drx` | rate-of-change for rotation around the first axis  |
| `dry` | rate-of-change for rotation around the second axis |
| `drz` | rate-of-change for rotation around the third axis  |
| `ds`  | rate-of-change for scaling factor |
| `t_epoch` | origin of the time evolution |
| `t_obs` | fixed value for observation time. Ignore fourth coordinate |
| `exact` | Do not use small-angle approximations when constructing the rotation matrix |
| `convention` | Either `position_vector` or `coordinate_frame`, as described above. Mandatory if any of the rotation parameters are used. |

**Example**:

```sh
geo:in | cart ellps=intl | helmert x=-87 y=-96 z=-120 | cart inv ellps=GRS80 | geo:out
```

**See also:** [PROJ documentation](https://proj.org/operations/transformations/helmert.html): *Helmert transform*. In general the two implementations should behave identically although the RG version implements neither the 4 parameter 2D Helmert variant, nor the 10 parameter 3D Molodensky-Badekas variant.

---

### Operator `lcc`

**Purpose:** Projection from geographic to Lambert conformal conic coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | Inverse operation: LCC to geographic |
| `ellps: name` | Use ellipsoid `name` for the conversion |
| `k_0` | Scaling factor |
| `lon_0` | Longitude of the projection center |
| `lat_0` | Latitude of the projection center |
| `lat_1` | First standard parallel |
| `lat_2` | Second standard parallel (optional) |
| `x_0` | False easting  |
| `y_0` | False northing |

**Example**:

```js
lcc lon_0=-100 lat_1=33 lat_2=45
```

**See also:** [PROJ documentation](https://proj.org/operations/projections/lcc.html): *Lambert Conformal Conic*. The RG implementation closely follows the PROJ version.

---

### Operator `merc`

**Purpose:** Projection from geographic to mercator coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | Inverse operation: Mercator to geographic |
| `ellps: name` | Use ellipsoid `name` for the conversion |
| `k_0` | Scaling factor |
| `lon_0` | Longitude of the projection center |
| `lat_0` | Latitude of the projection center |
| `lat_ts` | Latitude of true scale: alternative to `k_0` |
| `x_0` | False easting  |
| `y_0` | False northing |

**Example**:

```js
merc lon_0=9 lat_0=54 lat_ts=56
```

**See also:** [PROJ documentation](https://proj.org/operations/projections/merc.html): *Mercator*. The current implementation closely follows the PROJ version.

---

### Operator `molodensky`

**Purpose:** Transform between two geodetic datums using the full or abridged Molodensky formulas.

**Description:**
The full and abridged Molodensky transformations for 2D and 3D data. Closely related to the 3-parameter Helmert transformation, but operating directly on geographical coordinates.

This implementation is based:

- partially on the PROJ implementation by Kristian Evers,
- partially on OGP Publication 373-7-2: *Geomatics Guidance Note
number 7, part 2,* and
- partially on [R.E.Deakin, 2004:](http://www.mygeodesy.id.au/documents/Molodensky%20V2.pdf) *The Standard
and Abridged Molodensky Coordinate Transformation Formulae.*

**Note:**
We may use `ellps, da, df`, to parameterize the operator,
but `left_ellps, right_ellps` is a more likely set of
parameters to come across in real life.

| Argument | Description |
|----------|-------------|
| `inv` | Inverse operation |
| `ellps: name` | Use ellipsoid `name` for the conversion |
| `dx`  | offset along the first axis  |
| `dy`  | offset along the second axis |
| `dz`  | offset along the third axis  |
| `da` | change in semimajor axis between the ellipsoids of the source and target datums |
| `df` | change in flattening between the ellipsoids of the source and target datums |
| `left_ellps` | Ellipsoid of the source datum |
| `right_ellps` | Ellipsoid of the target datum |
| `abridged` | Use the abridged version of the transformation, which ignores the source height |

**Example**:

```js
molodensky left_ellps=WGS84 right_ellps=intl dx=84.87 dy=96.49 dz=116.95 abridged=false
```

**See also:** [PROJ documentation](https://proj.org/operations/transformations/molodensky.html): *Molodensky*. The current implementations differ between PROJ and RG: RG implements some minor numerical improvements and the ability to parameterize using two ellipsoids, rather than differences between them.

---

### Operator `nmea`, `dm`, `nmeass` and `dms`

**Purpose:** Convert from/to the [NMEA 0183](https://www.nmea.org/content/STANDARDS/NMEA_0183_Standard) DDDMM.mmm format, and from/to its logical extension DDDMMSS.sss.

**Description:**
This operator can be invoked under the names `nmea`, `dm`, `nmeass` and `dms`. The former 2 handles data in DDDMM.mmm format, the latter 2 in the DDDMMSS.sss format.

While "the real NMEA format" uses a postfix letter from the set `{N, S, W, E}` to indicate the sign of an angular coordinate, here we use common mathematical prefix signs. The output is a coordinate tuple in the RG internal format.

EXAMPLE: convert NMEA to decimal degrees.

```sh
$ echo 5530.15 -1245.15 | kp "nmea | geo inv"
> 55.5025  -12.7525 0 0
```

EXAMPLE: convert dms to decimal degrees.

```sh
$ echo 553036. -124509 | kp "dms | geo:out"
> 55.51  -12.7525 0 0
```

**See also:**

- [NMEA 0183](https://www.nmea.org/content/STANDARDS/NMEA_0183_Standard)
- NMEA 0183 on [Wikipedia](https://en.wikipedia.org/wiki/NMEA_0183)
- [GPSd](https://gpsd.gitlab.io/gpsd/NMEA.html) page about NMEA 0183

---

### Operator `noop`

**Purpose:** Do nothing

**Description:** `noop`, the no-operation, takes no arguments, does nothing and is good at it.

**Example**:

```sh
geo:in | noop all these parameters are=ignored | geo:out
```

---

### Operator `tmerc`

**Purpose:** Projection from geographic to transverse mercator coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | Inverse operation: transverse-mercator to geographic |
| `ellps: name` | Use ellipsoid `name` for the conversion |
| `lon_0` | Longitude of the projection center |
| `lat_0` | Latitude of the projection center |
| `k_0` | Scaling factor |
| `x_0` | False easting  |
| `y_0` | False northing |

**Example**: Implement UTM zone 32 using `tmerc` primitives

```js
tmerc lon_0=9 k_0=0.9996 x_0=500000
```

**See also:** [PROJ documentation](https://proj.org/operations/projections/tmerc.html): *Transverse Mercator*. The details of the current implementations differ between PROJ and RG.

---

### Operator `utm`

**Purpose:** Projection from geographic to universal transverse mercator (UTM) coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | Inverse operation: transverse-mercator to geographic |
| `ellps: name` | Use ellipsoid `name` for the conversion |
| `zone` | zone number. Between 1-60 |

**Example**: Use UTM zone 32

```js
utm zone=32
```

**See also:** [PROJ documentation](https://proj.org/operations/projections/utm.html): *Universal Transverse Mercator*. The current implementations differ between PROJ and RG. Within each 6 degrees wide zone, the differences should be immaterial.

### Document History

Major revisions and additions:

- 2021-08-20: Initial version
- 2021-08-21: All relevant operators described
- 2021-08-23: nmea, dm, nmeass, dms
- 2022-05-08: reflect syntax changes + a few minor corrections
