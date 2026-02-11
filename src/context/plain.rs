#[cfg(feature = "with_plain")]
use crate::authoring::*;
use crate::grid::GridSource;
use memmap2::Mmap;
use std::{
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

// ----- T H E   P L A I N   C O N T E X T ---------------------------------------------

/// A context provider, supporting built in and run-time defined operators,
/// external grids, and macros.
/// Sufficient for most uses, especially geodetic grid development.
/// May get somewhat clunky when working with large numbers of grids or macros,
#[derive(Debug)]
pub struct Plain {
    constructors: BTreeMap<String, OpConstructor>,
    resources: BTreeMap<String, String>,
    operators: BTreeMap<OpHandle, Op>,
    paths: Vec<std::path::PathBuf>,
    unigrid_elements: Vec<BTreeMap<String, Arc<BaseGrid>>>,
    memmapped_unigrids: Vec<Option<Mmap>>,
}

// Helper for Plain: Provide grid access for all `Op`s
// in all instantiations of `Plain` by handing out
// reference counted clones to a single heap allocation
static GLOBALLY_ALLOCATED_GRIDS: OnceLock<Mutex<GridCollection>> = OnceLock::new();

fn init_grids() -> Mutex<GridCollection> {
    Mutex::new(GridCollection(BTreeMap::<String, Arc<BaseGrid>>::new()))
}

struct GridCollection(BTreeMap<String, Arc<BaseGrid>>);
impl GridCollection {
    fn get_grid_from_global_collection(
        &mut self,
        name: &str,
        paths: &[PathBuf],
    ) -> Result<Arc<BaseGrid>, Error> {
        // If the grid is already there, just return a reference clone
        if let Some(grid) = self.0.get(name) {
            return Ok(grid.clone());
        }

        // Otherwise, we must look for it in the data path
        let n = PathBuf::from(name);
        let ext = n
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        for path in paths {
            // First we look in the base directory
            let mut gridpath = path.clone();
            gridpath.push(name);
            let mut grid = std::fs::read(gridpath);

            // If not found there: Look in a subdirectory named after the file extension
            if grid.is_err() {
                gridpath = path.clone();
                gridpath.push(ext);
                gridpath.push(name);
                grid = std::fs::read(gridpath);
            }
            let Ok(grid) = grid else {
                continue;
            };

            let key = name.to_string();
            match ext {
                "gsb" => {
                    let value = crate::grid::ntv2::ntv2_grid(&grid)?;
                    self.0.insert(key, Arc::new(value));
                }

                "gtx" => {
                    let value = crate::grid::gtx::gtx(&key, &grid)?;
                    self.0.insert(key, Arc::new(value));
                }

                _ => {
                    // Neither GSA, nor Gravsoft can be identified by extension alone,
                    // so we try GSA first, since it can identify itself from its
                    // magic bytes 'DSAA'
                    if let Ok(grid) = crate::grid::gsa::gsa(&key, &grid) {
                        self.0.insert(name.to_string(), Arc::new(grid));
                    } else {
                        // Gravsoft
                        let value = crate::grid::gravsoft::gravsoft(&key, &grid)?;
                        self.0.insert(key, Arc::new(value));
                    }
                }
            }

            if let Some(grid) = self.0.get(name) {
                return Ok(grid.clone());
            }
        }

        Err(Error::NotFound(name.to_string(), ": Grid".to_string()))
    }
}

const BAD_ID_MESSAGE: Error = Error::General("Plain: Unknown operator id");

impl Plain {
    /// To avoid having the heap allocated collection of grids stored in
    /// `GLOBALLY_ALLOCATED_GRIDS` growing through the roof, we may clear
    /// it occasionally.
    /// As the grids are behind an `Arc` reference counter, this is safe to do
    /// even though they may still be in use by some remaining operator
    /// instantiations.
    pub fn clear_grids() {
        if let Some(grids) = GLOBALLY_ALLOCATED_GRIDS.get() {
            grids.lock().unwrap().0.clear();
        }
    }
}

impl Default for Plain {
    fn default() -> Plain {
        let constructors = BTreeMap::new();
        let resources = BTreeMap::new();
        let operators = BTreeMap::new();
        let mut paths = Vec::new();

        let localpath: PathBuf = [".", "geodesy"].iter().collect();
        paths.push(localpath);

        if let Some(mut userpath) = dirs::data_local_dir() {
            userpath.push("geodesy");
            paths.push(userpath);
        }
        let unigrid_elements = Vec::new();
        let memmapped_unigrids = Vec::new();

        Plain {
            constructors,
            resources,
            operators,
            paths,
            unigrid_elements,
            memmapped_unigrids,
        }
    }
}

impl Context for Plain {
    fn new() -> Plain {
        let mut ctx = Plain::default();
        for item in BUILTIN_ADAPTORS {
            ctx.register_resource(item.0, item.1);
        }
        let Ok(unigrids) = crate::grid::unigrid::read_unigrid_index(&ctx.paths) else {
            return ctx;
        };
        ctx.unigrid_elements = unigrids;

        let mut memmapped_unigrids = Vec::new();
        for path in ctx.paths.iter() {
            let unifile_path = path.join("unigrid.grids");
            let Ok(unifile) = File::options().read(true).open(unifile_path) else {
                memmapped_unigrids.push(None);
                continue;
            };
            memmapped_unigrids.push(unsafe { memmap2::Mmap::map(&unifile).ok() });
        }
        ctx.memmapped_unigrids = memmapped_unigrids;
        ctx
    }

    /// Instantiate an operator. Recognizes PROJ syntax and converts it to Geodesy syntax.
    /// Bear in mind, however, that Geodesy does not support all PROJ operators, and that
    /// the input/output conventions differ. This functionality may be a better fit for
    /// somewhere between [`token::split_into_steps()`](crate::token::Tokenize::split_into_steps())
    /// and [`token::normalize()`](crate::token::Tokenize::normalize())
    fn op(&mut self, definition: &str) -> Result<OpHandle, Error> {
        // It may be a PROJ string, so we filter it through the PROJ parser
        let definition = parse_proj(definition)?;

        let op = Op::new(&definition, self)?;
        let id = OpHandle::new();
        self.operators.insert(id, op);
        assert!(self.operators.contains_key(&id));
        Ok(id)
    }

    fn apply(
        &self,
        op: OpHandle,
        direction: Direction,
        operands: &mut dyn CoordinateSet,
    ) -> Result<usize, Error> {
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        Ok(op.apply(self, operands, direction))
    }

    fn steps(&self, op: OpHandle) -> Result<Vec<String>, Error> {
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        Ok(op.descriptor.instantiated_as.split_into_steps())
    }

    fn params(&self, op: OpHandle, index: usize) -> Result<ParsedParameters, Error> {
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        if op.is_pipeline() {
            let steps = op.steps.as_ref().unwrap();
            if index >= steps.len() {
                return Err(Error::General("Plain: Bad step index"));
            }
            Ok(steps[index].params.clone())
        } else {
            // Not a pipeline
            if index > 0 {
                return Err(Error::General("Plain: Bad step index"));
            }
            Ok(op.params.clone())
        }
    }

    fn globals(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("ellps".to_string(), "GRS80".to_string())])
    }

    fn register_op(&mut self, name: &str, constructor: OpConstructor) {
        self.constructors.insert(String::from(name), constructor);
    }

    fn get_op(&self, name: &str) -> Result<OpConstructor, Error> {
        if let Some(result) = self.constructors.get(name) {
            return Ok(OpConstructor(result.0));
        }

        Err(Error::NotFound(
            name.to_string(),
            ": User defined constructor".to_string(),
        ))
    }

    fn register_resource(&mut self, name: &str, definition: &str) {
        self.resources
            .insert(String::from(name), String::from(definition));
    }

    fn get_resource(&self, name: &str) -> Result<String, Error> {
        // There may be an unidentified use case for user registered
        // resources lacking the ':'-sigil. So we postpone the check
        // for sigil until we know it is not a run-time user defined
        // resource we're looking for
        if let Some(result) = self.resources.get(name) {
            return Ok(result.to_string());
        }

        // TODO: Check for "known prefixes": 'ellps:', 'datum:', etc.
        let parts = name.split(':').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(Error::BadParam(
                "needing prefix:suffix format".to_string(),
                name.to_string(),
            ));
        }
        let prefix = parts[0];
        let suffix = parts[1];
        let section = "resources";

        // We do not know yet whether the resource is in a separate resource
        // file or in a resource register, so we generate file names for
        // both cases.
        let resource = prefix.to_string() + "_" + suffix + ".resource";
        let register = prefix.to_string() + ".md";
        let tag = "```geodesy:".to_string() + suffix + "\n";

        for path in &self.paths {
            // Is it in a separate file?
            let mut full_path = path.clone();
            full_path.push(section);
            full_path.push(&resource);
            if let Ok(result) = std::fs::read_to_string(full_path) {
                return Ok(result.trim().to_string());
            }

            // If not, search in a resource register
            let mut full_path = path.clone();
            full_path.push(section);
            full_path.push(&register);
            if let Ok(mut result) = std::fs::read_to_string(full_path) {
                result = result.replace('\r', "\n");
                let Some(mut start) = result.find(&tag) else {
                    continue;
                };
                start += tag.len();
                let Some(length) = result[start..].find("```") else {
                    // Search for end-of-item reached end-of-file
                    let result = result[start..].trim().to_string();
                    return Ok(result);
                };
                let result = result[start..start + length].trim().to_string();
                return Ok(result);
            }
        }

        Err(Error::NotFound(
            name.to_string(),
            ": User defined resource".to_string(),
        ))
    }

    fn get_blob(&self, name: &str) -> Result<Vec<u8>, Error> {
        let n = PathBuf::from(name);
        let ext = n
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        for path in &self.paths {
            let mut path = path.clone();
            path.push(ext);
            path.push(name);
            if let Ok(result) = std::fs::read(path) {
                return Ok(result);
            }
        }
        Err(Error::NotFound(name.to_string(), ": Blob".to_string()))
    }

    /// Access grid resources by identifier
    fn get_grid(&self, name: &str) -> Result<Arc<BaseGrid>, Error> {
        // First search among the run time loaded grids
        if let Ok(grid) = GLOBALLY_ALLOCATED_GRIDS
            .get_or_init(init_grids)
            .lock()
            .unwrap()
            .get_grid_from_global_collection(name, &self.paths)
        {
            return Ok(grid);
        }

        // Then among the unigrids
        for unigrid in self.unigrid_elements.iter() {
            if let Some(grid) = unigrid.get(name) {
                return Ok(grid.clone());
            }
        }

        // Not found
        Err(Error::NotFound(name.to_string(), ": Grid".to_string()))
    }

    fn get_grid_values(
        &self,
        grid: &BaseGrid,
        indices: &[usize],
        grid_values: &mut [Coor4D],
    ) -> usize {
        match &grid.grid {
            GridSource::External { level, offset } => {
                for (i, index) in indices.iter().enumerate() {
                    let mut val = Coor4D::nan();
                    if let Some(file) = &self.memmapped_unigrids[*level] {
                        for j in 0..grid.header.bands.min(4) {
                            let start = (index + j) * 4 + offset;
                            if start > file.len() - 4 {
                                // TODO: log message
                                return 0;
                            }
                            let range = start..start + 4;
                            val[j] = f32::from_le_bytes(file[range].try_into().unwrap()).into();
                        }
                    }
                    grid_values[i] = val;
                }
                indices.len()
            }
            GridSource::Internal { values } => {
                for (i, index) in indices.iter().enumerate() {
                    for j in 0..grid.header.bands.min(4) {
                        grid_values[i][j] = values[index + j].into()
                    }
                }
                indices.len()
            }
        }
    }
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn basic() -> Result<(), Error> {
        let mut ctx = Plain::new();

        // Test the check for syntactic correctness (i.e. prefix:suffix-form)
        assert!(matches!(
            ctx.get_resource("foo"),
            Err(Error::BadParam(_, _))
        ));
        // Do we get the proper error code for non-existing resources?
        assert!(matches!(
            ctx.get_resource("foo:bar"),
            Err(Error::NotFound(_, _))
        ));
        // ...and the proper error code for non-existing grids?
        assert!(matches!(ctx.get_grid("foo"), Err(Error::NotFound(_, _))));

        // Try to instantiate the "stupid way of adding 1" macro
        // from geodesy/resources/stupid_way.resource
        let op = ctx.op("stupid:way")?;

        // ...and it works as expected?
        let mut data = crate::test_data::coor2d();
        assert_eq!(data[0].x(), 55.);
        assert_eq!(data[1].x(), 59.);

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].x(), 56.);
        assert_eq!(data[1].x(), 60.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0].x(), 55.);
        assert_eq!(data[1].x(), 59.);

        // Now test that the look-up functionality works in general

        // Do we get the end address right in registers?
        assert!(ctx.get_resource("stupid:way_too")?.ends_with("addone"));
        // ...also at the end of the file?
        assert!(ctx.get_resource("stupid:way_too")?.ends_with("addone"));
        // And do we also get the start address right?
        assert!(ctx.get_resource("stupid:way_three")?.starts_with("addone"));

        // And just to be sure: once again for the plain resource file
        assert!(ctx.get_resource("stupid:way")?.starts_with('#'));
        assert!(ctx.get_resource("stupid:way")?.ends_with("addone"));

        // Now make sure, we can actually also *instantiate* a recipe
        // from a register
        let op = ctx.op("stupid:way_too")?;

        // ...and it works as expected?
        let mut data = crate::test_data::coor2d();

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].x(), 57.);
        assert_eq!(data[1].x(), 61.);

        // 3 Console tests from stupid.md
        let op = ctx.op("stupid:bad");
        assert!(matches!(op, Err(Error::Syntax(_))));

        let op = ctx.op("stupid:addthree")?;
        let mut data = crate::test_data::coor2d();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].x(), 58.);
        assert_eq!(data[1].x(), 62.);

        let op = ctx.op("stupid:addthree_one_by_one")?;
        let mut data = crate::test_data::coor2d();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].x(), 58.);
        assert_eq!(data[1].x(), 62.);

        // Make sure we can access "sigil-less runtime defined resources"
        ctx.register_resource("foo", "bar");
        assert!(ctx.get_resource("foo")? == "bar");

        // We are *not* supposed to be able to instantiate a sigil-less resource
        ctx.register_resource("baz", "utm zone=32");
        assert!(ctx.op("baz").is_err());

        // But this classic should work...
        let op = ctx.op("geo:in | utm zone=32")?;
        let mut data = crate::test_data::coor2d();
        ctx.apply(op, Fwd, &mut data)?;
        let expected = [691875.6321396609, 6098907.825005002];
        assert_float_eq!(data[0].0, expected, abs_all <= 1e-9);

        Ok(())
    }

    #[test]
    fn grids() -> Result<(), Error> {
        let mut ctx = Plain::new();

        // Here, we only invoke reference counting in the GridCollection. The tests in
        // gridshift and deformation makes sure that the correct grids are actually
        // provided by GridCollection::get_grid()
        let _op1 = ctx.op("gridshift grids=5458.gsb, 5458_with_subgrid.gsb")?;
        let _op2 = ctx.op("gridshift grids=5458.gsb, 5458_with_subgrid.gsb")?;
        let _op3 = ctx.op("gridshift grids=test.geoid")?;
        assert!(ctx.op("gridshift grids=non.existing").is_err());
        Ok(())
    }

    #[test]
    fn unigrid() -> Result<(), Error> {
        let ctx = Plain::new();
        let ellps = Ellipsoid::named("GRS80")?;

        let unigrid_test = ctx.get_grid("unigrid_test_datum")?;

        // A test point outside of the subgrid, and the correction grid value at that point
        let test_point = Coor4D::geo(55.1f64, 12.3f64, 0., 0.);
        let correction = unigrid_test.at(Some(&ctx), test_point, 0.).unwrap();
        let Some(subgrid) = unigrid_test.which_subgrid_contains(test_point, 0.0) else {
            return Err(Error::General("No (sub-)grid found for (55.1E, 12.3E)"));
        };
        assert_eq!("unigrid_test_datum[0]", subgrid);

        // Numerically the grid value IN ARCSEC should be identical to the grid location
        // IN DEGREES. Hence, to make the test_point (which is a coordinate in RADIANS)
        // comparable to res, which is given in arcsec, we must treat res as DEGREES, and
        // convert to radians
        let d = ellps.distance(&test_point, &correction.to_radians());
        assert!(d < 1e-9);

        // A test point within the subgrid, and the correction grid value at that point
        let test_point = Coor4D::geo(56.3, 12.1, 0., 0.);
        let correction = unigrid_test.at(Some(&ctx), test_point, 0.).unwrap();
        // The correction values are offset by 0.001 in the sub-grid
        let expected = Coor4D::geo(56.301, 12.101, 0., 0.);
        let d = ellps.distance(&expected, &correction.to_radians());
        dbg!((d * 1000.0, correction));
        assert!(d < 0.1);
        // The interpolated latitude above amounts to 56.30099945068359, leading to an
        // apparently enormous discrepancy of 66 mm.
        //
        // However, 56.301-56.30099945068359 = 0.000_000_5493, i.e. a deviation
        // at the 7th significant figure, which is as expected, since the backing grid consists
        // of single precision floats (f32), having a typical accuracy of 7 figures.
        //
        // In real applications, the correction is on the order of a few seconds of arc. The
        // deviation above is on the order of a microsecond of arc (uas).
        //
        // On the surface of the earth, 1 uas corresponds to approximately 30 micrometers,
        // i.e. 0.03 mm, which is much smaller than the expected accuracy of any current
        // or future datum shift estimate.

        // (56.3N, 12.1E) is inside the subgrid
        let Some(subgrid) = unigrid_test.which_subgrid_contains(test_point, 0.0) else {
            return Err(Error::General("No (sub-)grid found for (56.3E, 12.1E)"));
        };
        assert_eq!("unigrid_test_datum[1]", subgrid);

        Ok(())
    }
}
