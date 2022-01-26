
#![feature(async_closure)]

use {
    std::{
        io::Error,
        future::Future,
        collections::VecDeque,
        time::Duration,
    },
    tokio::{
        task::JoinHandle,
        spawn,
        time::sleep,
    },
};

struct AsyncHandle {
    pub handle: JoinHandle<()>,
}
impl AsyncHandle {
    pub fn new(handle: JoinHandle<()>) -> Self {
        AsyncHandle { handle }
    }
}

struct AsyncRunner {
    ring: VecDeque<AsyncHandle>,
    running: bool,
}

impl AsyncRunner {

    fn new() -> Self {
        AsyncRunner {
            ring: VecDeque::new(),
            running: false,
        }
    }

    pub fn is_running(&self) -> bool { self.running }

    pub fn add<F, Fut>(&mut self, afun: F) 
        where F: Fn(&mut AsyncRunner) -> Fut, Fut: Future<Output = ()> + Send + 'static
    {
        self.running = true;
        self.ring.push_back(AsyncHandle::new(spawn(afun(self))))
    }

    pub async fn run(&mut self) {
        while let Some(ahandle) = self.ring.pop_front() {
            let _ = ahandle.handle.await;
        }
    }

}


#[tokio::main]
async fn main() -> Result<(), Error> {

    let mut r = AsyncRunner::new();

    r.add(a1);
    //r.add(a2);

    r.run().await;

    Ok(())
}

async fn a1(runner: &mut AsyncRunner) {
    let mut i = 10;
    while i > 0 {
        sleep(Duration::from_millis(250)).await;
        println!("a1: {}", i);
        i -= 1;
    }
}
async fn a2(runner: &mut AsyncRunner) {
    let mut i = 5;
    while i > 0 {
        sleep(Duration::from_millis(100)).await;
        println!("a2: {}", i);
        i -= 1;
    }
}

