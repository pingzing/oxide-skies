extern crate serde;
extern crate serde_json;
extern crate hyper;

mod weather_structs;

use hyper::client::*; //todo replae the * with just what I'm using
use std::path::Path;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind, BufWriter, BufReader, Write, Read};
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
    let loc_result = get_cached_location();
    match loc_result {
        Ok(location) => {
            show_weather(client, api_key, location);
        }
        Err(_) => {
            match get_location_with_default_ip(&client) {
                Some(location) => {
                    show_weather(client, api_key, location);
                }
                None => println!("Unable to determine current location."),
            }
        }
    }
}

fn run_with_hostname(client: &Client, api_key: &str, hostname: &str) {
    let loc_result = get_cached_location();
    match loc_result {
        Ok(location) => {
            show_weather(client, api_key, location);
        }
        Err(_) => {
            match get_location_with_ip(&client, hostname) {
                Some(location) => {
                    show_weather(client, api_key, location);
                }
                None => panic!("Unable to determine current location."),
            }
        }
    }
}

fn get_location_with_ip(client: &Client, ip_string: &str) -> Option<Location> {
    let url_string = &(GEOLOCATION_URL.to_string() + &ip_string);
    println!("Retrieving location...");
    return get_location(client, url_string);
}

// todo: refactor this to call get_location_with_ip() and just use an empty string
fn get_location_with_default_ip(client: &Client) -> Option<Location> {
    println!("Retrieving location...");
    return get_location(client, GEOLOCATION_URL);
}

fn get_location(client: &Client, hostname: &str) -> Option<Location> {
    println!("get_location(): Calling {}", hostname);
    let geo_ip_response = client.get(hostname).send();

    match geo_ip_response {
        Ok(mut good_response) => {
            let mut response_string = String::new();
            match good_response.read_to_string(&mut response_string) {
                Ok(_) => {
                    println!("{:?}", response_string);
                    return deserialize_json::<Location>(&response_string);
                }
                Err(error) => {
                    println!("{:?}", error);
                    return None;
                }                  
            }
        }
        Err(error) => {
            println!("{}", error);
            return None;
        }
    }
}

fn show_weather(client: &Client, api_key: &str, location: Location) {
    cache_location(&location);
    if let Some(weather_result) = get_weather(&client, api_key, location) {
        println!("{:?}", weather_result);
    } else {
        println!("Unable to determine current weather.");
    }
}

fn get_weather(client: &Client, api_key: &str, location: Location) -> Option<WeatherResponse> {
    let url_string = format!("{url}lat={lat_val}&lon={lon_val}&appid={api_key}",
                             url = OPEN_WEATHER_MAP_LAT_LON_URL,
                             lat_val = location.latitude.to_string(),
                             lon_val = location.longitude.to_string(),
                             api_key = api_key);

    println!("Getting weather from {}", url_string);
    let response = client.get(&url_string).send();

    match response {
        Ok(mut good_response) => {
            let mut response_string = String::new();
            match good_response.read_to_string(&mut response_string) {
                Ok(_) => {
                    println!("{:?}", response_string);
                    return deserialize_json::<WeatherResponse>(&response_string);
                }
                Err(error) => {
                    println!("{:?}", error);
                    return None;
                }
            }
        }
        Err(error) => {
            println!("{}", error);
            return None;
        }
    }
}

fn get_cached_location() -> Result<Location, Error> {
    let mut options = OpenOptions::new();
    options.read(true);
    let in_path = Path::new(CACHED_LOCATION_FILE);
    let in_file = try!(options.open(in_path));

    let mut reader = BufReader::new(&in_file);
    let mut loc_string = String::new();
    try!(reader.read_to_string(&mut loc_string));

    match deserialize_json::<Location>(&loc_string) {
        Some(location) => return Ok(location),
        None => {
            println!("Failed to deserialize cached location.");
            return Err(std::io::Error::new(ErrorKind::InvalidData, "Deserialization failed"));
        }
    }
}

fn cache_location(location: &Location) -> Result<(), Error> {
    let mut options = OpenOptions::new();
    options.write(true);
    let out_path = Path::new(CACHED_LOCATION_FILE);
    let out_file = try!(options.open(out_path));

    let location_string = try!(serialize_to_json(location));

    let mut writer = BufWriter::new(&out_file);
    return writer.write_all(location_string.as_bytes());
}

fn get_owm_api_key() -> Result<String, Error> {
    let mut options = OpenOptions::new();
    options.read(true);
    let in_path = Path::new(OWM_API_KEY_FILENAME);
    let in_file = try!(options.open(in_path));

    let mut reader = BufReader::new(&in_file);
    let mut api_string = String::new();
    try!(reader.read_to_string(&mut api_string));
    return Ok(api_string);
}

fn serialize_to_json<T: serde::Serialize>(obj: T) -> Result<String, Error> {
    match serde_json::to_string(&obj) {
        Ok(string) => Ok(string),
        Err(_) => Err(Error::new(ErrorKind::InvalidInput, "Serialization failed")),
    }
}

fn deserialize_json<T: serde::Deserialize>(json_string: &str) -> Option<T> {
    let decode_result = serde_json::from_str(json_string);
    match decode_result {
        Ok(loc) => Some(loc),
        Err(err) => {
            println!("Failed to decode JSON. Error: {}", err);
            return None;
        }
    }
}
