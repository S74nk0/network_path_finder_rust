mod shared_memory_ipc;

use crate::shared_memory_ipc::*;
use ctrlc;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use shared_memory::*;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn get_link_name_pid_only(pid: u32) -> String {
    return format!("shm_{}_.link", pid);
}

static GLOBAL_LOCK_ID: usize = 0;
static SHM_SUBSCRIBE: &str = "shm_subscribe.link";
// use std::mem; maybe set the SHM_SUBSCRIBE size as a closest power of 2 size
// of ShmemHanshakeSubscribe struct
static SHM_SUBSCRIBE_SIZE: usize = 4096;

fn random_miliseconds(min: u64, max: u64) -> u64 {
    let mut rng = rand::thread_rng();
    return rng.gen_range(min, max);
}

fn get_now_as_f64() -> f64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_millis() as f64,
        _ => 0.0,
    }
}

fn create_or_open_linked_shmem(link_path: &str, size: usize) -> Result<SharedMem, SharedMemError> {
    match SharedMem::create_linked(link_path, LockType::Mutex, size) {
        // We created and own this mapping
        Ok(v) => Ok(v),
        // Link file already exists
        Err(SharedMemError::LinkExists) => SharedMem::open_linked(link_path),
        Err(e) => Err(e),
    }
}

fn create_or_open_linked_shmem_default(link_path: &str) -> Result<SharedMem, SharedMemError> {
    create_or_open_linked_shmem(link_path, SHM_SUBSCRIBE_SIZE)
}

type ArcAtomicBool = Arc<AtomicBool>;

fn main() -> Result<(), SharedMemError> {
    println!("Attempting to create/open custom shmem !");
    let subscribe_shmem = create_or_open_linked_shmem_default(SHM_SUBSCRIBE)?;
    println!("Mapping info : {}", subscribe_shmem);

    if subscribe_shmem.num_locks() != 1 {
        println!("Expected to only have 1 lock in shared mapping !");
        return Err(SharedMemError::InvalidHeader);
    }

    // TODO handle except correctly
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    if subscribe_shmem.is_owner() {
        master(subscribe_shmem, running)
    } else {
        slave(subscribe_shmem, running)
    }
}

fn slave(mut subscribe_shmem: SharedMem, running: ArcAtomicBool) -> Result<(), SharedMemError> {
    let pb = ProgressBar::new(10);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:4.cyan/blue}] {msg}")
            .progress_chars("#>-"),
    );
    let s_pid = std::process::id();
    println!(
        "[S][{}] Subscribing to master (pass my info and handshake)...",
        s_pid
    );
    // subscribe to master attemts TODO add number of tries instead of trying forever
    loop {
        if !running.load(Ordering::Relaxed) {
            println!("[S][{}] Subscribe to master canceled exiting...", s_pid);
            return Ok(());
        }
        // Scope to ensure proper Drop of lock
        {
            let mut subscribe_state =
                subscribe_shmem.wlock::<ShmemHanshakeSubscribe>(GLOBAL_LOCK_ID)?;
            if subscribe_state.new_sub == false {
                subscribe_state.sub_pid = s_pid;
                subscribe_state.new_sub = true;
                println!("[S][{}] Subscribe to master success...", s_pid);
                break;
            }
        }
        let wait_before_shm_link = random_miliseconds(100, 500);
        pb.set_message(&format!(
            "[S][{}] Subscribing to master wait for {}ms...",
            s_pid, wait_before_shm_link
        ));
        thread::sleep(Duration::from_millis(wait_before_shm_link));
    }
    // persistant data testing
    let mut last_slave_write_time: f64 = get_now_as_f64();
    let mut last_master_read_time: f64 = get_now_as_f64();
    let mut last_master_write_time: f64 = get_now_as_f64();
    let mut ca_delta_slave_write_time = ConstantAverage::new();
    let mut ca_delta_master_read_time = ConstantAverage::new();
    let mut ca_delta_master_write_time = ConstantAverage::new();
    let shmem_link = get_link_name_pid_only(s_pid);
    // here we don't care who owns the shared memory. Maybe this will be importantn in the future
    let mut shmem = create_or_open_linked_shmem_default(&shmem_link)?;
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(10)); // loop each second
                                                  /*loop pin pong to the shared memory*/
        // WRITE data
        // Scope to ensure proper Drop of lock
        {
            let mut shared_state = shmem.wlock::<ShmemPipeData>(GLOBAL_LOCK_ID)?;
            shared_state.counter_slave += 1;
            let prev_last_slave_time = last_slave_write_time;
            last_slave_write_time = get_now_as_f64();
            shared_state.last_slave_write_time = last_slave_write_time;
            let delta = last_slave_write_time - prev_last_slave_time;
            ca_delta_slave_write_time.add_to_average(delta);
            // progress
            pb.set_message(&format!(
                "WL [c_slave: {}] [c_master: {}] [avg_D S_wt: {:.2}] [avg_D S_rt: {:.2}] [avg_D M_wt: {:.2}]",
                shared_state.counter_slave,
                shared_state.counter_master,
                ca_delta_slave_write_time.get_average(),
                ca_delta_master_read_time.get_average(),
                ca_delta_master_write_time.get_average(),
            ));
            pb.set_position(0);
        }

        // READ data
        // Scope to ensure proper Drop of lock
        {
            let shared_state = shmem.rlock::<ShmemPipeData>(GLOBAL_LOCK_ID)?;
            let prev_last_master_time = last_master_read_time;
            last_master_read_time = get_now_as_f64();
            let delta = last_master_read_time - prev_last_master_time;
            ca_delta_master_read_time.add_to_average(delta);
            // Master write time
            let prev_last_master_write_time = last_master_write_time;
            last_master_write_time = shared_state.last_master_write_time;
            ca_delta_master_write_time
                .add_to_average(last_master_write_time - prev_last_master_write_time);
            // progress

            pb.set_message(&format!(
                "RL [c_slave: {}] [c_master: {}] [avg_delta S_wt: {:.2}] [avg_delta S_rt: {:.2}] [avg_D M_wt: {:.2}]",
                shared_state.counter_slave,
                shared_state.counter_master,
                ca_delta_slave_write_time.get_average(),
                ca_delta_master_read_time.get_average(),
                ca_delta_master_write_time.get_average(),
            ));
            pb.set_position(5);
        }
    }

    println!("[S][{}] Exiting loop!", s_pid);
    println!("[S][{}] Sleeping for 2 seconds !", s_pid);
    thread::sleep(Duration::from_secs(2));
    println!("[S][{}]\tDone", s_pid);

    Ok(())
}

struct SlaveInfo {
    pid: u32,
    id: u64,
}

struct SlaveSharedMemoryOpenError {
    info: SlaveInfo,
    err: SharedMemError,
}

impl fmt::Display for SlaveSharedMemoryOpenError {
    // TODO not sure about the format! macro
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&format!(
            "SlaveInfo id({}), pid({}) err: {}",
            self.info.id, self.info.pid, self.err,
        ))?;
        Ok(())
    }
}

// TODO this should probably be a generic look at 'last_slave_count'
struct SlaveIcpShm {
    info: SlaveInfo,
    shmem: shared_memory::SharedMem,
    // state make this a generic type
    last_slave_count: u32, // this should be some generic State that could hold any tiype
}

type SlaveSharedMemOpenResult = Result<SlaveIcpShm, SlaveSharedMemoryOpenError>;

// TODO make this as a generic
fn consume_and_open_ipc_shmem(info: SlaveInfo) -> SlaveSharedMemOpenResult {
    let shmem_link = get_link_name_pid_only(info.pid);
    match create_or_open_linked_shmem_default(&shmem_link) {
        Ok(shmem) => Ok(SlaveIcpShm {
            info: info,
            shmem: shmem,
            // state create a new function for the state
            last_slave_count: 0,
        }),
        Err(e) => Err(SlaveSharedMemoryOpenError { info: info, err: e }),
    }
}

enum MasterInfoMessage {
    Subscribe(SlaveInfo),
    Exit,
}

impl MasterInfoMessage {
    // consume and transform self to info
    fn consume_to_info(self) -> Option<SlaveInfo> {
        match self {
            MasterInfoMessage::Subscribe(info) => Some(info),
            _ => None,
        }
    }
}

fn master(mut subscribe_shmem: SharedMem, running: ArcAtomicBool) -> Result<(), SharedMemError> {
    let (tx, rx): (Sender<MasterInfoMessage>, Receiver<MasterInfoMessage>) = mpsc::channel();

    // create the IPC slave communication thread
    let slave_ipc_shm_reader_thread = thread::spawn(move || {
        let mut slave_infos: Vec<SlaveIcpShm> = Vec::new();
        loop {
            println!("SLAVE LOOP START");
            let (new_subs, exit_msgs): (Vec<_>, Vec<_>) = rx
                .try_iter()
                .map(|msg| msg.consume_to_info())
                .partition(Option::is_some);
            if !exit_msgs.is_empty() {
                println!("BREAK CALLED");
                break; // break the loop
            }
            // we didn't get exit message
            let (success_s_ipc, fail_s_ipc): (Vec<_>, Vec<_>) = new_subs
                .into_iter()
                .map(|info_opt| consume_and_open_ipc_shmem(info_opt.unwrap())) // it is safe to unwrap
                .partition(Result::is_ok);
            let mut success_s_ipc: Vec<SlaveIcpShm> = success_s_ipc
                .into_iter()
                // .map(Result::unwrap) // no Debug derive not going to work
                .map(Result::ok) // result ok and unwrap because result unwrap won't work
                .map(Option::unwrap)
                .collect();
            if success_s_ipc.len() > 0 {
                println!("NEW SUCCESS slave_infos {}", success_s_ipc.len());
            }
            if fail_s_ipc.len() > 0 {
                println!("NEW FAIL slave_infos {}", fail_s_ipc.len());
            }
            slave_infos.append(&mut success_s_ipc);

            // TODO map filter
            let results: Vec<Result<(), SharedMemError>> = slave_infos
                .iter_mut()
                .map(|s_ipc| -> Result<(), SharedMemError> {
                    // READ
                    {
                        let shared_state = s_ipc.shmem.rlock::<ShmemPipeData>(GLOBAL_LOCK_ID)?;
                        if shared_state.counter_slave != s_ipc.last_slave_count {
                            println!(
                                "[M]\tID\t{}\tpid\t{}\tcounted\t{}!",
                                s_ipc.info.id, s_ipc.info.pid, shared_state.counter_slave
                            );
                            s_ipc.last_slave_count = shared_state.counter_slave;
                        }
                        // TODO add random wait before dropping to see it in action
                        // Release global lock asap
                        drop(shared_state);
                    }
                    // WRITE
                    {
                        // TODO this should be fixed in the s_ipc type
                        let mut shared_state =
                            s_ipc.shmem.wlock::<ShmemPipeData>(GLOBAL_LOCK_ID)?;
                        // // SIMULATE WORK
                        // thread::sleep(Duration::from_millis(1000));
                        shared_state.counter_master += 1;
                        shared_state.last_master_write_time = get_now_as_f64();
                    }
                    Ok(())
                })
                .collect();
            for result in results {
                println!("[M]\tResult\t{} !", result.is_ok());
            }
            // TODO loop timeout
            thread::sleep(Duration::from_millis(10));
        }
        // do the cleanup here
    });

    // master subscribe thingy
    let mut id = 0;
    loop {
        if !running.load(Ordering::Relaxed) {
            tx.send(MasterInfoMessage::Exit)
                .unwrap_or_else(|e| println!("subscribe thread send err {}", e));
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
        let mut subscribe_state =
            subscribe_shmem.wlock::<ShmemHanshakeSubscribe>(GLOBAL_LOCK_ID)?;
        let s_pid = subscribe_state.sub_pid;
        let has_new_sub = subscribe_state.new_sub;
        // clear new sub
        subscribe_state.new_sub = false;
        if has_new_sub {
            let msg = MasterInfoMessage::Subscribe(SlaveInfo {
                pid: s_pid,
                id: id as u64,
            });
            tx.send(msg)
                .unwrap_or_else(|err| println!("tx.send error {}", err));
            id += 1;
        }
    }

    println!("[M] Exiting");
    slave_ipc_shm_reader_thread
        .join()
        .unwrap_or_else(|_| println!("thread join err"));
    println!("[M] Bye");
    Ok(())
}
