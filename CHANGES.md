# Geodesy

## 0.12.0 Release notes

### Improvements

- Alternative parameter names for the `helmert` operator (`translations`, `rotations` etc.) (Nic Hill)
- Improved syntax and parameter handling (#92) (Thomas Knudsen)
  - `>` and `<` can be used to indicate pipeline steps that are `omit_inv` and `omit_fwd` respectively (syntactic sugar for `| omit_inv`, etc.)
  - True utf-8 subscripts supported in parameter names (e.g. `x₀`), as alternatives to the existing `x_0`-style
  - More free form syntax: Modifiers `inv`, `omit_inv` and `omit_fwd` may now prefix the operator name, i.e. `inv utm zone=32` is functionally identical to `utm inv zone=32`
- Implement the `axisswap` operator (#84) (Thomas Knudsen)
- Replace `once_cell` with `std::sync::OneLock`. (Corey Farwell)
- KP: Guess output dimensionality (#82) (Thomas Knudsen)
- Implement `unitconvert` operator (#80) (Sean Rennie)
- Implement latlon, lonlat, latlong, and longlat as noop-aliases (Thomas Knudsen)

### Bug fixes

- LAEA: correct lon_0 in the inverse 'rho==0' case (#90) (Thomas Knudsen)
- Mark `clap-verbosity-flag` as an optional dep (#81) (Corey Farwell)
- Repair links in README.md (Thomas Knudsen)

### Acknowledgements

Geodesy v0.12.0 materialized through contributions from

- [Corey Farwell](https://github.com/frewsxcv)
- [Nic Hill](https://github.com/nrhill1)
- [Sean Rennie](https://github.com/Rennzie)
- [Thomas Knudsen](https://github.com/busstoptaktik)

## 0.11.0 Release notes

### Improvements

- Handle lists-of-grids, `@optional` grids, and the `@null` grid in `grids=` clauses
- Support NTv2 format datum shift grids
- Overall documentation brush up and extension
- Implement `somerc`, the Swiss Oblique Mercator operator
- Implement `deformation`, the 3D intrapalte deformation operator
- Rename the `NMEA` operators to `dm/dms` and `iso_dm/iso_dms`
- Support jacobian-of-projection and the corresponding deformation factors
- `proj_parse`: Translate PROJ syntax to Rust Geodesy syntax
  with partial support for PROJ ellipsoid definitions
- Through `proj_parse`, the `Plain` context, and hence `kp` now supports PROJ syntax
  (although with the limitations implied by Geodesy only supporting parts of the PROJ
  operator gamut, and by the differing input-output conventions between `kp` and `proj/cct`)
  as demonstrated by this example:

   ```sh
   echo 55 12 | kp geo:in | kp "proj=pipeline step proj=utm zone=32 step inv proj=utm zone=32" | kp geo:out
   ```

  which does nothing, in a very convoluted way
- Partial operator introspection
- General support for 2D, 3D, 4D, and 32 bit 2D coordinates (`Coor2D`, `Coor3D`, `Coor4D`, `Coor32`)
- Hence `Coord` is gone

### Bug fixes

- Avoid double correction for lat_0 in inverse tmerc

### Acknowledgements

A huge thank you goes to [Sean Rennie](https://github.com/Rennzie) who a.o. did most of the work on the improved grid support in 0.11.0, and to [Kyle Barron](https://github.com/kylebarron) for pushing Geodesy over the WASM barrier in 0.10.0
