use crate::client::Client;

#[derive(Clone)]
pub struct CommonState {
    pub client: Client,
}

impl CommonState {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}
