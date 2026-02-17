# NKG Register

## Geodesy implementations

This section contains the Rust Geodesy (RG) implementations of the NKG
transformations from ITRF2014 to the national realizations of ETRS89

---

### Sweden

```geodesy:itrf2014-sweref99

# Input is latitude/longitude/height/time with lat/lon in degrees, h in m, t in years CE,
# but the transformation machinery works in cartesian coordinates
|   adapt from=neuf_deg
|   cart ellps=GRS80

# Go from ITRF2014(t) to ETRF2014(t) using EUREF parameters
|   helmert
:       rotation = 1785, 11151, -16170
:       angular_velocity = 85, 531, -770 uas
:       t_epoch=1989
:       convention=position_vector

# Staying in ETRF2014, remove the frame deformation since 2000.0
|   deformation
:       inv t_epoch=2000.0
:       grids=nkgrf17vel

# Now, with t fixed at 2000.0, go from ETRF2014(2000) to ETRF97(2000),
# which is the frame SWEREF99 is based on
|   helmert
:       translation = 0.03054, 0.04606, -0.07944
:       rotation = 1419.58, 151.32, 1503.37 uas
:       scale = 0.003002
:       convention=position_vector

# Finally correct for the frame deformation from the pivot epoch of 2000
# to the realization epoch of 1999.5
|   deformation
:       dt=-0.5 grids=nkgrf17vel

# And get back to latitude/longitude/height/time
|   cart inv ellps=GRS80
|   adapt to=neuf_deg
```

---

### Denmark

The 'dt' of the last deformation step in the DK transformation may seem odd, as
it does not agree with the realization epoch of 1994.704 (i.e. 1994-09-15).
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

# Input is latitude/longitude/height/time with lat/lon in degrees, h in m, t in years CE,
# but the transformation machinery works in cartesian coordinates
|   adapt from=neuf_deg
|   cart ellps=GRS80

# Go from ITRF2014(t) to ETRF2014(t) using EUREF parameters
|   helmert
:       angular_velocity = 85, 531, -770 uas
:       t_epoch = 1989  convention = position_vector

# Staying in ETRF2014, remove the frame deformation since 2000.0
|   deformation inv
       t_epoch=2000.0 grids=eur_nkg_nkgrf17vel.deformation

# Now, with t fixed at 2000.0, go from ETRF2014(2000) to ETRF94(2000),
# which is the frame ETRS89-DNK is based on
|   helmert
:       translation = 0.66818, 0.04453, -0.45049
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
|   adapt to=neuf_deg
```

```console
# Testing itrf2014-etrs89dk

# Copenhagen (-ish)

$ echo 55 12 0 2026 | cs2cs -d 18 itrf2014 etrs89 --area Denmark
54.999994299501942407   11.999989796428588207 -0.014318614266812801 2026

$ echo 55 12 0 2026 | cargo r --release -- -d 18 nkg:itrf2014-etrs89dk
54.999994299501949513 11.999989796428593536 -0.014318615464977544 2026.000000000000000000

# PROJ and RG are geodesically indiscernible
$ echo 54.999994299501949513 11.999989796428593536 54.999994299501942407   11.999989796428588207 | kp "inv geodesic"
0.0000000000 0.0000000000 0.0000000000 180.0000000000

# And the height difference, at around a nanometre is absolutely acceptable
$ eva 0.014318615464977544-0.014318614266812801
0.0000000012

# But, as we can see here, the total deformation during 26 years is only 19 mm, so not much to handle
$ echo 55 12 0 2026 | cargo r --release -- -d 18 "geo:in | cart | deformation raw t_epoch=2000 grids=nkgrf17vel"
0.016842274583094469 -0.005351231340245891 0.008113172013357279 0.019445345204122853
```

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
```

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

```console

Odd difference between the EUREF expressions for SE and DK (difference in epoch balances differences in rx, ry, rz):

SE: +step +proj=helmert +rx=0.001785 +ry=0.011151 +rz=-0.01617 +drx=8.5e-05 +dry=0.000531 +drz=-0.00077 +t_epoch=2010 +convention=position_vector
DK: +step +proj=helmert +rx=0        +ry=0        +rz=0        +drx=8.5e-05 +dry=0.000531 +drz=-0.00077 +t_epoch=1989 +convention=position_vector
```
