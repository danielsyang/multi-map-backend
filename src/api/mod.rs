use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use validator::Validate;

use crate::AppState;

// curl -X POST -d '{
//     "textQuery" : "Spicy Vegetarian Food in Sydney, Australia",
//     "maxResultCount": "10"
//   }' \
//   -H 'Content-Type: application/json' -H 'X-Goog-Api-Key: KEY' \
//   -H 'X-Goog-FieldMask: places.id,places.displayName,places.formattedAddress,places.location' \
//   'https://places.googleapis.com/v1/places:searchText'

const CONTENT_TYPE: &str = "Content-type";
const JSON_TYPE: &str = "application/json";
const GOOGLE_FIELD_MASK_HEADER: &str = "X-Goog-FieldMask";
const FIELD_MASK: &str = "places.id,places.displayName,places.formattedAddress,places.location";
const GOOGLE_API_KEY_HEADER: &str = "X-Goog-Api-Key";
const GOOGLE_URL: &str = "https://places.googleapis.com/v1/places:searchText";
const GOOGLE_ROUTES_URL: &str = "https://routes.googleapis.com/directions/v2:computeRoutes";
const MAX_RESULT_COUNT_KEY: &str = "maxResultCount";
const MAX_RESULT_COUNT_VALUE: &str = "10";
const ROUTE_FIELD_MASK: &str =
    "routes.duration,routes.distanceMeters,routes.polyline.encodedPolyline";

#[derive(Debug, Deserialize, Serialize)]
struct DisplayName {
    text: String,
    #[serde(rename = "languageCode")]
    language_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Location {
    latitude: f32,
    longitude: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct GooglePlace {
    id: String,
    #[serde(rename = "formattedAddress")]
    formatted_address: String,
    #[serde(rename = "priceLevel")]
    price_level: Option<String>,
    #[serde(rename = "displayName")]
    display_name: DisplayName,
    location: Location,
}

#[derive(Debug, Deserialize, Serialize)]
struct GooglePlacesReponse {
    places: Option<Vec<GooglePlace>>,
}

#[derive(Deserialize, Validate)]
pub struct GooglePlacesRequest {
    #[validate(does_not_contain = "undefined")]
    text_query: String,
}

pub async fn get_places(
    State(s): State<AppState>,
    params: Query<GooglePlacesRequest>,
) -> impl IntoResponse {
    let p = params.0;

    if p.validate().is_err() {
        return (StatusCode::BAD_REQUEST, "Invalid request").into_response();
    }

    let mut map = HashMap::new();
    map.insert("textQuery", p.text_query);
    map.insert(MAX_RESULT_COUNT_KEY, MAX_RESULT_COUNT_VALUE.into());

    // We should add locationBias https://developers.google.com/maps/documentation/places/web-service/text-search#location-bias
    let request = s
        .client_reqwest
        .post(GOOGLE_URL)
        .json(&map)
        .header(GOOGLE_FIELD_MASK_HEADER, FIELD_MASK)
        .header(CONTENT_TYPE, JSON_TYPE)
        .header(GOOGLE_API_KEY_HEADER, s.google_key)
        .send()
        .await;

    match request {
        Ok(google_req) => match google_req.json::<GooglePlacesReponse>().await {
            Ok(google_places) => Json(google_places).into_response(),
            Err(e) => {
                println!("Error parsing response from Google Places API: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong. Try again later",
                )
                    .into_response()
            }
        },
        Err(e) => {
            println!("Error sending request to Google Places API: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong. Try again later",
            )
                .into_response()
        }
    }
}

// curl -X POST -d '{
//     "origin":{
//       "location":{
//         "latLng":{
//           "latitude": 37.419734,
//           "longitude": -122.0827784
//         }
//       }
//     },
//     "destination":{
//       "location":{
//         "latLng":{
//           "latitude": 37.417670,
//           "longitude": -122.079595
//         }
//       }
//     },
//     "travelMode": "DRIVE",
//     "routingPreference": "TRAFFIC_AWARE_OPTIMAL",
//     "departureTime": "2023-10-15T15:01:23.045123456Z",
//     "computeAlternativeRoutes": false,
//     "routeModifiers": {
//       "avoidTolls": false,
//       "avoidHighways": false,
//       "avoidFerries": false
//     },
//     "languageCode": "en-US",
//     "units": "IMPERIAL"
//   }' \
//   -H 'Content-Type: application/json' -H 'X-Goog-Api-Key: YOUR_API_KEY' \
//   -H 'X-Goog-FieldMask: routes.duration,routes.distanceMeters,routes.polyline.encodedPolyline' \
//   'https://routes.googleapis.com/directions/v2:computeRoutes'

#[derive(Debug, Deserialize)]
pub struct GetRouteRequestBody {
    #[serde(rename = "originLocation")]
    origin_location: Location,
    #[serde(rename = "destinationLocation")]
    destination_location: Location,
    #[serde(rename = "departureTime")]
    departure_time: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Polyline {
    #[serde(rename = "encodedPolyline")]
    encoded_polyline: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoutesResponse {
    #[serde(rename = "distanceMeters")]
    distance_meters: f32,
    duration: String,
    polyline: Polyline,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetRoutesReponse {
    routes: Vec<RoutesResponse>,
}

pub async fn get_routes(
    State(s): State<AppState>,
    Json(body): Json<GetRouteRequestBody>,
) -> impl IntoResponse {
    println!("body: {:?}", body);
    let req = json!({
        "origin":{
            "location":{
                "latLng":{
                "latitude": body.origin_location.latitude,
                "longitude": body.origin_location.longitude
                }
            }
        },
        "destination":{
            "location":{
                "latLng":{
                "latitude": body.destination_location.latitude,
                "longitude": body.destination_location.longitude
                }
            }
        },
        "departureTime": body.departure_time,
        "travelMode": "DRIVE",
        "routingPreference": "TRAFFIC_AWARE_OPTIMAL",
        "computeAlternativeRoutes": true,
        "routeModifiers": {
          "avoidTolls": false,
          "avoidHighways": false,
          "avoidFerries": false
        },
        "languageCode": "en-US",
        "units": "METRIC"
    });

    let request = s
        .client_reqwest
        .post(GOOGLE_ROUTES_URL)
        .json(&req)
        .header(GOOGLE_FIELD_MASK_HEADER, ROUTE_FIELD_MASK)
        .header(CONTENT_TYPE, JSON_TYPE)
        .header(GOOGLE_API_KEY_HEADER, s.google_key)
        .send()
        .await;

    match request {
        Ok(google_req) => match google_req.json::<GetRoutesReponse>().await {
            Ok(google_places) => Json(google_places).into_response(),
            Err(e) => {
                println!("Error parsing response from Google Routes API: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong. Try again later",
                )
                    .into_response()
            }
        },
        Err(e) => {
            println!("Error sending request to Google Routes API: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong. Try again later",
            )
                .into_response()
        }
    }
}
