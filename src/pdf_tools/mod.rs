use std::{collections::BTreeMap, io::{self, Write}, path::Path};
use core::fmt::Debug;

use grep::regex::RegexMatcher;
use serde::{Deserialize, Serialize};
use xpdf_tools::{PdfError, XpdfTools};

use crate::{grep_utils::{self, search_pdf_page}, settings::{FolderSearchMode, Settings}, utils, BoxError};

use rayon::prelude::*;

use termcolor::{StandardStream, WriteColor};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AvailablePdfTools {
    UseXpdfTools,
    UsePdfDummyTool,
}

pub trait PDFTools: std::fmt::Debug {

    fn pdf_info(&self, file_path: &Path) -> Result<BTreeMap<String, Option<String>>, BoxError>;
    fn pdf_text(&self, file_path: &Path) -> Result<Vec<u8>, BoxError>;
    fn search_file(&self, file: &Path, pattern: &String, settings: &Settings) -> Result<(), BoxError>;
}

// impl Debug for dyn PDFTools + std::marker::Send + std::marker::Sync {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "PDFTools")
//     }
// }
#[derive(Debug)]
pub struct XpdfWrapper {
    pub tools: XpdfTools,
}
#[derive(Debug)]
pub struct  PdfDummyTool {
    
}

//much slower and has error for glyph extract
// pub struct PdfExtractWrapper {

// }


// impl PDFTools for PdfExtractWrapper {
//     fn pdf_info(&self, _file_path: &Path) -> Result<BTreeMap<String, Option<String>>, BoxError> {
//         todo!()
//     }

//     fn pdf_text(&self, file_path: &Path) -> Result<Vec<u8>, BoxError> {
//         let bytes = std::fs::read(file_path).unwrap();
//         match pdf_extract::extract_text_from_mem(&bytes) {
//             Ok(text) => Ok(text.as_bytes().to_vec()),
//             Err(e) => Err(Box::new(e)),
//         }
//     }
// }
impl PDFTools for XpdfWrapper {
    
    fn pdf_info(&self, file_path: &Path) -> Result<BTreeMap<String, Option<String>>, BoxError> {
        match self.tools.pdf_info(file_path) {
            Ok(pdf_info) => Ok(pdf_info.info_map.0),
            Err(e) => Err(Box::new(e)) 
        }
    }

    fn pdf_text(&self, file_path: &Path) -> Result<Vec<u8>, BoxError> {
        match self.tools.pdf_text(file_path) {
            Ok(text) => Ok(text),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn search_file(&self, file: &Path, pattern: &String, settings: &Settings) -> Result<(), BoxError> {
        let mut stdout = settings.create_color_writer();
        let mut searcher = Settings::create_searcher();
        //let mut stdout = _stdout;
        let file_path = &file;
        
        match settings.tools.pdf_text(&file_path) {
            Ok(content) => {
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
                stdout.flush().unwrap();
            },
            Err(e) => {
                if let Some(pdf_error) = e.downcast_ref::<PdfError>() {
                    io::stderr().write(pdf_error.message.as_bytes()).unwrap();
                    io::stderr().write(b"\n").unwrap();
                    io::stderr().write(pdf_error.process_message.as_bytes()).unwrap();
                    
                } else {
                    io::stderr().write(e.to_string().as_bytes()).unwrap();
                }
                io::stderr().flush().unwrap();
            },
        };

        

        stdout.reset()?;
        Ok(())
    }

} 

impl PDFTools for PdfDummyTool {
    fn pdf_info(&self, _file_path: &Path) -> Result<BTreeMap<String, Option<String>>, BoxError> {
        let info = BTreeMap::from([
            ("Title".into(), Some("Test tool title".into())), ("Author".into(), Option::None)]);
        Ok(info)
    }

    fn pdf_text(&self, _file_path: &Path) -> Result<Vec<u8>, BoxError> {
        let text = "Some sample text from a test wrapper. \n Can you find something as a test?".as_bytes().to_vec();
        Ok(text)
    }

    fn search_file(&self, _file: &Path, _pattern: &String, _settings: &Settings) -> Result<(), BoxError> {
        unimplemented!();
    }
}

pub fn get_info_file(file_path: &Path, settings: &Settings) {
    
    match settings.tools.pdf_info(file_path) {
        Ok(pdf_info) => {
            println!("{:#?}", pdf_info);
        },
        Err(e) => println!("{:?}", e),
    }
}

pub fn get_info_dir(dir_path: &Path) {
    println!("info for: {:?}", dir_path);
}

// pub fn search_invoke_file(file: String, pattern: &String, settings: &Settings) {
   
//         let p = Path::new(&file);
//         match settings.tools.pdf_text(&p) {
//             Ok(content) => {
//                 grep_utils::search_file(&content, pattern, &settings, &p).unwrap();
//             },
//             Err(e) => {
//                 if let Some(pdf_error) = e.downcast_ref::<PdfError>() {
//                     io::stderr().write(pdf_error.message.as_bytes()).unwrap();
//                     io::stderr().write(b"\n").unwrap();
//                     io::stderr().write(pdf_error.process_message.as_bytes()).unwrap();
                    
//                 } else {
//                     io::stderr().write(e.to_string().as_bytes()).unwrap();
//                 }
//                 io::stderr().flush().unwrap();
//             },
//         } 
// }

pub fn search_invoke_folders(_dir: &str, list: &Vec<String>, pattern: &String, settings: &Settings) {
    
    for file in list {
        let p = Path::new(file);
        match settings.tools.pdf_text(&p) {
            Ok(content) => {
                grep_utils::search_file(&content, pattern, &settings, &p).unwrap();
            },
            Err(e) => {
                if let Some(pdf_error) = e.downcast_ref::<PdfError>() {
                    io::stderr().write(pdf_error.message.as_bytes()).unwrap();
                    io::stderr().write(b"\n").unwrap();
                    io::stderr().write(pdf_error.process_message.as_bytes()).unwrap();
                    
                } else {
                    io::stderr().write(e.to_string().as_bytes()).unwrap();
                }
                io::stderr().flush().unwrap();
            },
        }
    }
}

pub fn search_dir(dir_path: &Path, pattern: &String, settings: &Settings) -> Result<(), BoxError>{
   
    if settings.folder_search_mode == FolderSearchMode::ThreadPerFolder {
        // let pdf_map = utils::get_folder_tree(dir_path);

        // pdf_map.par_iter().for_each(|(dir, list)| {
        //     search_invoke_folders(dir, list, pattern, &settings);
        // });
    } else if settings.folder_search_mode == FolderSearchMode::ThreadPerFile {
        let pdf_files = utils::get_folder_files(dir_path);
        
        pdf_files.par_iter().for_each(|(_dir, file)| {
            //search_invoke_file(String::from(file), &pattern, &settings);
            settings.tools.search_file(Path::new(&file), &pattern, &settings).unwrap_or_else(|e| eprintln!("{e}"))
            //search_file_test(Path::new(&file), &pattern, &settings).unwrap_or_else(|e| eprintln!("{e}"))
        });
    }
    Ok(())
}

fn search_file_test(file: &Path, pattern: &String, settings: &Settings, stdout: &mut StandardStream) -> Result<(), BoxError> {
    //let mut stdout = settings.create_color_writer();
    let mut searcher = Settings::create_searcher();
    //let mut stdout = _stdout;
    let file_path = &file;
    
    match settings.tools.pdf_text(&file_path) {
        Ok(content) => {
            stdout.set_color(&settings.color_specs.extra_spec)?;
            writeln!(stdout, "Searching: {}", file_path.display())?;
            stdout.set_color(&settings.color_specs.text_spec)?;

            let s = format!(r"(?i){}", pattern);
            let matcher = RegexMatcher::new(s.as_str())?;
            
            let mut total = 0;
            for (page, split) in String::from_utf8_lossy(&content).split("\u{c}").enumerate() {
                let search_result = search_pdf_page(&matcher, split.as_bytes(), page, settings, stdout, &mut searcher);
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
            stdout.flush().unwrap();
        },
        Err(e) => {
            if let Some(pdf_error) = e.downcast_ref::<PdfError>() {
                io::stderr().write(pdf_error.message.as_bytes()).unwrap();
                io::stderr().write(b"\n").unwrap();
                io::stderr().write(pdf_error.process_message.as_bytes()).unwrap();
                
            } else {
                io::stderr().write(e.to_string().as_bytes()).unwrap();
            }
            io::stderr().flush().unwrap();
        },
    };

    

    stdout.reset()?;
    Ok(())
}
// pub fn search_file(file_path: &Path, pattern: &String, settings: &Settings) {
   

//     match settings.tools.pdf_text(file_path) {
//         Ok(content) => {
//             grep_utils::search_file(&content, pattern, settings, file_path).unwrap();
//         },
//         Err(e) => {
//             if let Some(pdf_error) = e.downcast_ref::<PdfError>() {
//                 io::stderr().write(pdf_error.message.as_bytes()).unwrap();
//                 io::stderr().write(b"\n").unwrap();
//                 io::stderr().write(pdf_error.process_message.as_bytes()).unwrap();
                
//             } else {
//                 io::stderr().write(e.to_string().as_bytes()).unwrap();
//             }
//             io::stderr().flush().unwrap();
//         },
//     }
    
// }