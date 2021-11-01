use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;
use anyhow::{Error, Result};
mod dotnet;
mod node;
mod python;

#[derive(Default, Debug, Deserialize, Clone)]
pub struct RawCloudWatchLog {
    pub time: String,
    pub r#type: String,
    pub record: serde_json::Value,
}

#[derive(Debug, Serialize, Clone)]
pub struct StructuredLog {
    pub timestamp: Option<String>,
    pub guid: Option<String>,
    pub level: Option<LogLevel>,
    pub data: Value,
}

impl TryFrom<String> for LogLevel {
    type Error = anyhow::Error;
    fn try_from(level: String) -> Result<Self> {
        match level.as_str() {
            "INFO" => Ok(LogLevel::Info),
            "WARN" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            _ => Err(Error::msg(format!("Unable to parse {} as LogLevel", level))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Log {
    Unformatted(StructuredLog),
    Formatted(serde_json::Value),
}


impl ToString for Log {
    fn to_string(&self) -> String {
        match self {
            Log::Unformatted(data) => serde_json::to_string(data).unwrap(),
            Log::Formatted(data) => data.to_string(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum LogLevel {
    #[serde(rename(serialize = "INFO"))]
    Info,
    #[serde(rename(serialize = "WARN"))]
    Warn,
    #[serde(rename(serialize = "ERROR"))]
    Error,
}

pub fn parse(logs: Vec<RawCloudWatchLog>) -> Vec<Log> {
    logs.into_iter()
        .filter(|log| match log.r#type.as_str() {
            "function" => true,
            _ => {
                println!("{:?}", log);
                false
            }
        })
        .map(|log| match log.record {
            Value::String(_) => try_parse_cloudwatch_log(&log),
            _ => Err(Error::msg(format!("Expected String {}", log.record))),
        })
        .flatten()
        .collect()
}

fn try_parse_cloudwatch_log(log: &RawCloudWatchLog) -> Result<Log> {
    match node::parse(log) {
        Some(dto) => {
            return Ok(dto);
        }
        _ => (),
    };
    match python::parse(log) {
        Some(dto) => {
            return Ok(dto);
        }
        _ => (),
    };
    match dotnet::parse(log) {
        Some(dto) => {
            return Ok(dto);
        }
        _ => (),
    };
    Err(Error::msg(format!("Unable to parse {:?}", log)))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::try_parse_cloudwatch_log;
    use crate::{LogLevel, RawCloudWatchLog, Log};

    #[test]
    fn can_parse_node() {
        let input =
            RawCloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\tHello World\n".to_string())
                , ..Default::default()
            };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output.unwrap() {
            Log::Unformatted(log) => {
                assert_eq!(log.timestamp.unwrap(), "2020-11-18T23:52:30.128Z");
                assert_eq!(log.guid.unwrap(), "6e48723a-1596-4313-a9af-e4da9214d637");
                assert_eq!(log.level.unwrap(), LogLevel::Info);
                assert_eq!(log.data, "Hello World\n");
            },
            _ => {
                panic!("Expected Cloudwatch formatted log");
            }
        }
    }

    #[test]
    fn can_parse_python() {
        let input = RawCloudWatchLog {
            record: serde_json::Value::String(
                "[INFO]	2020-11-18T23:52:30.128Z    6e48723a-1596-4313-a9af-e4da9214d637	Hello World\n"
                    .to_string(),
            ),
            ..Default::default()
        };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output.unwrap() {
            Log::Unformatted(log) => {
                assert_eq!(log.timestamp.unwrap(), "2020-11-18T23:52:30.128Z");
                assert_eq!(log.guid.unwrap(), "6e48723a-1596-4313-a9af-e4da9214d637");
                assert_eq!(log.level.unwrap(), LogLevel::Info);
                assert_eq!(log.data, "Hello World\n");
            },
            _ => {
                panic!("Expected Cloudwatch formatted log");
            }
        }
    }

    #[test]
    fn can_parse_dotnet() {
        let input = RawCloudWatchLog {
            record: serde_json::Value::String(
                "{ \"statusCode\": 200, \"body\": \"DotNet\" }".to_string(),
            ),
            time: "2020-11-18T23:52:30.128Z".to_string(),
            ..Default::default()
        };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output.unwrap() {
            Log::Formatted(log) => {
                assert_eq!(log["body"], "DotNet");
                assert_eq!(log["statusCode"], 200);
            }
            _ => {
                panic!("Expected Preformatted log");
            }
        }
    }

    #[test]
    fn cannot_parse() {
        let input = RawCloudWatchLog { record: serde_json::Value::String("Bad log".to_string()), ..Default::default()};
        let output = try_parse_cloudwatch_log(&input);
        assert_eq!(output.is_err(), true);
    }
}
