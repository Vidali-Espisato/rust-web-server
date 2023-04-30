use std::{sync::{mpsc, Arc, Mutex}, thread};

type Job = Box<dyn FnOnce() + Send + 'static>;
enum Message {
    New(Job),
    Terminate
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}


impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::New(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("Shutting down worker id: {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {

        let thread = Some(
            thread::spawn(move || loop {
                let message = receiver
                    .lock().unwrap()
                    .recv().unwrap();

                match message {
                    Message::New(job) => {
                        println!("Executing job using worker id: {}", id);
                        job();
                    },
                    Message::Terminate => {
                        println!("Terminating worker id: {}", id);
                        break;
                    }
                };
            })
        );

        Self { id, thread }
    }
}
