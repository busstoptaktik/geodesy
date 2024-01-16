# NKG Register

## Geodesy implementations

This section contains the Rust Geodesy (RG) implmentations of the NKG transformations from ITRF2014 to the national realizations of ETRS89


### Sweden

```geodesy:itrf2014-sweref99
|   adapt from=neuf_deg
|   cart ellps=GRS80
|   helmert
:      drx = 0.000085  dry = 0.000531  drz = -0.00077 ds = 0
:       t_epoch=1989    convention=position_vector
|   deformation
:       inv t_epoch=2000.0
:       grids=eur_nkg_nkgrf17vel.deformation
|   helmert
:       x = 0.03054 rx = 0.00141958
:       y = 0.04606 ry = 0.00015132
:       z =-0.07944 rz = 0.00150337
:       s = 0.003002
:       convention=position_vector
|   deformation dt=0.5 grids=eur_nkg_nkgrf17vel.deformation
|   cart inv ellps=GRS80
|   adapt to=neuf_deg
```

### Denmark

The 'dt' of the last deformation step in the DK transformation seems odd:
It does not seem to agree with the realization epoch of 1994.704 (i.e. 1994-09-15).
This is due to a minor adjustment of the DK realization at epoch 2015, in effect
solid-body lifting it from the old passive markers onto the active CORS network.

The RG implementation here fits extremely well with the PROJ implementation (to
at least 12 decimal places) at the test point (55N, 12E) - potentially because the
deformation is much smaller in Copenhagen than in Stockholm.

```geodesy:itrf2014-etrs89dk
|   adapt from=neuf_deg
|   cart ellps=GRS80
|   helmert
:       drx = 0.000085  dry = 0.000531  drz = -0.00077
:       t_epoch = 1989  convention = position_vector
|   deformation inv
       t_epoch=2000.0 grids=eur_nkg_nkgrf17vel.deformation
|   helmert
:       x = 0.66818  rx = 0.00312883
:       y = 0.04453  ry =-0.02373423
:       z =-0.45049  rz = 0.00442969
:       s =-0.003136 convention=position_vector
|   deformation inv
:       dt=15.829 grids=eur_nkg_nkgrf17vel.deformation
|   cart inv ellps=GRS80
|   adapt to=neuf_deg
```

```geodesy:test
|   adapt from=neuf_deg
|   cart ellps=GRS80
|   helmert
:       drx = 0.000085  dry = 0.000531  drz = -0.00077
:       t_epoch = 1989  convention = position_vector
|   helmert
:       x = 0.66818  rx = 0.00312883
:       y = 0.04453  ry =-0.02373423
:       z =-0.45049  rz = 0.00442969
:       s =-0.003136 convention=position_vector
|   cart inv ellps=GRS80
|   adapt to=neuf_deg
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
  +step +proj=helmert +x=0 +y=0 +z=0 +rx=0 +ry=0 +rz=0 +s=0 +dx=0 +dy=0 +dz=0
        +drx=8.5e-05 +dry=0.000531 +drz=-0.00077 +ds=0 +t_epoch=1989
        +convention=position_vector
  +step +inv +proj=deformation +t_epoch=2000.0 +grids=eur_nkg_nkgrf17vel.tif
  +step +proj=helmert +x=0.03054 +y=0.04606 +z=-0.07944 +rx=0.00141958
        +ry=0.00015132 +rz=0.00150337 +s=0.003002 +convention=position_vector
  +step +proj=deformation +dt=-0.5 +grids=eur_nkg_nkgrf17vel.tif
  +step +inv +proj=cart +ellps=GRS80
  +step +proj=unitconvert +xy_in=rad +xy_out=deg
  +step +proj=axisswap +order=2,1
```

Note that the direction of the last deformation has been swapped by swapping the
sign of 'dt', as this seems to fit best with PROJ at the test point (59N, 18E).
Still some tiny differences (1/100 mm), though - and some unclear things about the
interpretation of Fwd and Inv wrt. the deformation model implementation

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
