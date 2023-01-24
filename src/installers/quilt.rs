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

const QUILT_META: &str = "https://meta.quiltmc.org/v3/versions";
const QUILT_ICON: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAMAAAD04JH5AAAACXBIWXMAAAABAAAAAQBPJcTWAAAAsVBMVEX///8nov0zRP+XIv/cKd0nov0zRP+XIv8nov0zRP+XIv/cKd0nov0zRP+XIv/cKd2XIv/cKd0nov0zRP+XIv/cKd0nov2XIv/cKd0nov0zRP+XIv/cKd0nov0zRP+XIv/cKd0nov0zRP+XIv/cKd0nov0zRP+XIv/cKd0nov0zRP+XIv/cKd0nov0zRP+XIv/cKd0nov2XIv8nov0zRP+XIv/cKd0nov0zRP+XIv/cKd0/NEk8AAAAN3RSTlMAEBAQECAgIDAwMDBAQEBAUFBgYGBgcHBwgYGBgZGRkZGhoaGhsbGxscHBwcHR0dHR4eHx8fHxbU06QwAAA39JREFUeNrtmml3qjAQhkexWor7LrZ1qaIILtVCa///D7uOMQ3ewqB3qRHzfpqeg3mfyplJZiKAkpLSQblckdDDA3vKMEqEMhl8JpXKk5ITIJfzvC2pchmg0fgktVrhWvP5O6mnJxkBisVtjLpdANf9jBGu9R4j21YACuDaADyPlyFUVCquVqwMsVJEpaICuD4AlQUKQAEogO221SoWczl2JB2Nwq19v1IplXAtPHj+40J0YYD1WtPEk4YRDjCdBtcbDJIEgAiOU68D9Puu6/vRh3LXxbVsmz6Yyw7gecEWZb3+mzR8fRXxZnMtAA87ifTTNI5wPkCzmUpxhPFYHFLkBBCm+BePi0UAxxHNab9P2/s+fpp/2diI2zaLOx2ATodoTi8OAFAud/fCuHsQlt96HSO0R7Xbj4QMA5+5v+/sdXcHUK2yGGHyeYyazYgJycUBlBoNdy+M3YPwK8Xy67r9Pms4xmObULWKT+n6bC9dB3h+ZnGtBlCrYWRZ6bSMAGKTxb94jAcMXnzabYDxOO64hZ9+e/vYq1AAmM1YbJoApsliy5IRoFTipo2GaD9HI8Pg2+/joygrNMDHQcOhrnOYxSKbXSxYPJvJDRCl8wGilFwATERcC5PtFgFwwxXiKXdLAJsNbrbM3DR5+t0SgKoDCkABKIDBILjecHh7APM5v5gu7MSPX7cEkJwsWO0kBg7TaThApyOGToOBGMslASCzE48rFQCOEwTAwycfuKBpPn/9AJkMNwpvTn0fW/VqlQZgxYgqQihs1eUDUFLStOWSGtWyK3w+cMBEEs0Xbj98JBG9vvg5mJwAYlyv7RQE4GA4xOVtdxQAxtS/h9c+8gMsl9wS45eX4JWNGDwE28/hUBw/6NdLIEgEQF1e06OXcICgPYEgOQC/vNb1QoHeauLsIxGkBji+vNb10wHC7CMQpAYQl9d4/UCNn44BouxDESQH+JM0pOxDEBIHEGf/DSFhAKfY/4YgOcBk0u3i4aRWM01q9MAATrU/QpAawPOCpTibjQOIf51CjnMNAHg0O+cViJ+DxGsyuQ6Ac9OwXj/Nfrn8erkJAzgNIWB/FQCtFoBlnb4dxyEc2UsAANDrOYR6PXwmnbasGaHj8ROF8M1eAoD/oSiEH7KXACAc4QftJQD4jvDD9hIAHCNcwF4CAIFwIXsJABDBcSaTi9lTAL8AUD0mefbuyfMAAAAASUVORK5CYII=";

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
        .get(format!("{QUILT_META}/loader/{minecraft_version}/{loader_version}").as_str())
        .call()?;
    let version_name = format!("quilt-loader-{loader_version}-{minecraft_version}");
    let short_version_name = format!("quilt-loader-{minecraft_version}");
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
        .get(format!("{QUILT_META}/loader").as_str())
        .call()?
        .into_string()?;
    let versions: Vec<Loader> = serde_json::from_str(&request)?;
    let mut version = None;
    for x in versions {
        if !x.version.contains('-') {
            version = Some(x.version);
            break;
        }
    }
    Ok(version.ok_or(Error::NoVersion)?)
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
    extra: HashMap<String, Value>
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
    let mut new_profile = read_data.profiles
        .get(&short_version_name)
        .cloned()
        .unwrap_or_else(Profile::new);
    new_profile.name = short_version_name.clone();
    new_profile.last_used = current_time.clone();
    new_profile.last_version_id = version_name;
    new_profile.created = current_time;
    new_profile.icon = QUILT_ICON.to_string();
    new_profile._type = "custom".to_string();
    read_data.profiles.insert(short_version_name, new_profile);
    let mut file = File::create(directory)?;
    file.write_all(serde_json::to_string(&read_data)?.as_bytes())?;
    Ok(())
}
