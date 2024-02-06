# Teach yourself Geodesy in less than 600 seconds (of arc)

**Thomas Knudsen,** <thokn@sdfi.dk>, 2024-02-05

## Introduction

The following is a brief overview of the geodetic software package **Rust [Geodesy](https://github.com/busstoptaktik/geodesy)** (a.k.a. RG, or simply **Geodesy** - proper noun, capital G). The text is intended for consumption in its entirety, in one sitting, before trying out the software. It should provide you with a first feeling for the architecture, the concepts, and the syntax used, prior to taking the deep dive.

As a guide to further exploration, the text is followed by a collection of [usage examples](#examples), and a list of suggested [further reading](#further-reading).

**RG** shares many characteristics with the [PROJ](https://proj.org) transformation system, so the fundamental concepts are readily understood by PROJ users. The RG syntax is, however, slightly different, and much less verbose than its PROJ counterpart, so seasoned PROJ users may benefit from focusing primarily on the syntax descriptions below.

### About the title - and a bit about units

While in all likelihood it is impossible to teach yourself geodesy in 600 seconds, it is much more likely to teach yourself **Geodesy** in that amount of time.

The title, however, refers to 600 seconds *of arc*, which corresponds to *10 minutes* of arc. A minute of arc is the historical definition of one nautical mile, and a speed of one knot corresponds to one nautical mile per hour.

So to fit the 600 seconds of time to the 600 seconds of arc, you will have to navigate through this text at a speed of 60 knots. So better get going - see you at the finish line!

## Overview

At its most basic level, RG provides a number of elementary geodetic computational **operators**.

**Operators** read a stream of *input coordinates*, modify them by applying some specific algorithm, and write an identically sized stream of *output coordinates*.

Most operators exist in *forward* and *inverse* incarnations. For example:

- the **forward utm-operator** takes *geographical coordinates* as its input, and provides *utm coordinates* as its output, while
- the **inverse utm-operator** does the opposite: takes *utm coordinates* as input, and provides the corresponding *geographical coordinates* as output.

The *elementary operators* can be combined into more *complex operations* using the RG **pipeline mechanism**, where the *output* of one operator provides the *input* of another.

Pipelines can be generalized in the form of **macros**, with or without *parameters*. The macros can be collected in **registers**, organizing and documenting collections of (preferably) related macros.

### Operators

Most operators take **parameters,** which may be *mandatory* or *optional*. For example, the `utm` operator takes a mandatory parameter, `zone`, indicating which utm-zone it should operate in, e.g.

```geodesy
utm zone=32
```

Since utm coordinates are ellipsoidal, the `utm`-operator also needs to be told *which* ellipsoid to refer to. But that parameter is optional, and defaults to GRS80. So this more elaborate version:

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

For completeness' sake, let us look at a case where we want to convert geographical coordinates defined on one ellipsoid, to geographical coordinates defined on another (typically these kinds of work will also involve a datum shift step, which we for simplicity leave out here). In this case, we have two steps, each taking en `ellps` parameter, but where we need different *values* for the twor parameters:

```geodesy
cart ellps=$ellps_in(GRS80) | inv cart ellps=$ellps_out(GRS80)
```

Which can be invoked as `cart:utm ellps_in=intl ellps_out=GRS80`, to convert from coordinates on the International (Hayford) Ellipsoid, to coordinates on the GRS80 ellipsoid

### Registers

Registers are collections of (preferably) related macros - e.g. macros implementing pipelines for transformation from a given coordinate system, to a number of other coordinate systems, or e.g. transformations originating from a given publisher of geodetic parameters (of which the [EPSG](https://epsg.org) is probably the most well known).

For improved readability of long pipelines, using the
'step-separators-at-column-1' formatting, we introduce
':' as line continuation characters. They are ignored,
but potentially makes the pipeline slightly easier to
read.

Tests in token.rs and parsed_parameters.rs extended
correspondingly

Registers for the 'plain' context provider are now in MarkDown
format, for better communication to end users. Single element
resource files are still in the old format for rapid testing.

Tests for macro parameter defaults (foo=*0) and
lookups (foo=$bar) have been enhanced.

## Examples

### How to use the examples

The examples below are based on the Geodesy coordinate processing tool [**kp**](https://github.com/busstoptaktik/geodesy/blob/main/ruminations/003-rumination.md).

While the Geodesy operators typically take angular input in radians and (longitude, latitude)-order, humans tend to be more familiar with degrees and the nautical convention of geographical coordinates in degrees and (latitude, longitude)-order.

To mediate between the different representations, Geodesy provides a number of macros. For now, we need only consider the `geo:in` macro, which converts human readable geographical coordinates to the internal representation.

Using `geo:in`, we may convert the approximate geographical coordinates of Copenhagen, Denmark (55 N, 12 E), to UTM zone 32 coordinates, by saying:

```console
$ echo 55 12 | kp "geo:in | utm zone=32"

691875.63214 6098907.82501
```

Note that the output is in (easting, northing) order. We can use the `neu:out` macro to switch to (northing, easting, up) order, where the "up" part is ignored, since the input is two-dimensional:

```console
$ echo 55 12 | kp "geo:in | utm zone=32 | neu:out"

6098907.82501 691875.63214
```

For three-dimensional input. we get three-dimensional output:

```console
$ echo 55 12 100 | kp "geo:in | utm zone=32 | neu:out"

6098907.82501 691875.63214 100.00000
```

If we leave out the unit conversion, the numbers are interpreted as 55 radians east, 12 radians north, and the output is garbage:

```console
echo 55 12 | kp "utm zone=32"
-7198047.1103082076 -11321644.2251116671
```

## Further reading
