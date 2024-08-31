use std::{fs::File, path::PathBuf};

use polars::{
    frame::DataFrame,
    io::SerReader,
    prelude::{CsvParseOptions, CsvReadOptions, JsonLineReader, JsonReader, ParquetReader},
};

use crate::{
    args::{Args, InferSchema},
    utils::{as_ascii, safe_infer_schema},
    AppResult,
};

pub trait ReadToDataFrame {
    fn read_to_data_frame(&self, file: PathBuf) -> AppResult<DataFrame>;
}

pub trait BuildReader {
    fn build_reader(&self) -> Box<dyn ReadToDataFrame>;
}

impl BuildReader for Args {
    fn build_reader(&self) -> Box<dyn ReadToDataFrame> {
        match self.format {
            crate::args::Format::Dsv => Box::new(CsvToDataFrame::from_args(self)),
            crate::args::Format::Parquet => Box::new(ParquetToDataFrame),
            crate::args::Format::Jsonl => Box::new(JsonLineToDataFrame::from_args(self)),
            crate::args::Format::Json => Box::new(JsonToDataFrame::from_args(self)),
        }
    }
}

pub struct CsvToDataFrame {
    infer_schema: InferSchema,
    quote_char: char,
    separator_char: char,
    no_header: bool,
    ignore_errors: bool,
}

impl CsvToDataFrame {
    pub fn from_args(args: &Args) -> Self {
        Self {
            infer_schema: args.infer_schema,
            quote_char: args.quote_char,
            separator_char: args.separator,
            no_header: args.no_header,
            ignore_errors: args.ignore_errors,
        }
    }
}

impl ReadToDataFrame for CsvToDataFrame {
    fn read_to_data_frame(&self, file: PathBuf) -> AppResult<DataFrame> {
        let mut df = CsvReadOptions::default()
            .with_ignore_errors(self.ignore_errors)
            .with_infer_schema_length(self.infer_schema.to_csv_infer_schema_length())
            .with_has_header(!self.no_header)
            .with_parse_options(
                CsvParseOptions::default()
                    .with_quote_char(as_ascii(self.quote_char))
                    .with_separator(as_ascii(self.separator_char).expect("Invalid separator")),
            )
            .try_into_reader_with_file_path(file.into())?
            .finish()?;
        if matches!(self.infer_schema, InferSchema::Safe) {
            safe_infer_schema(&mut df);
        }
        Ok(df)
    }
}

pub struct ParquetToDataFrame;

impl ReadToDataFrame for ParquetToDataFrame {
    fn read_to_data_frame(&self, file: PathBuf) -> AppResult<DataFrame> {
        Ok(ParquetReader::new(File::open(&file)?)
            .set_rechunk(true)
            .finish()?)
    }
}

pub struct JsonLineToDataFrame {
    infer_schema: InferSchema,
    ignore_errors: bool,
}

impl JsonLineToDataFrame {
    pub fn from_args(args: &Args) -> Self {
        Self {
            infer_schema: args.infer_schema,
            ignore_errors: args.ignore_errors,
        }
    }
}

impl ReadToDataFrame for JsonLineToDataFrame {
    fn read_to_data_frame(&self, file: PathBuf) -> AppResult<DataFrame> {
        let mut df = JsonLineReader::new(File::open(file)?)
            .with_rechunk(true)
            .infer_schema_len(None)
            .with_ignore_errors(self.ignore_errors)
            .finish()?;
        if matches!(
            self.infer_schema,
            InferSchema::Safe | InferSchema::Full | InferSchema::Fast
        ) {
            safe_infer_schema(&mut df);
        }
        Ok(df)
    }
}

pub struct JsonToDataFrame {
    infer_schema: InferSchema,
    ignore_errors: bool,
}

impl JsonToDataFrame {
    pub fn from_args(args: &Args) -> Self {
        Self {
            infer_schema: args.infer_schema,
            ignore_errors: args.ignore_errors,
        }
    }
}

impl ReadToDataFrame for JsonToDataFrame {
    fn read_to_data_frame(&self, file: PathBuf) -> AppResult<DataFrame> {
        let mut df = JsonReader::new(File::open(file)?)
            .set_rechunk(true)
            .infer_schema_len(None)
            .with_ignore_errors(self.ignore_errors)
            .finish()?;
        if matches!(
            self.infer_schema,
            InferSchema::Safe | InferSchema::Full | InferSchema::Fast
        ) {
            safe_infer_schema(&mut df);
        }
        Ok(df)
    }
}
