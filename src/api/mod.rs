use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use validator::Validate;

use crate::AppState;
// https://places.googleapis.com/v1/places/ChIJj61dQgK6j4AR4GeTYWZsKWw?fields=id,displayName&key=KEY

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
const MAX_RESULT_COUNT_KEY: &str = "maxResultCount";
const MAX_RESULT_COUNT_VALUE: &str = "10";

#[derive(Debug, Deserialize, Serialize)]
struct DisplayName {
    text: String,
    #[serde(rename = "languageCode")]
    language_code: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Location {
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

    let key = match env::var("GOOGLE_PLACES_KEY") {
        Ok(key) => key,
        Err(e) => {
            println!("No GOOGLE_PLACES_KEY found. {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong. Try again later",
            )
                .into_response();
        }
    };

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
        .header(GOOGLE_API_KEY_HEADER, key)
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

pub async fn get_route() -> impl IntoResponse {}
