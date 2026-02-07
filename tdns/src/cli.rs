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
pub enum OutputFormat {
    #[value(alias = "json", alias = "JSON", alias = "Json")]
    Json,
    #[value(alias = "table", alias = "TABLE", alias = "Table")]
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
