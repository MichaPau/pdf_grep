use std::{collections::BTreeMap, io:: Write, path::Path};
use core::fmt::Debug;

use grep::regex::RegexMatcher;
use rand::Rng;
use serde::{Deserialize, Serialize};
use xpdf_tools::{PdfError, XpdfTools};

use crate::{grep_utils::{self}, settings::{FolderSearchMode, Settings}, utils, BoxError};

use rayon::prelude::*;

use termcolor::{BufferedStandardStream, Color, ColorSpec, WriteColor};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AvailablePdfTools {
    UseXpdfTools,
    UsePdfDummyTool,
}

pub trait PDFTools: std::fmt::Debug {

    fn pdf_info(&self, file_path: &Path) -> Result<BTreeMap<String, Option<String>>, BoxError>;
    fn pdf_text(&self, file_path: &Path) -> Result<Vec<u8>, BoxError>;
    fn split_pages<'a>(&self, text: &'a str) -> Result<Vec<&'a str>, BoxError>;
    fn search_file(&self, file: &Path, pattern: &str, settings: &Settings) -> Result<(), BoxError>;
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

    fn split_pages<'a>(&self, text: &'a str) -> Result<Vec<&'a str>, BoxError> {
        Ok(text.split('\u{c}').collect::<Vec<_>>())
    }
    fn search_file(&self, file: &Path, pattern: &str, settings: &Settings) -> Result<(), BoxError> {
        
        let mut printer = settings.create_printer();
        //let file_path = file;
        
        match settings.tools.pdf_text(file) {
            Ok(content) => {

                let file_header = format!("Searching: {}\n", file.display());
                let p = printer.get_mut();
                p.set_color(&settings.info_color_spec).unwrap();
                p.write_all(file_header.as_bytes()).unwrap();
                
                let s = format!(r"(?i){}", pattern);
                let matcher = RegexMatcher::new(s.as_str())?;
                
                let mut total = 0;
                for (page, split) in String::from_utf8_lossy(&content).split('\u{c}').enumerate() {
                    let search_result = grep_utils::search_pdf_page(&matcher, &mut printer, split.as_bytes(), page, settings);
                    match search_result {
                        Ok(count) => {
                            total += count;
                        },
                        Err(e) => eprint!("{}", e),
                    }
                }

                let file_footer = format!("End of file: found {} matches.\n\n", total);
                let p = printer.get_mut();
                p.set_color(&settings.info_color_spec).unwrap();
                p.write_all(file_footer.as_bytes()).unwrap();

                p.reset().unwrap();
            },
            Err(e) => {
                let mut stderr = BufferedStandardStream::stderr(termcolor::ColorChoice::Auto);
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red))).unwrap();
                if let Some(pdf_error) = e.downcast_ref::<PdfError>() {
                    stderr.write_all(pdf_error.message.as_bytes()).unwrap();
                    stderr.write_all(b"\n").unwrap();
                    stderr.write_all(pdf_error.process_message.as_bytes()).unwrap();
                    
                } else {
                    stderr.write_all(e.to_string().as_bytes()).unwrap();
                }
                stderr.flush().unwrap();
                stderr.reset().unwrap();
            },
        };

        

        //stdout.reset()?;
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

    fn split_pages<'a>(&self, _text: &'a str) -> Result<Vec<&'a str>, BoxError> {
        unimplemented!();
    }
    fn search_file(&self, _file: &Path, _pattern: &str, _settings: &Settings) -> Result<(), BoxError> {
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


pub fn search_dir(dir_path: &Path, pattern: &str, settings: &Settings) -> Result<(), BoxError> {
   
    if settings.folder_search_mode == FolderSearchMode::ThreadPerFolder {
        let pdf_map = utils::get_folder_tree(dir_path);

        pdf_map.par_iter().for_each(|(_, list)| {
            for file in list {
                settings.tools.search_file(Path::new(file), pattern, settings).unwrap_or_else(|e| eprintln!("{e}"))
            }
         });
    } else if settings.folder_search_mode == FolderSearchMode::ThreadPerFile {
        let pdf_files = utils::get_folder_files(dir_path);
        
        pdf_files.par_iter().for_each(|(_dir, file)| {
            settings.tools.search_file(Path::new(file), pattern, settings).unwrap_or_else(|e| eprintln!("{e}"))
        });
    }
    Ok(())
}

pub fn get_random_text(dir_path: &Path, settings: &Settings, snippet_length: usize) -> Result<(Vec<String>, String), BoxError> {
    
    let mut rng = rand::thread_rng();
    let pdf_files = utils::get_folder_files(dir_path);

    let file_index = rng.gen_range(0..pdf_files.len());
    let file = Path::new(&pdf_files[file_index].1);
    let text = settings.tools.pdf_text(file)?;

    if text.len() < snippet_length {
        let slice = String::from_utf8_lossy(text.as_slice()).to_string();
        
        Ok((vec![file.display().to_string()], slice))
    } else {
        let text_str = String::from_utf8_lossy(text.as_slice()).to_string();
        let pages = settings.tools.split_pages(&text_str)?;
        let page_index = rng.gen_range(0..pages.len());
        let page_text = pages[page_index];
        let start = rng.gen_range(0..page_text.len() - snippet_length -1);
        
        let left_index = utils::get_left_index_trim(&page_text, start, 1);
        let right_index = utils::get_right_index_trim(&page_text, start+snippet_length, 1);
        let slice = &page_text[left_index..right_index];

        let meta = vec![file.display().to_string(), format!("Page: {}", page_index), format!("range: {left_index}-{right_index}")];
        //let slice = &text[start..start+snippet_length];
        //let result = String::from_utf8_lossy(slice).to_string();
        Ok((meta, slice.to_string()))
    }
    


}

#[test]
fn test_s_len() {
    let sparkle_heart_vec = vec![240, 159, 146, 150];

    let sparkle_heart = String::from_utf8_lossy(&sparkle_heart_vec);

    println!("vec.len:{}, String len:{}", sparkle_heart_vec.len(), sparkle_heart.len());
    assert_eq!(sparkle_heart_vec.len(), sparkle_heart.len());
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