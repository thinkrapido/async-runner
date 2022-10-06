
use std::sync::{Arc, RwLock};
use anyhow::Result;
use anyhow::anyhow;
use tokio::task::JoinSet;
use tokio::signal::ctrl_c;
use std::time::Duration;
use futures::{
    select,
    FutureExt,
    pin_mut,
};

#[derive(Clone)]
pub struct AsyncRunner {
    state: Arc<AsyncRunnerState>,
}
impl AsyncRunner {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AsyncRunnerState::default()),
        }
    }
    pub async fn run(self) -> Result<()> {

        let premature_exit = async {
            ctrl_c().await.expect("failed to listen for event");
            tokio::time::sleep(Duration::from_secs(3)).await;
        }.fuse();

        let mut bla = self.state.set.write().unwrap();
        let set = bla.join_next().fuse();

        pin_mut!(set, premature_exit);

        select!{
            _ = set => (),
            _ = premature_exit => (),
        }

        Ok(())
    }

    pub fn spawn<T>(&self, a: fn(app: AsyncRunner) -> T)
        where
            T: std::future::Future<Output =()> + Send + 'static,
    {
        let _ = self.state.set.write().unwrap().spawn(a(self.clone()));
    }
    pub fn is_running(&self) -> bool {
        *self.state.is_running.read().unwrap()
    }
    pub fn stop(&self) {
        *self.state.is_running.write().unwrap() = false;
    }
}

struct AsyncRunnerState {
    set: RwLock<JoinSet<()>>,
    is_running: RwLock<bool>,
}
impl Default for AsyncRunnerState {
    fn default() -> Self {
        Self {
            set: RwLock::new(JoinSet::<()>::new()),
            is_running: RwLock::new(true),
        }
    }
}


#[tokio::main]
async fn main() -> Result<()> {

    let app = AsyncRunner::new();

    app.spawn(|app1: AsyncRunner| async move {
        let mut n = 1;
        while app1.is_running() {
            println!("{}", n);
            n += 1;
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    });    

    app.spawn(|app1: AsyncRunner| async move {
        tokio::time::sleep(Duration::from_millis(10000)).await;
        app1.stop();
    });    

    app.run().await?;

    Ok(())
}

