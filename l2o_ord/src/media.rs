use core::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

use brotli::enc::backward_references::BrotliEncoderMode::BROTLI_MODE_FONT;
use brotli::enc::backward_references::BrotliEncoderMode::BROTLI_MODE_GENERIC;
use brotli::enc::backward_references::BrotliEncoderMode::BROTLI_MODE_TEXT;
use brotli::enc::backward_references::BrotliEncoderMode::{self};
use mp4::MediaType;
use mp4::Mp4Reader;
use mp4::TrackType;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Media {
    Audio,
    Code(Language),
    Font,
    Iframe,
    Image,
    Markdown,
    Model,
    Pdf,
    Text,
    Unknown,
    Video,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Language {
    Css,
    JavaScript,
    Json,
    Python,
    Yaml,
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Css => "css",
                Self::JavaScript => "javascript",
                Self::Json => "json",
                Self::Python => "python",
                Self::Yaml => "yaml",
            }
        )
    }
}

impl Media {
    #[rustfmt::skip]
    const TABLE: &'static [(&'static str, BrotliEncoderMode, Media, &'static [&'static str])] = &[
    ("application/cbor",            BROTLI_MODE_GENERIC, Media::Unknown,                    &["cbor"]),
    ("application/json",            BROTLI_MODE_TEXT,    Media::Code(Language::Json),       &["json"]),
    ("application/octet-stream",    BROTLI_MODE_GENERIC, Media::Unknown,                    &["bin"]),
    ("application/pdf",             BROTLI_MODE_GENERIC, Media::Pdf,                        &["pdf"]),
    ("application/pgp-signature",   BROTLI_MODE_TEXT,    Media::Text,                       &["asc"]),
    ("application/protobuf",        BROTLI_MODE_GENERIC, Media::Unknown,                    &["binpb"]),
    ("application/x-javascript",    BROTLI_MODE_TEXT,    Media::Code(Language::JavaScript), &[]),
    ("application/yaml",            BROTLI_MODE_TEXT,    Media::Code(Language::Yaml),       &["yaml", "yml"]),
    ("audio/flac",                  BROTLI_MODE_GENERIC, Media::Audio,                      &["flac"]),
    ("audio/mpeg",                  BROTLI_MODE_GENERIC, Media::Audio,                      &["mp3"]),
    ("audio/wav",                   BROTLI_MODE_GENERIC, Media::Audio,                      &["wav"]),
    ("font/otf",                    BROTLI_MODE_GENERIC, Media::Font,                       &["otf"]),
    ("font/ttf",                    BROTLI_MODE_GENERIC, Media::Font,                       &["ttf"]),
    ("font/woff",                   BROTLI_MODE_GENERIC, Media::Font,                       &["woff"]),
    ("font/woff2",                  BROTLI_MODE_FONT,    Media::Font,                       &["woff2"]),
    ("image/apng",                  BROTLI_MODE_GENERIC, Media::Image,                      &["apng"]),
    ("image/avif",                  BROTLI_MODE_GENERIC, Media::Image,                      &[]),
    ("image/gif",                   BROTLI_MODE_GENERIC, Media::Image,                      &["gif"]),
    ("image/jpeg",                  BROTLI_MODE_GENERIC, Media::Image,                      &["jpg", "jpeg"]),
    ("image/png",                   BROTLI_MODE_GENERIC, Media::Image,                      &["png"]),
    ("image/svg+xml",               BROTLI_MODE_TEXT,    Media::Iframe,                     &["svg"]),
    ("image/webp",                  BROTLI_MODE_GENERIC, Media::Image,                      &["webp"]),
    ("model/gltf+json",             BROTLI_MODE_TEXT,    Media::Model,                      &["gltf"]),
    ("model/gltf-binary",           BROTLI_MODE_GENERIC, Media::Model,                      &["glb"]),
    ("model/stl",                   BROTLI_MODE_GENERIC, Media::Unknown,                    &["stl"]),
    ("text/css",                    BROTLI_MODE_TEXT,    Media::Code(Language::Css),        &["css"]),
    ("text/html",                   BROTLI_MODE_TEXT,    Media::Iframe,                     &[]),
    ("text/html;charset=utf-8",     BROTLI_MODE_TEXT,    Media::Iframe,                     &["html"]),
    ("text/javascript",             BROTLI_MODE_TEXT,    Media::Code(Language::JavaScript), &["js"]),
    ("text/markdown",               BROTLI_MODE_TEXT,    Media::Markdown,                   &[]),
    ("text/markdown;charset=utf-8", BROTLI_MODE_TEXT,    Media::Markdown,                   &["md"]),
    ("text/plain",                  BROTLI_MODE_TEXT,    Media::Text,                       &[]),
    ("text/plain;charset=utf-8",    BROTLI_MODE_TEXT,    Media::Text,                       &["txt"]),
    ("text/x-python",               BROTLI_MODE_TEXT,    Media::Code(Language::Python),     &["py"]),
    ("video/mp4",                   BROTLI_MODE_GENERIC, Media::Video,                      &["mp4"]),
    ("video/webm",                  BROTLI_MODE_GENERIC, Media::Video,                      &["webm"]),
  ];

    pub fn content_type_for_path(
        path: &Path,
    ) -> Result<(&'static str, BrotliEncoderMode), anyhow::Error> {
        let extension = path
            .extension()
            .ok_or_else(|| anyhow::anyhow!("file must have extension"))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("unrecognized extension"))?;

        let extension = extension.to_lowercase();

        if extension == "mp4" {
            Media::check_mp4_codec(path)?;
        }

        for (content_type, mode, _, extensions) in Self::TABLE {
            if extensions.contains(&extension.as_str()) {
                return Ok((*content_type, *mode));
            }
        }

        let mut extensions = Self::TABLE
            .iter()
            .flat_map(|(_, _, _, extensions)| extensions.first().cloned())
            .collect::<Vec<&str>>();

        extensions.sort();

        Err(anyhow::anyhow!(
            "unsupported file extension `.{extension}`, supported extensions: {}",
            extensions.join(" "),
        ))
    }

    pub fn check_mp4_codec(path: &Path) -> Result<(), anyhow::Error> {
        let f = File::open(path)?;
        let size = f.metadata()?.len();
        let reader = BufReader::new(f);

        let mp4 = Mp4Reader::read_header(reader, size)?;

        for track in mp4.tracks().values() {
            if let TrackType::Video = track.track_type()? {
                let media_type = track.media_type()?;
                if media_type != MediaType::H264 {
                    return Err(anyhow::anyhow!(
                        "Unsupported video codec, only H.264 is supported in MP4: {media_type}"
                    ));
                }
            }
        }

        Ok(())
    }
}

impl FromStr for Media {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for entry in Self::TABLE {
            if entry.0 == s {
                return Ok(entry.2);
            }
        }

        Err(anyhow::anyhow!("unknown content type: {s}"))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::assert_regex_match;

    #[test]
    fn for_extension() {
        assert_eq!(
            Media::content_type_for_path(Path::new("pepe.jpg")).unwrap(),
            ("image/jpeg", BrotliEncoderMode::BROTLI_MODE_GENERIC)
        );
        assert_eq!(
            Media::content_type_for_path(Path::new("pepe.jpeg")).unwrap(),
            ("image/jpeg", BrotliEncoderMode::BROTLI_MODE_GENERIC)
        );
        assert_eq!(
            Media::content_type_for_path(Path::new("pepe.JPG")).unwrap(),
            ("image/jpeg", BrotliEncoderMode::BROTLI_MODE_GENERIC)
        );
        assert_eq!(
            Media::content_type_for_path(Path::new("pepe.txt")).unwrap(),
            (
                "text/plain;charset=utf-8",
                BrotliEncoderMode::BROTLI_MODE_TEXT
            )
        );
        assert_regex_match!(
            Media::content_type_for_path(Path::new("pepe.foo")).unwrap_err(),
            r"unsupported file extension `\.foo`, supported extensions: apng .*"
        );
    }

    #[test]
    fn no_duplicate_extensions() {
        let mut set = HashSet::new();
        for (_, _, _, extensions) in Media::TABLE {
            for extension in *extensions {
                assert!(set.insert(extension), "duplicate extension `{extension}`");
            }
        }
    }
}
