use super::Region;
use vimwiki::{elements, vendor::chrono::NaiveDate, LC};

#[derive(async_graphql::Union, Debug)]
pub enum Placeholder {
    Title(PlaceholderTitle),
    NoHtml(PlaceholderNoHtml),
    Template(PlaceholderTemplate),
    Date(PlaceholderDate),
    Other(PlaceholderOther),
}

impl From<LC<elements::Placeholder>> for Placeholder {
    fn from(lc: LC<elements::Placeholder>) -> Self {
        let region = Region::from(lc.region);
        match lc.element {
            elements::Placeholder::Title(title) => {
                Self::from(PlaceholderTitle { region, title })
            }
            elements::Placeholder::NoHtml => {
                Self::from(PlaceholderNoHtml { region })
            }
            elements::Placeholder::Template(template) => {
                Self::from(PlaceholderTemplate { region, template })
            }
            elements::Placeholder::Date(date) => {
                Self::from(PlaceholderDate { region, date })
            }
            elements::Placeholder::Other { name, value } => {
                Self::from(PlaceholderOther {
                    region,
                    name,
                    value,
                })
            }
        }
    }
}

/// Represents a single document title placeholder
#[derive(async_graphql::SimpleObject, Debug)]
pub struct PlaceholderTitle {
    /// The segment of the document this placeholder covers
    region: Region,

    /// The title associated with this placeholder
    title: String,
}

/// Represents a single document nohtml placeholder
#[derive(async_graphql::SimpleObject, Debug)]
pub struct PlaceholderNoHtml {
    /// The segment of the document this placeholder covers
    region: Region,
}

/// Represents a single document template placeholder
#[derive(async_graphql::SimpleObject, Debug)]
pub struct PlaceholderTemplate {
    /// The segment of the document this placeholder covers
    region: Region,

    /// The template associated with this placeholder
    template: String,
}

/// Represents a single document date placeholder
#[derive(async_graphql::SimpleObject, Debug)]
pub struct PlaceholderDate {
    /// The segment of the document this placeholder covers
    region: Region,

    /// The date associated with this placeholder
    date: NaiveDate,
}

/// Represents a single document other placeholder
#[derive(async_graphql::SimpleObject, Debug)]
pub struct PlaceholderOther {
    /// The segment of the document this placeholder covers
    region: Region,

    /// The name associated with this placeholder
    name: String,

    /// The value associated with this placeholder
    value: String,
}
