use crate::helper_functions;

use std::net::SocketAddr;
use discv5::{enr, enr::{CombinedKey, Enr}, Discv5, Discv5ConfigBuilder};
use std::collections::HashSet;
use std::{str::FromStr, fs, fs::{File}, path::Path};

pub fn main_for_find_randomized(){
    // listening address and port
    let listen_addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();
 
    // construct a local ENR
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = enr::EnrBuilder::new("v4").build(&enr_key).unwrap();
    let origin_node_id = enr.clone().node_id();

    // build the tokio executor
    let runtime = tokio::runtime::Builder::new_multi_thread()
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
    let enr_bootnode = helper_functions::first_bootnode_enr();
    discv5.add_enr(enr_bootnode);
    //let first_from_node_id = enr.node_id();

    // start the discv5 server
    runtime.block_on(discv5.start(listen_addr));
 
    // run a find_node query, uncomment first line and comment out second line to use the reveresed approach
    //let found_nodes = test::loop_over_discover_beacon_nodes_rev(runtime, origin_node_id, &mut discv5);
    let _found_nodes = helper_functions::loop_over_discover_beacon_nodes(runtime, origin_node_id, &mut discv5);
}

pub fn main_for_jump_routing_tables(){

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

    // Create a specific node's ENR using their base64 encoded string
    let mut destination_node_enr:Enr<CombinedKey> = discv5::enr::Enr::from_str("enr:-LK4QA8FfhaAjlb_BXsXxSfiysR7R52Nhi9JBt4F8SPssu8hdE1BXQQEtVDC3qStCW60LSO7hEsVHv5zm8_6Vnjhcn0Bh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhAN4aBKJc2VjcDI1NmsxoQJerDhsJ-KxZ8sHySMOCmTO6sHM3iCFQ6VMvLTe948MyYN0Y3CCI4yDdWRwgiOM").unwrap();
    let mut current_routing_table:Vec<Enr<CombinedKey>> = Vec::new();
    // start the discv5 server
    runtime.block_on(discv5.start(listen_addr));

    let mut routing_tables: Vec<Vec<Enr<CombinedKey>>> = Vec::new();
    let mut discovered_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    discovered_nodes.push(destination_node_enr.clone());
    let mut current_destination_index = 0;

    let _save_file = File::create("test_enrs_base64.txt").expect("Failed to create save file");
    fs::create_dir_all("routing_tables");

    while current_destination_index <= discovered_nodes.len(){
        println!("Currently {} nodes in routing table before calling get_entire_routing_table", discv5.table_entries().len());
        current_routing_table = helper_functions::get_entire_routing_table(&mut discv5, &destination_node_enr, &runtime);
        routing_tables.push(current_routing_table.clone());
        let new_nodes = helper_functions::filter_new_nodes(&current_routing_table, &discovered_nodes);
        for node in new_nodes{
            discovered_nodes.push(node);
        }
        println!("Currently {} nodes in routing table after calling get_entire_routing_table", discv5.table_entries().len());
        let hash_set: HashSet<_> = HashSet::from_iter(discovered_nodes.clone());
        if hash_set.len() == discovered_nodes.len(){
            println!("Nodes in discovered_nodes are unique, all good");
        }else{
            println!("Nodes in discovered_nodes aren't unique, something went wrong");
        }
        println!("Currently {} nodes discovered after calling get_entire_routing_table", discovered_nodes.len());

        match helper_functions::save_enrs_to_file(&discovered_nodes, &"test_enrs_base64.txt") {
            Ok(_) => println!("Saved ENRs to file successfully."),
            Err(e) => {
                println!("Error saving ENRs to file: {:?}", e);
            }
        }

        let path = format!("routing_tables/routing_table_{}_base64.txt", destination_node_enr.node_id().to_string());
        let _save_file = File::create(path.clone()).expect("Failed to create save file");

        match helper_functions::save_enrs_to_file(&current_routing_table, &path) {
            Ok(_) => println!("Saved routing table to file successfully."),
            Err(e) => {
                println!("Error saving routing table to file: {:?}", e);
            }
        }
        current_destination_index += 1;
        destination_node_enr = discovered_nodes[current_destination_index].clone();
    };

    let retrieved_enrs: Vec<Enr<CombinedKey>> = helper_functions::load_enrs_from_file(&"test_enrs_base64.txt").unwrap(); 

    let file_string = "test_nodes.txt";
    let file_path = Path::new(file_string);
    match helper_functions::save_to_file(&file_path, &retrieved_enrs) {
        Ok(_) => println!("Data saved to file"),
        Err(e) => eprintln!("Error saving data to file: {:?}", e),
    }
}