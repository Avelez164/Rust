// Cargo.toml dependencies:
// [dependencies]
// tokio = { version = "1", features = ["full"] }

use std::env;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{sleep, Duration};

const N: usize = 5; // number of philosophers

// A fork is represented as a mutex.
struct Fork {
    mutex: Mutex<()>,
}

#[tokio::main]
async fn main() {
    // Allow a command-line parameter "loops" (default 3)
    let args: Vec<String> = env::args().collect();
    let loops: usize = if args.len() > 1 {
        args[1].parse().unwrap_or(3)
    } else {
        3
    };

    // Create a vector of forks (one per philosopher)
    let forks: Arc<Vec<Arc<Fork>>> = Arc::new(
        (0..N)
            .map(|_| Arc::new(Fork { mutex: Mutex::new(()) }))
            .collect(),
    );

    // The "room" semaphore limits concurrent eating attempts to 2.
    let room = Arc::new(Semaphore::new(2));

    // Spawn one asynchronous task per philosopher.
    let mut handles = vec![];
    for id in 0..N {
        let forks = Arc::clone(&forks);
        let room = Arc::clone(&room);
        handles.push(tokio::spawn(async move {
            philosopher(id, loops, forks, room).await;
        }));
    }

    // Wait for all philosophers to finish.
    for handle in handles {
        handle.await.unwrap();
    }
    println!("✅ All philosophers have finished.");
}

async fn philosopher(
    id: usize,
    loops: usize,
    forks: Arc<Vec<Arc<Fork>>>,
    room: Arc<Semaphore>,
) {
    for iter in 0..loops {
        // THINKING phase.
        println!("P#{} THINKING.", id);
        sleep(Duration::from_millis(100)).await;

        // Get hungry.
        println!("P#{} HUNGRY.", id);

        // Acquire a permit from the "room" semaphore.
        let permit = room.acquire().await.unwrap();

        // Determine the two fork indices: left and right.
        let left = id;
        let right = (id + 1) % N;
        // To avoid deadlock, lock the lower-numbered fork first.
        let (first, second) = if left < right { (left, right) } else { (right, left) };

        // Pick up forks (lock them).
        let first_fork = forks[first].mutex.lock().await;
        let second_fork = forks[second].mutex.lock().await;

        // Begin eating.
        println!("P#{} EATING (iteration {}/{})", id, iter + 1, loops);
        sleep(Duration::from_millis(100)).await;

        // Finished eating; forks are released when locks drop.
        drop(second_fork);
        drop(first_fork);
        drop(permit); // leave the room.

        println!("P#{} finished eating and is thinking again.", id);
    }
}

// Use pthread_cond variable
// or we can use tokio:: semaphore