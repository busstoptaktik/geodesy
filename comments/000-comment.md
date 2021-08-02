# Comments on Rust Geodesy

## Comment 000: Overall architecture and philosophy

Thomas Knudsen <knudsen.thomas@gmail.com>

2021-07-31

---

### Prologue

Rust Geodesy, RG, is a geodetic software system, not entirely unlike [PROJ](https://proj.org), but with much more limited transformation functionality. And while PROJ is mature, well supported, well tested, and production ready, RG is neither of these. Partially due to RG being a new born baby, partially due to its aiming at a (much) different set of use cases.

So when I liberally insert comparisons with PROJ in the following, it is for elucidation, not for mocking, neither of PROJ, nor of RG: I have spent much pleasant and instructive time with PROJ, both as a PROJ core developer and as a PROJ user (more about that in an upcomming *Comment on RG*). But I have also spent much pleasant time learning Rust and developing RG, so I feel deeply connected to both PROJ and RG.

PROJ and RG do, however, belong in two different niches of the geodetic software ecosystem: Where PROJ is the production work horse, with the broad community of end users and developers, RG aims at a much more narrow community of geodesists, for geodetic development work - e.g. for development of transformations that may eventually end up in PROJ. As stated in the [README](/README.md)-file, RG aims to:

1. Support experiments for evolution of geodetic standards.
2. Support development of geodetic transformations.
3. Hence, provide easy access to a number of basic geodetic operations, not limited to coordinate operations.
4. Support experiments with data flow and alternative abstractions. Mostly as a tool for aims (1, 2, and 3)

All four aims are guided by a wish to amend explicitly identified shortcomings in the existing geodetic system landscape.

### Getting beefy

Talking architecture and design philosophy out of thin air is at best counterproductive, so let's start with a brief example, demonstrating the Rust Geodesy idiom for converting geographical coordinates to UTM zone 32 coordinates.

```rust
fn main() {
    // [0] Use a brief name for some much used functionality
    use geodesy::CoordinateTuple as Coord;

    // [1] Build some context
    let mut ctx = geodesy::Context::new();

    // [2] Obtain a handle to the utm-operator
    let utm32 = ctx.operator("utm: {zone: 32}").unwrap();

    // [3] Coordinates of some Scandinavian capitals
    let copenhagen = Coord::geo(55., 12., 0., 0.);
    let stockholm  = Coord::geo(59., 18., 0., 0.);

    // [4] We put the coordinates into an array
    let mut data = [copenhagen, stockholm];

    // [5] Then do the forward conversion, i.e. geo -> utm
    ctx.fwd(utm32, &mut data);
    println!({:?}, data);

    // [6] And go back, i.e. utm -> geo
    ctx.inv(utm32, &mut data);
    Coord::geo_all(&mut data);
    println!({:?}, data);
}
```

(See also `[idiomatic Rust]` in the Notes section)

At comment `[0]`, we start by renaming the library functionality for coordinate handling, from `geodesy::CoordinateTuple` to `Coord`. Since coordinates are at the heart of what we're doing, it should have a brief and clear name. Then why giving it such a long name by design, you may wonder - well, `CoordinateTuple` is the ISO-19111 standard designation of what we colloquially would call *the coordinates*.

---

```rust
// [1] Build some context
let mut ctx = geodesy::Context::new();
```

At comment `[1]` we instantiate a `Context`, which should not come as a surprise if you have been using [PROJ](https:://proj.org) recently. The `Context` provides the interface to the messy world external to RG (files, threads, communication), and in general centralizes all the *mutable state* of the system.

Also, the `Context` is the sole interface between the `RG` transformation functionality and the application program: You may instantiate a transformation object, but the `Context` handles it for you. While you need a separate `Context` for each thread of your program, the `Context` itself is designed to eventually do its work in parallel, using several threads.

---

```rust
// [2] Obtain a handle to the utm-operator
let utm32 = ctx.operator("utm: {zone: 32}").unwrap();
```


At comment `[2]`, we use the `operator` method of the `Context` to instantiate an `Operator` (closely corresponding to the `PJ` object in PROJ). The parametrisation of the operator, i.e. the text `utm: {zone: 32}` is expressed in [YAML](https://en.wikipedia.org/wiki/YAML) using parameter naming conventions close to those used in PROJ, where the same operator would be described as `proj=utm zone=32`
(see also `[ellps implied]` in the Notes section).

So essentially, PROJ and RG uses identical operator parametrisations, but RG, being 40 years younger than PROJ, is able to leverage YAML, an already 20 years old, JSON compatible, generic data representation format. PROJ, on the other hand, was born 20 years prior to YAML, and had to implement its own domain specific format.

Note, however, that contrary to PROJ, when we instantiate an operator in RG, we do not actually get an `Operator` object back, but just a handle to an `Operator`, living its entire life embedded inside the `Context`.
And while the `Context` is mutable, the `Operator`, once created, is *immutable*.

This makes `Operator`s thread-sharable, so the `Context` will eventually (although not yet fully implemented), be able to automatically parallelize large transformation jobs, eliminating some of the need for separate thread handling at the application program level.

---

```rust
// [3] Coordinates of some Scandinavian capitals
let copenhagen = Coord::geo(55., 12., 0., 0.);
let stockholm  = Coord::geo(59., 18., 0., 0.);

// [4] We put the coordinates into an array
let mut data = [copenhagen, stockholm];
```

At comments `[3]` and `[4]` we produce the input data we want to transform. Internally, RG represents angles in radians, and follows the traditional GIS coordinate order of *longitide before latitude*. Externally, however, you may pick-and-choose.

In this case, we choose human readable angles in degrees, and the traditional coordinate order used in geodesy and navigation: *latitude before longitude*. The `Coord::geo(...)` function translates that into the internal representation. It has siblings `Coord::gis(...)` and `Coord::raw(...)` which handles GIS coordinate order and raw numbers, respectively. The latter is useful for projected coordinates, cartesian coordinates, and for coordinates with angles in radians. We may also simply give a `CoordinateTuple` as a naked array of four double precision floating point numbers:

```rust
let somewhere = Coord([1., 42., 3., 4.]);
```

The `CoordinateTuple` data type does not enforce any special interpretation of what kind of coordinate it stores: That is entirely up to the `Operation` to interpret. A `CoordinateTuple` simply consists of 4 numbers with no other implied interpretation than their relative order, given by the names *first, second, third, and fourth*, respectively.

RG operators take *arrays of `CoordinateTuples`* as input, rather than individual elements, so at comment `[4]` we put the elements into an array.

---

```rust
// [5] Then do the forward conversion, i.e. geo -> utm
ctx.fwd(utm32, &mut data);
println!({:?}, data);
```

At comment `[5]`, we do the actual forward conversion (hence `ctx.fwd(...)`) to utm coordinates. Behind the scenes, `ctx.fwd(...)` splits up the input array into chunks of 1000 elements, for parallel processing in a number of threads (that is: At time of writing, the chunking, but not the thread-parallelism, is implemented).

As the action goes on *in place*, we allow `fwd(..)` to mutate the input data, by using the `&mut`-operator in the method call.

The printout will show the projected data in (easting, northing)-coordinate order:

```
CoordinateTuple([ 691875.6321403517, 6098907.825001632, 0.0, 0.0])
CoordinateTuple([1016066.6135867655, 6574904.395327058, 0.0, 0.0])
```

---

```rust
// [6] And go back, i.e. utm -> geo
ctx.inv(utm32, &mut data);
Coord::geo_all(&mut data);
println!({:?}, data);
```

At comment `[6]`, we roundtrip back to geographical coordinates. Prior to print out, we let `Coord::geo_all(...)` convert from the internal coordinate representation, to the geodetic convention of "latitude before longitude, and angles in degrees".

### Redefining the world

Being intended for authoring of geodetic functionality, customization is very important. RG allows temporal overshadowing of built in functionality by registering user defined macros and operators. This is treated in detail in examples [02 (macros)](../examples/02-02-user_defined_macros.rs) and [03 (operators)](/examples/03-user_defined_operators.rs). Here, let just take a minimal look at the workflow, which can be described briefly as *define, register, instantiate, and use:*

First the macro case:

```rust
// Define a macro, using hat notation (^) for the macro parameters
let macro_text = "pipeline: {
        steps: [
            cart: {ellps: ^left},
            helmert: {dx: ^dx, dy: ^dy, dz: ^dz},
            cart: {inv: true, ellps: ^right}
        ]
    }";

// Register the macro, under the name "geohelmert"
ctx.register_macro("geohelmert", macro_text);

// Instantiate the geohelmert macro with replacement values
// for the parameters left, right, dx, dy, dz
ed50_wgs84 = ctx.operator("geohelmert: {
    left: intl,
    right: GRS80,
    dx: -87, dy: -96, dz: -120
}").unwrap();

// ... and use:
ctx.fwd(ed50_wgs84, data);
```

And then a user defined operator:

```rust
use geodesy::operator_construction::*;

// See examples/03-user-defined-operators.rs for implementation details
pub struct MyNewOperator {
    args: OperatorArgs,
    foo: f64,
    ...
}

// Register
ctx.register_operator("my_new_operator", MyNewOperator::operator);

// Instantiate
let my_new_operator_with_foo_as_42 = ctx.operator(
    "my_new_operator: {foo: 42}"
).unwrap();

// ... and use:
ctx.fwd(my_new_operator_with_foo_as_42, data);
```

Essentially, once they are registered, macros and user defined operators work exactly as the builtins. Also, they overshadow the builtin names, so testing alternative implementations of built in operators is as easy as registering a new operator with the same name as a builtin.

### Going ellipsoidal

Much functionality related to geometrical geodesy can be associated with the ellipsoid model in use, and hence, in a software context, be modelled as methods on the ellipsoid object.

In RG, the ellipsoid is represented by the `Ellipsoid` data type:

```rust
pub struct Ellipsoid {
    a: f64,
    ay: f64,
    f: f64,
}
```

In most cases, the ellipsoid in use will be rotationally symmetrical, but RG anticipates the use of triaxial ellipsoids. As can be seen, the `Ellipsoid` data type is highly restricted, containing only the bare essentials for defining the ellipsoidal size and shape. All other items are implemented as methods:

```rust
let GRS80 = geodesy::Ellipsoid::named("GRS80");

let E = GRS80.linear_eccentricity();
let b = GRS80.semiminor_axis();
let c = GRS80.polar_radius_of_curvature();
let n = GRS80.third_flattening();
let es = GRS80.eccentricity_squared();
```

The functionality also includes ancillary latitudes, and computation of geodesics on the ellipsoid - see [example 01](../examples/01-geometrical-geodesy.rs) for details.


### Comming attractions

RG is in early-stage development, so a number of additions are planned.

#### Geometric geodesy

In `[Knudsen et al 2019]` we identified a small number of operations collectively considered the "bare minimum requirements for a geodetic transformation system":

1. Geodetic-to-Cartesian coordinate conversion, and its inverse.
2. Helmert transformations of various kinds (2D, 3D, 4D or, equivalently: 4 parameter, 3/7 parameter and 14/15 parameter).
3. The Molodensky transformation.
4. Horizontal grid shift (“NADCON-transformation”).
5. Vertical grid shift (ellipsoidal-to-orthometric height transformation).

Of these only the first is fully implemented in RG. The Molodensky transformation has not even been started at, while the remaining parts are in various stages of completion. These are **need to do** elements for near future work.

Also, a number of additional projections are in the pipeline: first and foremost the Mercator projection (used in nautical charts), and the Lambert conformal conic projection (used in aeronautical charts).

#### Physical geodesy

Plans for invading the domain of physical geodesy are limited, although the `Ellipsoid` data type will probably very soon be extended with entries for the *International Gravity Formula, 1930* and *The GRS80 gravity formula*.

#### Coordinate descriptors

Combining the generic `CoordinateTuple`s with `CoordinateDescriptor`s will make it possible to sanity check pipelines, and automate coordinate order and unit conversions.

#### Logging

The Rust ecosystem includes excellent logging facilities, just waiting to be implemented in RG.

### Philosophy


### Discussion

b

### Conclusion

b

### References

**Reference:** `[Knudsen et al 2019]`

Thomas Knudsen, Kristian Evers, Geir Arne Hjelle, Guðmundur Valsson, Martin Lidberg and Pasi Häkli: *The Bricks and Mortar for Contemporary Reimplementation of Legacy Nordic Transformations*. Geophysica (2019), 54(1), 107–116.

### Notes

**Note:** `[ellps implied]`

In both cases, the use of the GRS80 ellipsoid is implied, but may be expressly stated as  `utm: {zone: 32, ellps: GRS80}` resp. `proj=utm zone=32 ellps=GRS80`

**Note:** `[idiomatic Rust]`

In production, we would check the return of `ctx.operator(...)`, rather than just `unwrap()`ping:

```rust
if let Some(utm32) = ctx.operator("utm: {zone: 32}") {
    let copenhagen = C::geo(55., 12., 0., 0.);
    let stockholm = C::geo(59., 18., 0., 0.);
    ...
}
```
In C, using PROJ, the demo program would resemble this untested snippet:

```C
#include <proj.h>

#int main() {
    PJ_CONTEXT *C = proj_context_create();
    PJ *P = proj_create(C, "proj=utm zone=32");

    PJ_COORD copenhagen = proj_coord(12, 55, 0, 0);
    PJ_COORD stockholm = proj_coord(18, 59, 0, 0);

    /* Forward */
    copenhagen = proj_trans(P, PJ_FWD, copenhagen);
    stockholm = proj_trans(P, PJ_FWD, stockholm);

    /* ... and back */
    copenhagen = proj_trans(P, PJ_INV, copenhagen);
    stockholm = proj_trans(P, PJ_INV, stockholm);

    proj_destroy(P);
    proj_context_destroy(C);
}
```
