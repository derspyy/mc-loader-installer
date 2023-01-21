use home::home_dir;
use std::error;
use std::path::PathBuf;
use thiserror::Error;

mod installers;
use installers::*;

// I love this crate oh my god
#[derive(Error, Debug)]
pub enum Error {
    #[error("Default directory not found.")]
    NoDirectory,
    #[error("Selected version was not found")]
    NoVersion,
}

pub enum Loader {
    Fabric,
    Quilt,
}

pub struct Installer {
    minecraft_version: String,
    loader: Loader,
    loader_version: Option<String>, // "None" for latest, "Some(value)" for a specific version.
    minecraft_location: Option<PathBuf>, // "None" for Vanilla Launcher location, "Some(value)" for a specific path.
}

impl Installer {
    pub fn install(self) -> Result<(), Box<dyn error::Error>> {
        let minecraft_directory = self.minecraft_location.unwrap_or(get_directory()?);
        match self.loader {
            Loader::Fabric => {
                fabric::install(
                    self.minecraft_version,
                    self.loader_version,
                    minecraft_directory,
                )?;
            }
            Loader::Quilt => {
                quilt::install(
                    self.minecraft_version,
                    self.loader_version,
                    minecraft_directory,
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn get_directory() -> Result<PathBuf, Box<dyn error::Error>> {
    let mut dir = home_dir().ok_or(Error::NoDirectory)?;
    dir.push("AppData");
    dir.push("Roaming");
    dir.push(".minecraft");
    if !dir.exists() {
        return Err(Box::new(Error::NoDirectory));
    }
    Ok(dir)
}

#[cfg(target_os = "linux")]
fn get_directory() -> Result<PathBuf, Box<dyn error::Error>> {
    let mut dir = home_dir().ok_or(Error::NoDirectory)?;
    dir.push(".minecraft");
    if !dir.exists() {
        return Err(Box::new(Error::NoDirectory));
    }
    Ok(dir)
}

#[cfg(target_os = "macos")]
fn get_directory() -> Result<PathBuf, Box<dyn error::Error>> {
    let mut dir = home_dir().ok_or(Error::NoDirectory)?;
    dir.push("Library");
    dir.push("Application Support");
    dir.push("minecraft");
    if !dir.exists() {
        return Err(Box::new(Error::NoDirectory));
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installing() {
        let installer = Installer {
            minecraft_version: String::from("1.19.3"),
            loader: Loader::Quilt,
            loader_version: None,
            minecraft_location: None,
        };
        match installer.install() {
            Err(x) => panic!("{x}"),
            Ok(_) => println!("Done!"),
        }
    }
}
