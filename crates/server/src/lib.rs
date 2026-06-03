use lru::LruCache;
use std::{
    hash::{Hash, Hasher},
    num::NonZeroUsize,
    sync::Arc,
};

use ::preprocess::SpliceText;
use tokio::sync::Mutex;

use crate::{hts_engine::HTSEngine, preprocess::PreprocessEngine};

pub mod async_engine;
pub mod hts_engine;
pub mod preprocess;
pub mod setting;

#[derive(PartialEq)]
struct CacheHash(String, String, f64);

impl Eq for CacheHash {}

impl Hash for CacheHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
        ((self.2 * 16384.0) as i64).hash(state);
    }
}

#[derive(Clone)]
pub struct State {
    pub preprocess: PreprocessEngine,
    hts: HTSEngine,
    pub splice: SpliceText,
    cache: Arc<Mutex<LruCache<CacheHash, Vec<i16>>>>,
    pub models: Vec<String>,
}

impl State {
    pub async fn new(hts_models: Vec<(String, String)>, cache_size: usize) -> anyhow::Result<Self> {
        let cores = num_cpus::get_physical();
        let preprocess = PreprocessEngine::new((), cores).await?;
        let models = hts_models
            .iter()
            .map(|(k, _v)| k.to_string())
            .collect::<Vec<_>>();
        let hts = HTSEngine::new(hts_models, cores).await?;
        let splice = SpliceText::new()?;
        let cache = Arc::new(Mutex::new(LruCache::new(
            NonZeroUsize::new(cache_size).ok_or(anyhow::anyhow!("invalid"))?,
        )));
        Ok(Self {
            preprocess,
            hts,
            splice,
            cache,
            models,
        })
    }
    pub async fn cached_run(
        &self,
        model: String,
        text: String,
        speed: f64,
    ) -> anyhow::Result<Option<Vec<i16>>> {
        let cache_hash = CacheHash(model.clone(), text.clone(), speed);
        if let Some(r) = self.cache.lock().await.get(&cache_hash) {
            return Ok(Some(r.to_vec()));
        }
        let Some(r) = self.hts.run((model, text, speed)).await? else {
            return Ok(None);
        };
        self.cache.lock().await.put(cache_hash, r.clone());
        Ok(Some(r))
    }
}
