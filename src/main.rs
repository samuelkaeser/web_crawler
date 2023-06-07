mod helper_functions;
use discv5::{enr::{CombinedKey, Enr, NodeId}, enr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{time::{Duration, Instant}, path::Path};
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::fs::File;
use std::io::Write;
use rayon::ThreadPoolBuilder;
use rand::Rng;
use crossbeam::channel;
use serde::{Serialize, Deserialize};
use std::fs;
use std::collections::HashSet;
use std::iter::FromIterator;
use rayon::prelude::*;
use crossbeam::channel::TryRecvError;
use simplelog::*;
use log::{info, warn};
use std::{fs::OpenOptions, io};
use std::collections::VecDeque;

fn main() {
    let file = File::create("log.txt").unwrap();
    let index_file = Arc::new(Mutex::new(File::create("index.txt").expect("Failed to create index file")));
    let _ = WriteLogger::init(LevelFilter::Info, Config::default(), file);

    let boot_node:Enr<CombinedKey> = discv5::enr::Enr::from_str("enr:-LS4QC0q8zzSXlOXa72v06kJHgsz83AaU3nZpNGLY5SxDdBwdOOznjAT1bz0lUEIFoo0vtPkEkm_9TJYIdneL6Uo8ZCCAceHYXR0bmV0c4gIAAAAAAAAAIRldGgykLuk2pYDAAAA__________-CaWSCdjSCaXCEM1EuoolzZWNwMjU2azGhA6AZT42UEnsHQMfm-mW6b1dPXUuLA6vdc6GgPw5i8tIAg3RjcIIjKIN1ZHCCIyg").unwrap();
    let ip_address = IpAddr::from_str("0.0.0.0").unwrap();
    let mut data = helper_functions::CustomNodes {
        discovered_nodes: [boot_node.clone()].iter().cloned().collect(),
        processed_nodes: HashSet::new(),
        nodes_to_be_processed: VecDeque::new(),
        listening_ports: (10000..=40000).collect(),
    };
    data.nodes_to_be_processed.push_back(boot_node.clone());
    let routing_table_counter = Arc::new(AtomicUsize::new(0));

    let (tx, rx) = channel::unbounded();

    let pool_1 = ThreadPoolBuilder::new().num_threads(200).build().unwrap();
    let pool_2 = ThreadPoolBuilder::new().num_threads(200).build().unwrap();

    let mut start = Instant::now();

    loop{

        if let Some(node_to_process) = data.nodes_to_be_processed.pop_front(){
            start = Instant::now();

            let listening_address = {
                let port = data.listening_ports.pop().unwrap();
                SocketAddr::new(ip_address, port)
            };
    
            let tx = tx.clone();
            info!("Launching {}, Queue-length: {}", node_to_process.to_base64(), data.nodes_to_be_processed.len());
            pool_1.spawn(move || {
                //println!("Launching thread for node {}", node_to_process);
                let thread_start = Instant::now();
                let routing_table = helper_functions::get_routing_table_custom(node_to_process.clone(), listening_address);
                let duration = thread_start.elapsed();
                info!("Processed {} Duration: {:?}", node_to_process.clone().to_base64(), duration);
                tx.send((node_to_process, routing_table, listening_address.clone())).unwrap();
            });
        }

        while let Ok((processed_node, routing_table, listening_address)) = rx.try_recv() {
            // This code will be executed by a worker thread in the pool
            let start_saving = Instant::now();
            let routing_table_clone = routing_table.clone();
            let processed_node_clone = processed_node.clone();
            let index_file_clone = Arc::clone(&index_file);
            let counter_clone = Arc::clone(&routing_table_counter);

            pool_2.spawn(move || {
                let node_id = processed_node_clone.node_id();
                let node_id_str = hex::encode(node_id);
                println!("NodeId: {}", node_id_str);
                let sequence_number = counter_clone.fetch_add(1, Ordering::SeqCst);
                let path = format!("routing_tables/{}.txt", sequence_number);
                let _save_file = File::create(path.clone()).expect("Failed to create save file");
                match helper_functions::save_routing_table_to_file(&routing_table_clone, &path) {
                    Ok(_) => {        
                        info!("Saved_routing_table {}", processed_node_clone.to_base64());
                        match helper_functions::count_files_in_directory("./routing_tables") {
                            Ok(count) => info!("Routing_tables_folder_count {}", count),
                            Err(e) => println!("Error reading routing_tables directory: {}", e),
                        }
                    },
                    Err(e) => {
                        info!("Error saving routing table to file: {:?}", e);
                    }
                }
                let mut index_file_guard = index_file_clone.lock().unwrap();
                writeln!(index_file_guard, "{}:{}", sequence_number, node_id_str).unwrap();
            });

            for node in routing_table {
                if data.discovered_nodes.insert(node.clone()){
                    helper_functions::append_node_to_file("discovered_nodes.txt", &node).unwrap();
                    data.nodes_to_be_processed.push_back(node.clone());
                    info!("Added_discovered_nodes {} currently {}", node.clone().to_base64(), data.discovered_nodes.len());
                }
            }
            data.processed_nodes.insert(processed_node.clone());
            helper_functions::append_node_to_file("processed_nodes.txt", &processed_node).unwrap();
            info!("Added_processed_nodes {} currently {}", processed_node.to_base64(), data.processed_nodes.len());

            data.listening_ports.push(listening_address.port());
            println!("Saving to file and updating variables took {:?}.", start_saving.elapsed())
        }
        let duration = start.elapsed();
        if duration > Duration::from_secs(60 * 60){
            break;
        }
    }
}