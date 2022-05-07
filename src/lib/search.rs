use crate::lib::models::{Event, Person, PersonRow};
use rayon::prelude::*;
use reqwest::blocking::Client;
use scraper::{ElementRef, Html, Selector};
use lazy_static::lazy_static;
use tracing::{instrument, Level};
use tracing_unwrap::{ResultExt, OptionExt};

use super::models::Source;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

lazy_static! {
    static ref ROW_SELECTOR: Selector = Selector::parse("#bodyContent > div.body > div.list > table > tbody > tr").unwrap_or_log();
    static ref FIRSTNAME_SELECTOR: Selector = Selector::parse("td:nth-child(1) > a").unwrap_or_log();
    static ref LASTNAME_SELECTOR: Selector = Selector::parse("td:nth-child(2) > a").unwrap_or_log();
    static ref TYPE_SELECTOR: Selector = Selector::parse("td:nth-child(1)").unwrap_or_log();
    static ref TIMESTAMP_SELECTOR: Selector = Selector::parse("td:nth-child(2)").unwrap_or_log();
    static ref LOCATION_SELECTOR: Selector = Selector::parse("td:nth-child(3)").unwrap_or_log();
    static ref SOURCES_SELECTOR: Selector = Selector::parse("td:nth-child(4) > a").unwrap_or_log();
    static ref GENERAL_SOURCES_SELECTOR: Selector = Selector::parse("#gedbas-sources > table > tbody > tr").unwrap_or_log();
    static ref GENERAL_SOURCE_NUM_SELECTOR: Selector = Selector::parse("td:nth-child(1) > a").unwrap_or_log();
    static ref GENERAL_SOURCE_DESCRIPTION_SELECTOR: Selector = Selector::parse("td:nth-child(2) > b").unwrap_or_log();
    static ref EVENT_SELECTOR: Selector = Selector::parse("#events > tbody > tr").unwrap_or_log();
    static ref HTTP_CLIENT: Client = Client::builder().user_agent(APP_USER_AGENT).build().unwrap_or_log();
}

#[cfg(test)]
mod tests {
    use tracing_unwrap::OptionExt;

    use super::*;

    #[test]
    fn test_parse_row() {
        let html = "<table><tr><td><a href='/person/show/id'>name</td><td><a>surname</a></td></tr></table>";
        let parsed_html = Html::parse_fragment(html);
        let selector = Selector::parse("tr").unwrap_or_log();

        let tr=  parsed_html.select(&selector).next().unwrap_or_log();


        let result = parse_row(tr);
        assert_eq!(result.firstname, "name");
        assert_eq!(result.lastname, "surname");
        assert_eq!(result.person_id, "id");
    }
}

#[instrument]
pub fn perform_search(lastname: &str, limit: usize, firstname: Option<String>) -> Vec<Person> {
    let rows = get_results(lastname, limit, &firstname.unwrap_or("".to_string()));

    rows.par_iter()
        .filter_map( get_person)
        .collect()
}

#[instrument(skip(row_element))]
fn parse_row(row_element: ElementRef) -> PersonRow {
    let name_element = row_element.select(&FIRSTNAME_SELECTOR).next().unwrap_or_log();
    let firstname = name_element.inner_html();
    let person_id = name_element.value().attr("href").unwrap_or_log()[13..].to_string();
    assert!(!person_id.is_empty());

    let lastname = row_element
        .select(&LASTNAME_SELECTOR)
        .next()
        .unwrap_or_log()
        .inner_html();

    PersonRow {
        person_id,
        firstname,
        lastname,
    }
}

#[instrument]
fn get_results(lastname: &str, limit: usize, firstname: &str) -> Vec<PersonRow> {
    let request_url = format!("https://gedbas.genealogy.net/search/simple?firstname={}&timelimit=none&lastname={}&placename=&offset=0&max={}", firstname, lastname,limit);
    let result = &HTTP_CLIENT.get(request_url).send().unwrap_or_log().text().unwrap_or_log();

    let html = Html::parse_document(result);

    let rows: Vec<PersonRow> = html.select(&ROW_SELECTOR).map(parse_row).collect();

    rows
}

#[instrument(skip(event_element))]
fn parse_event(event_element: ElementRef) -> Option<Event> {
    let event_type = event_element
        .select(&TYPE_SELECTOR)
        .next()?
        .inner_html()
        .trim()
        .to_string();

    let timestamp = event_element
        .select(&TIMESTAMP_SELECTOR)
        .next()?
        .text()
        .next()?
        .trim()
        .to_string();

    let location = event_element
            .select(&LOCATION_SELECTOR)
            .next()
            .and_then(|e| e.text().next())
            .map(|t| t.trim().to_string());

    let sources = event_element
            .select(&SOURCES_SELECTOR)
            .filter_map(parse_source)
            .collect();

    let event = Event {
        event_type,
        timestamp,
        location,
        sources,
    };

    tracing::event!(Level::DEBUG, "parsed event {:?}", event);

    Some(event)
}

#[instrument(skip(source_element))]
fn parse_source(source_element: ElementRef) -> Option<usize> {
    let source_link = source_element.value().attr("href")?;
    let source_num_text = source_link[8..].to_string();
    
    source_num_text.parse::<usize>().ok()
}

#[instrument]
fn get_person(person_row: &PersonRow) -> Option<Person> {
    let request_url = format!(
        "https://gedbas.genealogy.net/person/show/{}",
        person_row.person_id
    );
    let result = &HTTP_CLIENT.get(request_url).send().ok()?.text().ok()?;

    let html = Html::parse_document(result);

    let events: Vec<Event> = html
        .select(&EVENT_SELECTOR)
        .filter_map(parse_event)
        .collect();

    let sources = html
        .select(&GENERAL_SOURCES_SELECTOR)
        .filter_map(parse_general_source)
        .collect();

    Some(Person {
        person_id: person_row.person_id.to_string(),
        firstname: person_row.firstname.clone(),
        lastname: person_row.lastname.clone(),
        events,
        sources
    })
}

#[instrument(skip(source_element))]
fn parse_general_source(source_element: ElementRef) -> Option<Source> {
    let num_text = source_element
        .select(&GENERAL_SOURCE_NUM_SELECTOR)
        .next()?
        .value().attr("name")?[7..].to_string();

    tracing::trace!("num text: {}", num_text);
    let num = num_text.parse::<usize>().ok()?;

    let description = source_element
        .select(&GENERAL_SOURCE_DESCRIPTION_SELECTOR)
        .next()?
        .text()
        .next()?
        .trim()
        .to_string();

    let source = Source {
        num,
        description
    };

    tracing::event!(Level::DEBUG, "parsed general source {:?}", source);

    Some(source)
}
