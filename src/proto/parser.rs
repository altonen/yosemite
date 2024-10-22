// // Permission is hereby granted, free of charge, to any person obtaining a
// // copy of this software and associated documentation files (the "Software"),
// // to deal in the Software without restriction, including without limitation
// // the rights to use, copy, modify, merge, publish, distribute, sublicense,
// // and/or sell copies of the Software, and to permit persons to whom the
// // Software is furnished to do so, subject to the following conditions:
// //
// // The above copyright notice and this permission notice shall be included in
// // all copies or substantial portions of the Software.
// //
// // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// // OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// // FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// // DEALINGS IN THE SOFTWARE.

use crate::error::I2pError;

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take_until, take_while, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0, one_of, space1},
    combinator::{map, opt, recognize},
    error::{context, make_error, ErrorKind, VerboseError},
    multi::{many0, many0_count},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    Err, IResult, Parser,
};

use std::collections::HashMap;

/// Parsed command.
///
/// Represent a command that had value form but isn't necessarily
/// a command that `yosemite` recognizes.
struct ParsedCommand<'a> {
    /// Command
    ///
    /// Supported values: `HELLO`, `STATUS` and `STREAM`.
    command: &'a str,

    /// Subcommand.
    ///
    /// Supported values: `REPLY` for `HELLO`, `STATUS` for `SESSION`/`STREAM`.
    subcommand: Option<&'a str>,

    /// Parsed key-value pairs.
    key_value_pairs: &'a HashMap<&'a str, &'a str>,
}

/// Response received from SAMv3 server.
#[derive(Debug)]
pub enum Response {
    /// Response to `HELLO` message.
    Hello {
        /// Supported version or an error.
        version: Result<String, I2pError>,
    },

    /// Session message.
    Session {
        // Destination.
        destination: Result<String, I2pError>,
    },

    /// Stream message.
    Stream {
        /// Stream status.
        result: Result<(), I2pError>,
    },
}

impl<'a> TryFrom<ParsedCommand<'a>> for Response {
    type Error = ();

    fn try_from(value: ParsedCommand<'a>) -> Result<Self, Self::Error> {
        match (value.command, value.subcommand) {
            ("HELLO", Some("REPLY")) => match value.key_value_pairs.get("VERSION") {
                Some(version) => Ok(Response::Hello {
                    version: Ok(version.to_string()),
                }),
                None => {
                    // if `VERSION` doesn't exist, `RESULT` is expected to exist as `NOVERSION`
                    // an unexpected error since reporting version is optional as of v3.1 and
                    // `yosemite` doesn't send a version string to the router
                    let result = value.key_value_pairs.get("RESULT").ok_or(())?;
                    let message = value.key_value_pairs.get("MESSAGE");

                    Ok(Response::Hello {
                        version: Err(I2pError::try_from((*result, message.map(|value| *value)))?),
                    })
                }
            },
            ("SESSION", Some("STATUS")) => match value.key_value_pairs.get("DESTINATION") {
                Some(destination) => Ok(Response::Session {
                    destination: Ok(destination.to_string()),
                }),
                None => {
                    let result = value.key_value_pairs.get("RESULT").ok_or(())?;
                    let message = value.key_value_pairs.get("MESSAGE");

                    Ok(Response::Session {
                        destination: Err(I2pError::try_from((
                            *result,
                            message.map(|value| *value),
                        ))?),
                    })
                }
            },
            ("STREAM", Some("STATUS")) => match value.key_value_pairs.get("RESULT") {
                Some(value) if *value == "OK" => Ok(Response::Stream { result: Ok(()) }),
                Some(error) => {
                    let message = value.key_value_pairs.get("MESSAGE");

                    Ok(Response::Stream {
                        result: Err(I2pError::try_from((*error, message.map(|value| *value)))?),
                    })
                }
                None => return Err(()),
            },
            _ => todo!(),
        }
    }
}

impl Response {
    /// Attempt to parse `input` into `Response`.
    //
    // Non-public method returning `IResult` for cleaner error handling.
    fn parse_inner<'a>(input: &'a str) -> IResult<&'a str, Self> {
        let (rest, command) = alt((tag("HELLO"), tag("SESSION"), tag("STREAM")))(input)?;

        let (rest, (command, _, subcommand, _, key_value_pairs)) = tuple((
            alt((tag("HELLO"), tag("SESSION"), tag("STREAM"))),
            opt(char(' ')),
            opt(alt((tag("REPLY"), tag("STATUS")))),
            opt(char(' ')),
            opt(parse_key_value_pairs),
        ))(input)?;

        Ok((
            rest,
            Response::try_from(ParsedCommand {
                command,
                subcommand,
                key_value_pairs: &key_value_pairs.unwrap_or(HashMap::new()),
            })
            .map_err(|_| Err::Error(make_error(input, ErrorKind::Fail)))?,
        ))
    }

    /// Attempt to parse `input` into `Response`.
    pub fn parse(input: &str) -> Option<Self> {
        Some(Self::parse_inner(input).ok()?.1)
    }
}

fn parse_key_value_pairs(input: &str) -> IResult<&str, HashMap<&str, &str>> {
    let (input, key_value_pairs) = many0(preceded(multispace0, parse_key_value))(input)?;
    Ok((input, key_value_pairs.into_iter().collect()))
}

fn parse_key_value(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(parse_key, char('='), parse_value)(input)
}

fn parse_key(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))
    .parse(input)
}

fn parse_value(input: &str) -> IResult<&str, &str> {
    alt((
        parse_quoted_value,
        map(take_while1(|c: char| !c.is_whitespace()), |s: &str| s),
    ))(input)
}

fn parse_quoted_value(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        escaped(is_not("\\\""), '\\', alt((tag("\""), tag("\\")))),
        char('"'),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hello() {
        // success
        match Response::parse("HELLO REPLY RESULT=OK VERSION=3.3") {
            Some(Response::Hello {
                version: Ok(response),
            }) if response == "3.3".to_string() => {}
            response => panic!("invalid response: {response:?}"),
        }

        // failure
        match Response::parse("HELLO REPLY RESULT=I2P_ERROR MESSAGE=\"router error\"") {
            Some(Response::Hello {
                version: Err(error),
            }) if error == I2pError::I2pError(Some("router error".to_string())) => {}
            response => panic!("invalid response: {response:?}"),
        }
    }

    #[test]
    fn invalid_hello() {
        assert!(Response::parse("HELLO REPLY").is_none());
        assert!(Response::parse("HELLO REPLY KEY=VALUE").is_none());
        assert!(Response::parse("HELLO REPLY RESULT=NOVERSION").is_none());
        assert!(Response::parse("HELLO REPLY RESULT=UKNOWN_ERROR").is_none());
        assert!(Response::parse("HELLO REPLY RESULT=OK").is_none());
        assert!(Response::parse("HELLO REPLY MESSAGE=\"hello, world\"").is_none());
    }

    #[test]
    fn unrecognized_command() {
        assert!(Response::parse("TEST COMMAND KEY=VALUE").is_none());
    }

    #[test]
    fn session_status() {
        let response =  "SESSION STATUS RESULT=OK DESTINATION=TIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJkyG6cCLidWPZ3iUHuCcrTeb8MfiOghIzI~n1yyDQV4mTIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJkyG6cCLidWPZ3iUHuCcrTeb8MfiOghIzI~n1yyDQV4mTIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJmRZ8D0ewvPmy2QKbhZTS3Y9B~nR2m~2vf3yPdVWR7pokR0PeHn-vQ8Av0VNEKUete3L7pEvwrm8CxrIY2aUkV~CpNliKwvhfsJe7tSDSL32Ia42O45KTZbGkI9jvKDdFblwoOYpcd1ToDFZ5qWQ0bxACistfpu609-1Tw1y26neAAAA08XrilOIapGsMhNO1WihrFDLOycxcJlTlqbhV1NKKgekUa-RjUuL1n2hx7VjQK2iSK4FNUprfsr1GEIrOvaNKUD4B0fc7Xshbr43oZZ-LE0FxhNdOhz5KOEzW-eqE7V84PTWIfpY9to6Mm1JObl6ARHhVxPvSVQzkNMuuoFQoB2STMOw2osPXxr7tk~qVYnBrrHpZYrfGIyO1tN1MDCJPqTbFaCNb3Jtnxz3h7B~aJFAHzzEl~sHpMJx7IWAaVr-e2mIRin7fywJq3IhuPy8DdAJiIa-8qrjDDrNNg02a3BgSN4If6sTFooGRX-cXnuCjbbqjzg3dq8parcTekauEFtlTl6d17wFQ3o~JtFQ4ObzpGuW";
        let destination = "TIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJkyG6cCLidWPZ3iUHuCcrTeb8MfiOghIzI~n1yyDQV4mTIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJkyG6cCLidWPZ3iUHuCcrTeb8MfiOghIzI~n1yyDQV4mTIbpwIuJ1Y9neJQe4JytN5vwx-I6CEjMj-fXLINBXiZMhunAi4nVj2d4lB7gnK03m~DH4joISMyP59csg0FeJmRZ8D0ewvPmy2QKbhZTS3Y9B~nR2m~2vf3yPdVWR7pokR0PeHn-vQ8Av0VNEKUete3L7pEvwrm8CxrIY2aUkV~CpNliKwvhfsJe7tSDSL32Ia42O45KTZbGkI9jvKDdFblwoOYpcd1ToDFZ5qWQ0bxACistfpu609-1Tw1y26neAAAA08XrilOIapGsMhNO1WihrFDLOycxcJlTlqbhV1NKKgekUa-RjUuL1n2hx7VjQK2iSK4FNUprfsr1GEIrOvaNKUD4B0fc7Xshbr43oZZ-LE0FxhNdOhz5KOEzW-eqE7V84PTWIfpY9to6Mm1JObl6ARHhVxPvSVQzkNMuuoFQoB2STMOw2osPXxr7tk~qVYnBrrHpZYrfGIyO1tN1MDCJPqTbFaCNb3Jtnxz3h7B~aJFAHzzEl~sHpMJx7IWAaVr-e2mIRin7fywJq3IhuPy8DdAJiIa-8qrjDDrNNg02a3BgSN4If6sTFooGRX-cXnuCjbbqjzg3dq8parcTekauEFtlTl6d17wFQ3o~JtFQ4ObzpGuW".to_string();

        // success
        match Response::parse(&response) {
            Some(Response::Session {
                destination: parsed_destination,
            }) if Ok(destination) == parsed_destination => {}
            response => panic!("invalid response: {response:?}"),
        }

        // failure
        match Response::parse("SESSION STATUS RESULT=I2P_ERROR MESSAGE=\"router error\"") {
            Some(Response::Session {
                destination: Err(error),
            }) if error == I2pError::I2pError(Some("router error".to_string())) => {}
            response => panic!("invalid response: {response:?}"),
        }
    }

    #[test]
    fn stream_status() {
        // success
        match Response::parse("STREAM STATUS RESULT=OK") {
            Some(Response::Stream { result: Ok(()) }) => {}
            response => panic!("invalid response: {response:?}"),
        }

        // failure
        match Response::parse("STREAM STATUS RESULT=CANT_REACH_PEER MESSAGE=\"Connection failed\"")
        {
            Some(Response::Stream { result: Err(error) }) if error == I2pError::CantReachPeer => {}
            response => panic!("invalid response: {response:?}"),
        }
    }
}
