use crate::bibtex::BibtexType;

use indexmap::IndexMap;
use std::cmp::PartialOrd;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
// use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

/// Library "tag".
//
// How should I store this? One way is with name: String, parent: String.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Tag {}

/// Resource type.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct ResourceType {
    /// Resource name. This must match the value of "content" for each
    /// resource.
    name: String,
    /// BibTeX type associated with this resource type. This is used
    /// when exporting the resource to a BibTeX entry.
    bibtex: BibtexType,
}

/// Media type.
///
/// We've identified this as a MediaPrefix in order to distinguish it
/// from MediaType, but it technically designates the "type".
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum MediaPrefix {
    Application,
    Audio,
    Image,
    Message,
    Multipart,
    Text,
    Video,
    Font,
    Example,
    Model,
}

/// Media (formerly MIME) type.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(try_from = "&str", into = "String")]
pub struct MediaType {
    r#type: MediaPrefix,
    subtype: String,
}

#[derive(Debug)]
pub struct MediaTypeParseError {
    details: String,
}

impl MediaTypeParseError {
    fn new(msg: &str) -> MediaTypeParseError {
        MediaTypeParseError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for MediaTypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for MediaTypeParseError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl TryFrom<&str> for MediaType {
    type Error = MediaTypeParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let media_type_components: Vec<&str> = s.split("/").collect();

        if media_type_components.len() != 2 {
            return Err(MediaTypeParseError::new(
                "A media type must contain a type and subtype.",
            ));
        }

        let media_type_prefix = format!("\"{}\"", media_type_components[0]);
        Ok(MediaType {
            r#type: serde_json::from_str(&media_type_prefix).unwrap(),
            subtype: media_type_components[1].to_string(),
        })
    }
}

impl From<MediaType> for String {
    fn from(media_type: MediaType) -> Self {
        let type_string = serde_json::to_string(&media_type.r#type).unwrap();
        format!(
            "{}/{}",
            // serde_json includes quotes at the beginning and end of
            // the string that we don't want here.
            type_string[1..type_string.len() - 1].to_string(),
            media_type.subtype
        )
    }
}

/// Document type.
///
/// Classifies a document type according to an extension and media
/// type.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct DocumentType {
    pub extension: String,
    pub mime: Option<MediaType>,
}

/// DateTime.
///
/// The order of members in this struct is important since it is used
/// by `#[derive(PartialOrd)]`.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, PartialOrd)]
#[serde(try_from = "&str", into = "String")]
pub struct DateTime {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
    pub hour: Option<i32>,
    pub minute: Option<i32>,
    pub second: Option<i32>,
}

impl DateTime {
    pub fn new() -> DateTime {
        DateTime {
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
        }
    }
}

#[derive(Debug)]
pub struct DateTimeParseError {
    details: String,
}

impl DateTimeParseError {
    fn new(msg: &str) -> DateTimeParseError {
        DateTimeParseError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for DateTimeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for DateTimeParseError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl TryFrom<&str> for DateTime {
    type Error = DateTimeParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut datetime = DateTime::new();
        let len = s.len();

        if len >= 4 {
            datetime.year = Some(i32::from_str_radix(&s[..4], 10).unwrap());

            if len >= 7 {
                let month = i32::from_str_radix(&s[5..7], 10).unwrap();
                if month < 1 || month > 12 {
                    return Err(DateTimeParseError::new("month must be between 1 and 12"));
                }
                datetime.month = Some(month);

                if len >= 10 {
                    let day = i32::from_str_radix(&s[8..10], 10).unwrap();
                    if day < 1 || day > 31 {
                        return Err(DateTimeParseError::new("day must be between 1 and 31"));
                    }
                    datetime.day = Some(day);

                    if len >= 13 {
                        let hour = i32::from_str_radix(&s[11..13], 10).unwrap();
                        if hour < 0 || hour > 23 {
                            return Err(DateTimeParseError::new("hour must be between 0 and 23"));
                        }
                        datetime.hour = Some(hour);

                        if len >= 16 {
                            let minute = i32::from_str_radix(&s[14..16], 10).unwrap();
                            if minute < 0 || minute > 59 {
                                return Err(DateTimeParseError::new(
                                    "minute must be between 0 and 59",
                                ));
                            }
                            datetime.minute = Some(minute);

                            if len >= 19 {
                                let second = i32::from_str_radix(&s[17..19], 10).unwrap();
                                if second < 0 || second > 59 {
                                    return Err(DateTimeParseError::new(
                                        "second must be between 0 and 59",
                                    ));
                                }
                                datetime.second = Some(second);
                            }
                        }
                    }
                }
            }
        }

        Ok(datetime)
    }
}

impl From<DateTime> for String {
    fn from(datetime: DateTime) -> Self {
        match datetime.year {
            Some(y) => match datetime.month {
                Some(m) => match datetime.day {
                    Some(d) => match datetime.hour {
                        Some(h) => match datetime.minute {
                            Some(min) => match datetime.second {
                                Some(s) => format!(
                                    "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                                    y, m, d, h, min, s
                                ),
                                None => format!("{:04}-{:02}-{:02}T{:02}:{:02}", y, m, d, h, min),
                            },
                            None => format!("{:04}-{:02}-{:02}T{:02}", y, m, d, h),
                        },
                        None => format!("{:04}-{:02}-{:02}", y, m, d),
                    },
                    None => format!("{:04}-{:02}", y, m),
                },
                None => format!("{:04}", y),
            },
            None => format!(""),
        }
    }
}

/// Name.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(try_from = "&str", into = "String")]
pub struct Name {
    pub first: Option<String>,
    pub middle: Option<String>,
    pub last: Option<String>,
}

impl Name {
    pub fn new() -> Name {
        Name {
            first: None,
            middle: None,
            last: None,
        }
    }
}

#[derive(Debug)]
pub struct NameParseError {
    details: String,
}

impl NameParseError {
    fn new(msg: &str) -> NameParseError {
        NameParseError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for NameParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for NameParseError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl TryFrom<&str> for Name {
    type Error = NameParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut name = Name::new();
        let subnames: Vec<&str> = s.split(" ").collect();
        if subnames.len() > 3 {
            return Err(NameParseError::new(
                "A name can only contain a maximum of 3 parts.",
            ));
        } else if subnames.len() == 3 {
            name.first = Some(subnames[0].to_string());
            name.middle = Some(subnames[1].to_string());
            name.last = Some(subnames[2].to_string());
        } else if subnames.len() == 2 {
            name.first = Some(subnames[0].to_string());
            name.last = Some(subnames[1].to_string());
        } else if subnames.len() == 1 {
            name.last = Some(subnames[0].to_string());
        }

        Ok(name)
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        match name.last {
            Some(l) => match name.first {
                Some(f) => match name.middle {
                    Some(m) => format!("{} {} {}", f, m, l),
                    None => format!("{} {}", f, l),
                },
                None => format!("{}", l),
            },
            None => format!(""),
        }
    }
}

/// Library "resource". This represents one unit of library content,
/// which can either be a file (such as a document or video), or a
/// directory (e.g., holding the contents of a webpage).
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Resource {
    /// Title.
    pub title: String,
    /// Subtitle.
    pub subtitle: Option<String>,
    /// All resource authors.
    pub author: Option<Vec<Name>>,
    /// All resource editors.
    pub editor: Option<Vec<Name>>,
    /// A date and time that is meant to represent the last time the
    /// resource's content changed. For a publication, such as a book
    /// or scientific article, this is the date of publication. For a
    /// website, this is the last time the website contents were
    /// updated (if you don't know this information, use the archival
    /// date).
    pub date: Option<DateTime>,
    pub edition: Option<String>,
    /// Version or edition. While many editions are simple integers
    /// (e.g., first or second edition), many others are, so this can
    /// take any valid string.
    pub version: Option<String>,
    pub publisher: Option<String>,
    /// Organization or institution involved in creation of the
    /// resource. If the resource was a thesis, this should be the
    /// school.
    pub organization: Option<String>,
    /// Journal or magazine in which the resource was published.
    pub journal: Option<String>,
    /// Volume of a journal or multi-volume book or other resource.
    pub volume: Option<String>,
    /// The issue number of a journal, magazine, application note,
    /// etc. This is a string because issue numbers are not always
    /// numbers. For example, they often contain character suffixes as
    /// in "57A".
    pub number: Option<String>,
    /// TODO create a DOI struct with custom
    /// serialization/deserialization.
    /// Digital object identifier (DOI).
    pub doi: Option<String>,
    pub tags: Option<Vec<String>>,
    /// Document type (when applicable). This field is also used to
    /// associate a resource with a file extension.
    pub document: Option<String>,
    pub content: Option<String>,
    /// Upstream URL where the resource is maintained or where it was
    /// retreived.
    pub url: Option<Url>,
    /// Current SHA-1 checksum.
    pub checksum: String,
    /// An ordered collection (oldest to most recent) of all previous
    /// and current checksums of a resource. The current checksum is
    /// the last item in the container.
    pub historical_checksums: Vec<String>,
}

impl Resource {
    /// Concatenate fields into a single string, using a space as a
    /// delimeter between fields.
    ///
    /// This can be used for matching against multiple fields of a
    /// resource.
    ///
    /// TODO this creates too many spaces because empty strings still
    /// register.
    pub fn concat_fields(&self, fields: Vec<&str>) -> String {
        fields
            .iter()
            .filter_map(|x| self.field_string(x))
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// Return a string representation of a field.
    ///
    /// When an optional field is None, an empty string is
    /// returned. When a field contains a list of values, all items are
    /// concatenated separated by spaces.
    fn field_string(&self, field: &str) -> Option<String> {
        match field {
            "title" => Some(self.title.clone()),
            "subtitle" => match &self.subtitle {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "author" => match &self.author {
                Some(it) => Some(
                    it.iter()
                        .map(|x| String::from(x.clone()))
                        .collect::<Vec<String>>()
                        .join(" "),
                ),
                None => None,
            },
            "editor" => match &self.editor {
                Some(it) => Some(
                    it.iter()
                        .map(|x| String::from(x.clone()))
                        .collect::<Vec<String>>()
                        .join(" "),
                ),
                None => None,
            },
            "date" => match &self.date {
                Some(x) => Some(String::from(x.clone())),
                None => None,
            },
            "edition" => match &self.edition {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "version" => match &self.version {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "publisher" => match &self.publisher {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "organization" => match &self.organization {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "journal" => match &self.journal {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "volume" => match &self.volume {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "number" => match &self.number {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "doi" => match &self.doi {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "tags" => match &self.tags {
                Some(it) => Some(
                    it.iter()
                        .map(|x| x.clone())
                        .collect::<Vec<String>>()
                        .join(" "),
                ),
                None => None,
            },
            "document" => match &self.document {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "content" => match &self.content {
                Some(x) => Some(x.clone()),
                None => None,
            },
            "url" => match &self.url {
                Some(x) => Some(String::from(x.clone())),
                None => None,
            },
            "checksum" => Some(self.checksum.clone()),
            // TODO should probably exclude historical checksum that
            // is identical to checksum
            "historical_checksums" => Some(
                self.historical_checksums
                    .iter()
                    .map(|x| x.clone())
                    .collect::<Vec<String>>()
                    .join(" "),
            ),
            &_ => panic!("invalid field specifier"),
        }
    }

    /// The BibTeX type associated with the current resource.
    ///
    /// # Arguments
    ///
    /// * `content_types` - A collection of content types as defined
    /// in the catalog. The map key is a string identifying the
    /// content type and the map value is the associated BibTeX type.
    ///
    /// # Return
    ///
    /// Returns `None` if the content type for resource is not one of
    /// the content types defined in the catalog.
    pub fn bibtex_type(&self, content_types: &IndexMap<String, BibtexType>) -> Option<BibtexType> {
        match &self.content {
            Some(c) => Some(match content_types.get(c) {
                Some(ct) => ct.clone(),
                None => panic!(
                    "Failed to retrieve bibtex type for resource {:?}",
                    self.checksum
                ),
            }),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_concat_fields() {
        let resource: Resource = serde_json::from_str(
            "{
              \"title\": \"Classical Electrodynamics\",
              \"author\": [
                \"John David Jackson\"
              ],
              \"date\": \"1999\",
              \"edition\": \"3\",
              \"publisher\": \"John Wiley & Sons\",
              \"tags\": [
                \"physics\",
                \"electromagnetism\"
              ],
              \"document\": \"pdf\",
              \"content\": \"textbook\",
              \"checksum\": \"88259e88e7677e5ae8a31e33f177a2198cabe95c\",
              \"historical_checksums\": [
                \"88259e88e7677e5ae8a31e33f177a2198cabe95c\"
              ]
            }",
        )
        .unwrap();

        let actual = resource.concat_fields(vec![
            "title",
            "subtitle",
            "author",
            "editor",
            "date",
            "edition",
            "version",
            "publisher",
            "organization",
            "journal",
            "volume",
            "number",
            "doi",
            "tags",
            "document",
            "content",
            "url",
            "checksum",
            "historical_checksums",
        ]);
        let want = concat!(
            "Classical Electrodynamics ",
            "John David Jackson ",
            "1999 ",
            "3 ",
            "John Wiley & Sons ",
            "physics ",
            "electromagnetism ",
            "pdf ",
            "textbook ",
            "88259e88e7677e5ae8a31e33f177a2198cabe95c ",
            "88259e88e7677e5ae8a31e33f177a2198cabe95c"
        );
        println!("actual: {:?}", actual);
        println!("want: {:?}", want);
        assert!(actual == want);
    }
}
