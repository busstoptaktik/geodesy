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

impl Minimal {
    /// Returns a `Coor4D`, where the elements represent (dx/dλ, dy/dφ, dx/dφ, dy/dλ) (x_l, y_p, x_p, y_l)
    /// Mostly based on the PROJ function [pj_deriv](https://github.com/OSGeo/PROJ/blob/master/src/deriv.cpp),
    /// with appropriate adaptations to the fact that PROJ internally sets the semimajor axis, a = 1
    #[allow(dead_code)]
    #[rustfmt::skip]
    fn jacobian(&self, op: OpHandle, scale: [f64; 2], swap: [bool; 2], ellps: Ellipsoid, at: Coor2D) -> Result<Jacobian, Error> {
        const BAD_ID_MESSAGE: Error = Error::General("Minimal: Unknown operator id");
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;

        // If we have input in degrees, we must multiply the output by a factor of 180/pi
        // For user convenience, scale[0] is a "to degrees"-factor, i.e. scale[0]==1
        // indicates degrees, whereas scale[0]==180/pi indicates that input angles are
        // in radians.
        // To convert input coordinates to radians, divide by `angular_scale`
        let angular_scale = 1f64.to_degrees() / scale[0];

        // If we have output in feet, we must multiply the output by a factor of 0.3048
        // For user convenience, scale[1] is a "to metres"-factor, i.e. scale[1]==1
        // indicates metres, whereas scale[1]==0.3048 indicates that output lengths
        // are in feet, and scale[1]=201.168 indicates that output is in furlongs
        let linear_scale = 1.0 / scale[1];

        let h = 1e-5 / angular_scale;
        let d = (4.0 * h * ellps.semimajor_axis() / angular_scale * linear_scale).recip();

        let mut coo = [Coor2D::origin(); 4];

        let (e, n) = if swap[0] {(at[1], at[0])} else {(at[0], at[1])};

        // Latitude in degrees
        let latitude = n * scale[0];

        // North-east of POI
        coo[0] = Coor2D::raw(e + h, n + h);
        // South-east of POI
        coo[1] = Coor2D::raw(e + h, n - h);
        // South-west of POI
        coo[2] = Coor2D::raw(e - h, n - h);
        // North-west of POI
        coo[3] = Coor2D::raw(e - h, n + h);
        if swap[0] {
            coo[0] = Coor2D::raw(coo[0][1], coo[0][0]);
            coo[1] = Coor2D::raw(coo[1][1], coo[1][0]);
            coo[2] = Coor2D::raw(coo[2][1], coo[2][0]);
            coo[3] = Coor2D::raw(coo[3][1], coo[3][0]);
        }
        op.apply(self, &mut coo, Fwd);

        let (e, n) = if swap[1] {(1, 0)} else {(0, 1)};

        //        NE          SE         SW          NW
        let dx_dlam =  (coo[0][e] + coo[1][e] - coo[2][e] - coo[3][e]) * d;
        let dy_dlam =  (coo[0][n] + coo[1][n] - coo[2][n] - coo[3][n]) * d;
        let dx_dphi =  (coo[0][e] - coo[1][e] - coo[2][e] + coo[3][e]) * d;
        let dy_dphi =  (coo[0][n] - coo[1][n] - coo[2][n] + coo[3][n]) * d;

        Ok(Jacobian{latitude, dx_dlam, dy_dlam, dx_dphi, dy_dphi, ellps})
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
    fn jacobian() -> Result<(), Error> {
        let mut ctx = Minimal::new();

        let cph = Coor2D::geo(55., 12.);
        let op = ctx.op("utm zone=32")?;
        let steps = ctx.steps(op)?;
        assert!(steps.len()==1);
        let ellps = ctx.params(op, 0)?.ellps[0];
        let jac = ctx.jacobian(op, [1f64.to_degrees(),1.], [false, false], ellps, cph)?;
        //dbg!(&jac);
        dbg!(1f64.to_degrees());
        let factors = jac.factors();
        dbg!(factors);

        let cph = Coor2D::raw(12., 55.);
        let op = ctx.op("gis:in | utm zone=32")?;
        let jac = ctx.jacobian(op, [1.,1.], [false, false], ellps, cph)?;
        //dbg!(&jac);
        let factors = jac.factors();
        dbg!(factors);

        let cph = Coor2D::raw(55., 12.);
        let op = ctx.op("geo:in | utm zone=32")?;
        let jac = ctx.jacobian(op, [1.,1.], [true, false], ellps, cph)?;
        //dbg!(&jac);
        let factors = jac.factors();
        dbg!(factors);

        let op = ctx.op("geo:in | utm zone=32 |neu:out")?;
        let jac = ctx.jacobian(op, [1.,1.], [true, true], ellps, cph)?;
        //dbg!(&jac);
        let factors = jac.factors();
        dbg!(factors);

        let op = ctx.op("geo:in | utm zone=32 |neu:out | helmert scale=3.28083989501312300874")?;
        let jac = ctx.jacobian(op, [1.,0.3048], [true, true], ellps, cph)?;
        //dbg!(&jac);
        let factors = jac.factors();
        dbg!(factors);

        assert!(2==1);

        Ok(())
    }
}
