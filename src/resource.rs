use crate::bibtex::BibtexType;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
// use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

/// Library "tag".
//
// How should I store this? One way is with name: String, parent: String.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Tag {}

/// Resource type.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct ResourceType {
    /// Resource name. This must match the value of "content_type"
    /// for each resource.
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
///
/// TODO this should probably eventually have a custom deserializer so
/// we can write a media type like application/pdf, etc.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct MediaType {
    r#type: MediaPrefix,
    subtype: String,
}

/// Document type.
///
/// Classifies a document type according to an extension and media
/// type.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct DocumentType {
    extension: String,
    mime: Option<MediaType>,
}

/// DateTime.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
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
                                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                                    y, m, d, h, min, s
                                ),
                                None => format!("{:04}-{:02}-{:02} {:02}:{:02}", y, m, d, h, min),
                            },
                            None => format!("{:04}-{:02}-{:02} {:02}", y, m, d, h),
                        },
                        None => format!("{:04}-{:02}-{:02}", y, m, d),
                    },
                    None => format!("{:04}-{:02}", y, m),
                },
                None => format!("{:04}", y),
            },
            None => "".to_string(),
        }
    }
}

/// Name.
///
/// TODO implement custom serialization/deserialization.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Name {
    pub first: Option<String>,
    pub middle: Option<String>,
    pub last: Option<String>,
}

/// Library "resource". This represents one unit of library content,
/// which can either be a file (such as a document or video), or a
/// directory (e.g., with the contents of a webpage).
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Resource {
    /// Title.
    pub title: String,
    /// All resource authors.
    pub authors: Vec<Name>,
    /// A date and time that is meant to represent the last time the
    /// resource's content changed. For a publication, such as a book
    /// or scientific article, this is the date of publication. For a
    /// website, this is the last time the website contents were
    /// updated (if you don't know this information, use the archival
    /// date).
    pub datetime: Option<DateTime>,
    pub edition: Option<i32>,
    pub version: Option<String>,
    pub publisher: Option<String>,
    pub organization: Option<String>,
    pub tags: Vec<String>,
    /// Document type (when applicable). This field is also used to
    /// associate a resource with a file extension.
    pub document_type: Option<String>,
    pub content_type: Option<String>,
    /// URL.
    pub url: Option<Url>,
    /// Current SHA-1 checksum.
    pub checksum: String,
    /// An ordered collection (oldest to most recent) of all previous
    /// and current checksums of a resource. The current checksum is
    /// the last item in the container.
    pub historical_checksums: Vec<String>,
}

impl Resource {
    pub fn fuzzy_match(&self, query: &str) -> bool {
        self.fuzzy_match_field("title", query)
            || self.fuzzy_match_field("authors", query)
            || self.fuzzy_match_field("datetime", query)
            || self.fuzzy_match_field("edition", query)
            || self.fuzzy_match_field("version", query)
            || self.fuzzy_match_field("publisher", query)
            || self.fuzzy_match_field("organization", query)
            || self.fuzzy_match_field("tags", query)
            || self.fuzzy_match_field("document_type", query)
            || self.fuzzy_match_field("content_type", query)
            || self.fuzzy_match_field("url", query)
            || self.fuzzy_match_field("checksum", query)
            || self.fuzzy_match_field("historical_checksums", query)
    }

    // TODO remove unicode information to make fuzzy searching
    // easier. That is something like ä should be searched as though
    // it were a.
    pub fn fuzzy_match_field(&self, field: &str, query: &str) -> bool {
        let matcher = SkimMatcherV2::default();

        return match field {
            "title" => matcher.fuzzy_match(&self.title, query).is_some(),
            "authors" => self.authors.iter().any(|a| match &a.first {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            } || match &a.middle {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            } || match &a.last {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            }),
            "datetime" => match &self.datetime {
                Some(d) => {
                    (match d.year {
                        Some(f) => matcher.fuzzy_match(&f.to_string(), query).is_some(),
                        None => false,
                    }) || (match d.month {
                        Some(f) => matcher.fuzzy_match(&f.to_string(), query).is_some(),
                        None => false,
                    }) || (match d.day {
                        Some(f) => matcher.fuzzy_match(&f.to_string(), query).is_some(),
                        None => false,
                    }) || (match d.hour {
                        Some(f) => matcher.fuzzy_match(&f.to_string(), query).is_some(),
                        None => false,
                    }) || (match d.minute {
                        Some(f) => matcher.fuzzy_match(&f.to_string(), query).is_some(),
                        None => false,
                    }) || (match d.second {
                        Some(f) => matcher.fuzzy_match(&f.to_string(), query).is_some(),
                        None => false,
                    })
                }
                None => false,
            }
            "edition" => match self.edition {
                Some(f) => matcher.fuzzy_match(&f.to_string(), query).is_some(),
                None => false,
            },
            "version" => match &self.version {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "publisher" => match &self.publisher {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "organization" => match &self.organization {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "tags" => self
                .tags
                .iter()
                .any(|t| matcher.fuzzy_match(&t, query).is_some()),
            "document_type" => match &self.document_type {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "content_type" => match &self.content_type {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "url" => match &self.url {
                Some(f) => matcher.fuzzy_match(&f.to_string(), query).is_some(),
                None => false,
            },
            "checksum" => matcher.fuzzy_match(&self.checksum, query).is_some(),
            "historical_checksums" => self
                .historical_checksums
                .iter()
                .any(|c| matcher.fuzzy_match(&c, query).is_some()),
            &_ => panic!("invalid field"),
        };
    }
}
