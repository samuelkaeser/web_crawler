use discv5::{enr, enr::{CombinedKey, NodeId}, TokioExecutor, Discv5, Discv5ConfigBuilder};
use std::net::SocketAddr;

// use lighthouse_network::Enr;


pub fn script() {
 
    // listening address and port
    let listen_addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();
 
    // construct a local ENR
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = enr::EnrBuilder::new("v4").build(&enr_key).unwrap();
 
    // build the tokio executor
    let mut runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("Discv5-example")
        .enable_all()
        .build()
        .unwrap();
 
    // default configuration
    let config = Discv5ConfigBuilder::new().build();
 
    // construct the discv5 server
    let mut discv5: Discv5 = Discv5::new(enr, enr_key, config).unwrap();
 
    // In order to bootstrap the routing table an external ENR should be added
    // This can be done via add_enr. I.e.:
    // discv5.add_enr();
     
    // start the discv5 server
    runtime.block_on(discv5.start(listen_addr));
 
    // run a find_node query
    runtime.block_on(async {
       let found_nodes = discv5.find_node(NodeId::random()).await.unwrap();
       println!("Found nodes: {:?}", found_nodes);
    });
}