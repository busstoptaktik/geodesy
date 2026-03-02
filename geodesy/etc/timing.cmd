@echo off

if .%1==. goto nothing_to_do
if .%1==.prepare goto prepare
if .%1==.nkg goto nkg
if .%1==.noop goto noop
if .%1==.utm goto utm
if .%1==.subset goto subset

goto unknown_command_argument

:prepare
rem ----------------------------------------------------------
rem Prepare transformation timing by creating 10 million
rem random coordinates in untracked\timing\nkg_test.pts
rem ----------------------------------------------------------
if exist untracked\timing\nkg_test.pts goto nkg_test_pts_exist
echo creating test input file with 10_000_000 coordinates
echo in untracked\timing\nkg_test.pts (takes a while)
md untracked\timing 2>nul
python geodesy/etc/nkg_test.py
goto end


:nkg

rem ----------------------------------------------------------
rem NKG transformations: relative timing between kp and cs2cs
rem ----------------------------------------------------------
if not exist untracked\timing\nkg_test.pts goto missing_nkg_test_pts

rem echo --------------------------- %time%
rem echo Compiling
rem cargo -q b --release
echo --------------------------- %time%
echo Running kp
kp -vvd 10 nkg:itrf2014-sweref99 untracked\timing\nkg_test.pts> untracked\timing\nkg_test.kp
echo --------------------------- %time%
echo Running cs2cs
cs2cs -d 10 --3d --only-best=yes --no-ballpark itrf2014 sweref99 untracked\timing\nkg_test.pts >untracked\timing\nkg_test.cs2cs
echo --------------------------- %time%

rem
rem RESULTS
rem
rem Running with kp first.
rem     ITRF2014->SWEREF99  kp: 16.81 s, cct: 1238 s, kp  73.64 times faster
rem
rem $ geodesy\etc\timing
rem ---------------------------  5:56:45,29
rem Running kp
rem [2026-02-23T04:57:02Z INFO  kp] Read 10000000 coordinates and succesfully transformed 10000000 in 16.8152154s  (1.681µs each)
rem ---------------------------  5:57:02,61
rem Running cs2cs
rem ---------------------------  6:17:40,23
rem $ eva 1238/16.81
rem 73.6466389054
rem
rem Running with cs2cs first.
rem     ITRF2014->SWEREF99  kp: 16.29 s, cct: 1196 s, kp  73.41 times faster
rem
rem $ geodesy\etc\timing nkg
rem ---------------------------  7:48:14,45
rem Running cs2cs
rem ---------------------------  8:08:10,72
rem Running kp
rem [2026-02-23T07:08:27Z INFO  kp] Read 10000000 coordinates and succesfully transformed 10000000 in 16.2937762s  (1.629µs each)
rem ---------------------------  8:08:27,30
rem $ eva 1196/16.29
rem 73.4192756292


goto end


:noop

rem ----------------------------------------------------------
rem Noop: relative timing between kp and cct
rem ----------------------------------------------------------

rem echo --------------------------- %time%
rem echo Compiling
rem cargo -q b --release
echo One noop
echo --------------------------- %time%
echo Running cct
cct -d 10 +proj=noop untracked\timing\nkg_test.pts > untracked\timing\nkg_test.noop_cct
echo --------------------------- %time%
echo Running kp
kp -vvd 10 noop untracked\timing\nkg_test.pts > untracked\timing\nkg_test.noop_kp
echo --------------------------- %time%

echo Eight noops
echo --------------------------- %time%
echo Running cct
cct -d 10 +proj=pipeline +step +proj=noop +step +proj=noop +step +proj=noop +step +proj=noop +step +proj=noop +step +proj=noop +step +proj=noop +step +proj=noop untracked\timing\nkg_test.pts > untracked\timing\nkg_test.noop_cct
echo --------------------------- %time%
echo Running kp
kp -vvd 10 "noop|noop|noop|noop|noop|noop|noop|noop" untracked\timing\nkg_test.pts > untracked\timing\nkg_test.noop_kp
echo --------------------------- %time%

rem
rem RESULTS
rem
rem Running with kp first.
rem     One noop.     kp: 8.95 s, cct: 83 s, kp  9.27 times faster
rem     Eight noops.  kp: 9.38 s, cct: 97 s. kp 10.34 times faster
rem
rem $ geodesy\etc\timing noop
rem One noop
rem ---------------------------  4:42:05,39
rem Running kp
rem [2026-02-23T03:42:14Z INFO  kp] Read 10000000 coordinates and succesfully transformed 10000000 in 8.9521567s  (895ns each)
rem ---------------------------  4:42:14,65
rem Running cct
rem ---------------------------  4:43:37,90
rem Eight noops
rem ---------------------------  4:43:37,90
rem Running kp
rem [2026-02-23T03:43:47Z INFO  kp] Read 10000000 coordinates and succesfully transformed 10000000 in 9.3825174s  (938ns each)
rem ---------------------------  4:43:47,76
rem Running cct
rem ---------------------------  4:45:20,96
rem
rem Running with cct first.
rem     One noop.     kp: 9.56 s, cct: 83 s, kp  8.68 times faster
rem     Eight noops.  kp: 9.65 s, cct: 92 s. kp  9.53 times faster
rem
rem $ geodesy\etc\timing noop
rem One noop
rem ---------------------------  4:48:08,62
rem Running cct
rem ---------------------------  4:49:31,43
rem Running kp
rem [2026-02-23T03:49:41Z INFO  kp] Read 10000000 coordinates and succesfully transformed 10000000 in 9.5586385s  (955ns each)
rem ---------------------------  4:49:41,11
rem Eight noops
rem ---------------------------  4:49:41,11
rem Running cct
rem ---------------------------  4:51:13,25
rem Running kp
rem [2026-02-23T03:51:22Z INFO  kp] Read 10000000 coordinates and succesfully transformed 10000000 in 9.6539978s  (965ns each)
rem ---------------------------  4:51:22,99
rem
goto end


:utm

rem ----------------------------------------------------------
rem utm zone 32: relative timing between kp and cct
rem ----------------------------------------------------------

rem echo --------------------------- %time%
rem echo Compiling
rem cargo -q b --release
echo --------------------------- %time%
echo Running cct
cct -d 10 +proj=utm +zone=32 untracked\timing\nkg_test.pts > untracked\timing\nkg_test.noop_cct
echo --------------------------- %time%
echo Running kp
kp -vvd 10 "utm zone=32" untracked\timing\nkg_test.pts> untracked\timing\nkg_test.noop_kp
echo --------------------------- %time%

rem
rem RESULTS
rem
rem Running with kp first.
rem     utm zone=32   kp: 13.54 s, cct: 70 s, kp  5.17 times faster
rem     utm zone=32   kp: 12.37 s, cct: 96 s, kp  7.76 times faster
rem
rem $ geodesy\etc\timing utm
rem ---------------------------  5:34:27,33
rem Running kp
rem [2026-02-23T04:34:40Z INFO  kp] Read 10000000 coordinates and succesfully transformed 9938015 in 13.5335116s  (1.361µs each)
rem ---------------------------  5:34:40,93
rem Running cct
rem ---------------------------  5:35:50,84
rem $ geodesy\etc\timing utm
rem ---------------------------  5:39:34,09
rem Running kp
rem [2026-02-23T04:39:46Z INFO  kp] Read 10000000 coordinates and succesfully transformed 9938015 in 12.3728943s  (1.245µs each)
rem ---------------------------  5:39:46,84
rem Running cct
rem ---------------------------  5:41:22,41
rem
rem Running with cct first.
rem     utm zone=32   kp: 13.36 s, cct:  96 s, kp  7.18 times faster
rem     utm zone=32   kp: 13.57 s, cct: 101 s, kp  7.43 times faster
rem
rem $ geodesy\etc\timing utm
rem ---------------------------  5:45:39,07
rem Running cct
rem ---------------------------  5:47:15,87
rem Running kp
rem [2026-02-23T04:47:29Z INFO  kp] Read 10000000 coordinates and succesfully transformed 9938015 in 13.3654796s  (1.344µs each)
rem ---------------------------  5:47:29,67
rem $ eva 96/13.36
rem 7.1856287425
rem
rem $ geodesy\etc\timing utm
rem ---------------------------  5:48:16,31
rem Running cct
rem ---------------------------  5:49:57,34
rem Running kp
rem [2026-02-23T04:50:11Z INFO  kp] Read 10000000 coordinates and succesfully transformed 9938015 in 13.5755316s  (1.366µs each)
rem ---------------------------  5:50:11,12
rem $ eva 101/13.58
rem 7.4374079529
goto end


:subset

rem step-by-step consistency check between PROJ and RG implementations
rem of the NKG2020 transformation from itrf2014 to sweref99

echo off
rem echo Compiling and linking kp
rem cargo -q b --release

rem extract first 1000 points, and convert to cartesian
echo Extract
head -1000 untracked\timing\nkg_test.pts > untracked\timing\1000.pts
kp -d 5 "geo:in | cart" untracked\timing\1000.pts >untracked\timing\1000.xyz

rem Go from ITRF2014(t) to ETRF2014(t) using EUREF parameters
echo To ETRF2014
kp -d 5 "helmert uas angular_velocity = 85, 531, -770 rotation = 1785, 11151, -16170 t_epoch=2010 convention=position_vector" 1000.xyz >1000.kp
cct -d 5 proj=helmert rx=0.001785 ry=0.011151 rz=-0.01617 drx=8.5e-05 dry=0.000531 drz=-0.00077 t_epoch=2010 convention=position_vector -- untracked\timing\1000.xyz > untracked\timing\1000.cct
python geodesy\etc\takt_dist.py untracked\timing\1000.kp untracked\timing\1000.cct | tee untracked\timing\1000.dist

rem  Staying in ETRF2014, remove the frame deformation since 2000.0
echo Deformation
kp -d 5 "deformation inv t_epoch=2000 grids=nkgrf17vel ellps=GRS80" 1000.kp > untracked\timing\1000.kp_def
cct -d 5 inv proj=deformation t_epoch=2000 grids=eur_nkg_nkgrf17vel.tif ellps=GRS80 -- untracked\timing\1000.cct > untracked\timing\1000.cct_def
python geodesy\etc\takt_dist.py untracked\timing\1000.kp_def untracked\timing\1000.cct_def | tee untracked\timing\1000.dist_def

rem Now, with t fixed at 2000.0, go from ETRF2014(2000) to ETRF97(2000), which is the frame SWEREF99 is based on
echo To ETRF97
kp -d 5 "helmert translation = 30.54, 46.06, -79.44 mm rotation = 1419.58, 151.32, 1503.37 uas scale = 0.003002 convention=position_vector" untracked\timing\1000.kp_def > untracked\timing\1000.kp_ETRF97
cct -d 5 step proj=helmert x=0.03054 y=0.04606 z=-0.07944 rx=0.00141958 ry=0.00015132 rz=0.00150337 s=0.003002 convention=position_vector -- untracked\timing\1000.cct_def > untracked\timing\1000.cct_ETRF97
python geodesy\etc\takt_dist.py untracked\timing\1000.kp_ETRF97 untracked\timing\1000.cct_ETRF97 | tee untracked\timing\1000.dist_ETRF97

rem Finally correct for the frame deformation from the pivot epoch of 2000 to the realization epoch of 1999.5
echo Deformation
kp -d 5 "deformation dt=-0.5 grids=nkgrf17vel" untracked\timing\1000.kp_ETRF97 > untracked\timing\1000.kp_final
cct -d 5 proj=deformation dt=-0.5 grids=eur_nkg_nkgrf17vel.tif ellps=GRS80 -- untracked\timing\1000.cct_ETRF97 > untracked\timing\1000.cct_final
python geodesy\etc\takt_dist.py untracked\timing\1000.kp_final untracked\timing\1000.cct_final | tee untracked\timing\1000.dist_final



rem ----------------------------------------------------------
rem Error handling
rem ----------------------------------------------------------

:unknown_command_argument
echo Unknown argument: "%1". Valid arguments: prepare/nkg/noop/utm
goto end

:missing_nkg_test_pts
echo Missing the file "untracked\timing\nkg_test.pts"
echo Run "geodesy\etc\timing prepare" to generate
:goto end

:nkg_test_pts_exist
echo The file "untracked\timing\nkg_test.pts" already exists
echo Delete and re-run "geodesy\etc\timing prepare" to replace
goto end

:nothing_to_do
echo missing action argument (prepare/nkg/noop/utm)
:end
