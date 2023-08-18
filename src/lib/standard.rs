use super::*;
use pest::Parser;
use pest_derive::Parser;
use std::io::ErrorKind;

#[derive(Parser)]
#[grammar = "lib/standard.pest"]
pub struct StandardNotation;

impl RollExpression {
    pub fn from_pairs<'i>(mut value: pest::iterators::Pairs<'i, Rule>) -> Vec<RollExpression> {
        let Some(rolls) = value.nth(0) else {
            panic!("no matched patterns!");
        };

        if rolls.as_rule() != Rule::Rolls {
            panic!("not root");
        };

        let expressions: Vec<RollExpression> = rolls
            .into_inner()
            .filter_map(|r| {
                if r.as_rule() == Rule::EOI {
                    None
                } else {
                    Some(r.into())
                }
            })
            .collect();
        expressions
    }
}

impl Notation for StandardNotation {
    fn parse_from_str(input: &str) -> Result<Vec<RollExpression>, std::io::Error> {
        let pairs =
            StandardNotation::parse(Rule::Rolls, input).map_err(|_| ErrorKind::InvalidData)?;
        Ok(RollExpression::from_pairs(pairs))
    }
}

impl<'i> From<pest::iterators::Pair<'i, Rule>> for RollExpression {
    fn from(value: pest::iterators::Pair<'i, Rule>) -> Self {
        if value.as_rule() != Rule::RollExpression {
            panic!("expected a roll expression")
        };
        let roll_expression = value
            .into_inner()
            .collect::<Vec<pest::iterators::Pair<'i, Rule>>>();
        let dice = roll_expression.get(0).expect("no die expression");
        if dice.as_rule() != Rule::Dice {
            panic!("expected a die expression")
        };

        let mut inner = dice.clone().into_inner();
        let mut count: usize = 1;
        let mut faces: usize = 6;

        let mut t = inner.next().unwrap();
        if t.as_rule() == Rule::DiceCount {
            count = t.as_str().parse().unwrap();
            t = inner.next().unwrap();
        }

        if t.as_rule() == Rule::DiceType {
            match t.as_str() {
                "%" => faces = 100,
                n => faces = n.parse().unwrap(),
            }
        }

        let retention = roll_expression
            .iter()
            .find_map(|r| {
                if r.as_rule() == Rule::Retention {
                    let r = r.clone().into_inner().nth(0).unwrap();
                    match r.as_rule() {
                        Rule::RetentionHighest => Some(RollRetention::Highest(
                            r.into_inner().as_str().parse().unwrap(),
                        )),
                        Rule::RetentionLowest => Some(RollRetention::Lowest(
                            r.into_inner().as_str().parse().unwrap(),
                        )),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .unwrap_or(RollRetention::All);

        let modifiers = roll_expression
            .iter()
            .filter_map(|r| match r.as_rule() {
                Rule::Modifier => {
                    let modifier = r.clone().into_inner().nth(0).unwrap();
                    let n = modifier.clone().into_inner().as_str().trim().parse();
                    match modifier.as_rule() {
                        Rule::ModifierAdd => Some(RollModifier::Add(n.unwrap())),
                        Rule::ModifierSubtract => Some(RollModifier::Subtract(n.unwrap())),
                        Rule::ModifierMultiply => Some(RollModifier::Multiply(n.unwrap())),
                        Rule::ModifierDivide => Some(RollModifier::Divide(n.unwrap())),
                        Rule::ModifierExplode => Some(RollModifier::Explode(n.unwrap_or(faces))),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect::<Vec<RollModifier>>();

        RollExpression {
            faces,
            count,
            retention,
            modifiers,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pest::Parser;

    const LEGAL_ROLLS: &'static [&'static str] = &[
        "d20", "1d20", "3d10+3", "10d6 - 5", "d6x4", "8d8 / 2", "d%", "12d%",
    ];
    const ILLEGAL_ROLLS: &'static [&'static str] = &[
        "d0", "0d6", "3d10 3+", "%d", "-2d6", "2/1d8", "d%20", "%d10",
    ];

    #[test]
    pub fn parses_all_examples() {
        for input in LEGAL_ROLLS {
            let res = StandardNotation::parse(Rule::Rolls, input);
            assert!(res.is_ok());
        }
    }

    #[test]
    pub fn fails_all_bad_examples() {
        for input in ILLEGAL_ROLLS {
            let res = StandardNotation::parse(Rule::Rolls, input);
            assert!(res.is_err());
        }
    }
}
