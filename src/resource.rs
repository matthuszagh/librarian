use crate::bibtex::BibtexType;

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

/// Date.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Date {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

/// Name.
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
    pub date: Date,
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
    // fn fuzzy_match(&self, query: &str) -> bool {
    //     self.fuzzy_match_field("title", query)
    //         || self.fuzzy_match_field("authors", query)
    //         || self.fuzzy_match_field("date", query)
    //         || self.fuzzy_match_field("edition", query)
    //         || self.fuzzy_match_field("version", query)
    //         || self.fuzzy_match_field("publisher", query)
    //         || self.fuzzy_match_field("organization", query)
    //         || self.fuzzy_match_field("tags", query)
    //         || self.fuzzy_match_field("document_type", query)
    //         || self.fuzzy_match_field("content_type", query)
    //         || self.fuzzy_match_field("url", query)
    //         || self.fuzzy_match_field("checksum", query)
    //         || self.fuzzy_match_field("historical_checksums", query)
    // }

    // fn fuzzy_match_field(&self, field: &str, query: &str) -> bool {
    //     // TODO implement
    //     false
    // }
}