// use enr::{EnrBuilder, k256::ecdsa::SigningKey, Enr, CombinedKey};
use std::net::Ipv4Addr;
use rand::thread_rng;
use rand::Rng;
use std::str::FromStr;
use discv5::{enr, enr::{NodeId, CombinedKey, Enr}, TokioExecutor, Discv5, Discv5ConfigBuilder};

pub fn enr_provider()-> Enr<CombinedKey> {
    // let base_64_string = "enr:-Ly4QFPk-cTMxZ3jWTafiNblEZkQIXGF2aVzCIGW0uHp6KaEAvBMoctE8S7YU0qZtuS7By0AA4YMfKoN9ls_GJRccVpFh2F0dG5ldHOI__________-EZXRoMpCC9KcrAgAQIIS2AQAAAAAAgmlkgnY0gmlwhKh3joWJc2VjcDI1NmsxoQKrxz8M1IHwJqRIpDqdVW_U1PeixMW5SfnBD-8idYIQrIhzeW5jbmV0cw-DdGNwgiMog3VkcIIjKA";
    let enr_1:Enr<CombinedKey> = discv5::enr::Enr::from_str("enr:-Ly4QFPk-cTMxZ3jWTafiNblEZkQIXGF2aVzCIGW0uHp6KaEAvBMoctE8S7YU0qZtuS7By0AA4YMfKoN9ls_GJRccVpFh2F0dG5ldHOI__________-EZXRoMpCC9KcrAgAQIIS2AQAAAAAAgmlkgnY0gmlwhKh3joWJc2VjcDI1NmsxoQKrxz8M1IHwJqRIpDqdVW_U1PeixMW5SfnBD-8idYIQrIhzeW5jbmV0cw-DdGNwgiMog3VkcIIjKA").unwrap();
    return enr_1;
}