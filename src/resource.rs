use crate::bibtex::BibtexType;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
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
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(try_from = "&str", into = "String")]
struct MediaType {
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
    extension: String,
    mime: Option<MediaType>,
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
    pub document_type: Option<String>,
    pub content_type: Option<String>,
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
    // TODO I imagine there's a much more concise (and less
    // repetitive) way of fuzzy-matching all fields.
    pub fn fuzzy_match(&self, query: &str) -> bool {
        self.fuzzy_match_field("title", query)
            || self.fuzzy_match_field("subtitle", query)
            || self.fuzzy_match_field("author", query)
            || self.fuzzy_match_field("editor", query)
            || self.fuzzy_match_field("date", query)
            || self.fuzzy_match_field("edition", query)
            || self.fuzzy_match_field("version", query)
            || self.fuzzy_match_field("publisher", query)
            || self.fuzzy_match_field("organization", query)
            || self.fuzzy_match_field("journal", query)
            || self.fuzzy_match_field("volume", query)
            || self.fuzzy_match_field("number", query)
            || self.fuzzy_match_field("doi", query)
            || self.fuzzy_match_field("tags", query)
            || self.fuzzy_match_field("document_type", query)
            || self.fuzzy_match_field("content_type", query)
            || self.fuzzy_match_field("url", query)
            || self.fuzzy_match_field("checksum", query)
            || self.fuzzy_match_field("historical_checksums", query)
    }

    // TODO remove unicode information to make fuzzy searching
    // easier. E.g., 'ä' should be searched as 'a'.
    pub fn fuzzy_match_field(&self, field: &str, query: &str) -> bool {
        let matcher = SkimMatcherV2::default().ignore_case();

        return match field {
            "title" => matcher.fuzzy_match(&self.title, query).is_some(),
            "subtitle" => match &self.subtitle {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "author" => match &self.author {
                Some(oa) => {
                    oa.iter().any(|a| match &a.first {
                        Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                        None => false,
                    } || match &a.middle {
                        Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                        None => false,
                    } || match &a.last {
                        Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                        None => false,
                    })
                },
                None => false,
             },
            "editor" => match &self.author {
                Some(oe) => {
                    oe.iter().any(|a| match &a.first {
                        Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                        None => false,
                    } || match &a.middle {
                        Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                        None => false,
                    } || match &a.last {
                        Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                        None => false,
                    })
                },
                None => false,
             },
            "date" => match &self.date {
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
            "edition" => match &self.edition {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
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
            "journal" => match &self.journal {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "volume" => match &self.volume {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "number" => match &self.number {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "doi" => match &self.doi {
                Some(f) => matcher.fuzzy_match(&f, query).is_some(),
                None => false,
            },
            "tags" => match &self.tags {
                Some(ot) => ot
                    .iter()
                    .any(|t| matcher.fuzzy_match(&t, query).is_some()),
                None => false,
            },
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
        match &self.content_type {
            Some(c) => Some(content_types.get(c).unwrap().clone()),
            None => None,
        }
    }
}
