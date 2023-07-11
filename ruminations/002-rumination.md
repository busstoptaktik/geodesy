# Ruminations on Rust Geodesy

## Rumination 002: The missing manual

Thomas Knudsen <knudsen.thomas@gmail.com>

2021-08-20. Last [revision](#document-history) 2023-07-09

### Abstract

```sh
$ echo 553036. -124509 | kp "dms:in | geo:out"
> 55.51  -12.7525 0 0
```

---

### Contents

- [Prologue](#prologue)
- [A brief `kp` HOWTO](#a-brief-kp-howto)
- [`adapt`](#operator-adapt): The order-and-unit adaptor
- [`cart`](#operator-cart): The geographical-to-cartesian converter
- [`curvature`](#operator-curvature): Radii of curvature
- [`dm`](#operator-dm): DDMM.mmm encoding.
- [`dms`](#operator-dms): DDMMSS.sss encoding.
- [`geodesic`](#operator-geodesic): Origin, Distance, Azimuth, Destination and v.v.
- [`gridshift`](#operator-gridshift): NADCON style datum shifts in 1, 2, and 3 dimensions
- [`helmert`](#operator-helmert): The Helmert (similarity) transformation
- [`laea`](#operator-laea): The Lambert Authalic Equal Area projection
- [`latitude`](#operator-latitude): Auxiliary latitudes
- [`lcc`](#operator-lcc): The Lambert Conformal Conic projection
- [`merc`](#operator-merc): The Mercator projection
- [`molodensky`](#operator-molodensky): The full and abridged Molodensky transformations
- [`noop`](#operator-noop): The no-operation
- [`omerc`](#operator-omerc): The oblique Mercator projection
- [`pop`](#operator-pop): Pop a dimension from the stack into the operands
- [`proj`](#operator-proj): Invoke the `proj` executable to support all the projections PROJ supports.
- [`push`](#operator-push): Push a dimension from the operands onto the stack
- [`tmerc`](#operator-tmerc): The transverse Mercator projection
- [`utm`](#operator-utm): The UTM projection
- [`webmerc`](#operator-webmerc): The Web Pseudomercator projection

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
> 691875.63214 6098907.82501 0.00000 0.00000
```

While RG coordinates are always 4D, `kp` will provide a zero-value for left-out 3rd dimension values, and a NaN-value for left out 4th dimension values:

```sh
echo 55 12 | kp "geo:in | utm zone=32"
> 691875.6321 6098907.82501 0.0000 NaN
```

In the examples in the operator descriptions below, we will just give the operator representation, and imply the `echo ... | kp ...` part.

If in doubt, use `kp --help` or read [Rumination 003: `kp` - the RG Coordinate Processing program](/ruminations/003-rumination.md).

---

### Operator `adapt`

**Purpose:** Adapt source coordinate order and angular units to target ditto, using a declarative approach.

**Description:** Let us first introduce the **coordinate archetypes** *eastish, northish, upish, futurish*, and their geometrical inverses *westish, southish, downish, pastish*, with mostly evident meaning:

A coordinate is

- **eastish** if you would typically draw it along an abscissa (e.g. longitude or easting),
- **northish** if you would typically draw it along an ordinate (e.g. latitude or northing),
- **upish** if you would need to draw it out of the paper (e.g. height or elevation), and
- **futurish** if it represents ordinary, forward evolving time (e.g. time or time interval).

*Westish, southish, downish*, and *pastish* are the axis-reverted versions of the former four. These 8 spatio-temporal directional designations have convenient short forms,
`e, n, u, f` and `w, s, d, p`, respectively.

Also, we introduce the 3 common angular representations *degrees, gradians, radians*, conventionally abbreviated as `deg`, `gon` and `rad`.

The Rust Geodesy internal format of a four dimensional coordinate tuple is `e, n, u, f`, and the internal unit of measure for angular coordinates is radians. In `adapt`, terms, this is described as `enuf_rad`.

`adapt` covers much of the same ground as the `PROJ` operators [`axisswap`](https://proj.org/operations/conversions/axisswap.html) and [`unitconvert`](https://proj.org/operations/conversions/unitconvert.html), but using a declarative, rather than imperative, approach: You never tell `adapt` how you want things done, only what kind of result you want. You tell it where you want to go `from`, and where you want to go `to` (and in most cases actually only one of those). Then `adapt` figures out how to fulfill that wish.

**Example:** Read data in degrees, (latitude, longitude, height, time)-order, write homologous data in radians, (longitude, latitude, height, time)-order, i.e. latitude and longitude swapped.

```js
adapt from=neuf_deg  to=enuf_rad
```

But since the target format is identical to the default internal format, it can be left out, and the operation be written simply as:

```js
adapt from=neuf_deg
```

(end of example)

**Usage:** Typically, `adapt` is used in one or both ends of a pipeline, to match data between the RG internal representation and the requirements of the embedding system:

```sh
adapt from=neuf_deg | cart ... | helmert ... | cart inv ... | adapt to=neuf_deg
```

Note that `adapt to=...` and `adapt inv from=...` are equivalent. The latter form is sometimes useful: It is a.o. used behind the scenes when using RG's predefined macros, `geo` (latitude, longitude) and `gis` (longitude, latitude), as in:

```sh
geo:in | cart ... | helmert ... | cart inv ... | geo:out
```

where `geo:out` could be defined as `geo:in inv`.

---

### Operator `cart`

**Purpose:** Convert from geographic coordinates + ellipsoidal height to geocentric cartesian coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | Inverse operation: cartesian-to-geographic |
| `ellps=name` | Use ellipsoid `name` for the conversion|

**Example**:

```sh
geo:in | cart ellps=intl | helmert x=-87 y=-96 z=-120 | cart inv ellps=GRS80 | gis:out
```

cf. [Rumination no. 001](/ruminations/001-rumination.md) for details about this perennial pipeline.

---

### Operator `curvature`

**Purpose:**
Convert from geographic latitude to a selection of radii of curvature cases

**Description:**

| Argument | Description |
|----------|-------------|
| `ellps=name` | Use ellipsoid `name` for the conversion|
| `prime` | $N$, radius of curvature in the prime vertical|
| `meridian` | $M$, the meridian radius of curvature|
| `gauss` | Gaussian mean $R_a = \sqrt{M\times N}$|
| `mean` | Mean radius of curvature $R_m = \frac{2}{1/M + 1/N}$|
| `azimuthal` | Radius of curvature in the direction $\alpha$. $R_\alpha = \frac{1}{\cos^2\alpha/M+\sin^2\alpha/N}$|

Contrary to most other operators, in most cases `curvature` reads only the first dimension of the input coordinate, which is considered to be the latitude, $\varphi$ **in degrees**.

In the `curvature azimuthal` case, the two first dimensions are read, and considered a latitude, azimuth pair $(\varphi, \alpha)$, both expected to be **given in degrees**

**Example**:

```sh
curvature prime ellps=GRS80
```

**See also:** The [Earth radius](https://en.wikipedia.org/wiki/Earth_radius) article on Wikipedia

---

### Operator `dm`

**Purpose:** Convert from/to the ISO-6709 DDDMM.mmm format.

**Description:**
While "the real ISO-6709 format" uses a postfix letter from the set `{N, S, W, E}` to indicate the sign of an angular coordinate, here we use common mathematical prefix signs. The output is a coordinate tuple in the RG internal format.

The ISO-6709 formats are often used in nautical/navigational gear following the industry standard NMEA 0183.

EXAMPLE: convert DDMM.mmm to decimal degrees.

```sh
$ echo 5530.15 -1245.15 | kp "dm | geo inv"
> 55.5025  -12.7525 0 0
```

**See also:**

- [NMEA 0183](https://www.nmea.org/content/STANDARDS/NMEA_0183_Standard)
- NMEA 0183 on [Wikipedia](https://en.wikipedia.org/wiki/NMEA_0183)
- [GPSd](https://gpsd.gitlab.io/gpsd/NMEA.html) page about NMEA 0183

---

### Operator `dms`

**Purpose:** Convert from/to the ISO-6709 DDDMMSS.sss format.

**Description:**
While "the real ISO-6709 format" uses a postfix letter from the set `{N, S, W, E}` to indicate the sign of an angular coordinate, here we use common mathematical prefix signs. The output is a coordinate tuple in the RG internal format.

The ISO-6709 formats are often used in nautical/navigational gear following the industry standard NMEA 0183.

EXAMPLE: convert DDDMMSS.sss to decimal degrees.

```sh
$ echo 553036. -124509 | kp "dms | geo:out"
> 55.51  -12.7525 0 0
```

**See also:**

- [NMEA 0183](https://www.nmea.org/content/STANDARDS/NMEA_0183_Standard)
- NMEA 0183 on [Wikipedia](https://en.wikipedia.org/wiki/NMEA_0183)
- [GPSd](https://gpsd.gitlab.io/gpsd/NMEA.html) page about NMEA 0183

---

### Operator `geodesic`

**Purpose:**
Solve the two classical *geodetic main problems:*

- Determine where you are, given an origin, a bearing and the distance travelled
- Knowing where you are, determine which bearing and distance will bring you back to the origin

**Description:**

| Argument     | Description |
|--------------|-------------|
| `ellps=name` | Use ellipsoid `name` for the computations|
| `reversible` | in the forward case, provide output suitable for roundtripping|
| `inv`        | swap forward and inverse mode |

**In the forward case,** `geodesic` reads *one* 2D coordinate tuple, an azimuth and a distance from its 4D input. The tuple is expected to be in degrees and in latitude-longitude order. The azimuth is expected to be in degrees, and the distance in meters.

The 4D output represents the characteristics of a geodesic between the points:

- The forward azimuth at the origin
- The forward azimuth at the destination
- The distance between the points, and
- The return azimuth from the destination to the origin

**In the inverse case,** `geodesic` reads *a pair* of 2D coordinate tuples from its 4D input. The tuples are expected to be in degrees and in latitude-longitude order. The first pair represents the origin of a geodesic, the second represents its destination.

If the `reversible` option *is not* selected, the 4D output represents the characteristics of a geodesic between the points:

- The forward azimuth at the origin
- The forward azimuth at the destination
- The distance between the two points, and
- The return azimuth from the destination to the origin

If the `reversible` option *is* selected, the 4D output represents the characteristics of a geodesic between the points *in a way suitable for roundtrip testing*:

- The latitude of the destination point, in degrees
- The longitude of the destination point, in degrees
- The return azimuth from the destination to the origin
- The distance between the two points

i.e. the format expected by *the forward case.*

**Example**:

```sh
geodesic reversible ellps=GRS80
```

**See also:** The [Earth radius](https://en.wikipedia.org/wiki/Earth_radius) article on Wikipedia

---

### Operator `gridshift`

**Purpose:**
Datum shift using grid interpolation.

**Description:**
The `gridshift` operator implements datum shifts by interpolation in correction grids, for one-, two-, and three-dimensional cases.

`gridshift` follows the common, but potentially confusing, convention that when operating in the forward direction:

- For 1-D transformations (vertical datum shift),  the grid derived value is *subtracted* from the operand
- For 2-D transformations, the grid derived values are *added* to the operand

3-D and time dependent transformations are implemented by the `deformation` operator.

| Parameter | Description |
|-----------|-------------|
| `inv` | Inverse operation: output-to-input datum. For 2-D and 3-D cases, this involves an iterative refinement, typically converging after less than 5 iterations |
| `grids` | Name of the grid file to use. RG supports only one file for each operation, but maintains the plural form of the `grids` option for alignment with the PROJ precedent |

The `gridshift` operator has built in support for the **Gravsoft** grid format. Support for additional file formats depends on the `Context` in use.

**Units:**
For grids with angular (geographical) spatial units, the corrections are supposed to be given in seconds of arc, and internally converted to radians. For grids appearing to have linear (projected) spatial units, the corrections are supposed to be given in meters, and are kept unchanged. A grid is supposed to be in linear spatial units if any of its boundaries have a numerical value larger than `2×360`, i.e. clearly outside of the angular range.

**Example**:

```term
geo:in | gridshift grids=ed50.datum | geo:out
```

**See also:** PROJ documentation, [`hgridshift`](https://proj.org/operations/transformations/hgridshift.html) and [`vgridshift`](https://proj.org/operations/transformations/vgridshift.html). RG combines the functionality of the two: The dimensionality of the grid determines whether a plane or a vertical transformation is carried out.

---

### Operator `helmert`

**Purpose:**
Datum shift using a 3, 6, 7 or 14 parameter similarity transformation.

**Description:**
In strictly mathematical terms, the Helmert (or *similarity*) transformation transforms coordinates from their original coordinate system, *the source basis,* to a different system, *the target basis.* The target basis may be translated, rotated and/or scaled with respect to the source basis. The inter-axis angles are, however, fixed (hence, the *similarity* moniker).

So mathematically we may think of this as "*transforming* the coordinates from one well defined basis to another". But geodetically, it is more correct to think of the operation as *aligning* rather than *transforming,* since geodetic reference frames are very far from the absolute platonic ideals implied in the mathematical idea of bases.

Rather, geodetic reference frames are empirical constructions, realised using datum specific rules for survey and adjustment. Hence, coordinate tuples subjected to a given similarity transform, *do not* magically become realised using the survey rules of the target datum. But they gain a degree of *interoperability* with coordinate tuples from the target: The transformed (aligned) values represent our best knowledge about **what coordinates we would obtain,** if we re-surveyed the same physical point, using the survey rules of the target datum.

**Warning:**
Two different conventions are common in Helmert transformations involving rotations. In some cases the rotations define a rotation of the reference frame. This is called the "coordinate frame" convention (EPSG methods 1032 and 9607). In other cases, the rotations define a rotation of the vector from the origin to the position indicated by the coordinate tuple. This is called the "position vector" convention (EPSG methods 1033 and 9606).

Both conventions are common, and trivially converted between as they differ by sign only. To reduce this great source of confusion, the `convention` parameter must be set to either `position vector` or `coordinate_frame` whenever the operation involves rotations. In all other cases, all parameters are optional.

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

```js
geo:in | cart ellps=intl | helmert x=-87 y=-96 z=-120 | cart inv ellps=GRS80 | geo:out
```

**See also:** [PROJ documentation](https://proj.org/operations/transformations/helmert.html): *Helmert transform*. In general the two implementations should behave identically although the RG version implements neither the 4 parameter 2D Helmert variant, nor the 10 parameter 3D Molodensky-Badekas variant.

---

### Operator `laea`

**Purpose:** Projection from geographic to Lambert azimuthal equal area coordinates

**Description:**

| Argument     | Description |
|--------------|-------------|
| `inv`        | Inverse operation: LAEA to geographic |
| `ellps=name` | Use ellipsoid `name` for the conversion |
| `lon_0`      | Longitude of the projection center |
| `lat_0`      | Latitude of the projection center |
| `x_0`        | False easting  |
| `y_0`        | False northing |

**Example**:

The ETRS89-LAEA grid (used by a.o. The European Environmental Agency, for thematic mapping of the EU member and candidate states), is given by:

```js
laea lon_0=10  lat_0=52  x_0=4321000  y_0=3210000  ellps=GRS80
```

**See also:**

- [PROJ documentation](https://proj.org/operations/projections/laea.html): *Lambert Azimuthal Equal Area*.
- [IOGP, 2019](https://www.iogp.org/wp-content/uploads/2019/09/373-07-02.pdf): *Coordinate Conversions and Transformations including Formulas*. IOGP Geomatics Guidance Note Number 7, part 2, 162 pp.
- [Charles F.F. Karney, 2022](https://doi.org/10.48550/arXiv.2212.05818): *On auxiliary latitudes*

The RG implementation closely follows the IOGP (2019) exposition, but utilizes the work by Karney (2022) to obtain a higher accuracy in the handling of the conversion between authalic and geographic latitudes.

---

### Operator `latitude`

**Purpose:** Convert from geographic to an auxiliary latitude

**Description:**

| Argument | Description |
|--------------|-------------|
| `inv`        | Inverse operation: auxiliary to geographic |
| `ellps=name` | Use ellipsoid `name` for the conversion |
| `authalic`   | Convert to authalic latitude |
| `conformal`  | Convert to conformal latitude |
| `geocentric` | Convert to geocentric latitude |
| `parametric` | Convert to parametric latitude |
| `reduced`    | (synonym for `parametric`) |
| `rectifying` | Convert to rectifying latitude |

**Example**:

```js
latitude geocentric ellps=GRS80
```

**See also:** Charles F.F. Karney, 2022: [On auxiliary latitudes](https://doi.org/10.48550/arXiv.2212.05818)

---

### Operator `lcc`

**Purpose:** Projection from geographic to Lambert conformal conic coordinates

**Description:**

| Argument     | Description |
|--------------|-------------|
| `inv`        | Inverse operation: LCC to geographic |
| `ellps=name` | Use ellipsoid `name` for the conversion |
| `k_0`        | Scaling factor |
| `lon_0`      | Longitude of the projection center |
| `lat_0`      | Latitude of the projection center |
| `lat_1`      | First standard parallel |
| `lat_2`      | Second standard parallel (optional) |
| `x_0`        | False easting  |
| `y_0`        | False northing |

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
| `ellps=name` | Use ellipsoid `name` for the conversion |
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
| `ellps=name` | Use ellipsoid `name` for the conversion |
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
molodensky left_ellps=WGS84 right_ellps=intl dx=84.87 dy=96.49 dz=116.95 abridged
```

**See also:** [PROJ documentation](https://proj.org/operations/transformations/molodensky.html): *Molodensky*. The current implementations differ between PROJ and RG: RG implements some minor numerical improvements and the ability to parameterize using two ellipsoids, rather than differences between them.

---

### Operator `noop`

**Purpose:** Do nothing

**Description:** `noop`, the no-operation, takes no arguments, does nothing and is good at it. Any arguments provided are ignored. Probably most useful during development of transformation pipelines, for "commenting out" individual steps.

**Example**:

Ignore all parameters, do nothing

```sh
geo:in | noop all these parameters are=ignored | geo:out
```

**Example**:

Comment out a datum shift step in a pipeline

```sh
geo:in | cart | noop helmert x=84 y=96 z=116 | cart inv | merc
```

---

### Operator `omerc`

**Purpose:** Projection from geographic to oblique mercator coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | swap forward and inverse operations |
| `ellps=name` | Use ellipsoid `name` for the conversion |
| `lonc` | Longitude of the projection center |
| `latc` | Latitude of the projection center |
| `k_0` | Scaling factor (on the initial line) |
| `x_0` | False easting  |
| `y_0` | False northing |
| `alpha` | Azimuth of the initial line |
| `gamma` | Angle from the rectified grid to the oblique grid |
| `variant` | Use the "variant B" formulation (changes the interpretation of `x_0` and `y_0`) |
| `laborde` | Approximate the Laborde formultaion using "variant B" with `gamma = alpha`) |

**Example**: EPSG Guidance Note 7-2 implementation of Projected coordinate system
*Timbalai 1948 / R.S.O. Borneo*

```js
omerc ellps=evrstSS variant
x_0=590476.87 y_0=442857.65
latc=4 lonc=115
k_0=0.99984 alpha=53:18:56.9537 gamma_c=53:07:48.3685
```

**See also:** [PROJ documentation](https://proj.org/operations/projections/omerc.html): *Oblique Mercator*.
The parameter names differ slightly between PROJ and RG: PROJ's `lat_0` is `latc` here, to match `lonc`,
and RG does not support PROJ's "indirectly given azimuth" case.

---

### Operator `pop`

**Purpose:** Pop a coordinate dimension from the stack

**Description:**
Pop the top(s)-of-stack into one or more operand coordinate dimensions. If more than one dimension is given, they are pop'ed in reverse numerical order. Pop's complement, push, pushes in numerical order, so the dance `push v_3 v_2 | pop v_3 v_2` is a noop - no matter in which order the args are given.

| Argument | Description |
|----------|-------------|
| `v_1` | Pop the top-of-stack into the first coordinate of the operands |
| `v_2` | Pop the top-of-stack into the second coordinate of the operands |
| `v_3` | Pop the top-of-stack into the third coordinate of the operands |
| `v_4` | Pop the top-of-stack into the fourth coordinate of the operands |

(the argument names are selected for PROJ compatibility)

**See also:** [`push`](#operator-push)

---

### Operator `proj`

**Purpose:** Projection from geographic to any projected system supported by PROJ

**Description:**

Invokes the `proj` executable (if available) instantiating any PROJ supported projection. The syntax is identical to PROJ's, so check the [PROJ projection documentation](https://proj.org/operations/projections/index.html) for details

**Example**: UTM zone 32 implemented using PROJ

```js
proj proj=utm zone=32 ellps=GRS80
```

**See also:** [PROJ documentation](https://proj.org/operations/projections/index.html): *Projections*.

---

### Operator `push`

**Purpose:** Push a coordinate dimension onto the stack

**Description:**
Take a copy of one or more coordinate dimensions and push it onto the stack. If more than one dimension is given, they are pushed in numerical order. Push's complement, pop, pops in reverse numerical order, so the dance `push v_3 v_2 | pop v_3 v_2` is a noop - no matter in which order the args are given.

| Argument | Description |
|----------|-------------|
| `v_1` | Push the first coordinate onto the stack |
| `v_2` | Push the second coordinate onto the stack |
| `v_3` | Push the third coordinate onto the stack |
| `v_4` | Push the fourth coordinate onto the stack |

(the argument names are selected for PROJ compatibility)

**See also:** [`pop`](#operator-pop)

---

### Operator `tmerc`

**Purpose:** Projection from geographic to transverse mercator coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | Swap forward and inverse operations |
| `ellps=name` | Use ellipsoid `name` for the conversion |
| `lon_0` | Longitude of the projection center |
| `lat_0` | Latitude of the projection center |
| `k_0` | Scaling factor |
| `x_0` | False easting  |
| `y_0` | False northing |

**Example**: Implement UTM zone 32 using `tmerc` primitives

```js
tmerc lon_0=9 k_0=0.9996 x_0=500000
```

**See also:** [PROJ documentation](https://proj.org/operations/projections/tmerc.html): *Transverse Mercator*.

---

### Operator `utm`

**Purpose:** Projection from geographic to universal transverse mercator (UTM) coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | Swap forward and inverse operations |
| `ellps=name` | Use ellipsoid `name` for the conversion |
| `zone=nn` | zone number `nn`. Between 1-60 |

**Example**: Use UTM zone 32 on the default ellipsoid

```js
utm zone=32
```

**See also:** [PROJ documentation](https://proj.org/operations/projections/utm.html): *Universal Transverse Mercator*.

---

### Operator `webmerc`

**Purpose:** Projection from geographic to web pseudomercator coordinates

**Description:**

| Argument | Description |
|----------|-------------|
| `inv` | Swap forward and inverse operations |
| `ellps=name` | Use ellipsoid `name` for the conversion. Defaults to `WGS84` |

**Example**:

```js
webmerc
```

**See also:**

- [PROJ documentation](https://proj.org/operations/projections/webmerc.html): *Mercator*. The current implementation closely follows the PROJ version.
- [`merc`](#operator-merc)

### Document History

Major revisions and additions:

- 2021-08-20: Initial version
- 2021-08-21: All relevant operators described
- 2021-08-23: nmea, dm, nmeass, dms
- 2022-05-08: reflect syntax changes + a few minor corrections
- 2023-06-06: A number of minor corrections + note that since last
  registered update on 2022-05-08. a large number of new operators
  have been included and described
- 2023-07-09: dm and dms liberated from their NMEA overlord
