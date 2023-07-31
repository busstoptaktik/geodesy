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

    fn params(&self, op: OpHandle, index: usize) -> Result<&ParsedParameters, Error> {
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        // Leaf level?
        if op.steps.is_empty() {
            if index > 0 {
                return Err(Error::General("Minimal: Bad step index"));
            }
            return Ok(&op.params);
        }

        // Not leaf level
        if index >= op.steps.len() {
            return Err(Error::General("Minimal: Bad step index"));
        }
        Ok(&op.steps[index].params)
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
    fn jacobian(
        &self,
        op: OpHandle,
        operands: &dyn CoordinateSet,
    ) -> Result<(Ellipsoid, Vec<Coor4D>), Error> {
        const BAD_ID_MESSAGE: Error = Error::General("Minimal: Unknown operator id");
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;

        let ellps = op.params.ellps(0);

        let h = 1e-8;

        let mut j = Vec::with_capacity(operands.len());
        let mut coo = Vec::with_capacity(operands.len());

        // North-east of POI
        for i in 0..operands.len() {
            let coord = operands.get_coord(i);
            coo.push(Coor2D::raw(coord[0] + h, coord[1] + h));
        }
        op.apply(self, &mut coo, Fwd);

        for i in 0..operands.len() {
            let coord = coo[i];
            j.push(Coor4D::raw(coord[0], coord[1], coord[0], coord[1]));
        }

        // South-east of POI
        for i in 0..operands.len() {
            let coord = operands.get_coord(i);
            coo[i] = Coor2D::raw(coord[0] + h, coord[1] - h);
        }
        op.apply(self, &mut coo, Fwd);

        for i in 0..operands.len() {
            let x = coo[i][0];
            let y = coo[i][1];
            let ji = j[i];
            j[i] = Coor4D::raw(ji[0] + x, ji[1] - y, ji[2] - x, ji[3] + y);
        }


        // South-west of POI
        for i in 0..operands.len() {
            let coord = operands.get_coord(i);
            coo[i] = Coor2D::raw(coord[0] - h, coord[1] - h);
        }
        op.apply(self, &mut coo, Fwd);

        for i in 0..operands.len() {
            let x = coo[i][0];
            let y = coo[i][1];
            let ji = j[i];
            j[i] = Coor4D::raw(ji[0] - x, ji[1] - y, ji[2] - x, ji[3] - y);
        }

        // North-west of POI
        for i in 0..operands.len() {
            let coord = operands.get_coord(i);
            coo[i] = Coor2D::raw(coord[0] - h, coord[1] + h);
        }
        op.apply(self, &mut coo, Fwd);

        for i in 0..operands.len() {
            let x = coo[i][0];
            let y = coo[i][1];
            let ji = j[i];
            j[i] = Coor4D::raw(ji[0] - x, ji[1] + y, ji[2] + x, ji[3] - y);
        }

        // Normalize
        for i in 0..operands.len() {
            let d = 4.0 * h * ellps.semimajor_axis();
            j[i][0] /= d;
            j[i][1] /= d;
            j[i][2] /= d;
            j[i][3] /= d;
        }

        Ok((*ellps, j))
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

        let ellps = ctx.params(op, 1)?.ellps(0);
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
        assert_eq!("utm", ctx.params(op, 1)?.name);

        // All the 'common' elements (lat_?, lon_?, x_?, y_? etc.) defaults to 0,
        // while ellps_? defaults to GRS80 - so they are there even though we havent
        // set them
        let ellps = ctx.params(op, 1)?.ellps(0);
        assert_eq!(ellps.semimajor_axis(), 6378137.);
        assert_eq!(0., ctx.params(op, 1)?.lat(0));

        // The zone id is found among the natural numbers (which here includes 0)
        let zone = ctx.params(op, 1)?.natural("zone")?;
        assert_eq!(zone, 32);

        // Taking a look at the internals is not too hard either
        // let params = ctx.params(op, 0)?;
        // dbg!(params);

        Ok(())
    }

    #[test]
    fn jacobian() -> Result<(), Error> {
        let mut ctx = Minimal::new();

        let op = ctx.op("utm zone=32")?;

        let mut data = [
            Coor2D::geo(55., 12.),
            Coor2D::geo(59., 18.)
        ];

        // (dx/dλ, dy/dφ, dx/dφ, dy/dλ) (x_l, y_p, x_p, y_l)
        let (ellps, jac) = ctx.jacobian(op, &mut data)?;
        dbg!(&jac);

        let es = ellps.eccentricity_squared();

        let cosphi = data[0][1].cos();
        let x_l = jac[0][0];
        let y_p = jac[0][1];
        let x_p = jac[0][2];
        let y_l = jac[0][3];

        let h = x_p.hypot(y_p);
        dbg!(h);
        let k = x_l.hypot(y_l) / cosphi;
        dbg!(k);

        let t = data[0][1].sin();
        let t = 1. - es * t * t;
        let n = t.sqrt();
        let h = h * (t * n / (1. - es));
        let k = k * n;
        let r = t * t / (1. - es);
        dbg!(h);
        dbg!(k);


        let conv = -jac[0][2].atan2(jac[0][1]).to_degrees();
        dbg!(conv);
        assert_eq!(h, 3.45);

        Ok(())
    }
}
