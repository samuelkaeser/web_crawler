use rand::{Rng};
use std::{error::Error, io, str::FromStr, fs, fs::{File, OpenOptions}, io::{Read, Write}, path::Path, collections::HashSet, net::{SocketAddr, Ipv4Addr}, mem};
use discv5::{enr, enr::{CombinedKey, Enr, NodeId, EnrBuilder}, Discv5, Discv5ConfigBuilder};
use std::sync::{Arc};
use serde::{Serialize, Deserialize};
use tokio::time::{timeout, Duration, Instant, sleep};
use parking_lot::RwLock;
use discv5::handler::{HandlerIn, HandlerOut, NodeContact, Handler};
use discv5::{packet::DefaultProtocolId,
    rpc::{Request, Response, RequestBody, ResponseBody, RequestId, ResponseBody::Nodes},
};
use std::collections::VecDeque;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomNodes {
    pub discovered_nodes: HashSet<Enr<CombinedKey>>,
    pub processed_nodes: HashSet<Enr<CombinedKey>>,
    pub nodes_to_be_processed: VecDeque<Enr<CombinedKey>>,
    pub listening_ports: Vec<u16>,
}

pub fn count_files_in_directory(dir: &str) -> std::io::Result<usize> {
    let entries = fs::read_dir(dir)?;
    let count = entries
        .filter_map(Result::ok)  // Convert iterator of Result to iterator of DirEntry
        .filter(|e| e.path().is_file())  // Filter out directories
        .count();
    Ok(count)
}

pub fn append_node_to_file(file_path: &str, node: &Enr<CombinedKey>) -> io::Result<()> {
    let mut file = OpenOptions::new()
    .append(true)
    .create(true)
    .open(file_path)?;
    writeln!(file, "{}", serde_json::to_string(&node.to_base64())?)
}

macro_rules! arc_rw {
    ( $x: expr ) => {
        Arc::new(RwLock::new($x))
    };
}

pub fn find_node_custom(receiver_enr: discv5::enr::Enr<CombinedKey>, distance_vector:Vec<u64>, listening_address:SocketAddr) -> HashSet<Enr<CombinedKey>> {

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {

        const MAX_RETRIES: u32 = 10;
        let mut retries = 0;
        let mut _exit_send;
        let sender_handler;
        let mut sender_handler_recv;
        let ip: Ipv4Addr= "0.0.0.0".parse().unwrap();

        loop {
            let ip_1 = "0.0.0.0".parse().unwrap();
            let key = CombinedKey::generate_secp256k1();
            let config = Discv5ConfigBuilder::new().build();
            let sender_enr = EnrBuilder::new("v4")
                .ip4(ip_1)
                .udp4(listening_address.port())
                .build(&key)
                .unwrap();
        
            match Handler::spawn::<DefaultProtocolId>(
                arc_rw!(sender_enr.clone()),
                arc_rw!(key),
                sender_enr.udp4_socket().unwrap().into(),
                config.clone(),
            )
            .await
            {
                Ok((exit_send, handler, handler_recv)) => {
                    //println!("Started listening successfully on address {}", listening_address);
                    _exit_send = exit_send;
                    sender_handler = handler;
                    sender_handler_recv = handler_recv;
                    break;
                }
                Err(e) => {
                    println!("An error occurred: {}, retrying address {}", e, listening_address);
                    retries += 1;
                    if retries > MAX_RETRIES {
                        println!("Exceeded maximum number of retries, returning empty routing table for node: {}", receiver_enr);
                        let nodes_collector: HashSet<Enr<CombinedKey>> = HashSet::new();
                        return nodes_collector;
                    }
                    sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        let send_message = Box::new(Request {
            id: RequestId(vec![1]),
            body: RequestBody::Ping { enr_seq: 1 },
        });
        let find_node_message = Box::new(Request {
            id: RequestId(vec![2]),
            body: RequestBody::FindNode { distances: distance_vector },
        });

        // sender to send the first message then await for the session to be established
        //let clock_start = Instant::now();
        let receiver_node_contact = match NodeContact::try_from_enr(receiver_enr.clone(), discv5::IpMode::Ip4) {
            Ok(node) => node,
            Err(_) => {
                let nodes_collector: HashSet<Enr<CombinedKey>> = HashSet::new();
                return nodes_collector;
            }
        };
        
        let _ = sender_handler.send(HandlerIn::Request(
            receiver_node_contact.clone(),
            send_message.clone(),
        ));

        let pong_response = Response {
            id: RequestId(vec![1]),
            body: ResponseBody::Pong {
                enr_seq: 1,
                ip: ip.into(),
                port: listening_address.port(),
            },
        };
        let mut nodes_collector: HashSet<Enr<CombinedKey>> = HashSet::new();

        loop {
            match timeout(Duration::from_millis(2000) , sender_handler_recv.recv()).await {
                Ok(Some(message)) => {
                    //println!("Received message at time: {:?} , reset last_message_received", clock_start.elapsed());
                    match message {
                        HandlerOut::Established(_, _, _) => {
                            //println!("Established connection: {:?} after {:?} time", message, clock_start.elapsed());
                            //now the session is established, send the rest of the messages
                            let _ = sender_handler.send(HandlerIn::Request(
                                    receiver_node_contact.clone(),
                                    find_node_message.clone(),
                                ));
                                //println!("Sent message {}", find_node_message);
                            }
                        HandlerOut::WhoAreYou(wru_ref) => {
                            //println!("Sender received whoareyou packet with ref:{:?}", wru_ref);
                            //println!("Sender sent whoareyou packet with ref:{:?}", wru_ref);

                            let _ = sender_handler.send(HandlerIn::WhoAreYou(wru_ref, Some(receiver_enr.clone())));
                        }
                        HandlerOut::Request(addr, request) => {
                            //println!("Sender received request:{}", request);
                            // required to send a pong response to establish the session
                            let _ = sender_handler.send(HandlerIn::Response(addr.clone(), Box::new(pong_response.clone())));
                            println!("Sender sent response:{:?}", HandlerIn::Response(addr.clone(), Box::new(pong_response.clone())));
                        }
                        HandlerOut::Response(_, box_response) => {
                            if let Response { id: _, body: Nodes { total: _, nodes } } = *box_response {
                                // Now you can work with the `nodes` vector.
                                //println!("Nodes: {:?}", nodes);
                                nodes_collector.extend(nodes);
                            } else {
                                //println!("The body of the response is not of type Nodes: {}", box_response);
                            }
                        }
                        HandlerOut::RequestFailed(_, request_error) => {
                            println!("Request failed: {}", request_error);
                        }
                    }
                }
                Ok(None) => {
                    // The channel has been closed.
                    println!("The channel has been closed");
                },
                Err(_) => {
                    // Timeout expired without receiving a message.
                    //println!("Last message was received more than a second ago, breaking out from receiving loop");
                    return nodes_collector;
                    //break;
                }
            }
        }
    })
}

pub fn get_routing_table_custom(receiver_enr: discv5::enr::Enr<CombinedKey>, listening_address:SocketAddr) -> Vec<Enr<CombinedKey>>{
    let mut nodes_collector_1: HashSet<Enr<CombinedKey>> = HashSet::new();
        for i in 240..=256{
        let distance_vector:Vec<u64> = vec![i];
        let new_nodes: HashSet<Enr<CombinedKey>> = find_node_custom(receiver_enr.clone(), distance_vector, listening_address);
        nodes_collector_1.extend(new_nodes);
    }
    let routing_table: Vec<Enr<CombinedKey>> = nodes_collector_1.into_iter().collect();
    return routing_table;
}

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