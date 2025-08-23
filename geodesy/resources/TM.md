# TM register

Register of geographical systems and definitions based on the transverse mercator projection

### ITM

The Irish Transverse Mercator: ETRS89 based TM with origin, scale, and false origin fit
for Ireland

```geodesy:itm_core
tmerc lat_0=53.5 lon_0=-8 k_0=0.99982 x_0=600000 y_0=750000 ellps=GRS80
```

```geodesy:ITM
geo:in | TM:itm_core | enu:out
```

### DKTM

```geodesy:dktm1_core
tmerc lat_0=0 lon_0=9     k=0.99998 x_0=200000 y_0=-5000000 ellps=GRS80
```

```geodesy:DKTM1
# haha
geo:in | TM:dktm1_core | enu:out
```

```geodesy:dktm2_core
tmerc lat_0=0 lon_0=10    k=0.99998 x_0=400000 y_0=-5000000 ellps=GRS80
```

```geodesy:DKTM2
geo:in | TM:dktm2_core | enu:out
```

```geodesy:dktm3_core
tmerc lat_0=0 lon_0=11.75 k=0.99998 x_0=600000 y_0=-5000000 ellps=GRS80
```

```geodesy:DKTM3
geo:in | TM:dktm3_core | enu:out
```

```geodesy:dktm4_core
tmerc lat_0=0 lon_0=15    k=1       x_0=800000 y_0=-5000000 ellps=GRS80
```

```geodesy:DKTM4
geo:in | TM:dktm4_core | enu:out
```

```
<DKTM1_DVR90> proj=pipeline step init=DK:DVR90 step init=DK:DKTM1
<DKTM2_DVR90> proj=pipeline step init=DK:DVR90 step init=DK:DKTM2
<DKTM3_DVR90> proj=pipeline step init=DK:DVR90 step init=DK:DKTM3
<DKTM4_DVR90> proj=pipeline step init=DK:DVR90 step init=DK:DKTM4

<DKTM1_DNN> proj=pipeline step init=DK:DNN step init=DK:DKTM1
<DKTM2_DNN> proj=pipeline step init=DK:DNN step init=DK:DKTM2
<DKTM3_DNN> proj=pipeline step init=DK:DNN step init=DK:DKTM3
<DKTM4_DNN> proj=pipeline step init=DK:DNN step init=DK:DKTM4
```
