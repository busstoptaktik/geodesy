# X - the transformation register

Register of transformations - primarily based on similarity/Helmert transforms

## ITRF2020

Transformations with ITRF2020 as the source

### GR96(2021)

**Description from**
*GR96: Greenland Reference 1996, Geodætisk systembeskrivelse,*
GeoNotes 13, Version 2, 2025-08-01

> I forbindelse med overførelsen af den primære GR96-realisering fra REFGR til GNET,
> er der bestemt et sæt af transformationsparametre, som kan bruges til at transformere
> koordinater fra IGS20 epoke 2021-08-14 til GR96(2021).
>
> [desuden benyttes] de 3 rotationshastigheder for den nordamerikanske del af
> ITRF2020-plademodellen (Altamimi et al., 2023), som gør det muligt at transformere
> fra en vilkårlig IGS20 epoke.
>
> Rotationshastighederne svarer til en bevægelse på ca. 20 mm/år i nordvestlig retning.

#### itrf2020_gr96

```geodesy:itrf2020_gr96

helmert exact
    convention = position_vector
    translation = -0.30031, -1.17512, -0.30654           # meter
    rotation = 0.041614, -0.026303, -0.011214            # arc-seconds
    scale = -0.01626                                     # ppm
    angular_velocity = 0.000045, -0.000666, -0.000098    # arc-seconds/year
    t_epoch = 2021.6164                                  # 2021-08-14

```

---

## GR96

Transformations with GR96(1996) as the source. They provide the (shaky) connection
to the older Greenland datums Qoornoq, Ammassalik, and Scoresbysund.

---

### Qoornoq (Qôrnoκ?)

#### gr96_qoornoq

```geodesy:gr96_qoornoq

helmert exact inv convention = coordinate_frame
    translation = 197.8579, 146.5947, -108.8501    # meter
    rotation = -0.85735, 0.36082, 0.38626          # arc-seconds
    scale = -8.356137                              # ppm

```

#### geo_gr96_qoornoq

```geodesy:geo_gr96_qoornoq
geo:in | cart ellps=GRS80 | X:gr96_qoornoq | inv cart ellps=intl | geo:out
```

#### Compare results from KP and KMSTrans2

```console
KMSTrans2:
geoEqornoq   64                -51                 0
geoEgr96     63.998 719 082    -50.995 543 584    30.8619

KP:
echo 64 -51 0 | kp "X:geo_gr96_qoornoq inv" | clip
kp           63.998 719 0817   -50.995 543 5843 30.8619079967
```

#### qoornoq_gr96

```geodesy:qoornoq_gr96

helmert exact
    convention = coordinate_frame
    translation = 197.8579, 146.5947, -108.8501     # meter
    rotation = -0.85735, 0.36082, 0.38626           # arc-seconds
    scale = -8.356137                               # ppm

```

#### geo_qoornoq_gr96

```geodesy:geo_qoornoq_gr96
geo:in | cart ellps=intl | X:qoornoq_gr96 | inv cart ellps=GRS80 | geo:out
```

---

### Ammassalik

The published constants implement the Ammassalik->GR96 case, so we use helmert inv
to make the name fit the constants

#### gr96_ammassalik

```geodesy:gr96_ammassalik
helmert inv
    exact convention=coordinate_frame
    translation=308.9415,136.2020,986.3661
    rotation=-3.87420,3.77827,-7.61345
    scale=-171.673150
```

#### geo_gr96_ammassalik

```geodesy:geo_gr96_ammassalik
geo:in | cart ellps=GRS80 | X:gr96_ammassalik | inv cart ellps=intl | geo:out
```

#### geo_gr96_ammassalik_xyz

```geodesy:geo_gr96_ammassalik_xyz
geo:in | cart ellps=GRS80 | X:gr96_ammassalik
```

#### xyz_gr96_ammassalik_geo

```geodesy:xyz_gr96_ammassalik_geo
cart ellps=GRS80 | X:gr96_ammassalik | geo:out
```

---

### Scoresbysund

#### gr96_scoresbysund

```geodesy:gr96_scoresbysund
helmert inv
    exact convention=coordinate_frame
    translation=218.2330,270.6151,253.1391
    rotation=0.26337,-0.15733,-1.19862
    scale=-59.923872
```

#### geo_gr96_scoresbysund

```geodesy:geo_gr96_scoresbysund
geo:in | cart ellps=GRS80 | X:gr96_scoresbysund | inv cart ellps=intl | geo:out
```

#### Authoritative definition from KMSTrans2 def_lab.txt

```txt
#scosd
 22    gr96     Hayford  0  GL
 7                218.2330 m    270.6151 m    253.1391 m
 -59.923872 ppm     0.26337 sx   -0.15733 sx   -1.19862 sx
Scoresbysund datum
```
