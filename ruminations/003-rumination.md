# Ruminations on Rust Geodesy

## Rumination 003: `kp` - the RG Coordinate Processing program

Thomas Knudsen <knudsen.thomas@gmail.com>

2021-08-28. Last [revision](#document-history) 2023-11-24

### Abstract

```console
$ echo 55 12 | kp "geo:in | utm zone=32"
> 691875.6321 6098907.8250
```

---

### Prologue

`kp` is the Rust Geodesy **coordinate processing** program. The obvious
abbreviation of coordinate processing is `cp`, but since `cp` is the Unix file
copying program we substitute k for c - hence `kp`, which may be taken as a
reference to the Danish word for coordinate processing
**koordinatprocessering**.

Incidentally, `kp` was also the user-id and email address of the late **Knud
Poder** (1925-2019), during his work years at the Danish geodetic institute, GI
(and its successor, KMS), from the 1950s until his retirement in 1995.

For many years, Poder was in charge of the GI department for computational
geodesy where, for some years around 1980, his deputy was Carl Christian
Tscherning (1942-2014), for whom the [PROJ](https::/proj.org) transformation
program [cct](https://proj.org/apps/cct.html) was named. Among friends,
colleagues and collaborators worldwide, Knud Poder was regarded a Nestor of
computational geodesy.

### Usage

The basic operation of `kp` is very simple. Any complexity in `kp` usage is
related to the description of the operation to carry out, which is the subject
of [Rumination 002](/ruminations/002-rumination.md). The `kp` command line
syntax is:

```console
$ kp "operation" file1 file2 ...
> ...
```

or, with input from `stdin`:

```console
$ echo coordinate | kp "operation"
> ...
```

or, with output to the file `result`:

```console
$ kp -o result "operation" file1 file2 ...
> ...
```

### Examples

Convert the coordinate tuple (55 N, 12 E) from geographical coordinates on the
GRS80 ellipsoid to Universal Transverse Mercator, zone 32 coordinates on the
same (implied) ellipsoid:

```console
$ echo 55 12 0 0 | kp "geo:in | utm zone=32"
> 691875.6321 6098907.8250
```

While RG coordinates are always 4D, `kp` will provide zero-values for any
left-out postfix dimensions, and try to guess a proper number of output
dimensions (unless the `-D n` option is given):

```console
$ echo 55 12 | kp "geo:in | utm zone=32"
> 691875.63214 6098907.82501

$ echo 55 12 | kp -D3 "geo:in | utm zone=32"
> 691875.63214 6098907.82501 0.0000

$ echo 55 | kp "curvature mean"
> 6385431.75306

$ echo 55 | kp -D4 "curvature mean"
> 6385431.75306 0.00000 0.00000 NaN
```

The `roundtrip` option measures the roundtrip accuracy of a transformation (i.e.
how close to the origin you end up after a forward+inverse dance). Knud Poder
championed this practise with his ingeniously constructed *Poder dual
autochecking* method, which was essential at a time where computers were less
robust than today (more about that
[below](#a-few-more-words-about-knud-poder)).

```console
$ echo 55 12 | kp --roundtrip "geo:in | utm zone=32"
> -0.0000000000 -0.0000000000
```

The `inv` option runs the specified pipeline inversely:

```console
$ echo 691875.63214 6098907.82501 | kp --inv "geo:in | utm zone=32"
> 55.0000000000 12.0000000000
```

The `factors=` option provides deformation factors in a format inspired by the `proj`
options `-S` and `-V`. Contrary to `proj`, however, the `factors=` option requires an
argument, specifying the ellipsoid for the evaluation of the factors.

Also, contrary to `proj`, `kp` expects input in latitude-longitude order, and output
in northing-easting order, hence typically needing wrapping by a `geo:in | ... | neu:out`
pair. The output is given as 3 scales (meridional, parallel, areal), followed by 3 angular
items: (angular distortion, meridian/parallel, meridian convergence)

The long-format version (`proj`s `-V`) is invoked by combining the `factors` and `verbose`
options.

```console
$ echo 12 55 | proj -S +proj=utm +ellps=GRS80 +zone=32
691875.63  6098907.83  <1.00005 1.00005 1.0001 1.20736e-06 1.00005 1.00005>


$ echo 55 12 | kp --factors=GRS80 "geo:in | utm ellps=GRS80 zone=32 | neu:out"
6098907.82501 691875.63214   < 1.0000516809 1.0000516809 1.0001033645 | 0.00000 90.00000 2.45820 >


$ echo 12 55 | proj -V +proj=utm +ellps=GRS80 +zone=32
(...)
Longitude: 12dE [ 12 ]
Latitude:  55dN [ 55 ]
Easting (x):   691875.63
Northing (y):  6098907.83
Meridian scale (h) : 1.00005168  ( 0.005168 % error )
Parallel scale (k) : 1.00005168  ( 0.005168 % error )
Areal scale (s):     1.00010336  ( 0.01034 % error )
Angular distortion (w): 0.000
Meridian/Parallel angle: 90.00000
Convergence : 2d27'29.52" [ 2.45819987 ]
Max-min (Tissot axis a-b) scale error: 1.00005 1.00005


$ echo 55 12 | kp --factors=GRS80 --verbose "geo:in | utm ellps=GRS80 zone=32 | neu:out"
6098907.82501 691875.63214
Factors {
    meridional_scale: 1.0000516809310758,
    parallel_scale: 1.0000516808991158,
    areal_scale: 1.0001033645011084,
    angular_distortion: 2.1072335301308353e-8,
    meridian_parallel_angle: 89.99999879258172,
    meridian_convergence: 2.458199874328705,
    tissot_semimajor: 1.000051691451808,
    tissot_semiminor: 1.0000516703783837,
}
```

### Options

The `help` option gives the list of options:

```console
$ kp --help

KP: The Rust Geodesy 'Coordinate Processing' program

Usage: kp [OPTIONS] <OPERATION> [ARGS]...

Arguments:
  <OPERATION>  The operation to carry out e.g. 'kp "utm zone=32"'
  [ARGS]...    The files to operate on

Options:
      --inv                    Inverse operation
      --factors <FACTORS>      Specify a base ellipsoid for evaluation of deformation
                               factors, based on the jacobian
  -z, --height <HEIGHT>        Specify a fixed height for all coordinates
  -t, --time <TIME>            Specify a fixed observation time for all coordinates
  -d, --decimals <DECIMALS>    Number of decimals in output
  -D, --dimension <DIMENSION>  Output dimensionality - default: Estimate from input
      --debug                  Activate debug mode
  -r, --roundtrip              Report fwd-inv roundtrip deviation
  -e, --echo                   Echo input to output
  -v, --verbose...             Increase logging verbosity
  -q, --quiet...               Decrease logging verbosity
  -o, --output <OUTPUT>        Output file, stdout if not present
  -h, --help                   Print help
  -V, --version                Print version
```

### Operators

The current crop of RG operators is described in the
[missing manual](/ruminations/002-rumination.md)

### A few more words about Knud Poder

On the occasion of Knud Poder's 90th birthday in 2015, I wrote a few words about
one of his accomplishments on the
[PROJ mailing list](https://lists.osgeo.org/pipermail/proj/2015-October/006884.html):

> As described in a recent thread, for the next release, proj.4 will switch the
> default transverse mercator implementation from tmerc to etmerc.
>
> This is probably a good occasion to reiterate the history of the code for the
> etmerc implementation - especially since the original author, Knud Poder,
> turned 90 on October 19th. Having his transverse mercator implementation
> becoming the proj.4 default is a strikingly proper way of celebrating Poder,
> among colleagues and collaborators rightfully considered “the Nestor of
> computational geodesy”.
>
> Poder wrote the first version of what is now known as etmerc, around 1961. It
> was written in Algol-60 and ran on the GIER computer, built for the Danish
> Geodetic Institute (see [1] for details).
>
> The code was based on theoretical foundations published a decade earlier, by
> König & Weise ([2], building on prior work by Krüger, 1912 [3]).
>
> Poder’s work was characterized by great care with respect to numerical
> precision and accuracy (e.g. by using Clenshaw summation for recurrence
> series, and Horner’s scheme for polynomial evaluation).
>
> Also, Poder was noted for his ingeniously implemented “dual autochecking
> method” (not used in the proj.4 version), where the same code was used for
> forward and inverse projections and was run both ways and compared, to protect
> against both coding and hardware errors. The latter was very important at a
> time where the mean time between failure for computer systems was much shorter
> than today.
>
> During the 1970s Poder’s student, Karsten Engsager (the “E” in etmerc,
> “Engsager Extended Transverse Mercator”) took over maintenance and eventually
> extended König and Weise’s numerical series by another term, bringing the
> accuracy up to today’s standard.
>
> In 2008, through the efforts of a.o. Gerald Evenden, Frank Warmerdam and
> Karsten Engsager, etmerc was introduced in proj.4, while in 2013 Charles
> Karney provided 3 corrections - stressing the value and importance of open
> source code sharing.
>
> Poder retired 20 years ago, but has been taking active interest in the
> maintenance and development of his code ever since. Switching proj.4 to use a
> transverse mercator implementation based on his work is probably the best
> conceivable way of celebrating the 90th birthday of a great Nestor of
> computational geodesy.
>
> In celebration of Knud Poder!
>
> /Thomas Knudsen, Danish Geodata Agency
>
>
> [1] Thomas Knudsen, Simon L. Kokkendorff, Karsten E. Engsager (2012): A Vivid
> Relic Under Rapid Transformation, OSGeo Journal vol. 10, pp. 55-57, URL
> <https://journal.osgeo.org/index.php/journal/article/download/200/167>
>
> [2] R. König and K. H. Weise (1951): Mathematische Grundlagen der Höheren
> Geodäsie und Kartographie, Erster Band. Springer, Berlin/Göttingen/Heidelberg,
> 1951.
>
> [3] L. Krüger (1912): Konforme Abbildung des Erdellipsoids in der Ebene. Neue
> Folge 52. Royal Prussian Geodetic Institute, Potsdam. URL
> <http://bib.gfz-potsdam.de/pub/digi/krueger2.pdf>

### Document History

Major revisions and additions:

- 2021-08-28: Initial version
- 2022-05-08: Reflect current syntax
- 2023-08-17: Graphical clean up
- 2023-11-20: Reflect the current --help text
- 2023-11-24: Automatic selection of output dimensionality
- 2025-07-11: Reflow paragraphs to 85 characters
