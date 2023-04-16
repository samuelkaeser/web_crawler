mod helper_functions;
mod main_functions;

use discv5::{enr, enr::{CombinedKey, Enr, NodeId}, Discv5, Discv5ConfigBuilder};
use std::{str::FromStr, fs, fs::{File, OpenOptions}, io::{Read, Write}, path::Path, collections::HashSet, net::SocketAddr};
use tokio::sync::{Mutex as TokioMutex, Semaphore};
use tokio::runtime::Runtime;
use tokio::time::{timeout, error::Elapsed};
use std::time::Duration;
use discv5::QueryError;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::Instant;

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
    while current_destination_index <= discovered_nodes.len(){// && (counter < 2){
        let current_routing_table = helper_functions::get_entire_routing_table(&destination_node_enr).await;
        routing_tables.push(current_routing_table.clone());
        let new_nodes = helper_functions::filter_new_nodes(&current_routing_table, &discovered_nodes);
        new_nodes_tracker.push(new_nodes.len());
        //println!("Currently {} nodes discovered after calling get_entire_routing_table", discovered_nodes.len());
        let enr_with_features = helper_functions::process_enrs(new_nodes).await?;
        let new_responisve_nodes = helper_functions::extract_responsive_enrs(enr_with_features.clone());
        for node in &new_responisve_nodes{
            discovered_nodes.push(node.clone());
        }
        let hash_set: HashSet<_> = HashSet::from_iter(discovered_nodes.clone());
        if hash_set.len() == discovered_nodes.len(){
            //println!("Nodes in discovered_nodes are unique, all good");
        }else{
            println!("Nodes in discovered_nodes aren't unique, something went wrong");
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
    helper_functions::write_vector_to_file(new_nodes_tracker, "new_nodes_numbers.txt")?;

    Ok(())
}


