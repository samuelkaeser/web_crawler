/*use crate::helper_functions;

use std::net::SocketAddr;
use discv5::{enr, enr::{CombinedKey, Enr}, Discv5, Discv5ConfigBuilder};
use std::collections::HashSet;
use std::{fs, fs::File, path::Path};
use tokio::sync::{Mutex as TokioMutex, Semaphore};
use std::sync::Arc;



//fn main(){
//   //main_functions::main_for_find_randomized();
//   main_functions::main_for_jump_routing_tables()
//}

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
    let mut destination_node_enr:Enr<CombinedKey> = helper_functions::first_bootnode_enr();
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
        let current_routing_table = helper_functions::get_entire_routing_table(&mut discv5, &destination_node_enr);
        routing_tables.push(current_routing_table.clone());
        let new_nodes = helper_functions::filter_new_nodes(&current_routing_table, &discovered_nodes);
        for node in &new_nodes{
            discovered_nodes.push(node.clone());
        }
        println!("Currently {} nodes in routing table after calling get_entire_routing_table", discv5.table_entries().len());
        let hash_set: HashSet<_> = HashSet::from_iter(discovered_nodes.clone());
        if hash_set.len() == discovered_nodes.len(){
            println!("Nodes in discovered_nodes are unique, all good");
        }else{
            println!("Nodes in discovered_nodes aren't unique, something went wrong");
        }
        println!("Currently {} nodes discovered after calling get_entire_routing_table", discovered_nodes.len());

        match helper_functions::save_enrs_to_file(&new_nodes, &"test_enrs_base64.txt") {
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


#[tokio::main]
async fn main_parallel() {
    // listening address and port
    let listen_addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();

    // construct a local ENR
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = enr::EnrBuilder::new("v4").build(&enr_key).unwrap();

    // default configuration
    let config = Discv5ConfigBuilder::new().build();

    // construct the discv5 server
    let discv5: Discv5 = Discv5::new(enr, enr_key, config).unwrap();
    let discv5_mutex = Arc::new(TokioMutex::new(discv5));

    // start the discv5 server
    {
        let mut discv5 = discv5_mutex.lock().await;
        discv5.start(listen_addr).await.unwrap();
    }

    // Create a specific node's ENR using their base64 encoded string
    let destination_node_enr = helper_functions::first_bootnode_enr();

    let discovered_nodes_mutex = Arc::new(TokioMutex::new(Vec::new()));
    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));

    // Initialize discovered_nodes with the first node
    {
        let mut discovered_nodes_guard = discovered_nodes_mutex.lock().await;
        discovered_nodes_guard.push(destination_node_enr.clone());
    }

    helper_functions::process_nodes(discv5_mutex, discovered_nodes_mutex, semaphore).await;
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // listening address and port
    let listen_addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();
        
    // construct a local ENR
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = enr::EnrBuilder::new("v4").build(&enr_key).unwrap();

    // build the tokio executor
    //let mut runtime = tokio::runtime::Builder::new_multi_thread()
    //    .thread_name("Discv5-example")
    //    .enable_all()
    //    .build()
    //    .unwrap();

    // default configuration
    let config = Discv5ConfigBuilder::new().build();

    // construct the discv5 server
    let mut discv5: Discv5 = Discv5::new(enr, enr_key, config).unwrap();

    // Create a specific node's ENR using their base64 encoded string
    let mut destination_node_enr:Enr<CombinedKey> = helper_functions::first_bootnode_enr();
    // start the discv5 server
    discv5.start(listen_addr).await; // runtime.block_on(

    let mut routing_tables: Vec<Vec<Enr<CombinedKey>>> = Vec::new();
    let mut discovered_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    let mut new_nodes_tracker: Vec<usize> = Vec::new();
    discovered_nodes.push(destination_node_enr.clone());
    let mut current_destination_index = 0;
    let enr_file_name = "enr_list.json";
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(enr_file_name)?;

    // Start the JSON array
    file.write_all(b"[")?;
    fs::create_dir_all("routing_tables");

    let mut counter = 0;
    while current_destination_index <= discovered_nodes.len() && (counter < 1){
        println!("Currently {} nodes in routing table before calling get_entire_routing_table", discv5.table_entries().len());
        let current_routing_table = helper_functions::get_entire_routing_table(&mut discv5, &destination_node_enr).await;
        routing_tables.push(current_routing_table.clone());
        let new_nodes = helper_functions::filter_new_nodes(&current_routing_table, &discovered_nodes);
        for node in &new_nodes{
            discovered_nodes.push(node.clone());
        }
        new_nodes_tracker.push(new_nodes.len());
        println!("Currently {} nodes in routing table after calling get_entire_routing_table", discv5.table_entries().len());
        let hash_set: HashSet<_> = HashSet::from_iter(discovered_nodes.clone());
        if hash_set.len() == discovered_nodes.len(){
            println!("Nodes in discovered_nodes are unique, all good");
        }else{
            println!("Nodes in discovered_nodes aren't unique, something went wrong");
        }
        println!("Currently {} nodes discovered after calling get_entire_routing_table", discovered_nodes.len());
        let enr_with_features = helper_functions::process_enrs(new_nodes).await?;
        let new_responisve_nodes = helper_functions::extract_responsive_enrs(enr_with_features.clone());
        for node in &new_responisve_nodes{
            discovered_nodes.push(node.clone());
        }
        match helper_functions::save_enrs_to_file(&enr_with_features, &enr_file_name) {
            Ok(_) => println!("Saved ENRs to file successfully."),
            Err(e) => {
                println!("Error saving ENRs to file: {:?}", e);
            }
        }

        let path = format!("routing_tables/routing_table_{}_base64.txt", destination_node_enr.node_id().to_string());
        let _save_file = File::create(path.clone()).expect("Failed to create save file");

        match helper_functions::save_routing_table_to_file(&current_routing_table, &path) {
            Ok(_) => println!("Saved routing table to file successfully."),
            Err(e) => {
                println!("Error saving routing table to file: {:?}", e);
            }
        }
        current_destination_index += 1;
        destination_node_enr = discovered_nodes[current_destination_index].clone();
        counter += 1;
    };
    helper_functions::delete_last_character_and_add_closing_bracket(enr_file_name)?;
    helper_functions::write_vector_to_file(new_nodes_tracker, "new nodes numbers.txt")?;

    Ok(())
}


mod helper_functions;

use std::net::SocketAddr;
use discv5::{enr, enr::{CombinedKey, Enr, NodeId}, Discv5, Discv5ConfigBuilder};
use std::collections::HashSet;
use std::{fs, fs::{File, OpenOptions}, path::Path, io::Write, sync::Arc};
use tokio::sync::{Mutex as TokioMutex, Semaphore};
use std::time::{Duration, Instant};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination_node = helper_functions::first_bootnode_enr();


    // listening address and port
    let listen_addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();
        
    // construct a local ENR
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = enr::EnrBuilder::new("v4").build(&enr_key).unwrap();
 
    // build the tokio executor
    //let mut runtime = tokio::runtime::Builder::new_multi_thread()
    //    .thread_name("Discv5-example")
    //    .enable_all()
    //    .build()
    //    .unwrap();
 
    // default configuration
    let config = Discv5ConfigBuilder::new().build();
 
    // construct the discv5 server
    let mut discv5: Discv5 = Discv5::new(enr, enr_key, config).unwrap();
 
    // start the discv5 server
    discv5.add_enr(destination_node.clone());
    discv5.start(listen_addr).await; // runtime.block_on(

    
    let target_node_id = NodeId::random();
    let start = Instant::now();
    let _ = helper_functions::find_node_and_handle_error(&mut discv5, target_node_id);
    let elapsed_time = start.elapsed();
    println!("Took {:?} to respond to request", elapsed_time);



    let mut discovered_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    let mut new_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    let enr_file_name = "enr_list.json";
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(enr_file_name)?;

    // Start the JSON array
    file.write_all(b"[")?;
    new_nodes.push(destination_node);
    let enr_with_features = helper_functions::process_enrs(new_nodes).await?;
        let new_responisve_nodes = helper_functions::extract_responsive_enrs(enr_with_features.clone());
        for node in &new_responisve_nodes{
            discovered_nodes.push(node.clone());
        }
        match helper_functions::save_enrs_to_file(&enr_with_features, &enr_file_name) {
            Ok(_) => println!("Saved ENRs to file successfully."),
            Err(e) => {
                println!("Error saving ENRs to file: {:?}", e);
            }
        }
    Ok(())
}



// Create a custom error type for handling timeout errors
#[derive(Debug)]
enum CustomQueryError {
    Discv5(discv5::QueryError),
    TimedOut,
}

impl Display for CustomQueryError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            CustomQueryError::Discv5(err) => write!(f, "Discv5 error: {}", err),
            CustomQueryError::TimedOut => write!(f, "Query timed out"),
        }
    }
}

impl Error for CustomQueryError {}

impl From<discv5::QueryError> for CustomQueryError {
    fn from(err: discv5::QueryError) -> Self {
        CustomQueryError::Discv5(err)
    }
}

#[tokio::main]
async fn main() {
    // Set up Discv5 instance (make sure to replace "your_enr_key" with your actual key)
    let enr_key = CombinedKey::generate_secp256k1();
    let enr = enr::EnrBuilder::new("v4").build(&enr_key).unwrap();
    let config = Discv5ConfigBuilder::new().build();
    let listen_addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();

    let mut discv5: Discv5 = Discv5::new(enr, enr_key, config).unwrap();

    let enr_key = CombinedKey::generate_secp256k1();
    let random_enr = enr::EnrBuilder::new("v4").build(&enr_key).unwrap();
    let mut destination_node_enr:Enr<CombinedKey> = helper_functions::first_bootnode_enr();
    // destination_node_enr = random_enr.clone();
    let distance = 255;
    let bitstring_distance = helper_functions::set_bit(distance);
    let target_node_id: NodeId = helper_functions::random_node_id_at_distance(bitstring_distance, &helper_functions::node_id_to_array(destination_node_enr.node_id()));
    
    discv5.add_enr(destination_node_enr.clone());
    let timeout_duration = Duration::from_secs(30); // Timeout of 30 seconds

    // Create a new Tokio runtime and call the find_node function with a timeout
    discv5.start(listen_addr).await;
    let start = Instant::now();
    let result = discv5.find_node(target_node_id).await.unwrap();
    // Compute the duration of the code execution
    let duration = start.elapsed();
    if result.len() > 0{
        for enr in result{
            println!("{:?}", enr)
        }
    }

    // Print the elapsed time
    println!("Time elapsed: {:?} for distance of {:?}", duration, distance);

}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Create a specific node's ENR using their base64 encoded string
    let mut destination_node_enr:Enr<CombinedKey> = helper_functions::first_bootnode_enr();

    let mut routing_tables: Vec<Vec<Enr<CombinedKey>>> = Vec::new();
    let mut discovered_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    let mut new_nodes_tracker: Vec<usize> = Vec::new();
    discovered_nodes.push(destination_node_enr.clone());
    let mut current_destination_index = 0;
    let enr_file_name = "enr_list.json";
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(enr_file_name)?;

    // Start the JSON array
    file.write_all(b"[")?;
    fs::create_dir_all("routing_tables");

    let mut counter = 0;
    while current_destination_index <= discovered_nodes.len() && (counter < 1){
        let current_routing_table = helper_functions::get_entire_routing_table(&destination_node_enr).await;
        routing_tables.push(current_routing_table.clone());
        let new_nodes = helper_functions::filter_new_nodes(&current_routing_table, &discovered_nodes);
        for node in &new_nodes{
            discovered_nodes.push(node.clone());
        }
        new_nodes_tracker.push(new_nodes.len());
        let hash_set: HashSet<_> = HashSet::from_iter(discovered_nodes.clone());
        if hash_set.len() == discovered_nodes.len(){
            println!("Nodes in discovered_nodes are unique, all good");
        }else{
            println!("Nodes in discovered_nodes aren't unique, something went wrong");
        }
        println!("Currently {} nodes discovered after calling get_entire_routing_table", discovered_nodes.len());
        let enr_with_features = helper_functions::process_enrs(new_nodes).await?;
        let new_responisve_nodes = helper_functions::extract_responsive_enrs(enr_with_features.clone());
        for node in &new_responisve_nodes{
            discovered_nodes.push(node.clone());
        }
        match helper_functions::save_enrs_to_file(&enr_with_features, &enr_file_name) {
            Ok(_) => println!("Saved ENRs to file successfully."),
            Err(e) => {
                println!("Error saving ENRs to file: {:?}", e);
            }
        }

        let path = format!("routing_tables/routing_table_{}_base64.txt", destination_node_enr.node_id().to_string());
        let _save_file = File::create(path.clone()).expect("Failed to create save file");

        match helper_functions::save_routing_table_to_file(&current_routing_table, &path) {
            Ok(_) => println!("Saved routing table to file successfully."),
            Err(e) => {
                println!("Error saving routing table to file: {:?}", e);
            }
        }
        current_destination_index += 1;
        destination_node_enr = discovered_nodes[current_destination_index].clone();
        counter += 1;
    };
    helper_functions::delete_last_character_and_add_closing_bracket(enr_file_name)?;
    helper_functions::write_vector_to_file(new_nodes_tracker, "new nodes numbers.txt")?;

    Ok(())
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Create a specific node's ENR using their base64 encoded string
    let mut destination_node_enr:Enr<CombinedKey> = helper_functions::first_bootnode_enr();

    let mut routing_tables: Vec<Vec<Enr<CombinedKey>>> = Vec::new();
    let mut discovered_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    let mut new_nodes_tracker: Vec<usize> = Vec::new();
    discovered_nodes.push(destination_node_enr.clone());
    let mut current_destination_index = 0;
    let enr_file_name = "enr_list.json";
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(enr_file_name)?;

    // Start the JSON array
    file.write_all(b"[")?;
    fs::create_dir_all("routing_tables");

    let mut counter = 0;
    while current_destination_index <= discovered_nodes.len() && (counter < 1){
        let current_routing_table = helper_functions::get_entire_routing_table(&destination_node_enr).await;
        routing_tables.push(current_routing_table.clone());
        let new_nodes = helper_functions::filter_new_nodes(&current_routing_table, &discovered_nodes);
        for node in &new_nodes{
            discovered_nodes.push(node.clone());
        }
        new_nodes_tracker.push(new_nodes.len());
        let hash_set: HashSet<_> = HashSet::from_iter(discovered_nodes.clone());
        if hash_set.len() == discovered_nodes.len(){
            println!("Nodes in discovered_nodes are unique, all good");
        }else{
            println!("Nodes in discovered_nodes aren't unique, something went wrong");
        }
        println!("Currently {} nodes discovered after calling get_entire_routing_table", discovered_nodes.len());
        let enr_with_features = helper_functions::process_enrs(new_nodes).await?;
        let new_responisve_nodes = helper_functions::extract_responsive_enrs(enr_with_features.clone());
        for node in &new_responisve_nodes{
            discovered_nodes.push(node.clone());
        }
        match helper_functions::save_enrs_to_file(&enr_with_features, &enr_file_name) {
            Ok(_) => println!("Saved ENRs to file successfully."),
            Err(e) => {
                println!("Error saving ENRs to file: {:?}", e);
            }
        }

        let path = format!("routing_tables/routing_table_{}_base64.txt", destination_node_enr.node_id().to_string());
        let _save_file = File::create(path.clone()).expect("Failed to create save file");

        match helper_functions::save_routing_table_to_file(&current_routing_table, &path) {
            Ok(_) => println!("Saved routing table to file successfully."),
            Err(e) => {
                println!("Error saving routing table to file: {:?}", e);
            }
        }
        current_destination_index += 1;
        destination_node_enr = discovered_nodes[current_destination_index].clone();
        counter += 1;
    };
    helper_functions::delete_last_character_and_add_closing_bracket(enr_file_name)?;
    helper_functions::write_vector_to_file(new_nodes_tracker, "new nodes numbers.txt")?;

    Ok(())
}

*/