use crate::context_authoring::*;
use std::path::PathBuf;

// ----- T H E   M I N I M A L   P R O V I D E R ---------------------------------------

/// A minimalistic context provider, supporting only built in and run-time defined operators.
/// Usually sufficient for cartographic uses, and for internal test authoring.
#[derive(Debug, Default)]
pub struct Minimal {
    /// Constructors for user defined operators
    constructors: BTreeMap<String, OpConstructor>,
    /// User defined resources (macros)
    resources: BTreeMap<String, String>,
    /// Instantiations of operators
    operators: BTreeMap<OpHandle, Op>,
}

const BAD_ID_MESSAGE: Error = Error::General("Minimal: Unknown operator id");

impl Context for Minimal {
    fn new() -> Minimal {
        let mut ctx = Minimal::default();
        for item in BUILTIN_ADAPTORS {
            ctx.register_resource(item.0, item.1);
        }
        ctx
    }

    fn op(&mut self, definition: &str) -> Result<OpHandle, Error> {
        let op = Op::new(definition, self)?;
        let id = op.id;
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
        const BAD_ID_MESSAGE: Error = Error::General("Minimal: Unknown operator id");
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        Ok(op.apply(self, operands, direction))
    }

    fn globals(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("ellps".to_string(), "GRS80".to_string())])
    }

    fn steps(&self, op: OpHandle) -> Result<&Vec<String>, Error> {
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        Ok(&op.descriptor.steps)
    }

    fn params(&self, op: OpHandle, index: usize) -> Result<ParsedParameters, Error> {
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        // Leaf level?
        if op.steps.is_empty() {
            if index > 0 {
                return Err(Error::General("Minimal: Bad step index"));
            }
            return Ok(op.params.clone());
        }

        // Not leaf level
        if index >= op.steps.len() {
            return Err(Error::General("Minimal: Bad step index"));
        }
        Ok(op.steps[index].params.clone())
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
        if let Some(result) = self.resources.get(name) {
            return Ok(result.to_string());
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
        let path: PathBuf = [".", "geodesy", ext, name].iter().collect();
        Ok(std::fs::read(path)?)
    }

    /// Access grid resources by identifier
    fn get_grid(&self, _name: &str) -> Result<Grid, Error> {
        Err(Error::General(
            "Grid access by identifier not supported by the Minimal context provider",
        ))
    }
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn basic() -> Result<(), Error> {
        let mut ctx = Minimal::new();

        // The "stupid way of adding 1" macro from geodesy/macro/stupid_way.macro
        ctx.register_resource("stupid:way", "addone | addone | addone inv");
        let op = ctx.op("stupid:way")?;

        let mut data = some_basic_coor2dinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        let steps = ctx.steps(op)?;
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0], "addone");
        assert_eq!(steps[1], "addone");
        assert_eq!(steps[2], "addone inv");

        let params = ctx.params(op, 1)?;
        let ellps = params.ellps(0);
        assert_eq!(ellps.semimajor_axis(), 6378137.);

        Ok(())
    }

    #[test]
    fn introspection() -> Result<(), Error> {
        let mut ctx = Minimal::new();

        let op = ctx.op("geo:in | utm zone=32 | neu:out")?;

        let mut data = some_basic_coor2dinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        ctx.apply(op, Fwd, &mut data)?;
        let expected = [6098907.825005002, 691875.6321396609];
        assert_float_eq!(data[0].0, expected, abs_all <= 1e-10);

        // The text definitions of each step
        let steps = ctx.steps(op)?;
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0], "geo:in");
        assert_eq!(steps[1], "utm zone=32");
        assert_eq!(steps[2], "neu:out");

        // Behind the curtains, the two i/o-macros are just calls to the 'adapt' operator
        assert_eq!("adapt", ctx.params(op, 0)?.name);
        assert_eq!("adapt", ctx.params(op, 2)?.name);

        // While the utm step really is the 'utm' operator, not 'tmerc'-with-extras
        // (although, obviously it is, if we dig a level deeper down through the
        // abstractions, into the concretions)
        assert_eq!("utm", ctx.params(op, 1)?.name);

        // All the 'common' elements (lat_?, lon_?, x_?, y_? etc.) defaults to 0,
        // while ellps_? defaults to GRS80 - so they are there even though we havent
        // set them
        let params = ctx.params(op, 1)?;
        let ellps = params.ellps[0];
        assert_eq!(ellps.semimajor_axis(), 6378137.);
        assert_eq!(0., params.lat[0]);

        // The zone id is found among the natural numbers (which here includes 0)
        let zone = params.natural("zone")?;
        assert_eq!(zone, 32);

        // Taking a look at the internals is not too hard either
        // let params = ctx.params(op, 0)?;
        // dbg!(params);

        Ok(())
    }

    #[test]
    fn jacobian_test() -> Result<(), Error> {
        let mut ctx = Minimal::new();
        let expected = [
            0.5743697198254788,
            0.02465770672327687,
            -0.04289429341613819,
            0.9991676664740311,
        ];

        // First a plain case of scaling input in radians, but no swapping
        let cph = Coor2D::geo(55., 12.);
        let op = ctx.op("utm zone=32")?;
        let steps = ctx.steps(op)?;
        assert!(steps.len() == 1);
        let ellps = ctx.params(op, 0)?.ellps[0];
        let jac = Jacobian::new(
            &ctx,
            op,
            [1f64.to_degrees(), 1.],
            [false, false],
            ellps,
            cph,
        )?;
        assert_float_eq!(
            [jac.dx_dlam, jac.dy_dlam, jac.dx_dphi, jac.dy_dphi],
            expected,
            abs_all <= 1e-12
        );

        // Then input in degrees (i.e. no scaling), and no swapping
        let cph = Coor2D::raw(12., 55.);
        let op = ctx.op("gis:in | utm zone=32")?;
        let jac = Jacobian::new(&ctx, op, [1., 1.], [false, false], ellps, cph)?;
        assert_float_eq!(
            [jac.dx_dlam, jac.dy_dlam, jac.dx_dphi, jac.dy_dphi],
            expected,
            abs_all <= 1e-12
        );

        // Then input in degrees (i.e. no scaling), and swapping on input
        let cph = Coor2D::raw(55., 12.);
        let op = ctx.op("geo:in | utm zone=32")?;
        let jac = Jacobian::new(&ctx, op, [1., 1.], [true, false], ellps, cph)?;
        assert_float_eq!(
            [jac.dx_dlam, jac.dy_dlam, jac.dx_dphi, jac.dy_dphi],
            expected,
            abs_all <= 1e-12
        );

        // Then input in degrees (i.e. no scaling), and swapping on both input and output
        let op = ctx.op("geo:in | utm zone=32 |neu:out")?;
        let jac = Jacobian::new(&ctx, op, [1., 1.], [true, true], ellps, cph)?;
        assert_float_eq!(
            [jac.dx_dlam, jac.dy_dlam, jac.dx_dphi, jac.dy_dphi],
            expected,
            abs_all <= 1e-12
        );

        // Then input in degrees (i.e. no scaling), scaling on output, and swapping on both input and output
        // (yes - the 'helmert s=3e6' scales by a factor of 4: 1 + 3 million ppm = 4)
        let op = ctx.op("geo:in | utm zone=32 |neu:out | helmert s=3e6")?;
        let jac = Jacobian::new(&ctx, op, [1., 0.25], [true, true], ellps, cph)?;
        assert_float_eq!(
            [jac.dx_dlam, jac.dy_dlam, jac.dx_dphi, jac.dy_dphi],
            expected,
            abs_all <= 1e-12
        );

        Ok(())
    }
}
