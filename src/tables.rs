use clap::ValueEnum;
use tabled::{Table, settings::style::Style};

use crate::config;

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::Display, strum::EnumString, ValueEnum,
)]
#[strum(serialize_all = "snake_case")]
pub enum TableStyles {
    Ascii,
    Modern,
    Sharp,
    Rounded,
    Extended,
    Psql,
    Markdown,
    ReStructuredText,
    Dots,
    AsciiRounded,
    Blank,
    Empty,
}

impl TableStyles {
    pub fn apply(&self, table: &mut Table) {
        match &self {
            TableStyles::Ascii => {
                table.with(Style::ascii());
            }
            TableStyles::Modern => {
                table.with(Style::modern());
            }
            TableStyles::Sharp => {
                table.with(Style::sharp());
            }
            TableStyles::Rounded => {
                table.with(Style::rounded());
            }
            TableStyles::Extended => {
                table.with(Style::extended());
            }
            TableStyles::Psql => {
                table.with(Style::psql());
            }
            TableStyles::Markdown => {
                table.with(Style::markdown());
            }
            TableStyles::ReStructuredText => {
                table.with(Style::re_structured_text());
            }
            TableStyles::Dots => {
                table.with(Style::dots());
            }
            TableStyles::AsciiRounded => {
                table.with(Style::ascii_rounded());
            }
            TableStyles::Blank => {
                table.with(Style::blank());
            }
            TableStyles::Empty => {
                table.with(Style::empty());
            }
        }
    }

    fn needs_extra_line(&self) -> bool {
        match self {
            TableStyles::Modern => false,
            _ => true,
        }
    }

    pub fn output_table(
        &self,
        table: &mut Table,
        output_target: &config::OutputTarget,
    ) -> std::io::Result<()> {
        self.apply(table);
        output_target.write(&format!("{}", table))?;
        if self.needs_extra_line() {
            output_target.write("\n")?;
        }
        Ok(())
    }

    pub fn print_table(&self, table: &mut Table) {
        self.apply(table);
        println!("{}", table);
        if self.needs_extra_line() {
            println!("");
        }
    }
}
