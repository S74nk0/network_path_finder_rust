#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

mod network;
mod nodes;
mod search_stop_settings;

pub use network::*;
pub use search_stop_settings::SearchStopSettings;
