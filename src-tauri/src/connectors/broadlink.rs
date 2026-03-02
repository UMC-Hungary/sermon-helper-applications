use super::ConnectorStatus;

pub struct BroadlinkConnector;

impl BroadlinkConnector {
    pub fn new() -> Self {
        Self
    }

    pub fn get_status(&self) -> ConnectorStatus {
        ConnectorStatus::Disconnected
    }
}

impl Default for BroadlinkConnector {
    fn default() -> Self {
        Self::new()
    }
}
