use std::fmt::Debug;
use std::{collections::BTreeMap, path::Path};

use walkdir::WalkDir;
use walkdir::DirEntry;

fn is_pdf(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.ends_with(".pdf"))
         .unwrap_or(false)
}

pub fn get_folder_tree(dir_path: &Path) -> BTreeMap<String, Vec<String>> {
    let mut pdf_map:BTreeMap<String, Vec<String>> = BTreeMap::new();
    //c:\Data\Library\Books
    let walk_iter = WalkDir::new(dir_path).into_iter();
    for entry in walk_iter.map(Result::unwrap).filter(|entry| is_pdf(entry)) {
       
        pdf_map.entry(entry.path().parent().unwrap().to_str().unwrap().to_string())
            .and_modify(|v| v.push(entry.path().display().to_string()))
            .or_insert(vec![entry.path().display().to_string()]);
        //pdf_files.push(entry.path().display().to_string());
    }

    pdf_map

}

pub fn get_folder_files(dir_path: &Path) -> Vec<(String, String)> {
    let mut list:Vec<(String, String)> = vec![];

    let walk_iter = WalkDir::new(dir_path).into_iter();
    for entry in walk_iter.map(Result::unwrap).filter(|entry| is_pdf(entry)) {
       
       list.push((entry.path().parent().unwrap().to_str().unwrap().to_string(), entry.path().display().to_string()));
    }

    list
}

pub fn get_left_index_trim(line: &str, left_pos: usize, trim_value: usize) -> usize {
    let trim_start_left  = left_pos.saturating_sub(trim_value);
    let left_index = line.as_bytes().iter().collect::<Vec<_>>().iter().enumerate().rposition(|(i, c)| c.is_ascii_whitespace() && i < trim_start_left).unwrap_or(0);
    left_index+1
}
pub fn get_right_index_trim(line: &str, right_pos: usize, trim_value: usize) -> usize {
    let trim_start_right = std::cmp::min(right_pos - 1 + trim_value, line.len() - 1);
    let right_index = line.as_bytes().iter().enumerate().position(|(i, c)| c.is_ascii_whitespace() && i >= trim_start_right).unwrap_or(line.len() - 1);
    right_index
}
pub fn _dump<I, K, V, A>(iter: I) 
    where 
        I: IntoIterator<Item = (K, V)>, 
        K: Debug, 
        V: Debug + IntoIterator<Item=A>,
        A: Debug
{

        for (key, value) in iter {
            println!("{:?}", key);
            for entry in value.into_iter() {
                println!("\t{:?}", entry);
            }
            
        }

}