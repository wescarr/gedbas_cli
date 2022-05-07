use reqwest::blocking::Client;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use rayon::prelude::*;

#[derive(Serialize, Debug)]
struct PersonRow {
    person_id: String,
    name: String,
    surname: String,
}

fn parse_row(row_element: ElementRef) -> PersonRow {
    let name_selector = &Selector::parse("td:nth-child(1) > a").unwrap();
    let name_element = row_element.select(name_selector).next().unwrap();
    let name = name_element.inner_html();
    let person_id = name_element.value().attr("href").unwrap()[13..].to_string();

    let surname_selector = &Selector::parse("td:nth-child(2) > a").unwrap();
    let surname = row_element
        .select(surname_selector)
        .next()
        .unwrap()
        .inner_html();

    PersonRow {
        person_id: person_id,
        name: name,
        surname: surname,
    }
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

fn get_results(client: &Client, lastname: &str, limit: usize) -> Vec<PersonRow> {
    let request_url = format!("https://gedbas.genealogy.net/search/simple?firstname=&timelimit=none&lastname={}&placename=&offset=0&max={}", lastname,limit);
    let result = client.get(request_url).send().unwrap().text().unwrap();

    eprintln!("Fetched results");

    let html = Html::parse_document(&result);
    let selector = &Selector::parse("#bodyContent > div.body > div.list > table > tbody > tr")
        .expect("entry selector parsing failure");

    let rows: Vec<PersonRow> = html.select(selector).map(|e| parse_row(e)).collect();

    eprintln!("Parsed {} rows.", rows.len());

    rows
}

#[derive(Serialize, Debug)]
struct Person {
    person_id: String,
    name: String,
    surname: String,
    events: Vec<Event>,
}

#[derive(Serialize, Debug)]
struct Event {
    event_type: String,
    timestamp: String,
    location: Option<String>,
    source: Option<String>,
}

fn parse_event(event_element: ElementRef) -> Option<Event> {
    let type_selector = &Selector::parse("td:nth-child(1)").ok()?;
    let event_type = event_element
        .select(type_selector)
        .next()?
        .inner_html()
        .trim()
        .to_string();

        let timestamp_selector = &Selector::parse("td:nth-child(2)").ok()?;
    let timestamp = event_element
        .select(timestamp_selector)
        .next()?
        .text()
        .nth(0)?
        .trim()
        .to_string();

    let location_selector = &Selector::parse("td:nth-child(3)").ok();
    let location = match location_selector {
        Some(selector) => event_element.select(selector).next()
        .and_then(|e|e.text().next())
        .map(|t|t.trim().to_string()),
        None => None,
    };

    let source_selector = &Selector::parse("td:nth-child(3)").ok();
    let source = match source_selector {
        Some(selector) => event_element.select(selector).next()
        .and_then(|e|e.text().next())
        .map(|t|t.trim().to_string()),
        None => None,
    };

    Some(Event {
        event_type: event_type,
        timestamp: timestamp,
        location: location,
        source: source,
    })
}

fn get_person(client: &Client, person_row: &PersonRow) -> Option<Person> {
    let request_url = format!(
        "https://gedbas.genealogy.net/person/show/{}",
        person_row.person_id
    );
    let result = &client.get(request_url).send().ok()?.text().ok()?;

    eprintln!("Fetched person with id {}", person_row.person_id);

    let html = Html::parse_document(result);
    let event_selector = &Selector::parse("#events > tbody > tr").ok()?;

    let events: Vec<Event> = html
        .select(event_selector)
        .filter_map(|e| parse_event(e))
        .collect();

    eprintln!("parsed {} events for person {}", events.len(), person_row.person_id);

    Some(Person {
        person_id: person_row.person_id.to_string(),
        name: person_row.name.clone(),
        surname: person_row.surname.clone(),
        events: events,
    })
}
fn main() {
    let client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();

    let rows = get_results(&client, "Müller", 1000);
    
    let persons: Vec<Person> = rows
        .par_iter()
        .filter_map(|r|get_person(&client, r))
        .collect();


    let serialized = serde_json::to_string_pretty(&persons).unwrap();
    println!("{}", serialized);
}
