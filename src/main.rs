extern crate confluence;
extern crate confluence_publisher;
extern crate reqwest;
extern crate saml;
extern crate vip;

use {
    clap::{clap_app, ArgMatches},
    confluence::client::Confluence,
    confluence_publisher::{publisher::Publisher, reader::read_metadata_yml},
    lazy_static::lazy_static,
    openssl::rsa::{Padding, Rsa},
    regex::Regex,
    saml::IdP,
};

lazy_static! {
    pub static ref HOST_RE: Regex = Regex::new(r#"\w*.{3}[\w\-\.]*"#).unwrap();
}

fn split_once_to_str(in_string: String) -> (String, String) {
    let mut splitter = in_string.lines().next().unwrap().splitn(2, ':');
    let username = String::from_utf8(splitter.next().unwrap().as_bytes().to_vec()).unwrap();
    let password = String::from_utf8(splitter.next().unwrap().as_bytes().to_vec()).unwrap();
    (username, password)
}

fn get_args<'a>(matches: &'a ArgMatches, all_args: &Vec<&'a str>) -> Vec<Vec<&'a str>> {
    let mut args = Vec::new();
    for a in all_args.iter() {
        if matches.is_present(*a) {
            match matches.value_of(*a) {
                Some(val) => {
                    let mut vals = Vec::new();
                    vals.push(*a);
                    vals.push(val);
                    args.push(vals);
                }
                None => {
                    let mut vals = Vec::new();
                    vals.push(*a);
                    args.push(vals);
                }
            }
        }
    }
    args
}

pub fn main() {
    let resolve = |prop: &str, matches: &ArgMatches| -> Option<String> {
        let arg = vec![prop];
        let v = get_args(matches, &arg);
        if !v.is_empty() && v[0].len() == 2 {
            return Some(v[0][1].to_string());
        } else {
            return None;
        }
    };

    let matches = clap_app!(
        confpub =>
            (version: "0.0.1")
            (author: "Sebastian Kaiser")
            (about: "Call a saml+vip protected Confluence rest api.")
            (@arg USER: -u --user +takes_value "Sets username:password")
            (@arg KEY: -k --key requires[FILE] +takes_value "Sets private key file.")
            (@arg FILE: -f --file requires[KEY] +takes_value "Sets encrypted file.")
            (@arg ENDPOINT: -e --endpoint +required +takes_value "Sets protected REST endpoints")
            (@arg DATA: +required "Sets path to metadata file")
    )
    .get_matches();

    let user = resolve("USER", &matches);
    let key = resolve("KEY", &matches);
    let file = resolve("FILE", &matches);
    let endpoint = resolve("ENDPOINT", &matches);
    let data = resolve("DATA", &matches);

    match (user, key, file, endpoint, data) {
        (Some(u), None, None, Some(e), Some(d)) => {
            let (username, password) = split_once_to_str(u);
            run(&username, &password, &e, &d);
        }
        (None, Some(k), Some(f), Some(e), Some(d)) => {
            let encrypted_file = std::fs::read(&f).unwrap();
            let key_data = std::fs::read(&k).unwrap();
            let rsa = Rsa::private_key_from_pem(&key_data).unwrap();
            let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
            let _ = rsa
                .private_decrypt(&encrypted_file, &mut buf, Padding::PKCS1)
                .unwrap();
            let (username, password) = split_once_to_str(String::from_utf8(buf).unwrap());
            run(&username, &password, &e, &d);
        }
        (_, _, _, _, _) => {}
    };
}

fn run(username: &str, password: &str, endpoint: &str, data: &str) {
    let host = HOST_RE.captures(&endpoint).unwrap()[0].to_string();
    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let rc_client = std::rc::Rc::new(client);
    match IdP::with_client(&rc_client).authenticate(&host, username, password) {
        Ok(()) => {
            let publisher = Publisher::new(Confluence::with_client(rc_client, endpoint));
            match read_metadata_yml(&publisher, data) {
                Ok(_) => println!("Done"),
                Err(e) => println!("Error {:?}", e),
            }
        }
        Err(e) => println!("Error {:?}", e),
    }
}
