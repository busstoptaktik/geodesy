# Teach yourself Geodesy in less than 900 seconds (of arc)

**Thomas Knudsen,** <thokn@sdfi.dk>, 2024-02-05

## Introduction

The following is a brief overview of the geodetic software package **Rust [Geodesy](https://github.com/busstoptaktik/geodesy)** (a.k.a. RG, or simply **Geodesy** - proper noun, capital G). The text is intended for consumption in its entirety, in one sitting, before trying out the software. It should provide you with a first feeling for the architecture, the concepts, and the syntax used, prior to taking the deep dive.

As a guide to further exploration, the text is followed by a collection of [usage examples](#examples), and a list of suggested [further reading](#further-reading).

**RG** shares many characteristics with the [PROJ](https://proj.org) transformation system, so the fundamental concepts are readily understood by PROJ users. The RG syntax is, however, slightly different, and much less verbose than its PROJ counterpart, so seasoned PROJ users may benefit from focusing primarily on the syntax descriptions below.

### About the title - and a bit about units

While in all likelihood it is impossible to teach yourself geodesy in 900 seconds, it is much more likely to teach yourself **Geodesy** in that amount of time.

The title, however, refers to 900 seconds *of arc*, which corresponds to a quarter of a degree, i.e. *15 minutes* of arc. A minute of arc is the historical definition of one nautical mile, and a speed of one knot corresponds to one nautical mile per hour.

So to fit the 900 seconds of time to the 900 seconds of arc, you will have to navigate through this text at a speed of 60 knots. So better get going - see you at the finish line!

### Prerequisites

The intention with the following text is to give a quick introduction to the nuts and bolts of the software package **Geodesy**. Not to teach you the nuts and bolts of the *science* of geodesy.

Hence, as a prerequisite, you are supposed to understand enough about geographic coordinates to grasp the "About the title..." section above. Also, you need to feel comfortable with the concepts of *ellipsoids* and *UTM coordinates*, and at least to know the existence of [PROJ](https://proj.org) and of [cartesian geocentric](https://en.wikipedia.org/wiki/Earth-centered,_Earth-fixed_coordinate_system) coordinates.

## Overview

At its most basic level, RG provides a number of elementary geodetic **operators**.

An **operator** reads a stream of *input coordinates*, modifies them by applying its associated algorithm, and writes an identically sized stream of *output coordinates*.

Most operators exist in *forward* and *inverse* incarnations. For example:

- the **forward UTM-operator** takes *geographical coordinates* as its input, and provides *UTM coordinates* as its output, while
- the **inverse UTM-operator** does the opposite: takes *UTM coordinates* as input, and provides the corresponding *geographical coordinates* as output.

The *elementary operators* can be combined into more *complex operations* using the RG **pipeline mechanism**, where the *output* of one operator provides the *input* of another.

Pipelines can be generalized in the form of **macros**, with or without *parameters*. The macros can be collected in **registers**, organizing and documenting collections of (preferably) related macros.

### Operators

Most operators take **parameters,** which may be *mandatory* or *optional*. For example, the `utm` operator takes a mandatory parameter, `zone`, indicating which UTM zone it should operate in, e.g.

```geodesy
utm zone=32
```

Note that operators and parameters are conventionally given lower case names, so the operator implementing UTM projections, is called `utm`, rather than `UTM`.

Since UTM coordinates are ellipsoidal, the `utm`-operator also needs to be told *which* ellipsoid to refer to. But that parameter is optional, and defaults to GRS80. So this more elaborate version:

```geodesy
utm zone=32 ellps=GRS80
```

works identically to the previous.

#### Inverse operators

Inverse operators are instantiated by providing the `inv`-modifier:

```geodesy
inv utm zone=32
```

**Modifiers,** like `inv`, are *special kinds of parameters* which may be given anywhere in the operator definition (whereas *ordinary parameters* must be given *after* the operator name).

But since modifiers have such a drastic influence, it is useful to **place them in front** of the entire expression for better visibility.

Apart from `inv`, only two modifiers exist: `omit_fwd` and `omit_inv`. They are rare beasts though, and only used inside **pipelines**.

### Pipelines

**Pipelines** are collections of operators, (which are referred to as **steps**), organized in lockstep, such that the output of the first step goes to the input of the second, the output of the second to the input of the third, and so on.

Pipelines are built using the vertical bar syntax, e.g. to build a pipeline taking geocentric cartesian coordinates as input, and giving utm coordinates with ellipsoidal heights as output, you will say:

```geodesy
inv cart | utm zone=32
```

a syntax clearly mimicking the Unix shell pipe-construct, which is also the inspiration for the PROJ pipeline construct upon which Geodesy pipelines are modelled.

#### Inverted pipelines

Just Like elementary operators, a pipeline can also be inverted - at least as long as each of its steps can. When executing a pipeline in inverse mode, each step is inverted, and the pipeline is executed from back to front.

Hence, inverse execution of the pipeline above corresponds to forward execution of this pipeline:

```geodesy
inv utm zone=32 | cart
```

In some advanced use cases (out-of-scope for this text), you may need to omit some steps when executing a pipeline in either forward or inverse. Those steps should be modified using the `omit_fwd` or `omit_inv` modifiers mentioned above.

### Macros

**NOTE:** The impatient reader may now skip to the [**examples**](#examples), and return here when convenient.

If we often need the geocentric cartesian pipeline above, we may define it as a **macro**, so we don't have to type the entire definition every time we need it, but can make do with just typing a shorter macro name, e.g by defining `cart:utm` as:

```geodesy
inv cart | utm zone=32
```

This is, however, an unreasonably inflexible macro. Typically, we would want a macro to take parameters, fitting it to a given context. We can do that by marking parameter values as optional by wrapping them in parentheses:

```geodesy
inv cart | utm zone=(32)
```

In this case, invoking the macro as `cart:utm zone=42` will bring you zone 42 coordinates, while the plain `cart:utm` will bring you coordinates from the default zone 32.

You may even take the value from a differently named macro parameter, by using the dereference sigil '$':

```geodesy
inv cart | utm zone=$foo
```

Here, invoking the macro as `cart:utm foo=42` will bring you zone 42 coordinates, while the plain `cart:utm` will bring you a syntax error. This can be remedied by combining the two macro parameter value expansion functionalities:

```geodesy
inv cart | utm zone=$foo(32)
```

which will bring you zone 32 coordinates, unless the macro parameter `foo` is defined, in which case its value will be used for the zone parameter.

For completeness' sake, let's consider a case where we want to convert geographical coordinates defined on one ellipsoid, to geographical coordinates defined on another.

Typically these kinds of work will also involve a datum shift step, which, for simplicity, we leave out here. In this case, we have two steps, each taking an `ellps` parameter, but where we need different *values* for the two parameters:

```geodesy
cart ellps=$ellps_in(GRS80) | inv cart ellps=$ellps_out(GRS80)
```

Which can be invoked as `cart:utm ellps_in=intl ellps_out=GRS80`, to convert from coordinates on the International (Hayford) Ellipsoid, to coordinates on the GRS80 ellipsoid

### Contexts and Registers

Registers are collections of related macros - e.g. pipelines for transformation from a given coordinate system, to a number of other coordinate systems. Or transformations originating from a given publisher of geodetic parameters (of which the [EPSG](https://epsg.org) is probably the most well known).

To be of any use, registers must be made accessible ("published"), and preferably in a form convenient for the given use case. For example, the EPSG register is made available for human lookup through the [EPSG.org](https://epsg.org) web site, and, through the skillful hands of [Even Rouault](https://spatialys.com), as an [SQLite](https://sqlite.com) database, for automated lookup by the [PROJ](https://proj.org/en/9.3/resource_files.html#proj-db) transformation system.

No matter how the register is made available, it is fundamentally *external* to the transformation system. So some kind of *interface* between the system and the external world is needed. In **Geodesy** (and in PROJ), that kind of interface is called a **Context**.

**Geodesy**'s architecture allows end users to plug-in their own Context implementation, hence adapting Geodesy to the user's operational environment. But to be of any use out-of-the-box, Geodesy also provides two ready made Context implementations, called **Minimal** and **Plain**.

The **Minimal** Context is, as its name suggests, minimal. It does not provide any actual interfacing to an external operating environment, but it is useful for writing self-contained, reproducible test cases, where the external operating environment is simulated by provision of fixed value replacements (["mocking"](https://en.wikipedia.org/wiki/Mock_object)).

The **Plain** Context, on the other hand, is more generally useful. It is the Context used by the Geodesy command line interface `kp`, which you will encounter in the [**Examples**](#examples) section below. Plain is named after the plain text files it uses to access register information. Actually, as you will see in a moment, the "plain text" is written using [Markdown](https://en.wikipedia.org/wiki/Markdown) conventions, and hence may serve as a combined representation of human readable documentation and machine readable register items.

#### The Plain register format

Essentially, a register element is just a bunch of text representing a Geodesy pipeline, as shown in the examples in the [Pipelines](#pipelines) section above. But for Geodesy to be able to refer to the pipelines by name, the individual pipelines are highlighted in named sections using the Markdown code block syntax "```"

````text

```geodesy:pointless
# a rather pointless example of a register item
# doing nothing, by first turning geographical
# coordinates into UTM, then back again

utm zone=32 | inv utm zone=32
```
````

The `geodesy:pointless` identifier tells the Markdown formatter that *this is code using the Geodesy Pipeline format*, and that the name of this pipeline is `pointless`.

By placing the text block in the file `./geodesy/resources/my_register.md`, Geodesy, using the Plain Context, will know it as the macro `my_register:pointless`.

Now, as pipelines grow larger, the single-line format used above, becomes increasingly unreadable, and it becomes advantageous to utilize the free-format characteristics of pipelines. This makes it possible to place the step delimiters at the start of lines, split the steps by line breaks, and to use a few short cuts to make large step incantations more readable by splitting into continuation lines, as exemplified by this metasyntactical example:

````text

```geodesy:free_format_example

| this is the first step
| this is the second step
:    the second step continues here
:    and here
| this is the third step
> the fourth step is taken only in forward mode
:    i.e. ">" is shorthand for "| omit_inv"
< the fifth step is taken only in inverse mode
:    i.e. "<" is shorthand for "| omit_fwd"

| Blank lines and # inline comments are OK
# Block comments too

```
````

Also see the [NKG sample registry](https://github.com/busstoptaktik/geodesy/blob/main/geodesy/resources/nkg.md) in the Geodesy source code.

## Examples

### How to use the examples

The examples below are based on the Geodesy coordinate processing tool [**kp**](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/003-rumination.md).

While the Geodesy operators typically take angular input in radians and (longitude, latitude)-order, humans tend to be more familiar with degrees and the nautical convention of geographical coordinates in degrees and (latitude, longitude)-order.

To mediate between the different representations, Geodesy provides a number of macros. For now, we need only consider the `geo:in` macro, which converts human readable geographical coordinates to the internal representation.

### Example: UTM coordinates

Using `geo:in`, we may convert the approximate geographical coordinates of Copenhagen, Denmark (55 N, 12 E), to UTM zone 32 coordinates, by saying:

```console
$ echo 55 12 | kp "geo:in | utm zone=32"

691875.63214 6098907.82501
```

### Example: Selecting the output format

Note that the output is in (easting, northing) order. We can use the `neu:out` macro to switch to (northing, easting, up) order, where the "up" part is ignored, since the input is two-dimensional:

```console
$ echo 55 12 | kp "geo:in | utm zone=32 | neu:out"

6098907.82501 691875.63214
```

For three-dimensional input, we get three-dimensional output:

```console
$ echo 55 12 100 | kp "geo:in | utm zone=32 | neu:out"

6098907.82501 691875.63214 100.00000
```

### Example: garbage in, garbage out

If we leave out the unit conversion, the numbers are interpreted as 55 radians east, 12 radians north, and the output is garbage:

```console
$ echo 55 12 | kp "utm zone=32"

-7198047.1103082076 -11321644.2251116671
```

### Example: distances and directions on the ellipsoid

**Geodesy** includes functionality for computations involving geodesics on the ellipsoid. These kind of computations are historically known as "the two geodetic main problems":

- The **first** geodetic main problem: Knowing the point-of-departure, the navigation azimuth, and the distance travelled, determine the coordinates of the destination
- The **second** geodetic main problem: Knowing the point-of-departure, and the destination, determine the azimuth of the course, and the distance between the two.

Due to these historical naming conventions, the first geodetic main problem is implemented at the forward direction functionality of the `geodesic` operator, while the second geodetic main problem is implemented as its inverse direction functionality.

Contrary to the examples above, the `geodesic` operator takes four numbers as its input: Two geographical coordinate tuples for the second main problem. One geographical coordinate tuple, an azimuth, and a distance, for the first main problm.

Also, by convention, the `geodesic` operator takes input in latitude-longitude order and in degrees. Hence, no adapters needed.

First, let us compute the azimuth and distance between Copenhagen (55 N, 12 E) and Paris (49 N, 2 E):

```console
$ echo 55 12 49 2 | kp "inv geodesic"

-130.1540604204 -138.0525794184 956066.2319619625 41.9474205816
```
The output is interpreted as:

- The forward azimuth at the point of departure
- The forward azimuth at the destination
- The distance between the two points and
- The return azimuth from the destination

If this is a bit too many azimuths to grasp at once, you may instead use `geodesic`'s `reversible` option:

```console
$ echo 55 12 49 2 | kp "inv geodesic reversible"

49.0000000000 2.0000000000 41.9474205816 956066.2319619625
```
Where the output consists of:

- The latitude of the destination
- The longitude of the destination
- The return azimuth and
- The distance

The `reversible` moniker hints at the fact that its output format is identical to the input format expected for the `geodesic` forward direction operation, hence enabling an easy roundtrip check:

```console
$ echo 55 12 49 2 | kp "inv geodesic reversible | geodesic"

55.0000000000 12.0000000000 49.0000000000 2.0000000000
```
Which is seen to match exactly to a precision of at least 10 decimals.

## Further reading

### Geodesy ruminations

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
