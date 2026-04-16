pub mod page;
pub mod wikilinks;
pub mod index;
pub mod log;

pub use page::{PageType, WikiPage};
pub use wikilinks::{extract_wikilinks, title_to_slug};
