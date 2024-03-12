use shared_memory::*;

/// This is used for subscribing pipe
#[derive(SharedMemCast)]
pub struct ShmemHanshakeSubscribe {
    pub new_sub: bool,
    pub sub_pid: u32,
    pub subscribe_size: usize,
    pub subscribe_data: [u8; 2048],
}

// TODO add bytes and bincode here
// TODO
#[derive(SharedMemCast)]
pub struct ShmemPipeData {
    pub counter_slave: u32,
    pub counter_master: u32,
    pub last_slave_write_time: f64,
    pub last_master_write_time: f64,
}

#[derive(SharedMemCast)]
pub struct ShmemPipeData2 {
    pub ping_pong: bool,
    pub slave_time: f64,
    pub master_time: f64,
}

// https://stackoverflow.com/questions/22999487/update-the-average-of-a-continuous-sequence-of-numbers-in-constant-time
pub struct ConstantAverage {
    average: f64,
    size: f64,
}

impl ConstantAverage {
    pub fn new() -> Self {
        ConstantAverage {
            average: 0.0,
            size: 0.0,
        }
    }
    pub fn add_to_average(&mut self, value: f64) {
        self.average = (self.size * self.average + value) / (self.size + 1.0);
        self.size += 1.0;
    }
    pub fn get_average(&self) -> f64 {
        self.average
    }
}
