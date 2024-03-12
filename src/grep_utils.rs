
use {
   
    //grep::matcher::Matcher,
    grep::regex::RegexMatcher,
    grep::searcher::{Searcher, BinaryDetection, SearcherBuilder},
    grep::searcher::sinks::UTF8,
 
    
};

use termcolor::{ColorChoice, WriteColor};
//use std::path::Path;
use std::io::Write;
use std::io::IsTerminal;
use std::path::Path;
//use termcolor_output::colored;
use termcolor::{Color, StandardStream, ColorSpec};
//use crate::xpdf_tools;



//https://github.com/BurntSushi/ripgrep
pub struct Settings {
    pub stream: StandardStream,
    pub color_specs: SearchColorSpecs,
    pub searcher: Searcher,
    
}
impl Default for Settings {
    fn default() -> Self {
        let color_choice = if std::io::stdin().is_terminal() { ColorChoice::Auto} else { ColorChoice::Never};
        let stream = StandardStream::stdout(color_choice);

        let searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .multi_line(false)
            .line_number(true)
            .build();

        Settings {
            stream,
            color_specs: SearchColorSpecs::default(),
            searcher
        }
        
    }
}
pub struct SearchColorSpecs {
    pub match_spec: ColorSpec,
    pub text_spec: ColorSpec,
    pub info_spec: ColorSpec,
    pub extra_spec: ColorSpec,
}


impl Default for SearchColorSpecs {
    fn default() -> Self {
        let mut m_spec = ColorSpec::new();
        m_spec.set_fg(Some(Color::Rgb(255, 197, 12)));
        m_spec.set_bold(true);
        m_spec.set_underline(true);

        let mut t_spec = ColorSpec::new();
        t_spec.set_fg(Some(Color::Rgb(249, 246, 238)));

        let mut i_spec = ColorSpec::new();
        i_spec.set_fg(Some(Color::Rgb(52, 154, 179)));

        let mut e_spec = ColorSpec::new();
        e_spec.set_fg(Some(Color::Rgb(181, 235, 18)));

        SearchColorSpecs {
            match_spec: m_spec,
            text_spec: t_spec,
            info_spec: i_spec,
            extra_spec: e_spec,
        }
    }
}


pub fn search_file(content: &Vec<u8>, pattern: &String, settings: &mut Settings, file_path: &Path) -> Result<(), Box<dyn std::error::Error>>{
    
    settings.stream.set_color(&settings.color_specs.extra_spec)?;
    writeln!(settings.stream, "Searching: {}", file_path.display())?;
    settings.stream.set_color(&settings.color_specs.text_spec)?;
    let search_result = search_pdf_text(pattern, content, settings);
        //let search_result = grep_utils::search_to_vec(search_term, &file_result);
        
    match search_result {
        Ok(count) => {
            //println!("{result}");
            settings.stream.set_color(&settings.color_specs.extra_spec)?;
            writeln!(settings.stream, "End of file: found {} matches..\n", count)?;
            settings.stream.set_color(&settings.color_specs.text_spec)?;
            //print_result(&result);

        },
        Err(e) => println!("{:?}", e),
    }

    settings.stream.reset().unwrap();

    Ok(())
}

#[allow(dead_code)]
pub fn search_pdf_text(what: &str, from: &[u8], settings: &mut Settings) -> Result<u32, Box<dyn std::error::Error>> {
   
    let s = format!(r"(?i){}", what);
    let matcher = RegexMatcher::new(s.as_str())?;
    
    // let mut searcher = SearcherBuilder::new()
    //     .binary_detection(BinaryDetection::quit(b'\x00'))
    //     .multi_line(false)
    //     .line_number(true)
    //     .build();

    let searcher = &mut settings.searcher;
    let stdout = &mut settings.stream;
    let color_specs = &mut settings.color_specs;
    // let color_choice = if std::io::stdin().is_terminal() { ColorChoice::Auto} else { ColorChoice::Never};
    // let mut stdout = StandardStream::stdout(color_choice);
    
    //let color_specs = SearchColorSpecs::default();
    let mut match_count = 0;

    for (page, split) in String::from_utf8_lossy(from).split("\u{c}").enumerate() {
       
        let page = page as i32;
        let mut page_found:i32 = -1;
        let _result = searcher.search_slice(&matcher, split.as_bytes(),  UTF8(|lnum, line| {
            
                stdout.set_color(&color_specs.info_spec)?;

                if page != page_found {
                    writeln!(stdout, "Page: {}", page + 1)?;
                    page_found = page;
                }
                write!(stdout, "{}: ", lnum)?;
        
                let vec_split = get_match_vector_from_line(what, line);
                for item in vec_split {
                    if item.to_lowercase() == what.to_lowercase() {
                        stdout.set_color(&color_specs.match_spec)?;
                    } else {
                        stdout.set_color(&color_specs.text_spec)?;
                    }

                    write!(stdout, "{}", item)?;
                }
               
                match_count += 1;
                Ok(true)
        }));    
    }

    //stdout.reset()?;
   
    Ok(match_count)
}

fn get_match_vector_from_line<'a>(what: &'a str, line: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;
    let temp_line = line.to_lowercase();
    for (index, matched) in temp_line.match_indices(what.to_lowercase().as_str()) {
        if last != index {
            result.push(&line[last..index]);
        }
        //result.push(matched);
        result.push(&line[index..index + matched.len()]);
        last = index + matched.len();
    }

    if last < line.len() {
        result.push(&line[last..]);
    }

    result
}


pub fn search_to_vec(what: &str, from: &[u8]) -> Result<Vec<(u64, String)>, Box<dyn std::error::Error>> {
   
    let s = format!(r"(?i){}", what);
    println!("search for: {}", s);
    let matcher = RegexMatcher::new(s.as_str())?;
    
    
    let mut matches: Vec<(u64, String)> = Vec::new();
    Searcher::new().search_slice(&matcher, from, UTF8(|lnum, line| {
        // We are guaranteed to find a match, so the unwrap is OK.
        //let mymatch = matcher.find(line.as_bytes())?.unwrap();
        //matches.push((lnum, line[mymatch].to_string()));
        matches.push((lnum, line.to_string()));
        Ok(true)
    }))?;

    
    Ok(matches)
}



// #[allow(dead_code)]
// pub fn search_to_console(what: &str, from: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
   
//     let s = format!(r"(?i){}", what);
//     //println!("search for: {}", s);
//     let matcher = RegexMatcher::new(s.as_str())?;
    
//     let specs = build_color_specs();
//     let mut printer = StandardBuilder::new()
//         .color_specs(specs)
//         .build(cli::stdout(if std::io::stdout().is_terminal() {
//             ColorChoice::Auto
//         } else {
//             ColorChoice::Never
//         }));

//     let mut searcher = SearcherBuilder::new()
//         .binary_detection(BinaryDetection::quit(b'\x00'))
//         .multi_line(false)
//         .line_number(true)
//         .build();

    
//     searcher.search_slice(&matcher, from,  printer.sink(&matcher),)?;
   
//     Ok(())
// }
// pub fn build_color_specs () -> ColorSpecs {
//     let spec_match: UserColorSpec = "match:fg:0xff,0xc5,0x12".parse().unwrap();
//     let spec_match2: UserColorSpec = "match:style:underline".parse().unwrap();
//     let spec_match3: UserColorSpec = "match:style:bold".parse().unwrap();
//     let spec_line: UserColorSpec = "line:fg:0x04,0x85,0xd1".parse().unwrap();

//     let spec = ColorSpecs::new(&[spec_line, spec_match, spec_match2, spec_match3]);
    
//     spec
// }