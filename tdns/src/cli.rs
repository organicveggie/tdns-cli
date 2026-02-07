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
#[strum(serialize_all = "snake_case")]
pub enum SortOrder {
    Unsorted,
    #[value(alias = "asc")]
    Ascending,
    #[value(alias = "desc")]
    Descending,
}
