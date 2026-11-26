// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Files in the local filesystem, named by a path, or in an object store such
//! as S3, named by a URL.
//!
//! Each object store takes its settings and credentials from the environment
//! variables that the `object_store` crate reads, such as `AWS_ENDPOINT`,
//! `AWS_ACCESS_KEY_ID`, and `AWS_SECRET_ACCESS_KEY` for S3 and the stores
//! compatible with it.  The functions here block until a transfer ends, so a
//! caller running in an async runtime must call them from a blocking task.

use std::{
    ffi::OsStr,
    fmt,
    fs::{self, File},
    io::{BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use anyhow::{Result, anyhow, bail, ensure};
use object_store::{
    ObjectStore, ObjectStoreExt, ObjectStoreScheme, aws::AmazonS3Builder,
    azure::MicrosoftAzureBuilder, buffered::BufWriter, gcp::GoogleCloudStorageBuilder,
    http::HttpBuilder,
};
use tokio::{io::AsyncWriteExt, runtime::Runtime};
use url::Url;

/// Runtime that drives the requests to object stores.  It lives as long as
/// geoai, since the connections a store keeps open belong to it.
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("cannot start the runtime for object stores")
});

#[derive(Clone, Debug)]
pub enum Location {
    File(PathBuf),
    Object {
        url: Url,
        store: Arc<dyn ObjectStore>,
        /// Path of the object within its bucket
        path: object_store::path::Path,
    },
}

/// File open for reading, which a format may read in any order.
pub trait Source: Read + Seek {}

impl<T: Read + Seek> Source for T {}

impl Location {
    /// Location written as `text`: a URL if it contains `://`, such as
    /// `s3://bucket/flood.tif`, and otherwise a local path.  A `file://` URL
    /// also names a local path.
    pub fn parse(text: &str) -> Result<Self> {
        if !text.contains("://") {
            return Ok(Self::File(text.into()));
        }
        let url = Url::parse(text)?;
        let (scheme, path) = ObjectStoreScheme::parse(&url)?;
        match scheme {
            ObjectStoreScheme::Local => url
                .to_file_path()
                .map(Self::File)
                .map_err(|()| anyhow!("{url} names no local path")),
            scheme => Ok(Self::Object {
                store: store(&scheme, &url)?,
                url,
                path,
            }),
        }
    }

    /// Open the file for reading.  An object is downloaded whole first.
    pub fn open(&self) -> Result<Box<dyn Source>> {
        Ok(match self {
            Self::File(path) => Box::new(BufReader::new(File::open(path)?)),
            Self::Object { store, path, .. } => {
                let bytes = RUNTIME.block_on(async { store.get(path).await?.bytes().await })?;
                Box::new(Cursor::new(bytes))
            }
        })
    }

    /// Write `bytes` as the whole file, replacing any file already there.  A
    /// large object is uploaded in parts.
    pub fn write(&self, bytes: Vec<u8>) -> Result<()> {
        match self {
            Self::File(path) => fs::write(path, bytes)?,
            Self::Object { store, path, .. } => RUNTIME.block_on(async {
                let mut upload = BufWriter::new(Arc::clone(store), path.clone());
                upload.put(bytes.into()).await?;
                upload.shutdown().await
            })?,
        }
        Ok(())
    }

    /// Ensure that the directory of a local file exists.  Whether an object
    /// can be written is only known once it is.
    pub fn check_writable(&self) -> Result<()> {
        if let Self::File(path) = self {
            let directory = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            ensure!(
                directory.is_dir(),
                "{} is not a directory",
                directory.display()
            );
        }
        Ok(())
    }

    pub fn extension(&self) -> Option<&str> {
        match self {
            Self::File(path) => path.extension().and_then(OsStr::to_str),
            Self::Object { path, .. } => path.extension(),
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => write!(formatter, "{}", path.display()),
            Self::Object { url, .. } => write!(formatter, "{url}"),
        }
    }
}

/// Store that holds the object at `url`, configured by the environment.
fn store(scheme: &ObjectStoreScheme, url: &Url) -> Result<Arc<dyn ObjectStore>> {
    Ok(match scheme {
        ObjectStoreScheme::AmazonS3 => {
            Arc::new(AmazonS3Builder::from_env().with_url(url.as_str()).build()?)
        }
        ObjectStoreScheme::GoogleCloudStorage => Arc::new(
            GoogleCloudStorageBuilder::from_env()
                .with_url(url.as_str())
                .build()?,
        ),
        ObjectStoreScheme::MicrosoftAzure => Arc::new(
            MicrosoftAzureBuilder::from_env()
                .with_url(url.as_str())
                .build()?,
        ),
        ObjectStoreScheme::Http => Arc::new(
            HttpBuilder::new()
                .with_url(&url[..url::Position::BeforePath])
                .build()?,
        ),
        _ => bail!("unsupported URL scheme `{}`", url.scheme()),
    })
}
