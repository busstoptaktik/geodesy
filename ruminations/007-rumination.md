# Ruminations on Rust Geodesy

## Rumination 007: Operator parameter introspection

Thomas Knudsen <knudsen.thomas@gmail.com>

2023-08-01. Last [revision](#document-history) 2023-08-01

### Abstract

```rust
let mut ctx = Minimal::new();
let op = ctx.op("geo:in | utm zone=32 | neu:out")?;
let steps = ctx.steps(op)?;

assert_eq!(steps.len(), 3);
assert_eq!(steps[0], "geo:in");
assert_eq!(steps[1], "utm zone=32");
assert_eq!(steps[2], "neu:out");
```

---

### Introspection

As of 2023-08-01, RG now supports some very limited introspection functionality, making it possible to take a "look at the inside" of an operator, and dissect its internal organization.

The first, and most high-level feature, simply provides a vector of strings, representing the definition of each step, as shown in the abstract above.

The second is more low-level, and requires a good amount of understanding of RG's internal machinery to utilize: It provides raw (read-only) access to the `ParsedParameters`-struct used by the internal "inner" operators, to access their individual parameter setups.

There is not much more to say about the subject than what can be read from the test material in the `Minimal` context provider - which for convenience is also included below.

```rust
#[test]
fn introspection() -> Result<(), Error> {
    let mut ctx = Minimal::new();

    let op = ctx.op("geo:in | utm zone=32 | neu:out")?;

    let mut data = some_basic_coordinates();
    assert_eq!(data[0][0], 55.);
    assert_eq!(data[1][0], 59.);

    ctx.apply(op, Fwd, &mut data)?;
    assert!((data[0][0] - 6098907.82501).abs() < 1e-4);
    assert!((data[0][1] - 691875.63214).abs() < 1e-4);

    // The text definitions of each step
    let steps = ctx.steps(op)?;
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0], "geo:in");
    assert_eq!(steps[1], "utm zone=32");
    assert_eq!(steps[2], "neu:out");

    // Behind the curtains, the two i/o-macros are just calls to the 'adapt' operator
    assert_eq!("adapt", ctx.params(op, 0)?.name);
    assert_eq!("adapt", ctx.params(op, 2)?.name);

    // While the utm step really is the 'utm' operator, not 'tmerc'-with-extras
    assert_eq!("utm", ctx.params(op, 1)?.name);

    // All the 'common' elements (lat_?, lon_?, x_?, y_? etc.) defaults to 0,
    // while ellps_? defaults to GRS80 - so they are there even though we havent
    // set them
    let ellps = ctx.params(op, 1)?.ellps(0);
    assert_eq!(ellps.semimajor_axis(), 6378137.);
    assert_eq!(0., ctx.params(op, 1)?.lat(0));

    // The zone id is found among the natural numbers (which here includes 0)
    let zone = ctx.params(op, 1)?.natural("zone")?;
    assert_eq!(zone, 32);

    // Taking a look at the internals is not too hard either
    // let params = ctx.params(op, 0)?;
    // dbg!(params);

    Ok(())
}

```

### Document History

Major revisions and additions:

- 2023-08-01: First light
