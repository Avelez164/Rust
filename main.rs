use rand::Rng;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

const MAX_DENIES: usize = 3;

#[derive(Clone)]
struct Process {
    _id: usize,           // unused, silences warning
    _max: Vec<usize>,     // unused, silences warning
    allocation: Vec<usize>,
    need: Vec<usize>,
}

struct Bank {
    available: Vec<usize>,
    processes: Vec<Process>,
}

struct SafeSeq {
    seq: Vec<usize>,
    idx: usize,
}

fn read_input(filename: &str) -> (Bank, usize, usize) {
    let file = File::open(filename).expect("failed to open input file");
    let mut lines = BufReader::new(file).lines().map(|l| l.unwrap());

    let first = lines.next().unwrap();
    let mut parts = first.split_whitespace().map(|s| s.parse::<usize>().unwrap());
    let p = parts.next().unwrap();
    let r = parts.next().unwrap();

    let available: Vec<usize> = lines
        .next()
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect();

    let mut processes = Vec::with_capacity(p);
    for id in 0..p {
        let nums: Vec<usize> = lines
            .next()
            .unwrap()
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let max = nums[0..r].to_vec();
        let allocation = nums[r..2 * r].to_vec();
        let need = nums[2 * r..3 * r].to_vec();
        processes.push(Process {
            _id: id,
            _max: max,
            allocation,
            need,
        });
    }

    (Bank { available, processes }, p, r)
}

fn is_safe_state(bank: &Bank) -> bool {
    let p = bank.processes.len();
    let r = bank.available.len();
    let mut work = bank.available.clone();
    let mut finish = vec![false; p];

    for _ in 0..p {
        let mut found = false;
        for (i, proc) in bank.processes.iter().enumerate() {
            if !finish[i] && proc.need.iter().zip(&work).all(|(&n, &w)| n <= w) {
                for j in 0..r {
                    work[j] += proc.allocation[j];
                }
                finish[i] = true;
                found = true;
            }
        }
        if !found { return false; }
    }
    true
}

fn compute_safe_seq(bank: &Bank) -> Vec<usize> {
    let p = bank.processes.len();
    let r = bank.available.len();
    let mut work = bank.available.clone();
    let mut finish = vec![false; p];
    let mut seq = Vec::with_capacity(p);

    for _ in 0..p {
        let mut found = false;
        for (i, proc) in bank.processes.iter().enumerate() {
            if !finish[i] && proc.need.iter().zip(&work).all(|(&n, &w)| n <= w) {
                for j in 0..r {
                    work[j] += proc.allocation[j];
                }
                finish[i] = true;
                seq.push(i);
                found = true;
                break;
            }
        }
        if !found { panic!("No safe sequence!"); }
    }
    seq
}

fn main() {
    let filename = std::env::args().nth(1).unwrap_or_else(|| "Bankers_rs.txt".into());
    let (bank, p, r) = read_input(&filename);

    let safe_seq = compute_safe_seq(&bank);
    println!("Initial Available: {:?}", bank.available);
    println!("Safe sequence:   {:?}", safe_seq);

    let bank = Arc::new(Mutex::new(bank));
    let safe = Arc::new((Mutex::new(SafeSeq { seq: safe_seq, idx: 0 }), Condvar::new()));
    let total_requests = Arc::new(Mutex::new(0));
    let granted = Arc::new(Mutex::new(0));
    let denied = Arc::new(Mutex::new(0));

    let mut handles = Vec::with_capacity(p);
    for id in 0..p {
        let bank_cl   = Arc::clone(&bank);
        let safe_cl   = Arc::clone(&safe);
        let tot_cl    = Arc::clone(&total_requests);
        let ok_cl     = Arc::clone(&granted);
        let no_cl     = Arc::clone(&denied);

        handles.push(thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut denies = 0;

            loop {
                // Check if done
                {
                    let mut bk = bank_cl.lock().unwrap();
                    if bk.processes[id].need.iter().all(|&n| n == 0) {
                        println!("P{} has finished. Releasing resources.", id);
                        for j in 0..r {
                            bk.available[j] += bk.processes[id].allocation[j];
                            bk.processes[id].allocation[j] = 0;
                        }
                        break;
                    }
                }

                // Build random request or full-need fallback
                let fallback = denies >= MAX_DENIES;
                let mut req = vec![0; r];
                let mut valid = false;
                {
                    let bk = bank_cl.lock().unwrap();
                    for j in 0..r {
                        let need = bk.processes[id].need[j];
                        if need > 0 {
                            req[j] = if fallback { need } else { rng.gen_range(0..=need) };
                            if req[j] > 0 { valid = true; }
                        }
                    }
                }
                if !valid {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                // Fallback branch
                if fallback {
                    let (lock, cvar) = &*safe_cl;
                    // Acquire once, do wait, fallback, advance, notify
                    let mut seq = lock.lock().unwrap();
                    while seq.seq[seq.idx] != id {
                        seq = cvar.wait(seq).unwrap();
                    }

                    // Perform full-need request
                    {
                        let mut bk = bank_cl.lock().unwrap();
                        println!("P{} requesting full-need: {:?}", id, req);
                        *tot_cl.lock().unwrap() += 1;
                        for j in 0..r {
                            bk.available[j]  -= req[j];
                            bk.processes[id].allocation[j] += req[j];
                            bk.processes[id].need[j] = 0;
                        }
                        *ok_cl.lock().unwrap() += 1;
                        println!("Request GRANTED to P{} (fallback)", id);
                    }

                    // Release resources
                    {
                        let mut bk = bank_cl.lock().unwrap();
                        println!("P{} has finished. Releasing resources.", id);
                        for j in 0..r {
                            bk.available[j] += bk.processes[id].allocation[j];
                            bk.processes[id].allocation[j] = 0;
                        }
                    }

                    // Advance and notify
                    seq.idx += 1;
                    cvar.notify_all();
                    break;
                }

                // Normal randomized request
                {
                    let mut bk = bank_cl.lock().unwrap();
                    println!("P{} requesting: {:?}", id, req);
                    *tot_cl.lock().unwrap() += 1;

                    // Raw availability check
                    if req.iter().zip(&bk.available).all(|(&r, &a)| r <= a) {
                        // Pretend allocate
                        for j in 0..r {
                            bk.available[j]  -= req[j];
                            bk.processes[id].allocation[j] += req[j];
                            bk.processes[id].need[j] -= req[j];
                        }
                        // Safety check
                        if is_safe_state(&bk) {
                            println!("Request GRANTED to P{}", id);
                            *ok_cl.lock().unwrap() += 1;
                            denies = 0;
                        } else {
                            // Roll back
                            for j in 0..r {
                                bk.available[j]  += req[j];
                                bk.processes[id].allocation[j] -= req[j];
                                bk.processes[id].need[j] += req[j];
                            }
                            println!("Request DENIED to P{} (unsafe)", id);
                            *no_cl.lock().unwrap() += 1;
                            denies += 1;
                        }
                    } else {
                        println!("Request DENIED to P{} (not enough resources)", id);
                        *no_cl.lock().unwrap() += 1;
                        denies += 1;
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("\nAll processes completed.");
    println!("Total requests: {}", *total_requests.lock().unwrap());
    println!("Granted requests: {}", *granted.lock().unwrap());
    println!("Denied requests: {}", *denied.lock().unwrap());
}
