use std::borrow::BorrowMut;

use grep::{
    printer::Standard,
    regex::RegexMatcher, searcher::Searcher};
use termcolor::BufferedStandardStream;

use crate::settings::Settings;

pub fn search_pdf_page(matcher: &RegexMatcher, printer: &mut Standard<BufferedStandardStream>, from: &[u8], page: usize, _settings: &Settings) -> Result<u32, Box<dyn std::error::Error>> {
   
    let p = format!("Page: {}", page+1);
    let mut sink = printer.sink_with_path(matcher, p.as_str());
    Searcher::new().search_slice(matcher, from, sink.borrow_mut())?;
    let stats = sink.stats().unwrap();

    Ok(stats.matches() as u32)
}