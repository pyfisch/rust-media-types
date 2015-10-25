#[macro_use]
extern crate media_types;

use std::collections::HashMap;

use media_types::*;

#[test]
fn test_text_plain() {
    let tag: MediaType = "text/plain".parse().unwrap();
    assert_eq!(tag.type_, Some("text".into()));
    assert_eq!(tag.subtype, Some("plain".into()));
}

#[test]
fn test_application_vnd_oasis_opendocument_text() {
    let tag: MediaType = "application/vnd.oasis.opendocument.text".parse().unwrap();
    assert_eq!(tag.type_, Some("application".into()));
    assert_eq!(tag.tree, Some("vnd".into()));
    assert_eq!(tag.subtype, Some("oasis.opendocument.text".into()));
}

#[test]
fn test_image_svg_xml() {
    let tag: MediaType = "image/svg+xml".parse().unwrap();
    assert_eq!(tag.type_, Some("image".into()));
    assert_eq!(tag.subtype, Some("svg".into()));
    assert_eq!(tag.suffix, Some("xml".into()));
}

#[test]
fn test_audio_star() {
    let tag: MediaType = "audio/*".parse().unwrap();
    assert_eq!(tag.type_, Some("audio".into()));
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
    parameters.insert("codecs".into(), "sevc, s263".into());
    assert_eq!(tag.parameters, parameters);
    let tag: MediaType = "audio/3gpp2; codecs=mp4a.E1".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".into(), "mp4a.E1".into());
    assert_eq!(tag.parameters, parameters);

    let tag: MediaType = "example/*; codecs=a.bb.ccc.d".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".into(), "a.bb.ccc.d".into());
    assert_eq!(tag.parameters, parameters);

    let tag: MediaType = "example/*; codecs=\"a.bb.ccc.d, e.fff\"".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".into(), "a.bb.ccc.d, e.fff".into());
    assert_eq!(tag.parameters, parameters);

    let tag: MediaType = "example/*; codecs*=''fo%2e".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".into(), "fo.".into());
    assert_eq!(tag.parameters, parameters);

    let tag: MediaType = "example/*; codecs*=\"''%25%20xz, gork\"".parse().unwrap();
    let mut parameters = HashMap::new();
    parameters.insert("codecs".into(), "% xz, gork".into());
    assert_eq!(tag.parameters, parameters);
}

#[test]
fn test_rfc2231_types() {
    let tag: MediaType = "application/x-stuff; title*=us-ascii'en-us'This is%20%2A%2A%2Afun%2A%2A%2A".parse().unwrap();
    let mut expected: MediaType = Default::default();
    expected.type_ = Some("application".into());
    expected.subtype = Some("x-stuff".into());
    expected.parameters.insert("title".into(), "This is ***fun***".into());
    assert_eq!(tag, expected);
}

#[test]
fn test_rfc1341_types() {
    let tag: MediaType = "multipart/digest; boundary=\"---- next message ----\" ".parse().unwrap();
    let mut expected: MediaType = Default::default();
    expected.type_ = Some("multipart".into());
    expected.subtype = Some("digest".into());
    expected.parameters.insert("boundary".into(), "---- next message ----".into());
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
    let mut tag = MediaType::new(Some("example"), STANDARDS, Some("foobar"));
    assert_eq!(tag.to_string(), "example/foobar");
    tag = MediaType::new(Some("example"), Some("spam"), Some("foobar"));
    assert_eq!(tag.to_string(), "example/spam.foobar");
    tag = MediaType::new(Some("example"), UNREGISTERED, Some("foobar"));
    assert_eq!(tag.to_string(), "example/x.foobar");
    tag = MediaType::new_with_suffix(Some("example"), UNREGISTERED, Some("foobar"), Some("xml"));
    assert_eq!(tag.to_string(), "example/x.foobar+xml");
    tag.parameters.insert("charset".into(), "US-ASCII".into());
    assert_eq!(tag.to_string(), "example/x.foobar+xml; charset=US-ASCII");
    tag.parameters.insert("boundary".into(), "foo ,".into());
    assert_eq!(tag.to_string(), "example/x.foobar+xml; boundary=\"foo ,\"; charset=US-ASCII");
    tag.parameters.insert("z".into(), "1".into());
    assert_eq!(tag.to_string(), "example/x.foobar+xml; boundary=\"foo ,\"; charset=US-ASCII; z=1");
}
