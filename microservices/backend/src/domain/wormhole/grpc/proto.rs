// Include the compiled protos by root `build.rs` file

pub mod spy {
    pub mod v1 {
        tonic::include_proto!("spy.v1");
    }
}

pub mod gossip {
    pub mod v1 {
        tonic::include_proto!("gossip.v1");
    }
}

pub mod publicrpc {
    pub mod v1 {
        tonic::include_proto!("publicrpc.v1");
    }
}
