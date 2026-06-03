mod ffi;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid as cstring(null err)")]
    InvalidAsCString,
    #[error("load failed")]
    LoadFailed,
    #[error("synthesize failed")]
    SynthesizeFailed,
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
pub struct HTSModel {
    engine: ffi::HTSEngine,
}

impl HTSModel {
    pub fn new(f: &str) -> Result<Self> {
        let mut engine = ffi::HTSEngine::init();
        engine.load(&[f])?;
        Ok(Self { engine })
    }
    pub fn synthesize(&mut self, label: &str, speed: f64) -> Result<Vec<i16>> {
        self.engine.set_speed(speed);
        self.engine.synthesize(label)?;
        let mut v = vec![];
        self.engine.write(|sample| {
            v.push(sample);
            Ok::<_, Error>(())
        })?;
        Ok(v)
    }
}
