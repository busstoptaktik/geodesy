# Interpolation comparison

```console

# The PROJ value for lower left corner (40N, 0E)

$ echo 0 40 0 0 | cct -d 14 proj=hgridshift grids=100800401.gsb -- | awk "{print $2, $1}
> 39.99882421663926 -0.00120312787967

# The ESRI value for the same point, computed using https://github.com/Esri/ntv2-file-routines

$ ntv2_cvt -f 100800401.gsb 40 0
> 39.99882421665721 -0.001203127834531996

# The interpolated grid value using grid.interpolation(...)

[src\grid\ntv2\mod.rs:108] corr = Coor4D(
    [
        -0.001203127834531996,
        -0.0011757833427853057,
        0.0,
        0.0,
    ],
)

# The exact grid value
[src\grid\ntv2\mod.rs:113] lat_corr = -0.0011757833427853057
[src\grid\ntv2\mod.rs:114] lon_corr = 0.001203127834531996

# Difference between exact grid value and grid.interpolation(...)
[src\grid\ntv2\mod.rs:118] (dlat, dlon) = (
    0.0,
    0.0,
)

# Difference, in approx nanometers, between grid.interpolation(...) and PROJ
[src\grid\ntv2\mod.rs:123] (proj_dlat, proj_dlon) = (
    1077.0539278157075,
    4513.808250783402,
)

# Difference, in approx nanometers, between grid.interpolation(...) and ESRI
[src\grid\ntv2\mod.rs:128] (esri_dlat, esri_dlon) = (
    0.1553574976997929,
    0.0,
)
```

GDAL plays "off by half" tricks and misrepresents the grid extent:

```console
$ gdalinfo 100800401.gsb

> ...
> Corner Coordinates:
> Upper Left  (  -0.0416667,  43.0416667) (  0d 2'30.00"W, 43d 2'30.00"N)
> Lower Left  (  -0.0416667,  39.9583333) (  0d 2'30.00"W, 39d57'30.00"N)
> Upper Right (   3.5416667,  43.0416667) (  3d32'30.00"E, 43d 2'30.00"N)
> Lower Right (   3.5416667,  39.9583333) (  3d32'30.00"E, 39d57'30.00"N)
> Center      (   1.7500000,  41.5000000) (  1d45' 0.00"E, 41d30' 0.00"N)
> ...
```

compared to what you read from the NTv2-header:

```console

[src\grid\ntv2\mod.rs:105] h = SubGridHeader {
    num_nodes: 1591.0,
    nlat: 43.0,
    slat: 40.0,
    wlon: 0.0,
    elon: 3.4999999999999996,
    dlat: 5.0,
    dlon: 5.0,
    num_rows: 37.0,
    row_size: 43.0,
}

```
