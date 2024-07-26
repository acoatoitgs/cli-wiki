extern crate colored;
extern crate indicatif;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use colored::*;
use indicatif::ProgressBar;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::env;
use std::process;
use std::time::Duration;

#[derive(Deserialize)]
struct WikipediaResponse {
    #[serde(rename = "type")]
    response_type: String,
    extract: Option<String>,
}

#[derive(Deserialize)]
struct SearchResult {
    title: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    query: Query,
}

#[derive(Deserialize)]
struct Query {
    search: Vec<SearchResult>,
}

fn get_page_data(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", name);

    let client = Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "wiki-cli/1.0")
        .send()?;

    if response.status().is_success() {
        let body = response.text()?;
        Ok(body)
    } else {
        Err(Box::from(format!("Request error: {}", response.status())))
    }
}

fn search_wikipedia(term: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json",
        term
    );

    let client = Client::new();

    //Initiate GET request
    let response = client
        .get(&url)
        .header("User-Agent", "wiki-cli/1.0")
        .send()?;

    //Handle response
    if response.status().is_success() {
        let body = response.text()?;
        let search_response: SearchResponse = serde_json::from_str(&body)?;
        if let Some(first_result) = search_response.query.search.first() {
            return Ok(first_result.title.clone());
        }
        Err(Box::from("No results found."))
    } else {
        Err(Box::from(format!("Search error: {}", response.status())))
    }
}

fn parse_response(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response: WikipediaResponse = serde_json::from_str(input)?;
    let return_value = response
        .extract
        .unwrap_or_else(|| "No extract available.".to_string());

    //Page could be ambiguous,
    //TODO: let the user disambiguate without another query
    if response.response_type == "disambiguation" {
        Ok(
            "Ambiguous, please add further information. E.g. \"<term> engineering\"."
                .yellow()
                .to_string(),
        )
    } else {
        Ok(return_value)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("Usage: wiki <args>");
        process::exit(0);
    }

    //Combine arguments
    let mut argument = args[1].clone();
    for arg in &args[2..] {
        argument.push(' ');
        argument.push_str(arg);
    }

    //Initiate progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_message("Parsing input...");
    pb.enable_steady_tick(Duration::from_millis(100));

    //Search the page which most resembles the query
    let search_result = search_wikipedia(&argument)?;

    //Update progress bar
    pb.set_message(format!("Searching \"{}\"", search_result));

    //Get the json data from wikipedia and parse it into String
    let response = get_page_data(&search_result)?;
    let parsed_response = parse_response(&response)?;

    //Print the result

    let custom_color = Color::TrueColor {
        r: 255,
        g: 165,
        b: 0,
    };
    pb.finish_with_message(parsed_response.color(custom_color).to_string());

    Ok(())
}
