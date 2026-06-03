use crate::async_engine::{AsyncTaskEngine, TaskEngine};
use preprocess::Preprocess;

pub struct PreprocessEngineLo {
    inner: Preprocess,
}

impl TaskEngine for PreprocessEngineLo {
    type R = String;

    type S = String;

    type A = ();

    fn new(_a: Self::A) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            inner: Preprocess::new()?,
        })
    }

    fn run(&mut self, r: Self::R) -> anyhow::Result<Self::S> {
        Ok(self.inner.run(r))
    }
}

pub type PreprocessEngine = AsyncTaskEngine<PreprocessEngineLo>;
