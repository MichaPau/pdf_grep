use std::{fs::{File, self}, path::PathBuf};
use std::io::{Write, Read};
use directories::ProjectDirs;
use serde::{Serialize, Deserialize};


use crate::{pdf_tools::AvailablePdfTools, BoxError};


#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ConfigColorSpec {
    pub name: String,
    pub fg: Option<(u8, u8, u8)>,
    pub bg: Option<(u8, u8, u8)>,
    
    pub styles: Vec<(String, bool)>,
}

impl Default for ConfigColorSpec {
    fn default() -> Self {
        Self {
            name: "default".into(),
            fg: None,
            bg: None,
            styles: vec![
                ("bold".into(), false), ("intense".into(), false), ("underline".into(), false),
                ("dimmed".into(), false), ("italic".into(), false), ("reset".into(), true), ("strikethrough".into(), false)
            ],
        }
    }
}


const CONFIG_FOLDER_NAME: &str = "pdf_grep";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TomlSettings {
    //colors: ColorsConfig,
    pub use_pdf_tool: AvailablePdfTools,
    pub xpdf_tools_folder: Option<PathBuf>,
    pub colors: Vec<ConfigColorSpec>,
}

impl TomlSettings {
   pub fn load() -> Result<TomlSettings, BoxError> {
        if let Some(proj_dirs) = ProjectDirs::from("", "",  CONFIG_FOLDER_NAME) {
            let mut toml_path = PathBuf::from(proj_dirs.config_dir());
            toml_path.push(CONFIG_FILE_NAME);
            println!("Load config file from:{:?}", toml_path);
            let mut read_data = String::new();
            let mut read_file = File::open(&toml_path)?;
            read_file.read_to_string(&mut read_data)?;

            let loaded_config: TomlSettings = toml::from_str(read_data.as_str())?;
            Ok(loaded_config)
        } else {
            Err("Error loading config file".into())
        }


    }
    pub fn create_default() -> Result<TomlSettings, BoxError> {
        let config = TomlSettings {
            xpdf_tools_folder: Some(PathBuf::from("./")),
            use_pdf_tool: AvailablePdfTools::UseXpdfTools,
            colors: vec![
                ConfigColorSpec { name: "match".into(), fg: Some((255, 197, 12)), bg: None,
                styles: vec![
                    ("bold".into(), true), ("intense".into(), true), ("underline".into(), true),
                    ("dimmed".into(), false), ("italic".into(), false), ("reset".into(), true), ("strikethrough".into(), false)
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
        Ok(config)
    }

}

#[derive(Serialize, Deserialize)]
pub struct TextStyles {
    bold: bool,
    underline: bool,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path::PathBuf};

    use serde::{Serialize, Deserialize};
    
    #[derive(Serialize, Deserialize)]
    
    struct TestSettings {
        pub xpdf_tools_folder: Option<PathBuf>,
        
        pub colors: Vec<TestConfigColorSpec>,
    }
    #[derive(Serialize, Deserialize)]
    struct TestConfigColorSpec {
        pub name: String,
        pub fg: Option<(u8, u8, u8)>,
        pub bg: Option<(u8, u8, u8)>,
        
        #[serde(flatten)]
        pub styles: TestTextStyles,
    }
    #[derive(Serialize, Deserialize)]
    pub struct TestTextStyles {
        bold: bool,
        underline: bool,
    }
    #[ignore]
    #[test]
    fn test_enum_look() {
        let config = TestSettings {
            xpdf_tools_folder: Some(PathBuf::from("dir/some")),
            colors: vec![
                TestConfigColorSpec {
                    name: "test11".into(),
                    fg: Some((12, 12, 12)),
                    bg: None,
                    styles: TestTextStyles {
                        bold: true,
                        underline: false,
                    }
                },
                TestConfigColorSpec {
                    name: "test2".into(),
                    fg: None,
                    bg: None,
                    styles: TestTextStyles {
                        bold: false,
                        underline: false,
                    }
                },
            ],
        };
        let mut toml = toml::to_string(&config).unwrap();
        toml.insert_str(0, "# add a comment line\n");
        println!("{:?}", toml);
        let toml_path = PathBuf::from("./test_config.toml");
        let mut save_file = File::create(&toml_path).unwrap();
        save_file.write_all(toml.as_bytes()).unwrap();

    }
}
