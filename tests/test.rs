use std::convert::TryFrom;
use std::convert::TryInto;
use strnum::StrNum;

#[derive(StrNum, Debug, PartialEq)]
enum Values {
    One,
    Two,
    Three,
    Other(String),
}

#[derive(StrNum, Debug, PartialEq)]
enum RenamedValues {
    #[name = "one"]
    One,
    #[name = "two"]
    Two,
    #[name = "three"]
    Three,
    Other(String),
}

#[derive(StrNum, Debug, PartialEq)]
enum LimitedValues {
    One,
    Two,
    Three,
}

#[test]
fn test_values() {
    assert_eq!(Values::One, "One".into());
    assert_eq!(Values::Two, "Two".into());
    assert_eq!(Values::Three, "Three".into());
    assert_eq!(Values::Other("Four".to_string()), "Four".into());

    assert_eq!("One", Values::to_string(&Values::One));
    assert_eq!("Two", Values::to_string(&Values::Two));
    assert_eq!("Three", Values::to_string(&Values::Three));
    assert_eq!(
        "Four",
        Values::to_string(&Values::Other("Four".to_string()))
    );

    assert_eq!("One", String::from(Values::One));
    assert_eq!("Two", String::from(Values::Two));
    assert_eq!("Three", String::from(Values::Three));
    assert_eq!("Four", String::from(Values::Other("Four".to_string())));
}

#[test]
fn test_renamed() {
    assert_eq!(RenamedValues::One, "one".into());
    assert_eq!(RenamedValues::Two, "two".into());
    assert_eq!(RenamedValues::Three, "three".into());
    assert_eq!(RenamedValues::Other("four".to_string()), "four".into());

    assert_eq!("one", RenamedValues::to_string(&RenamedValues::One));
    assert_eq!("two", RenamedValues::to_string(&RenamedValues::Two));
    assert_eq!("three", RenamedValues::to_string(&RenamedValues::Three));
    assert_eq!(
        "four",
        RenamedValues::to_string(&RenamedValues::Other("four".to_string()))
    );

    assert_eq!("one", String::from(RenamedValues::One));
    assert_eq!("two", String::from(RenamedValues::Two));
    assert_eq!("three", String::from(RenamedValues::Three));
    assert_eq!(
        "four",
        String::from(RenamedValues::Other("four".to_string()))
    );
}

#[test]
fn test_limited() {
    assert_eq!(Ok(LimitedValues::One), "One".try_into());
    assert_eq!(Ok(LimitedValues::Two), "Two".try_into());
    assert_eq!(Ok(LimitedValues::Three), "Three".try_into());
    assert_eq!(Err("four".to_string()), LimitedValues::try_from("four"));

    assert_eq!("One", LimitedValues::to_string(&LimitedValues::One));
    assert_eq!("Two", LimitedValues::to_string(&LimitedValues::Two));
    assert_eq!("Three", LimitedValues::to_string(&LimitedValues::Three));

    assert_eq!("One", String::from(LimitedValues::One));
    assert_eq!("Two", String::from(LimitedValues::Two));
    assert_eq!("Three", String::from(LimitedValues::Three));
}
