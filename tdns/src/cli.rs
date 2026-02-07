#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    strum::Display,
    strum::EnumString,
    clap::ValueEnum,
)]
pub enum OutputFormat {
    Json,
    Table,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    strum::Display,
    strum::EnumString,
    clap::ValueEnum,
)]
pub enum SortOrder {
    Unsorted,
    Ascending,
    Descending,
}
