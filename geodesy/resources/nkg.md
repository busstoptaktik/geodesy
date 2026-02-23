# NKG Register

## NKG2020 transformation implementations

This section contains the Rust Geodesy (RG) implementations of the NKG2020
transformations from ITRF2014 to the national realizations of ETRS89, via
the "common deformation frame" NKG_ETRF2014, and the common frame ETRF2000,
as described by Häkli et al (2023)

- **Pasi Häkli,**
Kristian Evers, Lotti Jivall, Tobias Nilsson, Sveinung Himle,
Karin Kollo, Ivars Liepiņš, Eimuntas Paršeliūnas, Olav Vestøl
and Martin Lidberg, **2023:**
*NKG2020 transformation: An updated transformation between dynamic and static
reference frames in the Nordic and Baltic countries*.
Journal of Geodetic Science, 13(1), 2023, pp. 20220155.
[DOI](https://doi.org/10.1515/jogs-2022-0155)

---

### Sweden

```geodesy:itrf2014-sweref99

# ITRF2014 (geo, h, t) -> SWEREF99 (geo, h, t)

# Input is latitude/longitude/height/time with lat/lon in degrees, h in m, t in years CE,
# but the transformation machinery works in cartesian coordinates
|   geo:in
|   cart ellps=GRS80

# Go from ITRF2014(t) to ETRF2014(t) using EUREF parameters
|   helmert
:       angular_velocity = 85, 531, -770 uas
:       t_epoch = 1989  convention = position_vector

# Staying in ETRF2014, remove the frame deformation since 2000.0
|   deformation
:       inv t_epoch=2000.0
:       grids=nkgrf17vel

# Now, with t fixed at 2000.0, go from ETRF2014(2000) to ETRF97(2000),
# which is the frame SWEREF99 is based on
|   helmert
:       translation = 30.54, 46.06, -79.44 mm
:       rotation = 1419.58, 151.32, 1503.37 uas
:       scale = 0.003002
:       convention=position_vector

# Finally correct for the frame deformation from the pivot epoch of 2000
# to the realization epoch of 1999.5
|   deformation
:       dt=-0.5 grids=nkgrf17vel

# And get back to latitude/longitude/height/time
|   cart inv ellps=GRS80
|   geo:out
```

---

### Denmark

The 'dt' of the last deformation step in the DK transformation may seem odd, as it
does not agree with the formal realization epoch of 1994.704 (i.e. 1994-09-15).
This is due to a minor adjustment of the DK realization at epoch 2015, in effect
solid-body lifting it from the old passive markers onto the active CORS network.
Or, as described by Häkli et al (2023):

> The Danish ETRS89 realization was originally based on seven passive stations
> (Fankhauser and Gurtner 1995) which over time have shown some instability.
> To remedy this, the Danish ETRS89 realization was updated in 2019 to include
> CORS stations.
>
> ETRS89 coordinates for the CORS stations were determined using a Helmert
> transformation based on the EUREF2015 campaign that included observations of
> both the original passive stations and the CORS stations.
>
> The updated coordinates were determined in 2015, and hence, the intra-plate
> deformation epoch has changed to 2015.8 from the original epoch of 1992.7.
>
> Formally, the realization is unchanged, but in practice, the realization is
> now carried by the more geodynamically stable CORS stations.

The RG implementation here fits extremely well with the PROJ implementation (to
at least 12 decimal places) at the test point (55N, 12E) - potentially because the
deformation is much smaller in Denmark than in northern Sweden.

```geodesy:itrf2014-etrs89dk

# ITRF2014 (geo, h, t) -> ETRS89-DNK (geo, h, t)

# Input is latitude/longitude/height/time with lat/lon in degrees, h in m, t in years CE,
# but the transformation machinery works in cartesian coordinates
|   geo:in
|   cart ellps=GRS80

# Go from ITRF2014(t) to ETRF2014(t) using EUREF parameters
|   helmert
:       angular_velocity = 85, 531, -770 uas
:       t_epoch = 1989  convention = position_vector

# Staying in ETRF2014, remove the frame deformation since 2000.0
|   deformation inv
:      t_epoch=2000.0 grids=nkgrf17vel

# Now, with t fixed at 2000.0, go from ETRF2014(2000) to ETRF94(2000),
# which is the frame ETRS89-DNK is based on
|   helmert
:       translation = 668.18, 44.53, -450.49 mm
:       rotation = 3128.83, -23734.23, 4429.69 uas
:       scale =-0.003136
:       convention=position_vector

# Finally correct for the frame deformation from the pivot epoch of 2000
# to the *deformation realization* epoch of 2025.829 (replacing the original
# realization epoch of 1994.709)
|   deformation
:       dt=15.829 grids=eur_nkg_nkgrf17vel.deformation

# And get back to latitude/longitude/height/time
|   cart inv ellps=GRS80
|   geo:out
```

---

## PROJ implementations

### PROJ Sweden

Material extracted from PROJ, using the projinfo incantation
below. Output slightly edited for readability

```console
$ projinfo -o proj -s itrf2014 -t sweref99

Operation No. 1:
Conversion from ITRF2014 (geog2D) to ITRF2014 (geocentric) +
ITRF2014 to ETRF2014 (1) +
Inverse of NKG_ETRF14 to ETRF2014 +
NKG_ETRF14 to ETRF97@2000.0 +
ETRF97@2000.0 to ETRF97@1999.5 +
Conversion from SWEREF99 (geocentric) to SWEREF99 (geog2D)
0.02 m, Sweden - onshore and offshore.

+proj=pipeline
  +step +proj=axisswap +order=2,1
  +step +proj=unitconvert +xy_in=deg +xy_out=rad
  +step +proj=cart +ellps=GRS80
  +step +proj=helmert +x=0 +y=0 +z=0 +rx=0.001785 +ry=0.011151 +rz=-0.01617 +s=0
        +dx=0 +dy=0 +dz=0 +drx=8.5e-05 +dry=0.000531 +drz=-0.00077 +ds=0
        +t_epoch=2010 +convention=position_vector
  +step +inv +proj=deformation +t_epoch=2000 +grids=eur_nkg_nkgrf17vel.tif
        +ellps=GRS80
  +step +proj=helmert +x=0.03054 +y=0.04606 +z=-0.07944 +rx=0.00141958
        +ry=0.00015132 +rz=0.00150337 +s=0.003002 +convention=position_vector
  +step +proj=deformation +dt=-0.5 +grids=eur_nkg_nkgrf17vel.tif +ellps=GRS80
  +step +inv +proj=cart +ellps=GRS80
  +step +proj=unitconvert +xy_in=rad +xy_out=deg
  +step +proj=axisswap +order=2,1


And in single line format, suitable for cct

cct -d 10 proj=pipeline step proj=axisswap order=2,1 step proj=unitconvert xy_in=deg xy_out=rad step proj=cart ellps=GRS80 step proj=helmert drx=8.5e-05 dry=0.000531 drz=-0.00077 t_epoch=1989 convention=position_vector step inv proj=deformation t_epoch=2000 grids=eur_nkg_nkgrf17vel.tif ellps=GRS80 step proj=helmert x=0.03054 y=0.04606 z=-0.07944 rx=0.00141958 ry=0.00015132 rz=0.00150337 s=0.003002 convention=position_vector step proj=deformation dt=-0.5 grids=eur_nkg_nkgrf17vel.tif ellps=GRS80 step inv proj=cart ellps=GRS80 step proj=unitconvert xy_in=rad xy_out=deg step proj=axisswap order=2,1 -- untracked\nkg_test.pts >nkg_test.cct

cct Runs in 20 min 25 sec:

$ echo %time%
22:00:15,39

$ cct -d 10 proj=pipeline step proj=axisswap order=2,1 step proj=unitconvert xy_in=deg xy_out=rad step proj=cart ellps=GRS80  step proj=helmert drx=8.5e-05 dry=0.000531 drz=-0.00077 t_epoch=1989 convention=position_vector step inv proj=deformation t_epoch=2000 grids=eur_nkg_nkgrf17vel.tif ellps=GRS80 step proj=helmert x=0.03054 y=0.04606 z=-0.07944 rx=0.00141958 ry=0.00015132 rz=0.00150337 s=0.003002 convention=position_vector step proj=deformation dt=-0.5 grids=eur_nkg_nkgrf17vel.tif ellps=GRS80 step inv proj=cart ellps=GRS80 step proj=unitconvert xy_in=rad xy_out=deg step proj=axisswap order=2,1 -- untracked\nkg_test.pts >nkg_test.cct

$ echo %time%
22:20:40,21

kp runs in 17 sec

$ echo %time% && kp -vvd 10 nkg:itrf2014-sweref99 untracked\nkg_test.pts> nkg_test.kp
22:25:46,18
[2026-02-18T21:26:03Z INFO  kp] Read 10000000 coordinates and succesfully transformed 10000000 in 16.9419602s  (1.694µs each)
$ echo %time%
22:26:03,21

(fire-dev) C:\FLOW\AD\RG\geodesy>projsync --source-id eur_nkg
Downloading from https://cdn.proj.org into C:\Users\B004330\AppData\Local/proj
https://cdn.proj.org/eur_nkg_README.txt already downloaded.
https://cdn.proj.org/NKG already downloaded.
https://cdn.proj.org/eur_nkg_nkgrf03vel_realigned.tif already downloaded.
https://cdn.proj.org/eur_nkg_nkgrf17vel.tif already downloaded.

$ echo PROJ_NETWORK=%PROJ_NETWORK%
PROJ_NETWORK=ON

---

### PROJ Denmark

Material extracted from PROJ, using the projinfo incantation
below. Output slightly edited for readability

```console
$ projinfo -o proj -s itrf2014 -t etrs89 --area Denmark

Conversion from ITRF2014 (geog2D) to ITRF2014 (geocentric) +
ITRF2014 to ETRF2014 (1) +
Inverse of NKG_ETRF14 to ETRF2014 +
NKG_ETRF14 to ETRF92@2000.0 +
ETRF92@2000.0 to ETRF92@1994.704 +
Conversion from ETRS89 (geocentric) to ETRS89 (geog2D)
0.02 m, Denmark - onshore and offshore

+proj=pipeline
  +step +proj=axisswap +order=2,1
  +step +proj=unitconvert +xy_in=deg +xy_out=rad
  +step +proj=cart +ellps=GRS80
  +step +proj=helmert +x=0 +y=0 +z=0 +rx=0 +ry=0 +rz=0 +s=0 +dx=0 +dy=0 +dz=0
        +drx=8.5e-05 +dry=0.000531 +drz=-0.00077 +ds=0 +t_epoch=1989
        +convention=position_vector
  +step +inv +proj=deformation +t_epoch=2000.0 +grids=eur_nkg_nkgrf17vel.tif
  +step +proj=helmert +x=0.66818 +y=0.04453 +z=-0.45049 +rx=0.00312883
        +ry=-0.02373423 +rz=0.00442969 +s=-0.003136 +convention=position_vector
  +step +proj=deformation +dt=15.829 +grids=eur_nkg_nkgrf17vel.tif
  +step +inv +proj=cart +ellps=GRS80
  +step +proj=unitconvert +xy_in=rad +xy_out=deg
  +step +proj=axisswap +order=2,1
```
