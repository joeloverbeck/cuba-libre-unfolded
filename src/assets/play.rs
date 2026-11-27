use super::{CARD_BACK, CARD_FACES};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
use unfolded_assets::read_verified;

struct CardInput<'a> {
    name: &'a str,
    collection_dir: &'a str,
    length: u64,
    hash: &'a str,
}

impl CardInput<'_> {
    fn read(&self, root: &Path, relative: &str) -> Result<Vec<u8>> {
        read_verified(root, relative, self.length, self.hash).with_context(|| {
            format!(
                "Expected SHA-256 {} at {}",
                self.hash,
                root.join(relative).display()
            )
        })
    }

    fn check_installed(&self, cards: &Path) -> Result<()> {
        self.read(cards, self.name).with_context(|| {
            format!(
                "Installed card {} is incompatible. Move it aside and run scripts/play again.",
                cards.join(self.name).display()
            )
        })?;
        Ok(())
    }

    fn source_bytes(&self, source: &Path) -> Result<Vec<u8>> {
        let flat = source.join(self.name);
        if exists(&flat)? {
            return self.read(source, self.name);
        }
        self.read(source, &format!("{}/{}", self.collection_dir, self.name))
            .with_context(|| format!("Also looked for {}", flat.display()))
    }
}

/// Prepare the ordinary launch's private card directory from the same pinned
/// files that the tabletop admits. A supplied source replaces the remembered
/// path only after preparation succeeds.
pub fn prepare_play_cards(data: &Path, source: Option<&Path>) -> Result<()> {
    let inputs = CARD_FACES.iter().map(|face| CardInput {
        name: face.name,
        collection_dir: "images/cards",
        length: face.length,
        hash: face.hash,
    });
    let back = CardInput {
        name: CARD_BACK.name,
        collection_dir: "images/card-backs",
        length: CARD_BACK.length,
        hash: CARD_BACK.hash,
    };
    prepare(data, source, inputs.chain([back]))
}

fn prepare<'a>(
    data: &Path,
    source: Option<&Path>,
    inputs: impl IntoIterator<Item = CardInput<'a>>,
) -> Result<()> {
    let cards = data.join("cards");
    let config = data.join("card-source");
    check_card_directory(&cards)?;
    let mut missing = Vec::new();
    for card in inputs {
        if exists(&cards.join(card.name))? {
            card.check_installed(&cards)?;
        } else {
            missing.push(card);
        }
    }

    let explicit = source.map(resolve_source).transpose()?;
    if !missing.is_empty() {
        let names = missing
            .iter()
            .map(|card| card.name)
            .collect::<Vec<_>>()
            .join(", ");
        let source = match &explicit {
            Some(source) => source.clone(),
            None => fs::read_to_string(&config)
                .map(PathBuf::from)
                .and_then(|path| path.canonicalize())
                .with_context(|| format!(
                    "Missing private cards: {names}. Set the private source with scripts/play --card-source DIRECTORY (saved in {}).",
                    config.display()
                ))?,
        };
        let staged = missing
            .iter()
            .map(|card| card.source_bytes(&source).map(|bytes| (card, bytes)))
            .collect::<Result<Vec<_>>>()
            .with_context(|| format!(
                "Incompatible private card source {}. Supply the matching originals or run scripts/play --card-source DIRECTORY to select another source.",
                source.display()
            ))?;
        fs::create_dir_all(&cards)
            .with_context(|| format!("Cannot create card directory {}", cards.display()))?;
        check_card_directory(&cards)?;
        for (card, bytes) in staged {
            let path = cards.join(card.name);
            let temporary = write_temporary(&cards, &bytes)
                .with_context(|| format!("Cannot stage {}", path.display()))?;
            match temporary.persist_noclobber(&path) {
                Ok(_) => {}
                Err(error) if error.error.kind() == ErrorKind::AlreadyExists => {
                    card.check_installed(&cards)?
                }
                Err(error) => {
                    return Err(error.error)
                        .with_context(|| format!("Cannot install {}", path.display()));
                }
            }
        }
    }

    if let Some(source) = explicit {
        let source = source
            .to_str()
            .context("Use a private card source with a UTF-8 path")?;
        write_temporary(data, source.as_bytes())
            .and_then(|temporary| {
                temporary
                    .persist(&config)
                    .map(|_| ())
                    .map_err(|error| error.error)
            })
            .with_context(|| format!("Cannot remember the source in {}", config.display()))?;
    }
    Ok(())
}

fn check_card_directory(cards: &Path) -> Result<()> {
    match cards.symlink_metadata() {
        Ok(metadata) => ensure!(
            metadata.is_dir(),
            "The private card path {} must be a directory. Move it aside and run scripts/play again.",
            cards.display()
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("Cannot inspect {}", cards.display()));
        }
    }
    Ok(())
}

fn resolve_source(source: &Path) -> Result<PathBuf> {
    source
        .canonicalize()
        .and_then(|path| {
            if path.is_dir() {
                Ok(path)
            } else {
                Err(ErrorKind::NotADirectory.into())
            }
        })
        .with_context(|| format!("Cannot use {} as the private card source", source.display()))
}

fn exists(path: &Path) -> Result<bool> {
    match path.symlink_metadata() {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("Cannot inspect {}", path.display())),
    }
}

fn write_temporary(directory: &Path, bytes: &[u8]) -> std::io::Result<NamedTempFile> {
    let mut temporary = NamedTempFile::new_in(directory)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    Ok(temporary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::digest;
    use tempfile::tempdir;

    #[test]
    fn configured_source_supplies_a_newly_required_card_on_the_next_launch() {
        let home = tempdir().unwrap();
        let data = home.path().join("data");
        let source = home.path().join("my cards");
        fs::create_dir_all(source.join("images/cards")).unwrap();
        fs::write(source.join("card-19.png"), b"current").unwrap();
        fs::write(source.join("images/cards/card-20.png"), b"preview").unwrap();
        let current_hash = digest(b"current");
        let preview_hash = digest(b"preview");
        let current = || CardInput {
            name: "card-19.png",
            collection_dir: "images/cards",
            length: 7,
            hash: &current_hash,
        };
        let preview = CardInput {
            name: "card-20.png",
            hash: &preview_hash,
            ..current()
        };

        prepare(&data, Some(&source), [current()]).unwrap();
        let installed = data.join("cards/card-19.png");
        let original = fs::metadata(&installed).unwrap().modified().unwrap();
        prepare(&data, None, [current(), preview]).unwrap();
        assert_eq!(fs::read(&installed).unwrap(), b"current");
        assert_eq!(
            fs::metadata(&installed).unwrap().modified().unwrap(),
            original
        );
        assert_eq!(
            fs::read(data.join("cards/card-20.png")).unwrap(),
            b"preview"
        );
        assert_eq!(
            fs::read_to_string(data.join("card-source")).unwrap(),
            source.to_str().unwrap()
        );
    }

    #[test]
    fn refuses_incompatible_source_or_installed_bytes_without_replacing_files() {
        let home = tempdir().unwrap();
        let data = home.path().join("data");
        let source = home.path().join("source");
        fs::create_dir(&source).unwrap();
        let name = "cuba-libre-card-back.png";
        fs::write(source.join(name), b"wrong").unwrap();
        let hash = digest(b"right");
        let input = || CardInput {
            name,
            collection_dir: "images/card-backs",
            length: 5,
            hash: &hash,
        };

        let error = prepare(&data, Some(&source), [input()]).unwrap_err();
        assert!(format!("{error:#}").contains("Expected SHA-256"));
        assert!(!data.exists());
        fs::create_dir_all(data.join("cards")).unwrap();
        let installed = data.join("cards").join(name);
        fs::write(&installed, b"wrong").unwrap();
        fs::write(source.join(name), b"right").unwrap();
        let error = prepare(&data, Some(&source), [input()]).unwrap_err();
        assert!(format!("{error:#}").contains("Move it aside"));
        assert_eq!(fs::read(installed).unwrap(), b"wrong");
        assert!(!data.join("card-source").exists());

        fs::remove_file(data.join("cards").join(name)).unwrap();
        fs::remove_dir(data.join("cards")).unwrap();
        let unrelated = home.path().join("unrelated");
        fs::create_dir(&unrelated).unwrap();
        std::os::unix::fs::symlink(&unrelated, data.join("cards")).unwrap();
        let error = prepare(&data, Some(&source), [input()]).unwrap_err();
        assert!(format!("{error:#}").contains("must be a directory"));
        assert!(!unrelated.join(name).exists());
    }
}
