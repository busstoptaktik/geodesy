# Comments on Rust Geodesy

## Comment 001: A few words about an often-seen pipeline

Thomas Knudsen <knudsen.thomas@gmail.com>

2021-08-11. Last [revision](#document-history) 2021-08-11

---

### Prologue

In the Rust Geodesy source code, test cases, and documentation, you will often encounter this transformation pipeline:

```js
cart ellps:intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80
```

It was selected as the *go to* example because it is only marginally more complex than the identity operator, `noop`, while still doing real geodetic work. So by implementing just two operators, `cart` and `helmert` we can already:

- Provide instructive examples of useful geodetic work
- Test the RG operator instantiation
- Test the internal data flow architecture
- Develop test- and documentation workflows
- and in general get a good view of the RG *look and feel*

For these reasons, `cart` and `helmert` were the first two operators implemented in RG.

### The operators

**cart** converts from geographcal coordinates, to earth centered cartesian coordinates (and v.v. in the inverse case).

**helmert** performs the Helmert transformation which in the simple 3-parameter case used here simply adds the parameters `[x, y, z]` to the input coordinate `[X, Y, Z]`, so the output becomes `[X+x, Y+y, Z+z]`, or in our case: `[X-87, Y-96, Z-120]` (due to the negative signs of `x, y, z`).

### What happens?

From end-to-end:

1. The `cart` stage takes geographical coordinates given on the *international ellipsoid* (`ellps:intl`) and converts them to earth-centered cartesian coordinates
2. The `helmert` stage shifts the cartesian coordinates to a new origin `[x,y,z]`
3. Finally, the inverse `cart` stage converts the cartesian coordinates back to geographical coordinates. This time on the *GRS80 ellipsoid* (`ellps:GRS80`)

### What does it mean?

All-in-all, this amounts to a *datum shift* from the older "European Datum, 1950", *ED50*, to the current "European Terrestrial Reference Frame 1989", *ETRS89*.

It is not a particularly good datum shift, but it is sufficient in many cases: The expected transformation error is on the order of 5 m, whereas one will get an error of around 200 m if not transforming at all. In other words, this simple transformation reduces the coordinate error from "a few blocks down the road" to "the wrong side of the road".

### Where did it come from?

The pipeline described above is actually the [GYS](000-comment#gys-the-ghastly-yaml-shorthand) representation of datum transformation number [1134](https://epsg.org/transformation_1134/ED50-to-WGS-84-2.html) in the [EPSG](https://epsg.org/home.html) geodetic registry, where it is described as *EPSG:1134 - ED50 to WGS 84*. In RG contexts we refer to it as *ED50 to ETRS89* since, at the level of accuracy of EPSG:1134, ETRS89 and WGS84 are equivalent.

### Document History

Major revisions and additions:

- 2021-08-11: First version
