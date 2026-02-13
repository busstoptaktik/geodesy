# NKG Register

## Geodesy implementations

This section contains the Rust Geodesy (RG) implementations of the NKG
transformations from ITRF2014 to the national realizations of ETRS89

### Sweden

```geodesy:itrf2014-sweref99
|   adapt from=neuf_deg
|   cart ellps=GRS80
|   helmert
:       angular_velocity = 85, 531, -770 uas      # drx=8.5e-05 dry=0.000531 drz=-0.00077
:       scale = 0 ds = 0 t_epoch=1989
:       convention=position_vector
|   deformation
:       inv t_epoch=2000.0
:       grids=nkgrf17vel
|   helmert
:       translation = 0.03054, 0.04606, -0.07944  # x=0.03054 y=0.04606 z=-0.07944
:       rotation = 1419.58, 151.32, 1503.37 uas   # rx=0.00141958 ry=0.00015132 rz=0.00150337
:       scale = 0.003002
:       convention=position_vector
|   deformation
:       dt=-0.5 grids=nkgrf17vel
|   cart inv ellps=GRS80
|   adapt to=neuf_deg
```

#### Stepwise validation against the canonical PROJ implementation

```console
# Luleå

# RG
$ echo 66 23 0 2026 | cargo r --release -- -d 18 nkg:itrf2014-sweref99
65.999994796742242897 22.999985921476536532 -0.283234661271775323 2026

# PROJ
$  echo 66 23 0 2026 | cs2cs -d 18 itrf2014 sweref99
65.999994796942800690   22.999985921414740631 -0.283244616352021694 2026

# Geodesic distance between RG and PROJ: 2/100 of a millimeter. Acceptable, but note that the horizontal deformation component is minimal: Luleå is very close to the apex of the dome shaped deformation figure.
$ echo 65.999994796742242897 22.999985921476536532     65.999994796942800690   22.999985921414740631 | kp "geodesic inv"
-7.1478357533 -7.1478357533 0.0000225377 172.8521642467

# Also the height difference is quite acceptable at slightly less than 1/100 mm
$ eva 0.283234661271775323-0.283244616352021694
-0.0000099551

# RG roundtrip
echo 66 23 0 2026 | (cargo r --release -- -d 18 nkg:itrf2014-sweref99) | cargo r --release -- -d 18 --inv  nkg:itrf2014-sweref99
65.999999999999374722 23.000000000001225686 0.000000109806814470 2026.000000000000000000

# Geodesic distance between origin and roundtrip: 88 nm
$ echo 66 23  65.999999999999374722 23.000000000001225686 | kp "geodesic inv"
  141.2987046480 141.2987046480 0.0000000889 321.2987046480
# The height diffference, since the origin is a 0 m, is identical to the 3rd output coordinate,
# i.e. 89 nm

```


### Denmark

The 'dt' of the last deformation step in the DK transformation may seem odd, as
it does not agree with the realization epoch of 1994.704 (i.e. 1994-09-15).
This is due to a minor adjustment of the DK realization at epoch 2015, in effect
solid-body lifting it from the old passive markers onto the active CORS network.

The RG implementation here fits extremely well with the PROJ implementation (to
at least 12 decimal places) at the test point (55N, 12E) - potentially because the
deformation is much smaller in Denmark than in northern Sweden.

```geodesy:itrf2014-etrs89dk
|   adapt from=neuf_deg
|   cart ellps=GRS80
|   helmert
:       angular_velocity = 85, 531, -770 uas
:       # drx = 0.000085  dry = 0.000531  drz = -0.00077
:       t_epoch = 1989  convention = position_vector
|   deformation inv
       t_epoch=2000.0 grids=eur_nkg_nkgrf17vel.deformation
|   helmert
:       translation = 0.66818, 0.04453, -0.45049
:       rotation = 3128.83, -23734.23, 4429.69 uas
:       scale =-0.003136
:       convention=position_vector
:       # x = 0.66818  rx = 0.00312883
:       # y = 0.04453  ry =-0.02373423
:       # z =-0.45049  rz = 0.00442969
|   deformation
:       dt=15.829 grids=eur_nkg_nkgrf17vel.deformation
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
`´`

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

The 'dt' of the last deformation step in the DK transformation seems odd:
It does not seem to agree with the realization epoch of 1994.704 (i.e. 1994-09-15).
This is due to a minor adjustment of the DK realization at epoch 2015, in effect
solid-body lifting it from the old passive markers onto the active CORS network.

The RG implementation here fits extremely well with the PROJ implementation (to
at least 12 decimal places) at the test point (55N, 12E) - potentially because the
deformation is much smaller in Copenhagen than in Stockholm.
