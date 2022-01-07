use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, &'static str> {
        if size < 1 {
            return Err("Thread can not be less then 1.");
        }

        let (sender, receiver) = mpsc::channel();
        let mut threads = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            let worker = Worker::new(id, Arc::clone(&receiver))?;
            threads.push(worker);
        }
        threads
            .iter()
            .for_each(|worker| println!("Worker {} get ready!", worker.id));

        Ok(ThreadPool { threads, sender })
    }

    pub fn execute<F>(&self, job: F) -> Result<(), &'static str>
    where
        F: FnOnce() + Send + 'static,
    {
        let result = self.sender.send(Message::NewJob(Box::new(job)));

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err("Send job to worker failed."),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        self.threads
            .iter()
            .for_each(|_| self.sender.send(Message::Terminate).unwrap());

        self.threads.iter_mut().for_each(|worker| {
            println!("Shutting down woker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        });
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Result<Worker, &'static str> {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a new job, executing", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} start terminate", id);
                    break;
                }
            }
        });

        Ok(Worker {
            id,
            thread: Some(thread),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_worker() {
        let (_, receiver) = mpsc::channel();

        let worker = Worker::new(1, Arc::new(Mutex::new(receiver))).unwrap();

        assert_eq!(1, worker.id);
        worker.thread.unwrap().join().unwrap();
    }

    #[test]
    fn create_thread_pool() {
        let mut pool = ThreadPool::new(4).unwrap();

        for id in 0..4 {
            let worker = pool.threads.get(id).unwrap();
            assert_eq!(worker.id, id);
        }

        pool.threads
            .iter_mut()
            .for_each(|worker| worker.thread.take().unwrap().join().unwrap())
    }
}
