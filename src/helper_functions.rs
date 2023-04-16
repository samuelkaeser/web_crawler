// use enr::{EnrBuilder, k256::ecdsa::SigningKey, Enr, CombinedKey};
use rand::{Rng};
use std::{error::Error, str::FromStr, fs, fs::{File, OpenOptions}, io::{Read, Write}, path::Path, collections::HashSet, net::SocketAddr, mem};
use discv5::{enr, enr::{CombinedKey, Enr, NodeId}, Discv5, Discv5ConfigBuilder};
use rlp::decode;
use tokio::runtime::Runtime;
use tokio::sync::MutexGuard;
use tokio::sync::{Mutex as TokioMutex, Semaphore};
use std::sync::{Arc};
use tokio::task;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use futures::future::try_join_all;
use futures::TryFutureExt;



pub fn first_bootnode_enr()-> Enr<CombinedKey> {
    // let base_64_string = "enr:-Ly4QFPk-cTMxZ3jWTafiNblEZkQIXGF2aVzCIGW0uHp6KaEAvBMoctE8S7YU0qZtuS7By0AA4YMfKoN9ls_GJRccVpFh2F0dG5ldHOI__________-EZXRoMpCC9KcrAgAQIIS2AQAAAAAAgmlkgnY0gmlwhKh3joWJc2VjcDI1NmsxoQKrxz8M1IHwJqRIpDqdVW_U1PeixMW5SfnBD-8idYIQrIhzeW5jbmV0cw-DdGNwgiMog3VkcIIjKA";
    // Teku team's bootnode
		//"enr:-KG4QOtcP9X1FbIMOe17QNMKqDxCpm14jcX5tiOE4_TyMrFqbmhPZHK_ZPG2Gxb1GE2xdtodOfx9-cgvNtxnRyHEmC0ghGV0aDKQ9aX9QgAAAAD__________4JpZIJ2NIJpcIQDE8KdiXNlY3AyNTZrMaEDhpehBDbZjM_L9ek699Y7vhUJ-eAdMyQW_Fil522Y0fODdGNwgiMog3VkcIIjKA",
		//"enr:-KG4QL-eqFoHy0cI31THvtZjpYUu_Jdw_MO7skQRJxY1g5HTN1A0epPCU6vi0gLGUgrzpU-ygeMSS8ewVxDpKfYmxMMGhGV0aDKQtTA_KgAAAAD__________4JpZIJ2NIJpcIQ2_DUbiXNlY3AyNTZrMaED8GJ2vzUqgL6-KD1xalo1CsmY4X1HaDnyl6Y_WayCo9GDdGNwgiMog3VkcIIjKA",
		// Prylab team's bootnodes
		//"enr:-Ku4QImhMc1z8yCiNJ1TyUxdcfNucje3BGwEHzodEZUan8PherEo4sF7pPHPSIB1NNuSg5fZy7qFsjmUKs2ea1Whi0EBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpD1pf1CAAAAAP__________gmlkgnY0gmlwhBLf22SJc2VjcDI1NmsxoQOVphkDqal4QzPMksc5wnpuC3gvSC8AfbFOnZY_On34wIN1ZHCCIyg",
		//"enr:-Ku4QP2xDnEtUXIjzJ_DhlCRN9SN99RYQPJL92TMlSv7U5C1YnYLjwOQHgZIUXw6c-BvRg2Yc2QsZxxoS_pPRVe0yK8Bh2F0dG5ldHOIAAAAAAAAAACEZXRoMpD1pf1CAAAAAP__________gmlkgnY0gmlwhBLf22SJc2VjcDI1NmsxoQMeFF5GrS7UZpAH2Ly84aLK-TyvH-dRo0JM1i8yygH50YN1ZHCCJxA",
		//"enr:-Ku4QPp9z1W4tAO8Ber_NQierYaOStqhDqQdOPY3bB3jDgkjcbk6YrEnVYIiCBbTxuar3CzS528d2iE7TdJsrL-dEKoBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpD1pf1CAAAAAP__________gmlkgnY0gmlwhBLf22SJc2VjcDI1NmsxoQMw5fqqkw2hHC4F5HZZDPsNmPdB1Gi8JPQK7pRc9XHh-oN1ZHCCKvg",
		// Lighthouse team's bootnodes
		//"enr:-Jq4QItoFUuug_n_qbYbU0OY04-np2wT8rUCauOOXNi0H3BWbDj-zbfZb7otA7jZ6flbBpx1LNZK2TDebZ9dEKx84LYBhGV0aDKQtTA_KgEAAAD__________4JpZIJ2NIJpcISsaa0ZiXNlY3AyNTZrMaEDHAD2JKYevx89W0CcFJFiskdcEzkH_Wdv9iW42qLK79ODdWRwgiMo",
		//"enr:-Jq4QN_YBsUOqQsty1OGvYv48PMaiEt1AzGD1NkYQHaxZoTyVGqMYXg0K9c0LPNWC9pkXmggApp8nygYLsQwScwAgfgBhGV0aDKQtTA_KgEAAAD__________4JpZIJ2NIJpcISLosQxiXNlY3AyNTZrMaEDBJj7_dLFACaxBfaI8KZTh_SSJUjhyAyfshimvSqo22WDdWRwgiMo",
		// EF bootnodes
		//"enr:-Ku4QHqVeJ8PPICcWk1vSn_XcSkjOkNiTg6Fmii5j6vUQgvzMc9L1goFnLKgXqBJspJjIsB91LTOleFmyWWrFVATGngBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhAMRHkWJc2VjcDI1NmsxoQKLVXFOhp2uX6jeT0DvvDpPcU8FWMjQdR4wMuORMhpX24N1ZHCCIyg",
		//"enr:-Ku4QG-2_Md3sZIAUebGYT6g0SMskIml77l6yR-M_JXc-UdNHCmHQeOiMLbylPejyJsdAPsTHJyjJB2sYGDLe0dn8uYBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhBLY-NyJc2VjcDI1NmsxoQORcM6e19T1T9gi7jxEZjk_sjVLGFscUNqAY9obgZaxbIN1ZHCCIyg",
		//"enr:-Ku4QPn5eVhcoF1opaFEvg1b6JNFD2rqVkHQ8HApOKK61OIcIXD127bKWgAtbwI7pnxx6cDyk_nI88TrZKQaGMZj0q0Bh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhDayLMaJc2VjcDI1NmsxoQK2sBOLGcUb4AwuYzFuAVCaNHA-dy24UuEKkeFNgCVCsIN1ZHCCIyg",
		//"enr:-Ku4QEWzdnVtXc2Q0ZVigfCGggOVB2Vc1ZCPEc6j21NIFLODSJbvNaef1g4PxhPwl_3kax86YPheFUSLXPRs98vvYsoBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhDZBrP2Jc2VjcDI1NmsxoQM6jr8Rb1ktLEsVcKAPa08wCsKUmvoQ8khiOl_SLozf9IN1ZHCCIyg",
		// Nimbus bootnodes
		//"enr:-LK4QA8FfhaAjlb_BXsXxSfiysR7R52Nhi9JBt4F8SPssu8hdE1BXQQEtVDC3qStCW60LSO7hEsVHv5zm8_6Vnjhcn0Bh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhAN4aBKJc2VjcDI1NmsxoQJerDhsJ-KxZ8sHySMOCmTO6sHM3iCFQ6VMvLTe948MyYN0Y3CCI4yDdWRwgiOM",
		//"enr:-LK4QKWrXTpV9T78hNG6s8AM6IO4XH9kFT91uZtFg1GcsJ6dKovDOr1jtAAFPnS2lvNltkOGA9k29BUN7lFh_sjuc9QBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpC1MD8qAAAAAP__________gmlkgnY0gmlwhANAdd-Jc2VjcDI1NmsxoQLQa6ai7y9PMN5hpLe5HmiJSlYzMuzP7ZhwRiwHvqNXdoN0Y3CCI4yDdWRwgiOM",
    let enr_1:Enr<CombinedKey> = discv5::enr::Enr::from_str("enr:-Jq4QItoFUuug_n_qbYbU0OY04-np2wT8rUCauOOXNi0H3BWbDj-zbfZb7otA7jZ6flbBpx1LNZK2TDebZ9dEKx84LYBhGV0aDKQtTA_KgEAAAD__________4JpZIJ2NIJpcISsaa0ZiXNlY3AyNTZrMaEDHAD2JKYevx89W0CcFJFiskdcEzkH_Wdv9iW42qLK79ODdWRwgiMo").unwrap();
    return enr_1;
}

pub fn find_node(runtime:tokio::runtime::Runtime, call_node_id:NodeId, discv5:Discv5)->Vec<Enr<CombinedKey>>{
    let mut found_nodes_1: Vec<Enr<CombinedKey>> = Vec::new();

    runtime.block_on(async {
    found_nodes_1 = discv5.find_node(call_node_id).await.unwrap();
    });
    for enr in found_nodes_1.iter(){
        for (key, value) in enr.iter(){
            println!("key: {}", String::from_utf8_lossy(key));
            println!("value: {}", String::from_utf8_lossy(value));
        }
    }
    return found_nodes_1;
}

pub fn save (input:Vec<Enr<CombinedKey>>, file_path:String)-> std::io::Result<()>{
    let mut file = fs::File::create(file_path)?;

    for num in input {
        writeln!(file, "{}", num).expect("Failed to write to file");
    }
    Ok(())
}

pub fn check_beacon(enr_to_check:&Enr<CombinedKey>) -> bool {
    if let Some(encoded_value) = enr_to_check.get("eth2").map(|value| value.to_vec()) {
        // Decode the RLP-encoded value
        let decoded_value: Vec<u8> = match decode(&encoded_value) {
            Ok(value) => value,
            Err(_) => {
                println!("Discovered eth2-key but failed to decode RLP-encoded value.");
                return false;
            }
        };
        //println!("Decoded value: {:?}", decoded_value);
        return true;
    } else {
        return false;
    }
}

pub fn array_to_node_id(input_array:[u8; 32]) -> NodeId {
    let node_id: NodeId= match NodeId::try_from(input_array) {
        Ok(id) => {
            Some(id).unwrap()
        }
        Err(e) => {
            eprintln!("Failed to convert: {:?}", e);
            None.unwrap()
        }
    };
    return node_id;
}

pub fn node_id_to_array(node_id: NodeId) -> [u8; 32] {
    let bytes = node_id.as_ref();
    let mut array = [0u8; 32];
    array.copy_from_slice(bytes);
    return array;
}

pub fn subtract_one(mut array: [u8; 32]) -> [u8; 32] {
    for i in (0..array.len()).rev() {
        if array[i] == 0 {
            array[i] = 255; // wrap around to 255 if the current element is 0
        } else {
            array[i] -= 1;
            break; // exit the loop as soon as a non-zero element is found
        }
    }
    return array;
}

pub fn random_node_id_at_distance(distance: [u8; 32], from: &[u8; 32]) -> NodeId {
    let mut rng = rand::thread_rng();

    // Generate a random mask at the specified distance.
    let mut random_bytes = [0u8; 32];
    rng.fill(&mut random_bytes);
    
    let distance_minus_one_array = subtract_one(distance);
    
    let mut mask = [0u8; 32];
    for i in 0..32 {
        mask[i] = distance_minus_one_array[i] & random_bytes[i];
    }
    // Apply the mask and XOR it with the original Node ID.
    let mut result_1_array = [0u8; 32];

    for i in 0..32 {
        result_1_array[i] = distance[i] ^ mask[i];
    }
    
    let from_array = from;
    let mut result_2_array = [0u8; 32];

    for i in 0..32 {
        result_2_array[i] = from_array[i] ^ result_1_array[i];
    }
    let new_node_id = array_to_node_id(result_2_array);
    return new_node_id;
}

pub fn print_bitstring(array: &[u8]) {
    for byte in array {
        print!("{:08b}", byte);
    }
    println!();
}

pub fn set_bit(index: usize) -> [u8; 32] {
    let mut array = [0u8; 32];
    let mut byte_index = 0;
    let mut bit_index = 0;
    if index != 0{
        byte_index = (index-1) / 8;
        bit_index = (index-1) % 8;
    }
    if byte_index < 32 {
        array[31 - byte_index] |= 1 << bit_index;
    }
    return array;
}

pub async fn find_node_and_handle_error(
    discv5: &mut Discv5,
    target_node: NodeId,
) -> Result<Vec<Enr<CombinedKey>>, discv5::QueryError> {
    //println!("went into find_node_and_handle_error");
    match discv5.find_node(target_node).await {
        Ok(results) => {
            //println!("received good results within find_node_and_handle_error");
            Ok(results)
        }
        Err(e) => {
            eprintln!("Error in find_node: {:?}", e);
            Err(e)
        }
    }
}

pub fn save_to_file<T: ToString>(file_path: &Path, data: &[T]) -> std::io::Result<()> {
    let mut file = fs::File::create(file_path)?;

    for item in data {
        writeln!(file, "{}", item.to_string())?;
    }

    file.flush()?;
    Ok(())
}

// Discover beacon nodes iteratively using FINDNODE requests
pub fn discover_beacon_nodes_rev(runtime: &tokio::runtime::Runtime, origin_node_id:&NodeId, discv5: &mut Discv5) -> Vec<Enr<CombinedKey>> {
    let mut discovered_nodes = Vec::new();

    // Perform iterative FINDNODE requests to discover new nodes
    for distance in (0..256).rev() {
        let old_size = discovered_nodes.len();
        let bitstring_distance = set_bit(distance);
        let origin_array = node_id_to_array(*origin_node_id);
        let target_node = random_node_id_at_distance(bitstring_distance, &origin_array);
        let mut findnode_results: Result<Vec<Enr<CombinedKey>>, discv5::QueryError> = Err(discv5::QueryError::ServiceNotStarted);
        //println!("Running findnode request with targetnode: {}", target_node);
        runtime.block_on(async {
        findnode_results = find_node_and_handle_error(discv5, target_node).await});
        match &findnode_results {
            Ok(_) => {
                // Process the results
                //println!("Processing FindNode results:");
                // Filter the discovered nodes to identify beacon nodes
                for enr in findnode_results.unwrap() {
                    if check_beacon(&enr) {
                        if discovered_nodes.contains(&enr){
                            break;
                        }else{
                        discovered_nodes.push(enr);
                        }
                    }
                }
            }
            Err(e) => {
                // Handle the error
                eprintln!("Error processing FindNode results: {:?}", e);
                continue;
            }
        };
        if old_size == discovered_nodes.len(){
            break;
        }
    }
    return discovered_nodes;
}

pub fn loop_over_discover_beacon_nodes_rev(runtime: tokio::runtime::Runtime, origin_node_id:NodeId, discv5: &mut Discv5) -> Vec<Enr<CombinedKey>>{
    let mut found_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    let mut old_size:i128 = -1;
    while old_size < found_nodes.len().try_into().unwrap(){
        println!("First round with {} nodes in routing table", discv5.table_entries().len());
        let new_found_nodes = discover_beacon_nodes_rev(&runtime, &origin_node_id, discv5);
        println!("Found {} new nodes", new_found_nodes.len());
        old_size = found_nodes.len().try_into().unwrap();
        for enr in new_found_nodes{
            if !found_nodes.contains(&enr){
                found_nodes.push(enr);
            }
        }
        println!("Found {} nodes overall", found_nodes.len());
        for enr in &found_nodes{
            discv5.add_enr(enr.clone());
        }
        let file_string = &(origin_node_id.to_string() + ".txt");
        let file_path = Path::new(file_string);
        match save_to_file(&file_path, &found_nodes) {
            Ok(_) => println!("Data saved to file"),
            Err(e) => eprintln!("Error saving data to file: {:?}", e),
        }
        println!("Old size:{}, new size: {}", old_size, found_nodes.len());
    }
    return found_nodes;
}

pub fn filter_new_nodes(new_found:&Vec<Enr<CombinedKey>>, current_nodes:&Vec<Enr<CombinedKey>>) -> Vec<Enr<CombinedKey>>{
    let mut new_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    for node in new_found.iter(){
        if !current_nodes.contains(&node){
            new_nodes.push(node.clone());
        }
    }
    return new_nodes;
}

pub fn discover_beacon_nodes(runtime: &tokio::runtime::Runtime, origin_node_id:&NodeId, discv5: &mut Discv5) -> Vec<Enr<CombinedKey>> {
    let mut discovered_nodes: Vec<Enr<CombinedKey>> = Vec::new();

    // Perform iterative FINDNODE requests to discover new nodes
    for distance in 0..256 {
        let mut miss_counter = 0;
        while miss_counter < 3 {
            println!("Currently {} nodes in routing table", discv5.table_entries().len());
            println!("Currently {} nodes discovered", discovered_nodes.len());
            let bitstring_distance = set_bit(distance);
            let origin_array = node_id_to_array(*origin_node_id);
            let target_node = random_node_id_at_distance(bitstring_distance, &origin_array);
            let mut findnode_results: Result<Vec<Enr<CombinedKey>>, discv5::QueryError> = Err(discv5::QueryError::ServiceNotStarted);
            println!("Running findnode request with targetnode: {}", target_node);
            runtime.block_on(async {
            findnode_results = find_node_and_handle_error(discv5, target_node).await});
            match &findnode_results {
                Ok(_) => {
                    let new_nodes = filter_new_nodes(&findnode_results.unwrap(), &discovered_nodes);
                    if new_nodes.is_empty(){
                        miss_counter += 1;
                        if miss_counter == 3{
                            println!("Tried 3 times with distance {} and didn't find any new nodes, continuing with next distance", distance)
                        }
                    }else{
                        miss_counter = 0;
                        println!("Found {} new nodes in distance {}", new_nodes.len(), distance);
                        for enr in new_nodes {
                            discovered_nodes.push(enr);
                        }
                    }
                }
                Err(e) => {
                    // Handle the error
                    eprintln!("Error processing FindNode results: {:?}", e);
                    continue;
                }
            }
        }
    };
    return discovered_nodes;
}

pub fn loop_over_discover_beacon_nodes(runtime: tokio::runtime::Runtime, origin_node_id:NodeId, discv5: &mut Discv5) -> Vec<Enr<CombinedKey>>{
    let mut found_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    found_nodes = discover_beacon_nodes(&runtime, &origin_node_id, discv5);
    let file_string = &(origin_node_id.to_string() + ".txt");
    let file_path = Path::new(file_string);
    match save_to_file(&file_path, &found_nodes) {
        Ok(_) => println!("Data saved to file"),
        Err(e) => eprintln!("Error saving data to file: {:?}", e),
    }
    return found_nodes;
}

pub fn remove_nodes_from_routing_table(discv5: &mut Discv5) -> &mut Discv5 {
    // Remove all nodes from the routing table
    let node_ids: Vec<NodeId> = discv5.table_entries_id();
    for node_id in node_ids {
        discv5.remove_node(&node_id);
    }
    discv5
}

fn xor_distance_msb(a: &[u8; 32], b: &[u8; 32]) -> usize {
    let mut distance = 0;
    for i in (0..a.len()).rev() {
        for j in (0..8).rev() {
            let same = ((a[i] >> j)& 1) == ((b[i] >> j)& 1);
            if !same {
                distance = 256 - (8*i + 7 - j);
                break;
            }
        }
    }
    distance
}

pub fn determine_furthest_node_distance(new_found_nodes: &HashSet<Enr<CombinedKey>>, destination_node_enr: &Enr<CombinedKey>) -> usize {
    let mut max_distance = 0;
    let mut furthest_node : Enr<CombinedKey> = destination_node_enr.clone();
    for node in new_found_nodes{
        let distance = xor_distance_msb(&node_id_to_array(node.node_id()), &node_id_to_array(destination_node_enr.node_id()));
        if distance > max_distance{
            max_distance = distance;
            furthest_node = node.clone();
        }
    }
    //println!("Distance between node {} and node {} is {}, therefore the distance in the next round is {}", furthest_node.node_id().to_string(), destination_node_enr.node_id().to_string(), max_distance, max_distance);
    return max_distance
}

pub async fn get_entire_routing_table(
    destination_node_enr: &Enr<CombinedKey>, 
) -> Vec<Enr<CombinedKey>>{
    
    let mut new_found_nodes = HashSet::new();
    let mut done: bool = false;
    let mut distance = 235;
    //let bitstring_distance = set_bit(distance);
    //let target_node_id: NodeId = random_node_id_at_distance(bitstring_distance, &node_id_to_array(destination_node_enr.node_id()));

    while !done {
        let listen_addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();
        // construct a local ENR
        let enr_key = CombinedKey::generate_secp256k1();
        let enr = enr::EnrBuilder::new("v4").build(&enr_key).unwrap();
        // default configuration
        let config = Discv5ConfigBuilder::new().build();

        // construct the discv5 server
        let mut discv5: Discv5 = Discv5::new(enr, enr_key, config).unwrap();
        match discv5.start(listen_addr).await {
            Ok(_) => {
                println!("Server in get_entire_routing_table started successfully on {:?}", listen_addr);
            }
            Err(e) => {
                eprintln!("Server in get_entire_routing_table failed to start on {:?}: {:?}", listen_addr, e);
            }
        }

        //println!("Currently {} nodes in routing table before calling add_enr", discv5.table_entries().len());
        let old_size = new_found_nodes.len();
        discv5.add_enr(destination_node_enr.clone());
        //println!("Currently {} nodes in routing table after calling add_enr", discv5.table_entries().len());

        let bitstring_distance = set_bit(distance);
        let target_node_id: NodeId = random_node_id_at_distance(bitstring_distance, &node_id_to_array(destination_node_enr.node_id()));
        let mut result: Result<Vec<Enr<CombinedKey>>, discv5::QueryError> = Err(discv5::QueryError::ServiceNotStarted);
        //println!("Destination node: {}", destination_node_enr.node_id().to_string());
        result = find_node_and_handle_error(&mut discv5, target_node_id).await;

        match &result {
            Ok(_) => {
                // Process the results
                //println!("Processing FindNode results:");
                for enr in result.as_ref().unwrap() {
                    new_found_nodes.insert(enr.clone());
                    //println!("{}", enr.to_string())
                }
                //println!("Found {} new nodes after calling find_node with distance: {} and traget node: {}", result.unwrap().len(), distance, target_node_id.to_string());
                if old_size == new_found_nodes.len(){
                    if distance == 256{
                        //println!("Didn't find any new nodes and reached the max distance");
                        done = true;
                    }else{
                        //println!("Didn't find any new nodes, increasing distance from {} by one to {}", distance, distance+1);
                        distance += 1;
                    }
                }else{
                let furthest_distance = determine_furthest_node_distance(&new_found_nodes, &destination_node_enr);
                if furthest_distance != 0{
                    distance = furthest_distance;
                }
                }
            }
            Err(e) => {
                // Handle the error
                eprintln!("Error processing FindNode results: {:?}, continuing loop", e);
            }
        };
        // Wrap the Discv5 instance with an Option
        let mut discv5_option = Some(discv5);

        // To stop the server, explicitly drop the discv5 instance and set the Option to None
        mem::drop(discv5_option.take());

        // Check if the instance has been dropped
        if discv5_option.is_none() {
        //println!("Server stopped, port released");
        }else{
        println!("Motherfucking server didn't stop");
        }
    }
    let routing_table: Vec<Enr<CombinedKey>> = new_found_nodes.into_iter().collect();
    return routing_table;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnrPlus {
    enr: Enr<CombinedKey>,
    responisve: bool,
}

pub fn save_routing_table_to_file(enrs: &Vec<Enr<CombinedKey>>, file: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true) // Append to the file instead of truncating
        .open(file)?;

    for enr in enrs {
        writeln!(file, "{}", enr.to_base64())?;
    }
    Ok(())
}

pub fn save_enrs_to_file(enrs: &Vec<EnrPlus>, file: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true) // Append to the file instead of truncating
        .open(file)?;

    for (index, enr) in enrs.iter().enumerate() {
        let json_data = serde_json::to_string(&enr)?;
        file.write_all(json_data.as_bytes())?;
        file.write_all(b",")?;
    }
    Ok(())
}

pub fn delete_last_character_and_add_closing_bracket(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the file content
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Remove the last character from the content
    content.pop();

    // Add the closing bracket
    content.push(']');

    // Overwrite the file with the modified content
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

pub fn load_enrs_from_file(file_path: &str) -> Result<Vec<Enr<CombinedKey>>, Box<dyn std::error::Error>> {
    // Read the JSON data from the file
    let mut file = File::open(file_path)?;
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)?;

    // Deserialize the JSON data to a vector of EnrWithFeature structs
    let enr_with_feature_list: Vec<EnrPlus> = serde_json::from_str(&json_data)?;

    // Extract the ENR entries and create a new vector
    let enrs: Vec<Enr<CombinedKey>> = enr_with_feature_list.into_iter().map(|entry| entry.enr).collect();

    Ok(enrs)
}

pub fn write_vector_to_file(vec: Vec<usize>, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Convert the vector of numbers to a string representation
    let vec_string = vec.iter()
        .map(|num| num.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    // Write the string representation to the file
    let mut file = File::create(file_path)?;
    file.write_all(vec_string.as_bytes())?;

    Ok(())
}

async fn is_node_responsive(destination_enr: Enr<CombinedKey>) -> Result<bool, Box<dyn std::error::Error>> {
    let listen_addr = "0.0.0.0:9000".parse::<SocketAddr>().unwrap();
    // construct a local ENR
    let enr_key_server = CombinedKey::generate_secp256k1();
    let enr_server = enr::EnrBuilder::new("v4").build(&enr_key_server).unwrap();
    // default configuration
    let config = Discv5ConfigBuilder::new().build();

    // construct the discv5 server
    let mut discv5: Discv5 = Discv5::new(enr_server, enr_key_server, config).unwrap();
    discv5.add_enr(destination_enr.clone());
    //println!("call is_node_responsive with the following enr: {:?}", destination_enr);
    match discv5.start(listen_addr).await {
        Ok(_) => {
            //println!("Server in is_node_responsive started successfully on {:?}", listen_addr);
        }
        Err(e) => {
            eprintln!("Server in is_node_responsive failed to start on {:?}: {:?}", listen_addr, e);
        }
    }
    // Send a FINDNODE request to the target ENR
    let target_node_id = destination_enr.node_id();
    let start = Instant::now();
    //println!("arrived before find_node_and_handle_error - call");
    let result = find_node_and_handle_error(&mut discv5, target_node_id).await;
    //println!("went past the find_node_and_handle_error - call");
    match &result {
        Ok(_) => {
            // Process the results
            //println!("Processing FindNode results:");
            //for enr in result.as_ref().unwrap() {
            //    println!("{}", enr.to_string())
           // }
           // println!("Found {} new nodes after calling find_node with traget node: {}", result.unwrap().len(), target_node_id.to_string());
        }
        Err(e) => {
            // Handle the error
            eprintln!("Error processing FindNode results: {:?}", e);
        }
    }

    let elapsed_time = start.elapsed();
    // Wrap the Discv5 instance with an Option
    let mut discv5_option = Some(discv5);

    // To stop the server, explicitly drop the discv5 instance and set the Option to None
    mem::drop(discv5_option.take());

    // Check if the instance has been dropped
    if discv5_option.is_none() {
        //println!("Server stopped, port released");
    }else{
        println!("Motherfucking server didn't stop");
    }


    // Check if the response time is more than 1 millisecond
    Ok(elapsed_time > Duration::from_millis(1))
}

pub async fn process_enrs(enrs: Vec<Enr<CombinedKey>>) -> Result<Vec<EnrPlus>, Box<dyn Error>> {
    let mut results = Vec::new();

    for enr in enrs {
        let is_responsive = is_node_responsive(enr.clone()).await?;
        results.push((enr, is_responsive));
    }

    let enr_with_features: Vec<EnrPlus> = results
        .into_iter()
        .map(|(enr, is_responsive)| EnrPlus { enr, responisve: is_responsive })
        .collect();

    Ok(enr_with_features)
}

pub fn extract_responsive_enrs(enr_with_features: Vec<EnrPlus>) -> Vec<Enr<CombinedKey>> {
    enr_with_features
        .into_iter()
        .filter(|enr_with_feature| enr_with_feature.responisve)
        .map(|enr_with_feature| enr_with_feature.enr)
        .collect()
}

pub fn remove_nodes_from_routing_table_parallel(
    discv5: &mut MutexGuard<Discv5>,
) {
    // Remove all nodes from the routing table
    let node_ids: Vec<NodeId> = discv5.table_entries_id();
    for node_id in node_ids {
        discv5.remove_node(&node_id);
    }
}

pub async fn find_node_and_handle_error_parallel<'a>(
    discv5: &'a Discv5,
    target_node_id: NodeId,
) -> Result<Vec<Enr<CombinedKey>>, discv5::QueryError> {
    match discv5.find_node(target_node_id).await {
        Ok(results) => {        
            Ok(results)
        }
        Err(e) => {
            eprintln!("Error in find_node: {:?}", e);
            Err(e)
        }
    }
}

pub async fn get_entire_routing_table_parallel(
    discv5_mutex: Arc<TokioMutex<Discv5>>,
    destination_node_enr: &Enr<CombinedKey>,
) -> Vec<Enr<CombinedKey>> {
    
    let mut new_found_nodes = HashSet::new();
    let mut done: bool = false;
    let mut distance = 0;
    //let bitstring_distance = set_bit(distance);
    //let target_node_id: NodeId = random_node_id_at_distance(bitstring_distance, &node_id_to_array(destination_node_enr.node_id()));

    while !done {
        // Lock the mutex and get the Discv5 instance
        let mut discv5 = discv5_mutex.lock().await;
        println!("Currently {} nodes in routing table before calling add_enr", discv5.table_entries().len());
        let old_size = new_found_nodes.len();
        match discv5.add_enr(destination_node_enr.clone()) {
            Ok(_) => println!("Remote ENR added to the routing table successfully."),
            Err(e) => {
                println!("Error adding remote ENR to the routing table: {:?}", e);
                break;
            }
        }
        println!("Currently {} nodes in routing table after calling add_enr", discv5.table_entries().len());

        let bitstring_distance = set_bit(distance);
        let target_node_id: NodeId = random_node_id_at_distance(bitstring_distance, &node_id_to_array(destination_node_enr.node_id()));
        let mut result: Result<Vec<Enr<CombinedKey>>, discv5::QueryError> = Err(discv5::QueryError::ServiceNotStarted);
        println!("Destination node: {}", destination_node_enr.node_id().to_string());
        result = find_node_and_handle_error_parallel(&*discv5, target_node_id).await;

        match &result {
            Ok(_) => {
                // Process the results
                println!("Processing FindNode results:");
                for enr in result.as_ref().unwrap() {
                    new_found_nodes.insert(enr.clone());
                    println!("{}", enr.to_string())
                }
                println!("Found {} new nodes after calling find_node with distance: {} and traget node: {}", result.unwrap().len(), distance, target_node_id.to_string());
                if old_size == new_found_nodes.len(){
                    if distance == 256{
                        println!("Didn't find any new nodes and reached the max distance");
                        done = true;
                    }else{
                        println!("Didn't find any new nodes, increasing distance from {} by one to {}", distance, distance+1);
                        distance += 1;
                    }
                }else{
                distance = determine_furthest_node_distance(&new_found_nodes, &destination_node_enr);
                }
            }
            Err(e) => {
                // Handle the error
                eprintln!("Error processing FindNode results: {:?}, continuing loop", e);
            }
        };
        remove_nodes_from_routing_table_parallel(&mut discv5);
    }
    let routing_table: Vec<Enr<CombinedKey>> = new_found_nodes.into_iter().collect();
    return routing_table;
}

pub fn filter_new_nodes_parallel(
    new_found:&Vec<Enr<CombinedKey>>, 
    current_nodes:&Vec<Enr<CombinedKey>>,
) -> Vec<Enr<CombinedKey>>{
    let mut new_nodes: Vec<Enr<CombinedKey>> = Vec::new();
    for node in new_found.iter(){
        if !current_nodes.contains(&node){
            new_nodes.push(node.clone());
        }
    }
    return new_nodes;
}

async fn process_node(
    discv5_mutex: Arc<TokioMutex<Discv5>>,
    discovered_nodes_mutex: Arc<TokioMutex<Vec<Enr<CombinedKey>>>>,
    semaphore: Arc<Semaphore>,
    destination_node_enr: Enr<CombinedKey>,
) {
    let _permit = semaphore.acquire().await;
    let discv5_guard = discv5_mutex.lock().await;
    let current_routing_table = get_entire_routing_table_parallel(
        discv5_mutex.clone(),
        &destination_node_enr,
    )
    .await;
    drop(discv5_guard);

    let discovered_nodes_guard = discovered_nodes_mutex.lock().await;
    let new_nodes = filter_new_nodes_parallel(&current_routing_table, &*discovered_nodes_guard);
    {
        let mut discovered_nodes_guard = discovered_nodes_mutex.lock().await;
        discovered_nodes_guard.extend(new_nodes);
    }
    drop(_permit);
}


// get a set of listening_addresses that are going to be distributed among the get_routing_table function calls and have to
// be released again, once the function call has returned the routing_table
pub async fn process_nodes(
    discv5_mutex: Arc<TokioMutex<Discv5>>,
    discovered_nodes_mutex: Arc<TokioMutex<Vec<Enr<CombinedKey>>>>,
    semaphore: Arc<Semaphore>,
) {
    let mut index = 0;

    loop {
        let destination_node_enr = {
            let discovered_nodes_guard = discovered_nodes_mutex.lock().await;
            if index < discovered_nodes_guard.len() {
                Some(discovered_nodes_guard[index].clone())
            } else {
                None
            }
        };

        if let Some(enr) = destination_node_enr {
            let discv5_mutex_clone = Arc::clone(&discv5_mutex);
            let discovered_nodes_mutex_clone = Arc::clone(&discovered_nodes_mutex);
            let semaphore_clone = Arc::clone(&semaphore);

            task::spawn(async move {
                process_node(
                    discv5_mutex_clone,
                    discovered_nodes_mutex_clone,
                    semaphore_clone,
                    enr,
                )
                .await;
            });

            index += 1;
        } else {
            // wait a bit until there are nodes again to process
            //tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // You can add a delay here if needed
        // tokio::time::sleep(Duration::from_millis(100)).await;
    }
}