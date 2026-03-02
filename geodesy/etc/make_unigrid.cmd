@echo off
rem generate demo unigrid file-set
rem execute from the geodesy root directory as geodesy\etc\make_unigrid
rem AFTER having removed and/or backed up unigrid.grids and unigrid.index
rem writes a new set in geodesy\geodesy

cargo r --bin ug add geodesy/deformation/nkgrf17vel.deformation geodesy/datum/test_datum_with_subset_as_subgrid.datum
cargo r --bin ug add geodesy/gsa/egm96_15_subset.gsa geodesy/geoid/test.geoid geodesy/gsb/100800401.gsb
cargo r --bin ug add kalle
cargo r --bin ug list --verbose
