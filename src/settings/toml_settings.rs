use std::{fs::{File, self}, path::PathBuf};
use std::io::{Write, Read};
use directories::ProjectDirs;
use serde::{Serialize, Deserialize};

use crate::BoxError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct ConfigColorSpec {
    name: String,
    fg: Option<(u8, u8, u8)>,
    bg: Option<(u8, u8, u8)>,
    //styles: TextStyles,
    //styles: HashMap<String, bool>,
    styles: Vec<(String, bool)>,
}

impl Default for ConfigColorSpec {
    fn default() -> Self {
        Self {
            name: "default".into(),
            fg: None,
            bg: None,
            //styles: TextStyles::default(),
            //styles: HashMap::from([("bold".into(), false), ("unbderline".into(), false)]),
            styles: vec![
                ("bold".into(), false), ("intense".into(), false), ("underline".into(), false),
                ("dimmed".into(), false), ("italic".into(), false), ("reset".into(), false), ("strikethrough".into(), false)
            ],
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ColorsConfig {
    match_color_spec: ConfigColorSpec,
    text_color_spec: ConfigColorSpec,
    info_color_spec: ConfigColorSpec,
    extra_color_spec: ConfigColorSpec,

}

const CONFIG_FOLDER_NAME: &str = "pdf_grep";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TomlSettings {
    //colors: ColorsConfig,
    xpdf_tools_folder: Option<PathBuf>,
    colors: Vec<ConfigColorSpec>,
}

impl TomlSettings {
   pub fn load() -> Result<TomlSettings, BoxError> {
        if let Some(proj_dirs) = ProjectDirs::from("", "",  CONFIG_FOLDER_NAME) {
            let mut toml_path = PathBuf::from(proj_dirs.config_dir());
            toml_path.push(CONFIG_FILE_NAME);
            let mut read_data = String::new();
            let mut read_file = File::open(&toml_path)?;
            read_file.read_to_string(&mut read_data)?;

            let loaded_config: TomlSettings = toml::from_str(read_data.as_str())?;
            Ok(loaded_config)
        } else {
            Err("Error loading config file".into())
        }


    }
    fn create_default() -> Result<(), BoxError> {
        let config = TomlSettings {
            xpdf_tools_folder: None,
            colors: vec![
                ConfigColorSpec { name: "match".into(), fg: Some((255, 197, 12)), bg: None,
                styles: vec![
                    ("bold".into(), true), ("intense".into(), true), ("underline".into(), true),
                    ("dimmed".into(), false), ("italic".into(), false), ("reset".into(), false), ("strikethrough".into(), false)
                ]
                },
                ConfigColorSpec { name: "text".into(), fg: Some((249, 246, 238)), bg: None,
                    ..Default::default()
                },
                ConfigColorSpec { name: "info".into(), fg: Some((52, 154, 179)), bg: None,
                    ..Default::default()
                },
                ConfigColorSpec { name: "extra".into(), fg: Some((181, 235, 18)), bg: None,
                    ..Default::default()
                }
            ],
        };

        let mut toml_path = PathBuf::new();
    
        if let Some(proj_dirs) = ProjectDirs::from("", "",  CONFIG_FOLDER_NAME) {
            
            toml_path.push(proj_dirs.config_dir());
            fs::create_dir_all(&toml_path)?;
            toml_path.push(CONFIG_FILE_NAME);

            let toml = toml::to_string(&config)?;
    
            let mut save_file = File::create(&toml_path)?;
            save_file.write_all(toml.as_bytes())?;
            println!("Config toml created at: {:?}", toml_path);

        } else {
            return Err("Could not create config file.".into());
        }
        Ok(())
    }

}