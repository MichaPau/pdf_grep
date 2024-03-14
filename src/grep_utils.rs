
use {
   
    crate::settings::Settings, 
    grep::{regex::RegexMatcher, searcher::{sinks::UTF8, Searcher}}
 
};
use termcolor::WriteColor;
use std::io::Write;
use std::path::Path;


//https://github.com/BurntSushi/ripgrep


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