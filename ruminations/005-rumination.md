# Ruminations on Rust Geodesy

## Rumination 005: Divided by a common language

Thomas Knudsen <knudsen.thomas@gmail.com>

2021-04-06. Last [revision](#document-history) 2022-05-30

### Abstract

```sh
$ echo 55 12 | kp "geodesy:in | geomatics:out"
> 691875.6321 6098907.8250 0.0000 0.0000
```

---

### Prologue

The following was submitted as an abstract for a talk given at the [ISO/TC-211](https://isotc211.org) workshop on "Shared Concepts" 2021-04-13/14.

The motto for the talk was George Bernard Shaw's observation that

- *England and America are two countries divided by a common language*,

with the modification that

- *Geodesy and Geoinformatics are two sciences divided by a common language*

And, since the time slot for the talk was limited, also the warning was given that

> Exaggeration promotes understanding, so I’ll leave out the details to make room for the exaggeration.
>
> So consider the following a sketchy caricature, intended to introduce a potentially illustrative case of “mismatched shared concepts”.
>
> The reason for the mismatch is historical, but the consequences will be tangible in a not-so-distant future.

With that noted, here comes the abstract

---

### Divided by a common language: Bridging the gap between the ideal world of geoinformatics and the messy realities of geodesy

**Geodetic terminology** has always been messy - for instance, geodesists in general do not care whether point identification is given as geographic, geocentric or projected coordinates, as long as it makes geodetic sense, i.e. as long as the reference frame in use allows lossless conversion between those coordinates.

Neither do coordinate order (latitude/longitude vs. longitude/latitude), units (radians, degrees, gradians, meter, feet, furlongs), or angular representation (DD.ddd, DD+MM.mmm, DD+MM+SS.sss) matter at all to your average anecdotal geodesist.

What matters is basically

1. The reference frame ("datum"),
2. Which operations can be carried out meaningfully within that reference frame, and
3. Which transformations can meaningfully be implemented between that and other reference frames.

**Geoinformatics terminology**, on the other hand, as reflected by the ISO-19100 series, is strict, consistent, and in some cases not at all in accordance with geodetic practice: The strict terminology describes a world of platonic ideals, as if a branch mathematics, while the geodetic world view is one of geophysics, with all the messy real world noise, exceptions and imprecisions that follow.

The ISO-19100 world view, as it stands today, is nice, clear and sufficient if you are *either* satisfied with decimeter level accuracy, *or* if you do all your work within a single reference frame. If you need consistent-and-better-than-decimeter accuracy across reference frames, you also need to dip your toes into the muddy, shark-laden waters of geodesy.

Essentially this is because in geoinformatics, coordinate reference systems ("CRS") are real objects with internal structure, that can be described, and lead one to infer transformations, while in geodesy, a CRS is simply a label, and transformations are the real objects.

The task for the comming years will be to arrange for two marriages:

- One between geodesy in general and the strict terminology of geoinformatics.
- The other between general geodetic transformations and the platonic world view of geoinformatics.

---

### Epilogue

The actual presentation paraphrased the conclusion of the abstract with these words:

**ISO 19111:** Well defined concepts built on the illusion of an axiomatic foundation

**Geodesy:** Sloppy linguistic concepts, but built on empirical observations, not requiring axiomatic foundations.

**Mission:** Marry geodesy and geoinformatics

- A simplified ISO 19111, (better) reflecting the real world
- Improved conceptual rigor in geodesy
- The changes needed are probably surprisingly small
- ... but that’s the subject of another talk

---

### Document History

Major revisions and additions:

- 2021-04-06: Initial (abstract) version
- 2021-04-13: Presentation
- 2022-05-30: Rebuilt as rumination
- 2023-06-06: Minor corrections
