/// M15: nom-based parser for the custom .rustpipe DSL format.
///
/// DSL syntax:
/// ```text
/// pipeline my-pipeline
///
/// stage lint
///   runs-on rust:latest
///   step "Run clippy"
///     run cargo clippy -- -D warnings
///   end
/// end
///
/// stage test
///   runs-on rust:latest
///   needs lint
///   step "Run tests"
///     run cargo test --all
///   end
/// end
/// ```
use crate::pipeline::model::{Pipeline, Stage, Step};
use nom::{
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, line_ending, multispace0, not_line_ending, space1},
    combinator::opt,
    multi::many0,
    sequence::{delimited, terminated},
    IResult,
};
use std::collections::HashMap;

type ParseResult<'a, T> = IResult<&'a str, T>;

fn identifier(input: &str) -> ParseResult<'_, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.')(input)
}

fn quoted_string(input: &str) -> ParseResult<'_, &str> {
    delimited(char('"'), take_until("\""), char('"'))(input)
}

fn rest_of_line(input: &str) -> ParseResult<'_, &str> {
    terminated(not_line_ending, opt(line_ending))(input)
}

fn parse_step(input: &str) -> ParseResult<'_, Step> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("step")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = quoted_string(input)?;
    let (input, _) = opt(line_ending)(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = tag("run")(input)?;
    let (input, _) = space1(input)?;
    let (input, run_cmd) = rest_of_line(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = tag("end")(input)?;
    let (input, _) = opt(line_ending)(input)?;

    Ok((
        input,
        Step {
            name: name.to_string(),
            run: run_cmd.trim().to_string(),
            artifact: None,
            retry: None,
        },
    ))
}

fn parse_needs(input: &str) -> ParseResult<'_, Vec<String>> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("needs")(input)?;
    let (input, _) = space1(input)?;
    let (input, line) = rest_of_line(input)?;
    let deps: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
    Ok((input, deps))
}

fn parse_when(input: &str) -> ParseResult<'_, String> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("when")(input)?;
    let (input, _) = space1(input)?;
    let (input, expr) = rest_of_line(input)?;
    Ok((input, expr.trim().to_string()))
}

fn parse_stage(input: &str) -> ParseResult<'_, (String, Stage)> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("stage")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = terminated(identifier, opt(line_ending))(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = tag("runs-on")(input)?;
    let (input, _) = space1(input)?;
    let (input, runs_on) = terminated(identifier, opt(line_ending))(input)?;

    let (input, needs) = opt(parse_needs)(input)?;
    let (input, when) = opt(parse_when)(input)?;
    let (input, steps) = many0(parse_step)(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = tag("end")(input)?;
    let (input, _) = opt(line_ending)(input)?;

    Ok((
        input,
        (
            name.to_string(),
            Stage {
                runs_on: runs_on.to_string(),
                steps,
                needs,
                when,
                timeout_secs: None,
                matrix: None,
                fail_fast: false,
            },
        ),
    ))
}

/// Parse a `.rustpipe` DSL file into a Pipeline.
pub fn parse_dsl(input: &str) -> anyhow::Result<Pipeline> {
    let (input, _) = multispace0::<&str, nom::error::Error<&str>>(input)
        .map_err(|e| anyhow::anyhow!("DSL parse error: {}", e))?;
    let (input, _) = tag::<&str, &str, nom::error::Error<&str>>("pipeline")(input)
        .map_err(|e| anyhow::anyhow!("Expected 'pipeline': {}", e))?;
    let (input, _) = space1::<&str, nom::error::Error<&str>>(input)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let (input, name) = rest_of_line(input)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let (_, stage_list) = many0(parse_stage)(input)
        .map_err(|e| anyhow::anyhow!("DSL stage parse error: {}", e))?;

    let stages: HashMap<String, Stage> = stage_list.into_iter().collect();

    Ok(Pipeline {
        name: name.trim().to_string(),
        trigger: None,
        stages,
        secrets: None,
        notify: None,
    })
}
