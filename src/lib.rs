use std::{thread, sync::{mpsc, Mutex, Arc}};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

#[derive(Debug)]
pub enum PoolCreationError {
    InvalidNumberOfThreads
}

impl ThreadPool {
    /// Creates a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool.
    /// 
    /// # Panics
    /// 
    /// The `new` function will panic if the size is zero.
    /// 
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        ThreadPool::build(size).unwrap()
    }

    /// Creates a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool.
    /// 
    /// If the size is zero, it returns a PoolCreationError
    /// 
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size > 0 {
            let (sender, receiver) = mpsc::channel();

            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            Ok( ThreadPool { workers, sender } )
        } else {
            Err( PoolCreationError::InvalidNumberOfThreads )
        }
    }

    /// Executes a closure on a new thread from the pool
    /// 
    /// The f closure is what will be executed by the ThreadPool
    /// 
    pub fn execute<F>(&self, f: F) 
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().expect("Error encountered while executing a previous job; Stopping execution.").recv().unwrap();

            println!("Worker {id} got a job; executing.");

            job();
        });

        Worker { id, thread }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;