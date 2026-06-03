use std::collections::HashMap;

use htsengine::HTSModel;
use jpreprocess::{
    DefaultTokenizer, JPreprocess, SystemDictionaryConfig, kind::JPreprocessDictionaryKind,
};

use crate::async_engine::{AsyncTaskEngine, TaskEngine};

pub struct HTSEngineLo {
    models: HashMap<String, htsengine::HTSModel>,
    jp: JPreprocess<DefaultTokenizer>,
}

impl TaskEngine for HTSEngineLo {
    type R = (String, String, f64);
    type S = Option<Vec<i16>>;
    type A = Vec<(String, String)>;
    fn new(a: Self::A) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let dic = SystemDictionaryConfig::Bundled(JPreprocessDictionaryKind::NaistJdic).load()?;
        let mut models = HashMap::new();
        for (k, v) in a {
            models.insert(k, HTSModel::new(&v)?);
        }
        Ok(Self {
            models,
            jp: JPreprocess::with_dictionaries(dic, None),
        })
    }

    fn run(&mut self, (name, text, speed): Self::R) -> anyhow::Result<Self::S> {
        let Some(model) = self.models.get_mut(&name) else {
            return Ok(None);
        };
        let label: Vec<_> = self
            .jp
            .extract_fullcontext(&text)?
            .into_iter()
            .map(|l| l.to_string())
            .collect();
        let v = model.synthesize(&label.join("\n"), speed)?;
        Ok(Some(v))
    }
}

pub type HTSEngine = AsyncTaskEngine<HTSEngineLo>;
