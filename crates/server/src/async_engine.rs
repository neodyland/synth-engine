use std::marker::PhantomData;

use async_channel::{Sender, bounded};
use tokio::sync::oneshot;

pub trait TaskEngine
where
    Self::A: Send + Sync + Clone + 'static,
    Self::R: Send + Sync + 'static,
    Self::S: Send + Sync + 'static,
{
    type R;
    type S;
    type A;
    fn new(a: Self::A) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn run(&mut self, r: Self::R) -> anyhow::Result<Self::S>;
}

pub struct AsyncTaskEngine<T>
where
    T: TaskEngine,
{
    #[allow(clippy::type_complexity)]
    tx: Sender<(T::R, oneshot::Sender<anyhow::Result<T::S>>)>,
    __phantom: PhantomData<T>,
}

impl<T> Clone for AsyncTaskEngine<T>
where
    T: TaskEngine,
{
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            __phantom: PhantomData,
        }
    }
}

unsafe impl<T> Send for AsyncTaskEngine<T> where T: TaskEngine {}
unsafe impl<T> Sync for AsyncTaskEngine<T> where T: TaskEngine {}

impl<T> AsyncTaskEngine<T>
where
    T: TaskEngine,
{
    pub async fn new(a: T::A, threads: usize) -> anyhow::Result<Self> {
        let (tx, rx) = bounded::<(T::R, oneshot::Sender<anyhow::Result<T::S>>)>(32);
        for i in 0..threads {
            tracing::info!("Spawning job thread: {i}");
            let (start_tx, start_rx) = oneshot::channel::<anyhow::Result<()>>();
            let a = a.clone();
            let rx = rx.clone();
            std::thread::spawn(move || {
                let mut t = match T::new(a) {
                    Ok(t) => {
                        start_tx.send(anyhow::Ok(())).ok();
                        t
                    }
                    Err(e) => {
                        start_tx
                            .send(Err(anyhow::anyhow!("thread: {i}, error: {e:?}")))
                            .ok();
                        return Err(anyhow::anyhow!("thread: {i}, error: {e:?}"));
                    }
                };
                while let Ok((r, sender)) = rx.recv_blocking() {
                    tracing::info!("thread {i} got job");
                    sender.send(t.run(r)).ok();
                }
                anyhow::Ok(())
            });
            start_rx.await??;
        }
        Ok(Self {
            tx,
            __phantom: PhantomData,
        })
    }
    pub async fn run(&self, r: T::R) -> anyhow::Result<T::S> {
        let (tx, rx) = oneshot::channel();
        self.tx.send((r, tx)).await?;
        rx.await?
    }
}
