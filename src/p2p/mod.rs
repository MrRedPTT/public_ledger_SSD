pub(super) mod private {
    pub(super) mod res_handler;
    pub(super) mod req_handler;

    pub(super) mod broadcast_api;
}

pub(super) mod peer_modules {
    pub(super) mod peer_rpc_server;
    pub(super) mod peer_rpc_client_handler;
    pub(super) mod peer_rpc_client;
    pub(super) mod peer_blockchain_events;
}
pub mod peer;
