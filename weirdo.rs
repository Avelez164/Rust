use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use std::env;

const NUM_PHILOSOPHERS: usize = 5;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Thinking,
    Hungry,
    Eating,
}

struct Table {
    states: Vec<State>,
    condvars: Vec<Condvar>,
}

fn left(id: usize) -> usize {
    (id + NUM_PHILOSOPHERS - 1) % NUM_PHILOSOPHERS
}

fn right(id: usize) -> usize {
    (id + 1) % NUM_PHILOSOPHERS
}

fn test(id: usize, table: &mut Table) {
    if table.states[id] == State::Hungry
        && table.states[left(id)] != State::Eating
        && table.states[right(id)] != State::Eating
    {
        table.states[id] = State::Eating;
        table.condvars[id].notify_one();
    }
}

fn pickup_forks(id: usize, table: &Arc<(Mutex<Table>, Condvar)>) {
    let (lock, _) = &**table;
    let mut guard = lock.lock().unwrap();
    guard.states[id] = State::Hungry;
    test(id, &mut guard);

    // Extract the condvar reference early to avoid borrow issues
    let condvar_ptr = std::ptr::addr_of!(guard.condvars[id]);
    while guard.states[id] != State::Eating {
        // SAFELY get condvar again without violating borrow rules
        let condvar = unsafe { &*condvar_ptr };
        guard = condvar.wait(guard).unwrap();
    }
}

fn putdown_forks(id: usize, table: &Arc<(Mutex<Table>, Condvar)>) {
    let (lock, _) = &**table;
    let mut data = lock.lock().unwrap();
    data.states[id] = State::Thinking;
    test(left(id), &mut data);
    test(right(id), &mut data);
}

fn think(id: usize) {
    println!("P#{} THINKING.", id);
    thread::sleep(Duration::from_millis(100));
}

fn eat(id: usize, round: usize) {
    println!("P#{} EATING ({} / {}).", id, round, round_total());
    thread::sleep(Duration::from_millis(100));
    println!("P#{} finished eating and is thinking again.", id);
}

fn round_total() -> usize {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(3)
    } else {
        3
    }
}

fn main() {
    let loops = round_total();

    let states = vec![State::Thinking; NUM_PHILOSOPHERS];
    let mut condvars = vec![];
    for _ in 0..NUM_PHILOSOPHERS {
        condvars.push(Condvar::new());
    }

    let table = Arc::new((
        Mutex::new(Table {
            states,
            condvars,
        }),
        Condvar::new(), // unused global condvar (could remove if not using it)
    ));

    let mut handles = vec![];

    for id in 0..NUM_PHILOSOPHERS {
        let table_clone = Arc::clone(&table);
        let handle = thread::spawn(move || {
            for round in 1..=loops {
                think(id);
                pickup_forks(id, &table_clone);
                eat(id, round);
                putdown_forks(id, &table_clone);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("All philosophers have finished.");
}
