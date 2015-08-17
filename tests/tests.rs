#[macro_use]
extern crate media_types;

use std::collections::HashMap;

use media_types::*;

#[test]
fn test_text_plain() {
    let tag: MediaType = "text/plain".parse().unwrap();
    assert_eq!(tag.type_, Some("text".to_owned()));
    assert_eq!(tag.subtype, Some("plain".to_owned()));
}

#[test]
fn test_application_vnd_oasis_opendocument_text() {
    let tag: MediaType = "application/vnd.oasis.opendocument.text".parse().unwrap();
    assert_eq!(tag.type_, Some("application".to_owned()));
    assert_eq!(tag.tree, Some("vnd".to_owned()));
    assert_eq!(tag.subtype, Some("oasis.opendocument.text".to_owned()));
}

#[test]
fn test_image_svg_xml() {
    let tag: MediaType = "image/svg+xml".parse().unwrap();
    assert_eq!(tag.type_, Some("image".to_owned()));
    assert_eq!(tag.subtype, Some("svg".to_owned()));
    assert_eq!(tag.suffix, Some("xml".to_owned()));
}

#[test]
fn test_audio_star() {
    let tag: MediaType = "audio/*".parse().unwrap();
    assert_eq!(tag.type_, Some("audio".to_owned()));
    assert_eq!(tag.subtype, None);
}

#[test]
fn test_any() {
    let tag: MediaType = "*/*".parse().unwrap();
    assert_eq!(tag, Default::default());
}

#[test]
fn test_empty() {
    assert_eq!("".parse::<MediaType>(), Err(Error::Invalid))
}

#[test]
fn test_rfc6381_types() {
    let tag: MediaType = "video/3gpp2; codecs=\"sevc, s263\"".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".to_owned(), "sevc, s263".to_owned());
    assert_eq!(tag.parameters, parameters);
    let tag: MediaType = "audio/3gpp2; codecs=mp4a.E1".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".to_owned(), "mp4a.E1".to_owned());
    assert_eq!(tag.parameters, parameters);

    let tag: MediaType = "example/*; codecs=a.bb.ccc.d".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".to_owned(), "a.bb.ccc.d".to_owned());
    assert_eq!(tag.parameters, parameters);

    let tag: MediaType = "example/*; codecs=\"a.bb.ccc.d, e.fff\"".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".to_owned(), "a.bb.ccc.d, e.fff".to_owned());
    assert_eq!(tag.parameters, parameters);

    let tag: MediaType = "example/*; codecs*=''fo%2e".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".to_owned(), "fo.".to_owned());
    assert_eq!(tag.parameters, parameters);

    let tag: MediaType = "example/*; codecs*=\"''%25%20xz, gork\"".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".to_owned(), "% xz, gork".to_owned());
    assert_eq!(tag.parameters, parameters);
}

#[test]
fn test_rfc2231_types() {
    let tag: MediaType = "application/x-stuff; title*=us-ascii'en-us'This is%20%2A%2A%2Afun%2A%2A%2A".parse().unwrap();
    let mut expected: MediaType = Default::default();
    expected.type_ = Some("application".to_owned());
    expected.subtype = Some("x-stuff".to_owned());
    expected.parameters.insert("title".to_owned(), "This is ***fun***".to_owned());
    assert_eq!(tag, expected);
}

#[test]
fn test_rfc1341_types() {
    let tag: MediaType = "multipart/digest; boundary=\"---- next message ----\" ".parse().unwrap();
    let mut expected: MediaType = Default::default();
    expected.type_ = Some("multipart".to_owned());
    expected.subtype = Some("digest".to_owned());
    expected.parameters.insert("boundary".to_owned(), "---- next message ----".to_owned());
    assert_eq!(tag, expected);
    assert_eq!(tag.boundary(), Ok("---- next message ----"));

    let tag: MediaType = "multipart/mixed; boundary=\"simple boundary\"".parse().unwrap();
    assert_eq!(tag.boundary(), Ok("simple boundary"));

    let tag: MediaType = "multipart/mixed; boundary=\"  foo\"".parse().unwrap();
    assert_eq!(tag.boundary(), Ok("  foo"));

    let tag: MediaType = "multipart/mixed; boundary=awesrdtfhzujiomkomnihuzbtrcdewsasrtczubmiokinibuvztcrxeyrctzbunimnbuvzcxxrctzubinnibuvzctxrxrtczvubinbuvzctxxrcvzbuhn".parse().unwrap();
    assert_eq!(tag.boundary(), Err(Error::Invalid));

    let tag: MediaType = "multipart/mixed; boundary=\"foo\\\"bar\"".parse().unwrap();
    assert_eq!(tag.boundary(), Err(Error::Invalid));
}

#[test]
fn test_rfc2046_types() {
    let tag: MediaType = "text/plain; charset=iso-8859-1".parse().unwrap();
    assert_eq!(tag.charset(), Ok(Charset::Iso88591));
}

#[test]
fn test_format() {
    let mut tag = MediaType::new(Some("example"), STANDARDS, Some("foobar"), None);
    assert_eq!(tag.to_string(), "example/foobar");
    tag = MediaType::new(Some("example"), Some("spam"), Some("foobar"), None);
    assert_eq!(tag.to_string(), "example/spam.foobar");
    tag = MediaType::new(Some("example"), UNREGISTERED, Some("foobar"), None);
    assert_eq!(tag.to_string(), "example/x.foobar");
    tag = MediaType::new(Some("example"), UNREGISTERED, Some("foobar"), Some("xml"));
    assert_eq!(tag.to_string(), "example/x.foobar+xml");
    tag.parameters.insert("charset".to_owned(), "US-ASCII".to_owned());
    assert_eq!(tag.to_string(), "example/x.foobar+xml; charset=US-ASCII");
    tag.parameters.insert("boundary".to_owned(), "foo ,".to_owned());
    assert_eq!(tag.to_string(), "example/x.foobar+xml; boundary=\"foo ,\"; charset=US-ASCII");
    tag.parameters.insert("z".to_owned(), "1".to_owned());
    assert_eq!(tag.to_string(), "example/x.foobar+xml; boundary=\"foo ,\"; charset=US-ASCII; z=1");
}
