extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate filetime;
extern crate time;

mod weather_structs;
mod error_extensions;

use hyper::client::*; //todo replae the * with just what I'm using
use std::path::Path;
use std::fs;
use filetime::FileTime;
use std::io;
use std::io::{Read, Write};
use std::error::Error;
use std::env;
use weather_structs::location::Location;
use weather_structs::weather::WeatherResponse;

const GEOLOCATION_URL: &'static str = "https://freegeoip.net/json/";
const OPEN_WEATHER_MAP_LAT_LON_URL: &'static str = "http://api.openweathermap.org/data/2.\
                                                    5/weather?";
const OWM_API_KEY_FILENAME: &'static str = "owm_api_key.txt";
const CACHED_LOCATION_FILE: &'static str = "cached_location.txt";

fn main() {
    if let Ok(api_key) = get_owm_api_key() {
        let args: Vec<String> = env::args().collect();
        let client = Client::new();
        match args.len() {
            0 => panic!("How did you even do that?"),
            1 => run_with_defaults(&client, &api_key),
            2 => run_with_hostname(&client, &api_key, &args[1]),
            _ => panic!("Too many args, scrub."),
        }
    } else {
        panic!("Failed to get OpenWeatherMap API key from owm_api_key.txt. Does the file exist?");
    }
}

// todo: refactor this to call run_with_hostname
fn run_with_defaults(client: &Client, api_key: &str) {
    run_with_hostname(&client, &api_key, "");
}

fn run_with_hostname(client: &Client, api_key: &str, hostname: &str) {
    let loc_result = get_cached_location();
    match loc_result {
        Ok(location) => show_weather(client, api_key, location),
        Err(e) => {
            println!("Not using cached location from file {}, {}",CACHED_LOCATION_FILE, e.description());
            match get_location_with_ip(&client, hostname) {
                Ok(location) => show_weather(client, api_key, location),                
                Err(_) => panic!("Unable to determine current location."),
            }
        }
    }
}

fn get_location_with_ip(client: &Client, ip_string: &str) -> Result<Location, error_extensions::ErrorExt> {
    let url_string = &(GEOLOCATION_URL.to_string() + &ip_string);    
    return get_location(client, url_string);
}

fn get_location(client: &Client, hostname: &str) -> Result<Location, error_extensions::ErrorExt> {
    println!("Retrieving location from {}", hostname);
    let mut geo_ip_response = try!(client.get(hostname).send());    
    let mut response_string = String::new();
    try!(geo_ip_response.read_to_string(&mut response_string)); 
    println!("{:?}", response_string);
    let deser_loc = try!(deserialize_json::<Location>(&response_string));
    try!(cache_location(&deser_loc));
    return Ok(deser_loc);                           
}


fn show_weather(client: &Client, api_key: &str, location: Location) {    
    if let Ok(weather_result) = get_weather(&client, api_key, location) {
        println!("{:?}", weather_result);
    } else {
        println!("Unable to determine current weather.");
    }
}

fn get_weather(client: &Client, api_key: &str, location: Location) -> Result<WeatherResponse, error_extensions::ErrorExt> {
    let url_string = format!("{url}lat={lat_val}&lon={lon_val}&appid={api_key}",
                             url = OPEN_WEATHER_MAP_LAT_LON_URL,
                             lat_val = location.latitude.to_string(),
                             lon_val = location.longitude.to_string(),
                             api_key = api_key);

    println!("Getting weather from {}", url_string);
    let mut response = try!(client.get(&url_string).send());
    let mut response_string = String::new();
    try!(response.read_to_string(&mut response_string)); 
    println!("{:?}", response_string);
    return deserialize_json::<WeatherResponse>(&response_string);
}

fn get_cached_location() -> Result<Location, error_extensions::ErrorExt> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    let in_path = Path::new(CACHED_LOCATION_FILE);
    let in_file = try!(options.open(in_path));

    let mut reader = io::BufReader::new(&in_file);
    let mut loc_string = String::new();
    try!(reader.read_to_string(&mut loc_string));
    
    let now = time::now().to_timespec(); //aka second since jan 1 1970
    let create_time = FileTime::from_last_modification_time(
                                &fs::metadata(in_path).unwrap())
                                .seconds_relative_to_1970();
    if now.sec - create_time as i64 > 86400 { //seconds in a day
        return Err(error_extensions::ErrorExt::DataTooOld);
    }     

    match deserialize_json::<Location>(&loc_string) {
        Ok(location) => return Ok(location),
        Err(e) => {
            println!("Failed to deserialize cached location.");            
            return Err(e);
        }
    }
}

fn cache_location(location: &Location) -> Result<(), error_extensions::ErrorExt> {    
    let out_file = try!(fs::OpenOptions::new().write(true).create(true).open(Path::new(CACHED_LOCATION_FILE)));

    let location_string = try!(serialize_to_json(location));

    let mut writer = io::BufWriter::new(&out_file);        
    match writer.write_all(location_string.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(error_extensions::ErrorExt::IoError(e))
    }
}

fn get_owm_api_key() -> Result<String, error_extensions::ErrorExt> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    let in_path = Path::new(OWM_API_KEY_FILENAME);
    let in_file = try!(options.open(in_path));

    let mut reader = io::BufReader::new(&in_file);
    let mut api_string = String::new();
    try!(reader.read_to_string(&mut api_string));
    return Ok(api_string);
}

fn serialize_to_json<T: serde::Serialize>(obj: T) -> Result<String, error_extensions::ErrorExt> {
    match serde_json::to_string(&obj) {
        Ok(string) => Ok(string),
        Err(e) => Err(error_extensions::ErrorExt::SerdeError(e)),
    }
}

fn deserialize_json<T: serde::Deserialize>(json_string: &str) -> Result<T, error_extensions::ErrorExt> {
    let decode_result = serde_json::from_str(json_string);
    match decode_result {
        Ok(loc) => Ok(loc),
        Err(err) => {
            println!("Failed to decode JSON. Error: {}", err);
            return Err(error_extensions::ErrorExt::SerdeError(err));
        }
    }
}