use std::io::Write;
use std::path::PathBuf;
use std::fs::{File, create_dir_all};
use std::error;
use super::super::Error;
use ureq::{Agent, AgentBuilder};
use miniserde::{Deserialize, json};

const FABRIC_META: &str = "https://meta.fabricmc.net/v2/versions";

pub fn install(minecraft_version: String, loader_version_opt: Option<String>, mut minecraft_directory: PathBuf) -> Result<(), Box<dyn error::Error>> {
    let agent = AgentBuilder::new().build();
    let loader_version = match loader_version_opt {
        Some(value) => value,
        None => get_latest(&agent)?,
    };
    let json_data = agent.get(format!("{FABRIC_META}/loader/{minecraft_version}/{loader_version}").as_str())
        .call()?;
    minecraft_directory.push("versions");
    minecraft_directory.push(format!("fabric-loader-{loader_version}-{minecraft_version}").as_str());
    if minecraft_directory.exists() { return Ok(()) }; // An early return is done (successfully) if the version is already found to exist.
    create_dir_all(&minecraft_directory)?;
    minecraft_directory.push(format!("fabric-loader-{loader_version}-{minecraft_version}.json").as_str());
    File::create(&minecraft_directory)?.write_all(json_data.into_string()?.as_bytes())?;
    minecraft_directory.set_extension("jar");
    File::create(&minecraft_directory)?; // This empty file is created because it is expected by the launcher.
    
    Ok(())
}

#[derive(Deserialize)]
struct Loader {
    version: String,
}

fn get_latest(agent: &Agent) -> Result<String, Box<dyn error::Error>>{
    let request = agent.get(format!("{FABRIC_META}/loader").as_str())
        .call()?
        .into_string()?;
    let versions: Vec<Loader> = json::from_str(&request)?;
    if versions.is_empty() { return Err(Box::new(Error::NoVersion)) }
    Ok(versions[0].version.clone())
}