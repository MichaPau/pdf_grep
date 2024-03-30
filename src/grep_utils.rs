use crate::settings::ShortenLineMode;

use {
   
    crate::settings::Settings, 
    grep::{regex::RegexMatcher, searcher::{sinks::UTF8, Searcher}}
 
};

use grep::matcher::Matcher;

use termcolor::WriteColor;
use std::io::Write;
use std::path::Path;

//https://github.com/BurntSushi/ripgrep

pub fn search_file(content: &Vec<u8>, pattern: &String, settings: &Settings, file_path: &Path) -> Result<(), Box<dyn std::error::Error>>{
    
    let mut stdout = settings.create_color_writer();
    let mut searcher = Settings::create_searcher();

    stdout.set_color(&settings.color_specs.extra_spec)?;
    writeln!(stdout, "Searching: {}", file_path.display())?;
    stdout.set_color(&settings.color_specs.text_spec)?;

    let s = format!(r"(?i){}", pattern);
    let matcher = RegexMatcher::new(s.as_str())?;
    
    let mut total = 0;
    for (page, split) in String::from_utf8_lossy(&content).split("\u{c}").enumerate() {
        let search_result = search_pdf_page(&matcher, split.as_bytes(), page, settings, &mut stdout, &mut searcher);
        match search_result {
            Ok(count) => {
                total += count;
                ()
            },
            Err(e) => eprint!("{}", e),
        }
    }
    stdout.set_color(&settings.color_specs.extra_spec)?;
    writeln!(stdout, "End of file: found {} matches..\n", total)?;
    stdout.set_color(&settings.color_specs.text_spec)?;

    stdout.reset()?;

    Ok(())
}

pub fn search_file2(content: &Vec<u8>, pattern: &String, settings: &Settings, file_path: &Path) -> Result<(), Box<dyn std::error::Error>>{
    
    let mut stdout = settings.create_color_writer();

    stdout.set_color(&settings.color_specs.extra_spec)?;
    writeln!(stdout, "Searching: {}", file_path.display())?;
    stdout.set_color(&settings.color_specs.text_spec)?;

    let search_result = search_pdf_text(pattern, content, settings, &mut stdout);
    
    match search_result {
        Ok(count) => {
            stdout.set_color(&settings.color_specs.extra_spec)?;
            writeln!(stdout, "End of file: found {} matches..\n", count)?;
            stdout.set_color(&settings.color_specs.text_spec)?;
            ()
        },
        Err(e) => eprint!("{}", e),
    }

    stdout.reset()?;

    Ok(())
}

#[allow(dead_code)]
pub fn search_pdf_page<W>(matcher: &RegexMatcher, from: &[u8], page: usize, settings: &Settings, stdout: &mut W, searcher: &mut Searcher) -> Result<u32, Box<dyn std::error::Error>>
where W: std::io::Write + WriteColor {

    let color_specs = &settings.color_specs;
    
    let mut match_count = 0;
    let page = page as i32;
    let mut page_found:i32 = -1;

    
    let _result = searcher.search_slice(&matcher, from,  UTF8(|lnum, line| {
        
            let first_match = matcher.find(line.as_bytes())?.unwrap();
        
            stdout.set_color(&color_specs.info_spec)?;
            if page != page_found {
                writeln!(stdout, "Page: {}", page + 1)?;
                page_found = page;
            }
            write!(stdout, "{}: ", lnum)?;
            let mut index = 0;
            

            if let ShortenLineMode::Trim(len) = settings.shorten_line_mode {
                if first_match.start() as isize - len as isize > 0 {
                    index = first_match.start() - len;
                    write!(stdout, "..").unwrap();
                } 
            } else {

            }
            let line_bytes = line.as_bytes(); 
            
            let _ = matcher.find_iter_at(line_bytes, index, |m| {
                //println!("{:?}", m); true
                stdout.set_color(&color_specs.text_spec).unwrap();
                write!(stdout, "{}", line[index..m.start()].to_owned()).unwrap();
                stdout.set_color(&color_specs.match_spec).unwrap();
                write!(stdout, "{}", line[m.start()..m.end()].to_owned()).unwrap();
                index = m.end();
                true
            });
            stdout.set_color(&color_specs.text_spec).unwrap();
            if let ShortenLineMode::Trim(len) = settings.shorten_line_mode {
                let mut end_index = index + len;
                if end_index > line.len() {
                    end_index = line.len();
                }
                let mut end = line[index..end_index].to_owned();
                end.push_str("..\n");
                write!(stdout, "{}", end).unwrap();
            } else {
                write!(stdout, "{}", line[index..].to_owned()).unwrap();
            }
            

            // if settings.print_text {
            //     let vec_split = get_match_vector_from_line(what, line);
            //     for (i, &item) in vec_split.iter().enumerate() {
                    
            //         if item.to_lowercase() == what.to_lowercase() {
            //             stdout.set_color(&color_specs.match_spec)?;
            //         } else {
            //             stdout.set_color(&color_specs.text_spec)?;
            //         }

            //         if let ShortenLineMode::Trim(len) = settings.shorten_line_mode {
            //             let mut shortened:String = String::new();
            //             if i == 0 && item.to_lowercase() != what.to_lowercase(){
                            
            //                 shortened.push_str("..");
            //                 let start_rev:String = item.chars().rev().enumerate().take_while(|(i, c)| {
            //                     if *i + 1 >= len && c.is_whitespace() {
            //                         false
            //                     } else {
            //                         true
            //                     }
            //                 }).map(|(_, c)| c).collect();

            //                 shortened.push_str(start_rev.chars().rev().collect::<String>().as_str());
                            

            //             } else if i == vec_split.len() - 1 && item.to_lowercase() != what.to_lowercase() {
                            
            //                 shortened = item.chars().enumerate().take_while(|(i, c)| {
            //                     if *i + 1 >= len && c.is_whitespace() {
            //                         false
            //                     } else {
            //                         true
            //                     }
            //                 }).map(|(_, c)| c).collect();
            //                 shortened.push_str("..\n");
                            
            //             } else {
            //                 shortened = String::from(item);
            //             }
            //             write!(stdout, "{}", shortened)?;
                        
            //         } else {
            //             write!(stdout, "{}", item)?;
            //         }
            //     }
            // }
            
            match_count += 1;
            Ok(true)
        }));

        //stdout.reset()?;
        Ok(match_count)
}

#[allow(dead_code)]
pub fn search_pdf_text<W>(what: &str, from: &[u8], settings: &Settings, stdout: &mut W) -> Result<u32, Box<dyn std::error::Error>>
    where W: std::io::Write + WriteColor {
   
    let s = format!(r"(?i){}", what);
    let matcher = RegexMatcher::new(s.as_str())?;

    let mut searcher = Settings::create_searcher();
    
    let color_specs = &settings.color_specs;
    
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
        
                if settings.print_text {
                    let vec_split = get_match_vector_from_line(what, line);
                    for (i, &item) in vec_split.iter().enumerate() {
                        
                        if item.to_lowercase() == what.to_lowercase() {
                            stdout.set_color(&color_specs.match_spec)?;
                        } else {
                            stdout.set_color(&color_specs.text_spec)?;
                        }

                        if let ShortenLineMode::Trim(len) = settings.shorten_line_mode {
                            let mut shortened:String = String::new();
                            if i == 0 && item.to_lowercase() != what.to_lowercase(){
                                
                                shortened.push_str("..");
                                let start_rev:String = item.chars().rev().enumerate().take_while(|(i, c)| {
                                    if *i + 1 >= len && c.is_whitespace() {
                                        false
                                    } else {
                                        true
                                    }
                                }).map(|(_, c)| c).collect();

                                //let start:String = start_rev.chars().rev().collect();
                                //let start_rev:String = item.chars().take(len).take_while(|c| !c.is_whitespace()).collect();
                                shortened.push_str(start_rev.chars().rev().collect::<String>().as_str());
                                

                            } else if i == vec_split.len() - 1 && item.to_lowercase() != what.to_lowercase() {
                                //shortened  = item.chars().take(len).take_while(|c| !c.is_whitespace()).collect();
                                
                                shortened = item.chars().enumerate().take_while(|(i, c)| {
                                    if *i + 1 >= len && c.is_whitespace() {
                                        false
                                    } else {
                                        true
                                    }
                                }).map(|(_, c)| c).collect();
                                shortened.push_str("..\n");
                                
                            } else {
                                shortened = String::from(item);
                            }
                            write!(stdout, "{}", shortened)?;
                            
                        } else {
                            write!(stdout, "{}", item)?;
                        }
                    }
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

fn get_match_vector_from_line2<'a>(what: &'a str, line: &'a str, trim: ShortenLineMode) -> Vec<&'a str> {
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

    if let ShortenLineMode::Trim(len) = trim {
        //result.first_mut().unwrap().replace(from, to)
        let trim_result = result.iter().enumerate().map(|(i ,&item)| {
            if i == 0 && item.len() > len {
                let sub = &item[item.len() - 1 - len..];
                sub
            }
            else if i == result.len() -1 && item.len() > len {
                let sub = &item[..len];
                sub
            } else {
                item
            }
        }).collect();

        trim_result
    } else {
        result
    }
    
}

fn search_to_vec(what: &str, from: &[u8]) -> Result<Vec<(u64, String)>, Box<dyn std::error::Error>> {
   
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

#[ignore]
#[test]
fn test_tokenizerish() {
    let hay = "this is some sample text to look for a word. There are more words in the Text, because it's a text, we'll talk more about this!";
    let result = get_match_vector_from_line2("text", hay, ShortenLineMode::Trim(7));

    //let result:Vec<_> = hay.split(|c: char| !c.is_alphanumeric()).collect();
    println!("{:?}", result);
}

#[ignore]
#[test]
fn test_format_write() {
    let str = " àéù ç is a sentence, with some text and !!!! 💖💖 💖💖💖 ";
    let max = 5;
    //let len = str.chars().count();

    //to first whitespace after max
    let end:String = str.chars().enumerate().take_while(|(i, c)| {
        if *i + 1 >= max && c.is_whitespace() {
            false
        } else {
            true
        }
    }).map(|(_, c)| c).collect();
    //let end:String   = str.chars().take(max).take_while(|c| !c.is_whitespace()).collect();
    
    let start_rev:String = str.chars().rev().enumerate().take_while(|(i, c)| {
        if *i + 1 >= max && c.is_whitespace() {
            false
        } else {
            true
        }
    }).map(|(_, c)| c).collect();
    let start:String = start_rev.chars().rev().collect();

    let mut stdout = std::io::stdout();
    writeln!(stdout, "----------------------------").unwrap();
    writeln!(stdout, "start: ..{}", start).unwrap();
    writeln!(stdout, "end: {}..", end).unwrap();
    writeln!(stdout, "----------------------------").unwrap();
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