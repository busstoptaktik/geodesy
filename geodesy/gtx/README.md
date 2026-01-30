# Lineage

## egm96_15_subset

The `egm96_15_subset.gtx` file descends from the
[`egm96_15.gtx` file in the PROJ-datumgrid](https://github.com/OSGeo/proj-datumgrid/blob/master/egm96_15.gtx)
repository.

The parent file is in the public domain, and originates from the
[NGA](https://earth-info.nga.mil/index.php?dir=wgs84&action=wgs84),
back in the 1990's. It is known as the "Worldwide EGM96 15 minute interpolation grid".

The original material is still available from the
[NGA download server](https://earth-info.nga.mil/php/download.php?file=egm-96interpolation).

The GTX-format version is, however, related to the NOAA/NOS VDatum program.
The format description is available from the NOAA
[VDatum page](https://vdatum.noaa.gov/docs/gtx_info.html#dev_gtx_binary).

The subsetting was done using `gdal_translate`:

```console
$ gdal_translate -projwin 7.875 58.125 16.125 53.875 egm96_15.gtx egm96_15_subset.gtx

$ gdalinfo egm96_15_subset.gtx

Driver: GTX/NOAA Vertical Datum .GTX
...
Size is 33, 17
Origin = (7.875000000000000,58.125000000000000)
Pixel Size = (0.250000000000000,-0.250000000000000)
Corner Coordinates:
Upper Left  (   7.8750000,  58.1250000) (  7d52'30.00"E, 58d 7'30.00"N)
Lower Left  (   7.8750000,  53.8750000) (  7d52'30.00"E, 53d52'30.00"N)
Upper Right (  16.1250000,  58.1250000) ( 16d 7'30.00"E, 58d 7'30.00"N)
Lower Right (  16.1250000,  53.8750000) ( 16d 7'30.00"E, 53d52'30.00"N)
Center      (  12.0000000,  56.0000000) ( 12d 0' 0.00"E, 56d 0' 0.00"N)
Band 1 Block=33x1 Type=Float32, ColorInterp=Undefined
  NoData Value=-88.8888
```

The intention is to extract the grid nodes inside, or on the border of,
the area 8E 58N 16E 54N.

GDAL, however, employs the image (area), rather than grid (point) convention for describing
grid coordinate systems. Hence, we need to extend the area by half-a-nodesize in all
directions, to get what we aim at.

A few point values were extracted for implementation of tests, and for checking conformance
between the original and the subset grid:

```console
# Directly on the grid center node
$ gdallocationinfo -valonly -geoloc -r bilinear egm96_15.gtx 12 56
36.5965881347656
$ gdallocationinfo -valonly -geoloc -r bilinear egm96_15_subset.gtx 12 56
36.5965881347656

# A nearby point requiring interpolation
$ gdallocationinfo -valonly -geoloc -r bilinear egm96_15.gtx 12.1 56.1
36.6803439331055
$ gdallocationinfo -valonly -geoloc -r bilinear egm96_15_subset.gtx 12.1 56.1
36.6803439331055

# Check that the georeference is correct: "Interpolation" directly onto a grid node
# returns the exact value of that grid node:

# Northwestern corner
$ gdallocationinfo -valonly -geoloc -r bilinear egm96_15_subset.gtx 8 58
40.7404823303223
# Same, but in grid relative coordinates
$ gdallocationinfo -valonly -r bilinear egm96_15_subset.gtx 0 0
40.7404823303223

# Southeastern corner
$ gdallocationinfo -valonly -geoloc -r bilinear egm96_15_subset.gtx 16 54
33.721004486084
$ gdallocationinfo -valonly -r bilinear egm96_15_subset.gtx 32 16
33.7333202362061
# ^^^^^^^^^^^^^^ wrong!
$ gdallocationinfo -valonly -r bilinear egm96_15_subset.gtx 33 17
# No response - appears to be considered outside of the grid
$ gdallocationinfo -valonly -r bilinear egm96_15_subset.gtx 32.9999999 16.9999999
33.721004486084
# ^^^^^^^^^^^^^ correct!

# We also get the correct value, using nearest neighbour "interpolation"
$ gdallocationinfo -valonly -r nearest egm96_15_subset.gtx 32 16
33.721004486084
$ gdallocationinfo -valonly -r nearest egm96_15_subset.gtx 0 0
40.7404823303223
```

- So gdal appears to extract a grid the size of 33 x 17 grid nodes, in a grid stretching
  from grid address (0, 0) to (32, 16).
- GDAL also appears to map those to the correct outline of 8E-16E and 54N-58N.

Also:

- Due to the fence post effect, 33 x 17 nodes result in 32 x 16 steps.
- This correctly results in 15' intervals:
  (16 - 8)° / 32 = 0.25° = 15', and
  (58 - 14)° / 16 = 0.25° = 15'.

But why must we extend the adressing almost all the way towards the (non-existing)
grid node at (33, 17), in order to obtain the proper grid value for (32, 16)?
