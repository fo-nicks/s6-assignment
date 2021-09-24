
use std::{collections::HashMap, sync::Mutex};
use serde::Deserialize;
use serde::Serialize;
use regex::Regex;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};

struct UrlData {
    current_url_code: u32,
    url_map: HashMap<String, String>
}
struct AppState {
    url_data: Mutex<UrlData>
}


#[derive(Serialize, Deserialize)]
struct ShortenedResponse {
    short_url_code: String
}
// TODO: need to normalize the input
#[derive(Deserialize, Clone)]
struct ShortenedRequest {
    url: String
}


#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/")]
async fn shorten_post(req: web::Json<ShortenedRequest>, data: web::Data<AppState>) -> impl Responder {
    // let mut url_map = data.url_data.lock().unwrap();
    // list.push(req_body);
    //HttpResponse::Ok().body(format!("test", url_map))
    let raw_url = String::from(req.url.clone());
    let shortened_url = shorten_url(raw_url, data);
    
    HttpResponse::Ok().body("test")
}

#[get("/")]
async fn shorten_get(req: web::Json<ShortenedRequest>, data: web::Data<AppState>) -> impl Responder {
    let normalized = normalize_url(req.url.clone());
    let url_map = &data.url_data.lock().unwrap().url_map;
    if url_map.contains_key(&normalized) {
        let url = url_map.get(&normalized).unwrap().to_string();
        return HttpResponse::MovedPermanently().json(ShortenedResponse { short_url_code: url})
    } else {
        return  HttpResponse::NotFound().body("Shortened url not found.");
    }
}

#[test]
fn normalize_url_test() {
    let result = normalize_url(String::from("http://www.singularity6.com")); 
    assert_eq!(result, "singularity6.com");
}

fn normalize_url(url: String) -> String {
    let re = Regex::new(r"^https*:[/]+[w]*[.]*([a-zA-Z0-9_-]+\.+[a-zA-Z0-9_-]+)").unwrap();
    let normalized = String::from(re.captures(&url)
        .unwrap().get(1)
        .unwrap().to_owned().as_str());
    normalized
}

#[test]
fn shorten_url_test() {
    let state = web::Data::new(AppState {
        url_data: Mutex::new(
            UrlData {
                url_map: HashMap::new(),
                current_url_code: 0
            }
        )
    });
    let result = shorten_url(
        String::from("http://www.singularity6.com"), 
        state
    );
    assert_eq!(result, "http://localhost:8080/1");
    
}

fn shorten_url(url: String, data: web::Data<AppState> ) -> String {
    let mut url_data = data.url_data.lock().unwrap();
    let normalized = normalize_url(url);
    if url_data.url_map.contains_key(&normalized) {
        return url_data.url_map.get(&normalized).unwrap().to_string();
    }
    else {
        url_data.current_url_code += 1;
        let shortened = "http://localhost:8080/".to_owned() + &url_data.current_url_code.to_string();
        url_data.url_map.insert(normalized.clone(), shortened.clone());
        return shortened;
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        url_data: Mutex::new(
            UrlData {
                url_map: HashMap::new(),
                current_url_code: 0
            }
        )
    });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(index)
            .service(shorten_post)
            .service(shorten_get)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
