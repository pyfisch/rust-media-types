#![deny(missing_docs)]

//! Media Types also known as MIME types describe the nature of data.
//!
//! There are many RFCs describing media types the two most important for this crate is
//! [RFC 2046 - Multipurpose Internet Mail Extensions (MIME) Part Two: Media Types]
//! (https://tools.ietf.org/html/rfc2046).

extern crate charsets;

use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::error::Error as ErrorTrait;
use std::fmt::{self, Display, Formatter};
use std::str::{FromStr, Utf8Error, from_utf8};
use std::string::FromUtf8Error;

pub use type_::*;
pub use tree::*;
pub use charsets::Charset;

mod utils;

#[derive(Debug, Eq, PartialEq)]
/// Defines an Error type for media types.
pub enum Error {
    /// Parsing the given string as a media type failed.
    Invalid,
    /// The media type does not have this parameter.
    NotFound,
    /// Decoding a string as UTF-8 (or ASCII) failed.
    Utf8Error(Utf8Error)
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        return "TODO"
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl From<charsets::Error> for Error {
    fn from(_: charsets::Error) -> Error {
        Error::Invalid
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::Utf8Error(err.utf8_error())
    }
}

/// Result type used for this library.
pub type Result<T> = ::std::result::Result<T, Error>;

/// A Media Type commonly used to describe the contents of a resource.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct MediaType {
    /// The top-level type or `None` to match all types.
    pub type_: Option<String>,
    /// A registration tree, the standards tree uses `None`.
    pub tree: Option<String>,
    /// A subtype giving describing a concrete file format.
    pub subtype: Option<String>,
    /// Some types use a suffix to refer to the base format like XML or JSON.
    pub suffix: Option<String>,
    /// Media types can contain optional parameters for example for charsets or video codes.
    pub parameters: HashMap<String, String>
}

fn u<'a>(x: &'a Option<String>) -> Option<&'a str> {
    x.as_ref().map(|x| &x[..])
}

impl MediaType {
    /// Creates a new media type without parameters, they can be added later.
    pub fn new(type_: Option<&str>,
               tree: Option<&str>,
               subtype: Option<&str>,
               suffix: Option<&str>) -> MediaType {
        MediaType {
            type_: type_.map(|x| x.to_string()),
            tree: tree.map(|x| x.to_string()),
            subtype: subtype.map(|x| x.to_string()),
            suffix: suffix.map(|x| x.to_string()),
            parameters: HashMap::new()
        }
    }

    /// The boundary parameter is used to separate different blocks of multipart resources.
    ///
    /// It is defined in [RFC2046 - Multipurpose Internet Mail Extensions (MIME) Part Two:
    /// Media Types #5.1.  Multipart Media Type](https://tools.ietf.org/html/rfc2046#section-5.1).
    pub fn boundary(&self) -> Result<&str> {
        let boundary = try!(self.parameters.get("boundary").ok_or(Error::NotFound));
        if !utils::boundary(boundary) {
            return Err(Error::Invalid);
        }
        Ok(&boundary[..])
    }

    /// The charset parameter is defined for `text/*` types, it carries information about the
    /// charset.
    ///
    /// The relevant RFCs are [RFC2046 - Multipurpose Internet Mail Extensions (MIME) Part Two:
    /// Media Types #4.1.2. Charset Parameter](https://tools.ietf.org/html/rfc2046#section-4.1.2)
    /// and [RFC6657 - Update to MIME regarding "charset" Parameter Handling in Textual Media Types]
    /// (https://tools.ietf.org/html/rfc6657).
    pub fn charset(&self) -> Result<Charset> {
        let charset = try!(self.parameters.get("charset").ok_or(Error::NotFound));
        Ok(try!(charset.parse()))
    }

    /// Compares the mime type portion (the media type without parameters) of two media types.
    pub fn eq_mime_portion(&self, other: &MediaType) -> bool {
        self.type_ == other.type_
        && self.tree == other.tree
        && self.subtype == other.subtype
        && self.suffix == other.suffix
    }

    /// Returns true if the mime type portions differ, strict inverse of `eq_mime_portion()`.
    pub fn ne_mime_portion(&self, other: &MediaType) -> bool {
        !self.eq_mime_portion(other)
    }

    /// Checks if the media type is an image type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_image_type(&self) -> bool {
        u(&self.type_) == IMAGE
    }

    /// Checks if the media type is an audio or video type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_audio_or_video_type(&self) -> bool {
        u(&self.type_) == AUDIO
        || u(&self.type_) == VIDEO
        || MediaType::new(APPLICATION, STANDARDS, Some("ogg"), None).eq_mime_portion(self)
    }

    /// Checks if the media type is a font type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_font_type(&self) -> bool {
        self == &MediaType::new(APPLICATION, STANDARDS, Some("font-ttf"), None)
        || [
            MediaType::new(APPLICATION, STANDARDS, Some("font-cff"), None),
            MediaType::new(APPLICATION, STANDARDS, Some("font-off"), None),
            MediaType::new(APPLICATION, STANDARDS, Some("font-sfnt"), None),
            MediaType::new(APPLICATION, VENDOR, Some("ms-opentype"), None),
            MediaType::new(APPLICATION, STANDARDS, Some("font-woff"), None),
            MediaType::new(APPLICATION, VENDOR, Some("ms-fontobject"), None)
        ].iter().any(|x| x.eq_mime_portion(self))
    }

    /// Checks if the media type is a zip based type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_zip_based_type(&self) -> bool {
        u(&self.suffix) == Some("zip")
        || MediaType::new(APPLICATION, STANDARDS, Some("zip"), None).eq_mime_portion(self)
    }

    /// Checks if the media type is an archive type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_archive_type(&self) -> bool {
        self == &MediaType::new(APPLICATION, STANDARDS, Some("x-rar-compressed"), None)
        || [
            MediaType::new(APPLICATION, STANDARDS, Some("zip"), None),
            MediaType::new(APPLICATION, STANDARDS, Some("x-gzip"), None)
        ].iter().any(|x| x.eq_mime_portion(self))
    }

    /// Checks if the media type is an XML type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_xml_type(&self) -> bool {
        u(&self.suffix) == Some("xml")
        || [
            MediaType::new(TEXT, STANDARDS, Some("xml"), None),
            MediaType::new(APPLICATION, STANDARDS, Some("xml"), None)
        ].iter().any(|x| x.eq_mime_portion(self))
    }

    /// Checks if the media type is a scriptable type, HTML or PDF.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_scriptable_mime_type(&self) -> bool {
        [
            MediaType::new(TEXT, STANDARDS, Some("html"), None),
            MediaType::new(APPLICATION, STANDARDS, Some("pdf"), None)
        ].iter().any(|x| x.eq_mime_portion(self))
    }
}

/// top-level type name / [ tree. ] subtype name [ +suffix ] [ ; parameters ]
impl FromStr for MediaType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::Invalid)
        }
        let mut media_type: MediaType = Default::default();
        let s = s.trim();
        let mut parts = s.splitn(2, ';');
        let mime_type_portion = try!(parts.next().ok_or(Error::Invalid));
        let parameters_portion = parts.next();
        let mut parts = mime_type_portion.splitn(2, '/');
        media_type.type_ = Some(try!(parts.next().ok_or(Error::Invalid)).to_ascii_lowercase())
            .and_then(parse_wildcard);
        let subtype_portion = try!(parts.next().ok_or(Error::Invalid));
        let suffixed_portion = if subtype_portion.contains('.') {
            let mut parts = subtype_portion.splitn(2, '.');
            media_type.tree = Some(parts.next().unwrap().to_ascii_lowercase());
            parts.next().unwrap()
        } else {
            subtype_portion
        };
        media_type.subtype = Some(if suffixed_portion.contains('+') {
            let mut parts = suffixed_portion.rsplitn(2, '+');
            media_type.suffix = Some(parts.next().unwrap().to_ascii_lowercase());
            parts.next().unwrap()
        } else {
            suffixed_portion
        }.to_ascii_lowercase()).and_then(parse_wildcard);
        if let Some(parameters_portion) = parameters_portion {
            for (key, value) in try!(parameters_portion.split(';').map(|x| {
                let mut parts = x.splitn(2, '=');
                let key = try!(parts.next().map(|x| x.trim()).ok_or(Error::Invalid));
                let value = try!(parts.next().map(utils::unquote_string).ok_or(Error::Invalid));
                decode_param(key, value)
            }).collect::<Result<Vec<(&str, String)>>>()) {
                media_type.parameters.insert(key.to_ascii_lowercase(), value);
            }
        }
        return Ok(media_type);

        fn decode_param<'a>(key: &'a str, value: &'a str) -> Result<(&'a str, String)> {
            Ok(if key.ends_with("*") {
                (&key[0..key.len() -1],
                try!(try!(String::from_utf8(utils::percent_decode(value.as_bytes())))
                    .splitn(3, '\'').nth(2)
                    .ok_or(Error::Invalid)).to_owned())
            } else {
                (key, value.to_owned())
            })
        }

        fn parse_wildcard(s: String) -> Option<String> {
            if s != "*" {
                Some(s)
            } else {
                None
            }
        }
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}/", self.type_.as_ref().map(|x| &x[..]).unwrap_or("*")));
        if let Some(ref tree) = self.tree {
            try!(write!(f, "{}.", tree));
        }
        try!(f.write_str(self.subtype.as_ref().map(|x| &x[..]).unwrap_or("*")));
        if let Some(ref suffix) = self.suffix {
            try!(write!(f, "+{}", suffix));
        }
        let mut items: Vec<(&String, &String)> = self.parameters.iter().collect();
        items.sort_by(|&(ref first, _), &(ref second, _)| first.cmp(second));
        for (ref key, ref value) in items {
            if utils::token(&value) {
                try!(write!(f, "; {}={}", key, value));
            } else {
                try!(write!(f, "; {}=\"{}\"", key, value));
            }
        };
        Ok(())
    }
}

/// Provides the five discrete and the two composite top-level media types.
pub mod type_ {
    /// The "text" top-level type is intended for sending material that is
    /// principally textual in form.
    pub const TEXT: Option<&'static str> = Some("text");
    /// A top-level type of "image" indicates that the content specifies one
    /// or more individual images.
    pub const IMAGE: Option<&'static str> = Some("image");
    /// A top-level type of "audio" indicates that the content contains audio data.
    pub const AUDIO: Option<&'static str> = Some("audio");
    /// A top-level type of "video" indicates that the content specifies a
    /// time-varying-picture image, possibly with color and coordinated sound.
    pub const VIDEO: Option<&'static str> = Some("video");
    /// The "application" top-level type is to be used for discrete data that
    /// do not fit under any of the other type names, and particularly for
    /// data to be processed by some type of application program.
    pub const APPLICATION: Option<&'static str> = Some("application");
    /// The "multipart" top-level type is to be used for data consisting of multiple
    /// entities of independent data types..
    pub const MULTIPART: Option<&'static str> = Some("multipart");
    /// A body of media type "message" is itself all or a portion of some
    /// kind of message object.
    pub const MESSAGE: Option<&'static str> = Some("message");
}

/// Provides the four registration trees.
pub mod tree {
    /// The standards tree is intended for types of general interest to the Internet community.
    pub const STANDARDS: Option<&'static str> = None;
    /// The vendor tree is used for media types associated with publicly available products.
    pub const VENDOR: Option<&'static str> = Some("vnd");
    /// Registrations for media types created experimentally or as part of
    /// products that are not distributed commercially may be registered in
    /// the personal or vanity tree.
    pub const PERSONAL: Option<&'static str> = Some("prs");
    /// Subtype names with "x." as the first facet may be used for types intended exclusively for
    /// use in private, local environments.
    pub const UNREGISTERED: Option<&'static str> = Some("x");
}
