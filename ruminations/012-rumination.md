# Ruminations on Rust Geodesy

## Rumination 012: Unigrids and the `UG` grid maintenance utility

Thomas Knudsen <knudsen.thomas@gmail.com>

2026-03-02. Last [revision](#document-history) 2026-03-02

### Abstract

```rust
let using_unigrid = ctx.op("gridshift grids=ed50")?;
let using_stand_alone_grid = ctx.op("gridshift grids=ed50.ntv2")?;
```

---

### Unigrids - don't implement poorly, what the OS already provides excellently!

Version 0.15 of Rust Geodesy introduces the concept of *unigrids*. Unigrids are
conglomerates of geodetic grids (1-D geoid models, 2-D datum transformation
grids and 3-D deformation grids), concatenated into a single file, supported by
a sidecar index, providing information about the offset, length, and
geometrical characteristics of each grid.

The intention with unigrids is to avoid stepping on the toes of the operating
system by uneducated second-guessing of details, which are better left for the
OS to handle. And I suspect that this kind of toe-stepping is exactly what we
do, when introducing internally layered caching and complex access methods for
gridded data.

By combining all grids into a single file, and by accessing its contents through
a read-only memory mapping of that file, we leave it up to the OS to figure out,
what should, at any given time, reside in core memory, and what should stay on
disk. The hope is that this may give the OS a better chance to optimize the
overall perfomrance of the system, by not obscuring its real-time view of our
true, immediate memory needs.

By using a read only mapping, the OS is free to re-use the actual physical
memory for more than one process, needing to access the relevant data: The
memory may be mapped into different logical adresses in the memory map of each
process, but the physical memory backing it need not be duplicated. The OS is
excellently equipped to do this efficiently at the kernel level, whereas it is a
non-trivial task to approximate in user space

Hence, the unigrid concept is an attempt to follow the tenet of *don't implement
poorly, what the OS already provides excellently!*

#### Contemporary evidence

Preliminary evidence, from a modest amount of testing, using MS Windows,
indicate that the unigrid concept really does provide a respectable access
speed. The reader is encouraged to try it out, and report any successes and/or
failures.

#### Historical/anecdotal evidence

The idea behind unigrids is based on experience from around 2010, when I was
part of a team struggling to maintain and modernize
[trlib](https://github.com/busstoptaktik/trlib), the (already then) aging
foundational library behind the
[KMSTrans](https://github.com/busstoptaktik/kmstrans) transformation program.

During this work, we did two very different implementations of the grid based
datum transformation method:

- one preloading the grid into a flat array, and accessing the grid elements directly
- one doing a `fopen(...)` of the grid, and doing every single grid element
  access by a combined `fseek(...)`/`fread(...)`-dance.

To our surprise, *the difference in performance was immaterial!*

Some years later, however, an OS internals connoisseur explained to me, that
this was plausible since a file opened as read-only, would secretly, behind my
back, be memory mapped by Windows. And clever handshaking between the OS and the
`stdio` layer virtually eliminates the number of context switches needed, even
under heavily random access patterns.

This explanation, whether credible or not, is the direct inspiration for the
implementation of unigrids.

#### Search paths

Unigrids are searched in the same order as stand alone grids, i.e. if a
directory named `geodesy` exists in the current working directory, and a unigrid
file pair (`unigrid.grids` and `unigrid.index`) is present, that will be
searched first. Then, in turn, the directories returned by the `data_local_dir` and
`data_dir` functions of the Rust [dirs](https://docs.rs/dirs/latest/dirs/) crate.

### `UG` - the unigrid maintenance tool

To use a unigrid, it must first be constructed. And to keep using it with
confidence, it must be maintained. To this (these) end(s), employ the (still
rudimentary) `UG` unigrid tool.

UG provides a rudimentary set of sub-commands, of which `ug add` is probably the
most important, adding new grids to an existing unigrid, creating it if it does
not already exist.

`ug add` functions in strict append mode: It appends the new files to the end of
the existing unigrid. If another file with the same name already exists in the
unigrid, `ug` refuses to append, unless the `--force` flag is provided. But due
to the *append only* mode, the existing file is not overwritten, it is simply
shadowed by the new, and hence ignored in future use.

Evidently, it would be practical now and then to remove the shadowed grids, a
process known as *vacuuming*. This functionality is, alas, not yet implemented.

The entire `UG` functionality is implemented as just 5 subcommands, of which one,
*vacuum* is just a stub.

```console
$ ug help

Handling Rust Geodesy Unigrids

Usage: ug.exe <COMMAND>

Commands:
  add     Add grids to geodesy/unigrid.grids
  list    List contents of unigrid in ./geodesy
  paths   Show unigrid search paths
  vacuum  Remove shadowed gridfiles (unimplemented)
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

$ ug add -h

Add grids to geodesy/unigrid.grids

Usage: ug.exe add [OPTIONS] <PATH>...

Arguments:
  <PATH>...  Grids to add

Options:
      --force  Let new grids shadow older ones with the same name
  -h, --help   Print help

```

The Windows command file `make_unigrid.cmd` in `geodesy/geodesy/etc` gives an
example of actual use:

```cmd
@echo off
rem generate demo unigrid file-set
rem execute from the geodesy root directory as geodesy\etc\make_unigrid
rem AFTER having removed and/or backed up unigrid.grids and unigrid.index
rem writes a new set in geodesy\geodesy

cargo r --bin ug add geodesy/deformation/nkgrf17vel.deformation geodesy/datum/test_datum_with_subset_as_subgrid.datum
cargo r --bin ug add geodesy/gsa/egm96_15_subset.gsa geodesy/geoid/test.geoid geodesy/gsb/100800401.gsb
cargo r --bin ug list --verbose

```

### Docuument History

Major revisions and additions:

- 2026-03-02: Initial version
