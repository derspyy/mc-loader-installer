use super::super::Error;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;
use ureq::{Agent, AgentBuilder};

const FABRIC_META: &str = "https://meta.fabricmc.net/v2/versions";
const FABRIC_ICON: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAACABAMAAAAxEHz4AAAAGFBMVEUAAAA4NCrb0LTGvKW8spyAem2uppSakn5SsnMLAAAAAXRSTlMAQObYZgAAAJ5JREFUaIHt1MENgCAMRmFWYAVXcAVXcAVXcH3bhCYNkYjcKO8dSf7v1JASUWdZAlgb0PEmDSMAYYBdGkYApgf8ER3SbwRgesAf0BACMD1gB6S9IbkEEBfwY49oNj4lgLhA64C0o9R9RABTAvp4SX5kB2TA5y8EEAK4pRrxB9QcA4QBWkj3GCAMUCO/xwBhAI/kEsCagCHDY4AwAC3VA6t4zTAMj0OJAAAAAElFTkSuQmCC";

pub fn install(
    minecraft_version: String,
    loader_version_opt: Option<String>,
    mut minecraft_directory: PathBuf,
) -> Result<(), Box<dyn error::Error>> {
    let agent = AgentBuilder::new().build();
    let loader_version = match loader_version_opt {
        Some(value) => value,
        None => get_latest(&agent)?,
    };
    let json_data = agent
        .get(format!("{FABRIC_META}/loader/{minecraft_version}/{loader_version}").as_str())
        .call()?;
    let version_name = format!("fabric-loader-{loader_version}-{minecraft_version}");
    let short_version_name = format!("fabric-loader-{minecraft_version}");
    write_json(
        minecraft_directory.clone(),
        version_name.clone(),
        short_version_name,
    )?;
    minecraft_directory.push("versions");
    minecraft_directory.push(&version_name);
    if minecraft_directory.exists() {
        return Ok(());
    }; // An early return is done (successfully) if the version is already found to exist.
    create_dir_all(&minecraft_directory)?;
    minecraft_directory.push(format!("{version_name}.json").as_str());
    File::create(&minecraft_directory)?.write_all(json_data.into_string()?.as_bytes())?;
    minecraft_directory.set_extension("jar");
    File::create(&minecraft_directory)?; // This empty file is created because it is expected by the launcher.

    Ok(())
}

#[derive(Deserialize)]
struct Loader {
    version: String,
}

fn get_latest(agent: &Agent) -> Result<String, Box<dyn error::Error>> {
    let request = agent
        .get(format!("{FABRIC_META}/loader").as_str())
        .call()?
        .into_string()?;
    let versions: Vec<Loader> = serde_json::from_str(&request)?;
    if versions.is_empty() {
        return Err(Box::new(Error::NoVersion));
    }
    Ok(versions[0].version.clone())
}

#[derive(Deserialize, Serialize, Debug)]
struct LauncherProfiles {
    profiles: HashMap<String, Profile>,
    settings: Value,
    version: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Profile {
    name: String,
    last_used: String,
    last_version_id: String,
    created: String,
    icon: String,
    #[serde(rename = "type")]
    _type: String,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl Profile {
    fn new() -> Profile {
        Profile {
            name: String::new(),
            last_used: String::new(),
            last_version_id: String::new(),
            created: String::new(),
            icon: String::new(),
            _type: String::new(),
            extra: HashMap::new(),
        }
    }
}

fn write_json(
    mut directory: PathBuf,
    version_name: String,
    short_version_name: String,
) -> Result<(), Box<dyn error::Error>> {
    directory.push("launcher_profiles.json");
    let current_time = Utc::now().to_string();
    let mut read_data: LauncherProfiles = serde_json::from_str(&read_to_string(&directory)?)?;
    let mut new_profile = read_data
        .profiles
        .get(&short_version_name)
        .cloned()
        .unwrap_or_else(Profile::new);
    new_profile.name = short_version_name.clone();
    new_profile.last_used = current_time.clone();
    new_profile.last_version_id = version_name;
    new_profile.created = current_time;
    new_profile.icon = FABRIC_ICON.to_string();
    new_profile._type = "custom".to_string();
    read_data.profiles.insert(short_version_name, new_profile);
    let mut file = File::create(directory)?;
    file.write_all(serde_json::to_string(&read_data)?.as_bytes())?;
    Ok(())
}
