#![cfg_attr(feature = "dev", deny(missing_docs, missing_debug_implementations,
                                  missing_copy_implementations, trivial_casts,
                                  trivial_numeric_casts, unsafe_code,
                                  unused_import_braces, unused_qualifications,
                                  warnings))]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
//! Media Types also known as MIME types describe the nature of data they are
//! used in email to describe the file type of attachments and in HTTP to to
//! give
//! the type of a resource.
//!
//! There are many RFCs describing media types the two most important for this
//! crate is
//! [RFC 2046 - Multipurpose Internet Mail Extensions (MIME) Part Two: Media
//! Types]
//! (https://tools.ietf.org/html/rfc2046).

extern crate charsets;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::str::{FromStr, from_utf8};

pub use charsets::Charset;

pub use self::Type::{Application, Audio, Image, Message, Model, Multipart, Text, Video};
pub use self::Tree::{Personal, Private, Standards, Vendor};
pub use error::{Error, Result};

mod error;
mod utils;

/// A Media Type commonly used to describe the contents of a resource.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MediaType {
    /// The top-level type or `None` to match all types.
    pub type_: Option<Type>,
    /// A subtype describing the concrete file format. The first element of the tuple is the
    /// registration tree, it describes if they are registered by a standards organization,
    /// a vendor, or if they are only for private use. The second tuple element is the subtype,
    /// it describes the resource. The last part is the suffix it tells how the file was encoded
    /// common values are "xml" and "json".
    pub subtype: Option<(Tree, Cow<'static, str>, Option<Cow<'static, str>>)>,
    /// Media types can contain optional parameters for example for charsets or video codes.
    pub parameters: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

/// Provides the six discrete and the two composite top-level media types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    /// The "text" top-level type is intended for sending material that is
    /// principally textual in form.
    Text,
    /// A top-level type of "image" indicates that the content specifies one
    /// or more individual images.
    Image,
    /// A top-level type of "audio" indicates that the content contains audio data.
    Audio,
    /// A top-level type of "video" indicates that the content specifies a
    /// time-varying-picture image, possibly with color and coordinated sound.
    Video,
    /// The "application" top-level type is to be used for discrete data that
    /// do not fit under any of the other type names, and particularly for
    /// data to be processed by some type of application program.
    Application,
    /// The "multipart" top-level type is to be used for data consisting of multiple
    /// entities of independent data types.
    Multipart,
    /// A body of media type "message" is itself all or a portion of some
    /// kind of message object.
    Message,
    /// The "model" media type is used for 3D-models.
    Model,
    /// Less common top-level types.
    Unregistered(Cow<'static, str>),
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match *self {
            Text => "text",
            Image => "image",
            Audio => "audio",
            Video => "video",
            Application => "application",
            Multipart => "multipart",
            Message => "message",
            Model => "model",
            Type::Unregistered(ref string) => &string[..],
        })
    }
}

/// Provides the four registration trees.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tree {
    /// The standards tree is intended for types of general interest to the Internet community.
    Standards,
    /// The vendor tree is used for media types associated with publicly available products.
    Vendor,
    /// Registrations for media types created experimentally or as part of
    /// products that are not distributed commercially may be registered in
    /// the personal or vanity tree.
    Personal,
    /// Subtype names with "x." as the first facet may be used for types intended exclusively for
    /// use in private, local environments.
    Private,
    /// Other unofficial trees.
    Unregistered(Cow<'static, str>),
}

impl Display for Tree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Standards => Ok(()),
            Vendor => f.write_str("vnd."),
            Personal => f.write_str("prs."),
            Private => f.write_str("x."),
            Tree::Unregistered(ref string) => write!(f, "{}.", &string[..]),
        }
    }
}

impl MediaType {
    /// Creates the wildcard media type `*/*`.
    pub fn wildcard() -> MediaType {
        MediaType {
            type_: None,
            subtype: None,
            parameters: HashMap::new(),
        }
    }

    /// Creates a media type with only a concrete type and no subtype like `image/*`.
    pub fn wildcard_subtype(type_: Type) -> MediaType {
        MediaType {
            type_: Some(type_),
            subtype: None,
            parameters: HashMap::new(),
        }
    }

    /// Creates a new media type.
    pub fn new<A>(type_: Type, tree: Tree, subtype: A) -> MediaType
        where A: Into<Cow<'static, str>>
    {
        MediaType {
            type_: Some(type_),
            subtype: Some((tree, subtype.into(), None)),
            parameters: HashMap::new(),
        }
    }

    /// Creates a new media type with suffix.
    pub fn new_with_suffix<A, B>(type_: Type, tree: Tree, subtype: A, suffix: B) -> MediaType
        where A: Into<Cow<'static, str>>,
              B: Into<Cow<'static, str>>
    {
        MediaType {
            type_: Some(type_),
            subtype: Some((tree, subtype.into(), Some(suffix.into()))),
            parameters: HashMap::new(),
        }
    }

    /// Accesses the tree component of the subtype.
    pub fn tree(&self) -> Option<&Tree> {
        if let Some(ref subtype) = self.subtype {
            Some(&subtype.0)
        } else {
            None
        }
    }

    /// Accesses the sub component of the subtype containing the resource type.
    pub fn sub(&self) -> Option<&Cow<'static, str>> {
        if let Some(ref subtype) = self.subtype {
            Some(&subtype.1)
        } else {
            None
        }
    }

    /// Accesses the suffix of the type.
    pub fn suffix(&self) -> Option<&Cow<'static, str>> {
        if let Some(ref subtype) = self.subtype {
            if let Some(ref string) = subtype.2 {
                return Some(&string);
            }
        }
        None
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

    /// Sets the charset parameter to the given charset and returns the old value if present.
    pub fn set_charset(&mut self, charset: Charset) -> Option<Cow<'static, str>> {
        self.parameters.insert("charset".into(), Cow::Owned(charset.to_string()))
    }

    /// Sets the charset to UTF-8.
    pub fn set_charset_utf8(&mut self) -> Option<Cow<'static, str>> {
        self.set_charset(Charset::Utf8)
    }

    /// Compares the mime type portion (the media type without parameters) of two media types.
    pub fn eq_mime_portion(&self, other: &MediaType) -> bool {
        self.type_ == other.type_ && self.subtype == other.subtype
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
        self.type_ == Some(Image)
    }

    /// Checks if the media type is an audio or video type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_audio_or_video_type(&self) -> bool {
        self.type_ == Some(Audio) || self.type_ == Some(Video) ||
        MediaType::new(Application, Standards, "ogg").eq_mime_portion(self)
    }

    /// Checks if the media type is a font type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_font_type(&self) -> bool {
        [MediaType::new(Application, Standards, "font-ttf"),
         MediaType::new(Application, Standards, "font-cff"),
         MediaType::new(Application, Standards, "font-off"),
         MediaType::new(Application, Standards, "font-sfnt"),
         MediaType::new(Application, Vendor, "ms-opentype"),
         MediaType::new(Application, Standards, "font-woff"),
         MediaType::new(Application, Vendor, "ms-fontobject")]
            .iter()
            .any(|x| x.eq_mime_portion(self))
    }

    /// Checks if the media type is a zip based type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_zip_based_type(&self) -> bool {
        self.suffix() == Some(&"zip".into()) ||
        MediaType::new(Application, Standards, "zip").eq_mime_portion(self)
    }

    /// Checks if the media type is an archive type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_archive_type(&self) -> bool {
        [MediaType::new(Application, Standards, "x-rar-compressed"),
         MediaType::new(Application, Standards, "zip"),
         MediaType::new(Application, Standards, "x-gzip")]
            .iter()
            .any(|x| x.eq_mime_portion(self))
    }

    /// Checks if the media type is an XML type.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_xml_type(&self) -> bool {
        self.suffix() == Some(&"xml".into()) ||
        [MediaType::new(Text, Standards, "xml"), MediaType::new(Application, Standards, "xml")]
            .iter()
            .any(|x| x.eq_mime_portion(self))
    }

    /// Checks if the media type is a scriptable type, HTML or PDF.
    ///
    /// Implements the [MIME Sniffing standard]
    /// (https://mimesniff.spec.whatwg.org/#mime-type-groups) for MIME type groups.
    pub fn is_scriptable_mime_type(&self) -> bool {
        [MediaType::new(Text, Standards, "html"), MediaType::new(Application, Standards, "pdf")]
            .iter()
            .any(|x| x.eq_mime_portion(self))
    }
}

/// top-level type name / [ tree. ] subtype name [ +suffix ] [ ; parameters ]
impl FromStr for MediaType {
    type Err = Error;
    fn from_str(s: &str) -> Result<MediaType> {
        let (raw_type, raw_subtype, raw_parameters) = try!(utils::parse_media_type(s.as_bytes()));
        let type_ = match &raw_type[..] {
            b"*" => None,
            b"text" => Some(Text),
            b"image" => Some(Image),
            b"audio" => Some(Audio),
            b"video" => Some(Video),
            b"application" => Some(Application),
            b"multipart" => Some(Multipart),
            b"message" => Some(Message),
            b"model" => Some(Model),
            _ => Some(Type::Unregistered(Cow::Owned(try!(String::from_utf8(raw_type))))),
        };
        let mut parameters = HashMap::new();
        for (key, value) in raw_parameters {
            parameters.insert(try!(String::from_utf8(key)).into(),
                              try!(String::from_utf8(value)).into());
        }
        if raw_subtype == b"*" {
            Ok(MediaType {
                type_: type_,
                subtype: None,
                parameters: parameters,
            })
        } else {
            let subtype = try!(String::from_utf8(raw_subtype));
            let (prefix, suffix) = if subtype.contains('+') {
                let mut parts = subtype.rsplitn(2, '+');
                let suffix = parts.next().unwrap();
                let prefix = parts.next().unwrap();
                (prefix, Some(suffix))
            } else {
                (&subtype[..], None)
            };
            let (tree, sub) = if prefix.contains('.') {
                let mut parts = prefix.splitn(2, '.');
                let tree = match parts.next().unwrap() {
                    "vnd" => Vendor,
                    "prs" => Personal,
                    "x" => Private,
                    s => Tree::Unregistered(Cow::Owned(s.to_owned())),
                };
                (tree, parts.next().unwrap())
            } else {
                (Standards, prefix)
            };
            Ok(MediaType {
                type_: type_,
                subtype: Some((tree,
                               Cow::Owned(sub.to_owned()),
                               suffix.map(|x| Cow::Owned(x.to_owned())))),
                parameters: parameters,
            })
        }
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(ref type_) = self.type_ {
            try!(write!(f, "{}/", type_));
            if let Some((ref tree, ref subtype, ref suffix_opt)) = self.subtype {
                try!(tree.fmt(f));
                try!(subtype.fmt(f));
                if let Some(ref suffix) = *suffix_opt {
                    try!(write!(f, "+{}", suffix));
                }
            } else {
                try!(f.write_str("*"));
            }
        } else {
            try!(f.write_str("*/*"))
        }
        let mut items: Vec<(&Cow<'static, str>, &Cow<'static, str>)> = self.parameters
                                                                           .iter()
                                                                           .collect();
        items.sort_by(|&(ref first, _), &(ref second, _)| first.cmp(second));
        for (ref key, ref value) in items {
            if utils::token(&value) {
                try!(write!(f, "; {}={}", key, value));
            } else {
                try!(write!(f, "; {}=\"{}\"", key, value));
            }
        }
        Ok(())
    }
}
