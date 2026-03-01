use super::ConnectorStatus;

pub struct AtemConnector;

impl AtemConnector {
    pub fn new() -> Self {
        Self
    }

    pub fn get_status(&self) -> ConnectorStatus {
        ConnectorStatus::Disconnected
    }
}

impl Default for AtemConnector {
    fn default() -> Self {
        Self::new()
    }
}
