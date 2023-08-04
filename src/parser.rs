use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_until},
    character::complete::{digit1, multispace0, multispace1},
    multi::many0,
    sequence::{delimited, preceded},
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct ParsedTask<'a> {
    pub index: Option<usize>,
    pub command: Option<&'a str>,
    pub args: Vec<&'a str>,
}

pub struct InputIterator<'a> {
    input: &'a str,
}

impl<'a> InputIterator<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }
}

impl<'a> Iterator for InputIterator<'a> {
    type Item = ParsedTask<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.input = self.input.trim_start_matches(|c| "; \t".contains(c));
        if self.input.is_empty() {
            return None;
        }
        match parse_item(self.input) {
            Ok((input, parsed)) => {
                self.input = input;
                Some(parsed)
            }
            // If error, skip to next item
            Err(_) => {
                self.input = self.input.trim_start_matches(|c| c != ';');
                self.next()
            }
        }
    }
}

fn parse_item(input: &str) -> IResult<&str, ParsedTask> {
    let index: Option<usize>;
    let command: Option<&str>;
    let remainder: &str;
    match preceded(multispace0::<&str, nom::error::Error<&str>>, digit1)(input) {
        Ok((rem, i)) => {
            index = Some(i.parse::<usize>().unwrap());
            command = None;
            remainder = rem;
        }
        Err(_) => {
            index = None;
            let (rest, parsed_command) = preceded(multispace0, arg)(input)?;
            (remainder, command) = (rest, Some(parsed_command));
        }
    };
    let (input, args) = many0(preceded(multispace1, alt((quoted_string, arg))))(remainder)?;
    Ok((
        input,
        ParsedTask {
            index,
            command,
            args,
        },
    ))
}

fn quoted_string(input: &str) -> IResult<&str, &str> {
    let single_quoted = delimited(tag("'"), take_until("'"), tag("'"));
    let double_quoted = delimited(tag("\""), take_until("\""), tag("\""));
    alt((single_quoted, double_quoted))(input)
}

fn arg(input: &str) -> IResult<&str, &str> {
    take_till1(|c| " \t;".contains(c))(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(quoted_string("'hello'"), Ok(("", "hello")));
        assert_eq!(quoted_string("\"hello\""), Ok(("", "hello")));
        assert_eq!(quoted_string("\"'ello, gov\" "), Ok((" ", "'ello, gov")));
        // It's important these error so `alt` switches to the 'arg' parser
        assert!(quoted_string("hello").is_err());
        assert!(quoted_string("arg_1 'hello'").is_err());
        assert!(quoted_string("'hello").is_err());
        assert!(quoted_string(" hello").is_err());
        assert!(quoted_string(" 'hello'").is_err());
    }

    #[test]
    fn test_parse_arg() {
        assert_eq!(arg("hello"), Ok(("", "hello")));
        assert_eq!(arg("hello world"), Ok((" world", "hello")));
        assert_eq!(arg("a1b2_c3 d2"), Ok((" d2", "a1b2_c3")));
        assert!(arg(" hello").is_err());
    }

    #[test]
    fn test_parse_item_with_index() {
        let expected = ParsedTask {
            index: Some(1),
            command: None,
            args: vec!["hello", "world"],
        };
        let (input, parsed) = parse_item("1 hello world; 4").unwrap();
        assert_eq!(input, "; 4");
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_item_with_index_no_args() {
        let expected = ParsedTask {
            index: Some(4),
            command: None,
            args: vec![],
        };
        let (input, parsed) = parse_item("4;1 hello world").unwrap();
        assert_eq!(input, ";1 hello world");
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_item_with_command() {
        let expected = ParsedTask {
            index: None,
            command: Some("echo"),
            args: vec!["hello world"],
        };
        let (input, parsed) = parse_item("echo 'hello world';3").unwrap();
        assert_eq!(input, ";3");
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_collecting_tasks() {
        let expected = vec![
            ParsedTask {
                index: Some(7),
                command: None,
                args: vec![],
            },
            ParsedTask {
                index: Some(4),
                command: None,
                args: vec!["staging"],
            },
            ParsedTask {
                index: None,
                command: Some("echo"),
                args: vec!["hello; world"],
            },
        ];
        let collected: Vec<ParsedTask<'_>> =
            InputIterator::new("7; 4 staging;echo 'hello; world'").collect();
        assert_eq!(collected, expected);
    }
}
