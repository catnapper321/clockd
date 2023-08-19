
use nom::{
    character::complete::{char, alpha1, space1, space0, digit1, u8 as parse_u8, i32 as parse_i32, i64 as parse_i64},
    bytes::complete::tag_no_case,
    sequence::separated_pair,
    branch::alt,
    combinator::{map, map_res, map_opt, verify},
    multi::separated_list1,
    IResult,
};
type ParseResult<'a, T> = IResult<&'a str, T>;

#[derive(Debug, Eq, PartialEq)]
pub enum TimePart {
    HM(u8, u8),
    MD(u8, u8),
    Today,
    Tomorrow,
    WeekDay(u8),
    Month(u8),
    Year(i32),
    YMD(i32, u8, u8),
    // BareNumber(u8),
    Hours(i64),
    Minutes(i64),
    Seconds(i64),
}

impl TimePart {

    fn parse_month_name(input: &str) -> ParseResult<TimePart> {
        let mp = map_opt(alpha1, parse_month);
        map(mp, TimePart::Month)(input)
    }

    fn parse_weekday(input: &str) -> ParseResult<TimePart> {
        let wp = map_opt(alpha1, parse_weekday);
        map(wp, TimePart::WeekDay)(input)
    }

    fn parse_hm(input: &str) -> ParseResult<TimePart> {
        let p = separated_pair(parse_u8, char(':'), parse_u8);
        map(p, |(h, m)| Self::HM(h, m))(input) 
    }

    /// accepts formats "11 jun" or "06-11" or "6-11"
    fn parse_md(input: &str) -> ParseResult<TimePart> {
        let p1 = separated_pair(parse_u8, space1, map_opt(alpha1, parse_month));
        let p1a = map(p1, |(d, m)| Self::MD(m, d));
        let p2 = separated_pair(parse_u8, char('-'), parse_u8);
        let p2a = map(p2, |(m, d)| Self::MD(m, d));
        alt((p1a, p2a))(input)
    }
    fn parse_today(input: &str) -> ParseResult<TimePart> {
        map(tag_no_case("today"), |_| Self::Today)(input)
    }
    fn parse_tomorrow(input: &str) -> ParseResult<TimePart> {
        map(tag_no_case("tomorrow"), |_| Self::Tomorrow)(input)
    }
    fn parse_year(input: &str) -> ParseResult<TimePart> {
        let p = verify(digit1, |x: &str| x.len() == 4);
        let p1 = map_res(p, |x: &str| x.parse::<i32>());
        map(p1, Self::Year)(input)
    }
    fn parse_ymd(input: &str) -> ParseResult<TimePart> {
        todo!()        
    }
    fn parse_hours(input: &str) -> ParseResult<TimePart> {
        let p = separated_pair(parse_i64, space0, char('h'));
        map(p, |(x, _)| Self::Hours(x))(input) 
    }
    fn parse_minutes(input: &str) -> ParseResult<TimePart> {
        let p = separated_pair(parse_i64, space0, char('m'));
        map(p, |(x, _)| Self::Minutes(x))(input) 
    }
    fn parse_seconds(input: &str) -> ParseResult<TimePart> {
        let p = separated_pair(parse_i64, space0, char('s'));
        map(p, |(x, _)| Self::Seconds(x))(input) 
    }
    pub fn parse_line(input: &str) -> ParseResult<Vec<TimePart>> {
        let mut p = alt((
            Self::parse_hm,
            Self::parse_md,
            Self::parse_today,
            Self::parse_tomorrow,
            Self::parse_year,
            Self::parse_hours,
            Self::parse_minutes,
            Self::parse_seconds,
            Self::parse_month_name,
            Self::parse_weekday,
                ));
        separated_list1(space1, p)(input)
    }

}

fn parse_weekday(input: &str) -> Option<u8> {
    let input = input.to_lowercase();
    match input.as_str() {
        "sun" => Some(0),
        "mon" => Some(1),
        "tue" => Some(2),
        "wed" => Some(3),
        "thu" => Some(4),
        "fri" => Some(5),
        "sat" => Some(6),
        _ => None,
    }
}

fn parse_month(input: &str) -> Option<u8> {
    let input = input.to_lowercase();
    match input.as_str() {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }
}

#[cfg(test)]
mod checks {
    use super::*;
    #[test]
    fn name() {
        assert!(true);
    }
    #[test]
    fn parsing_hm() {
        let input = "12:30";
        let r = TimePart::parse_hm(input).unwrap();
        assert_eq!(("", TimePart::HM(12, 30)), r);
    }
    #[test]
    fn parsing_md() {
        let input = "16 aug";
        let r = TimePart::parse_md(input).unwrap();
        assert_eq!(("", TimePart::MD(8, 16)), r);
        let input = "08-16";
        let r = TimePart::parse_md(input).unwrap();
        assert_eq!(("", TimePart::MD(8, 16)), r);
    }
    #[test]
    fn parsing_tomorrow() {
        let input = "tomorrow";
        let r = TimePart::parse_tomorrow(input).unwrap();
        assert_eq!(("", TimePart::Tomorrow), r);
    }
    #[test]
    fn parsing_hours() {
        let input = "73h";
        let r = TimePart::parse_hours(input).unwrap();
        assert_eq!(("", TimePart::Hours(73)), r);
    }
    #[test]
    fn parsing_minutes() {
        let input = "71m";
        let r = TimePart::parse_minutes(input).unwrap();
        assert_eq!(("", TimePart::Minutes(71)), r);
    }
    #[test]
    fn parsing_seconds() {
        let input = "3600s";
        let r = TimePart::parse_seconds(input).unwrap();
        assert_eq!(("", TimePart::Seconds(3600)), r);
        let input = "3600 s";
        let r = TimePart::parse_seconds(input).unwrap();
        assert_eq!(("", TimePart::Seconds(3600)), r);
    }
    #[test]
    fn parsing_line() {
        let input = "54h 10m";
        let r = TimePart::parse_line(input).unwrap().1;
        assert_eq!(vec![TimePart::Hours(54), TimePart::Minutes(10)], r);
        let input = "12 sep 1992";
        let r = TimePart::parse_line(input).unwrap().1;
        assert_eq!(vec![TimePart::MD(9, 12), TimePart::Year(1992)], r);
        let input = "tomorrow 13:45";
        let r = TimePart::parse_line(input).unwrap().1;
        assert_eq!(vec![TimePart::Tomorrow, TimePart::HM(13, 45)], r);
    }
}
