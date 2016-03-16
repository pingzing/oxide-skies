pub mod location {

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Location {
        pub ip: String,
        pub country_code: String,
        pub country_name: String,
        pub region_code: String,
        pub region_name: String,
        pub city: String,
        pub zip_code: String,
        pub time_zone: String,
        pub latitude: f64,
        pub longitude: f64,
        pub metro_code: u32,
    }
}

pub mod weather {

    #[derive (Serialize, Deserialize, Debug)]
    pub struct WeatherResponse {
        coord: Option<LatLong>,
        weather: Option<Vec<WeatherDetails>>,
        base: Option<String>,
        main: Option<WeatherBasics>,
        wind: Option<Wind>,
        clouds: Option<Clouds>,
        dt: Option<u64>,
        sys: Option<OWMSystem>,
        id: Option<u64>,
        name: Option<String>,
        cod: Option<u32>,
    }

    #[derive (Serialize, Deserialize, Debug)]
    pub struct LatLong {
        lon: f64,
        lat: f64,
    }

    #[derive (Serialize, Deserialize, Debug)]
    pub struct WeatherDetails {
        id: u64,
        main: String,
        description: String,
        icon: String,
    }

    #[derive (Serialize, Deserialize, Debug)]
    pub struct WeatherBasics {
        temp: Option<f64>,
        pressure: Option<f64>,
        humidity: Option<f64>,
        temp_min: Option<f64>,
        temp_max: Option<f64>,
        sea_level: Option<f64>,
        grnd_level: Option<f64>,
    }

    #[derive (Serialize, Deserialize, Debug)]
    pub struct Wind {
        speed: Option<f64>,
        deg: Option<f64>,
    }

    #[derive (Serialize, Deserialize, Debug)]
    pub struct Clouds {
        all: Option<u64>,
    }

    #[derive (Serialize, Deserialize, Debug)]
    pub struct Rain {
        #[serde(rename="3h")]
        last_3_hours: Option<f64>,
    }

    #[derive (Serialize, Deserialize, Debug)]
    pub struct Snow {
        #[serde(rename="3h")]
        last_3_hours: Option<f64>,
    }

    #[derive (Serialize, Deserialize, Debug)]
    pub struct OWMSystem {
        #[serde(rename="3h")]
        system_type: Option<u32>,
        id: Option<u32>,
        message: Option<f64>,
        country: Option<String>,
        sunrise: Option<u64>,
        sunset: Option<u64>,
    }
}