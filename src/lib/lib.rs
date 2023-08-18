use rand::Rng;
use std::{fmt::Debug, fmt::Display, io::Error};

pub mod standard;

#[derive(Debug)]
pub enum RollRetention {
    Highest(usize),
    Lowest(usize),
    All,
}

#[derive(Debug, Clone)]
pub enum RollQuality {
    Good,
    Regular,
    Bad,
}

#[derive(Debug, Clone)]
pub struct RollItem {
    pub value: usize,
    pub retained: bool,
    pub quality: RollQuality,
}

#[derive(Debug)]
pub struct RollResult {
    pub input: String,
    pub total: isize,
    pub rolls: Vec<RollItem>,
}

impl Display for RollResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::{ColoredString, Colorize};

        write!(f, "{:<10}: {:<4} [", self.input, self.total)?;

        for (i, r) in self.rolls.iter().enumerate() {
            let mut k: ColoredString = r.value.to_string().normal();
            if !r.retained {
                k = k.strikethrough();
            }
            match r.quality {
                RollQuality::Good => {
                    k = k.green();
                }
                RollQuality::Bad => {
                    k = k.red();
                }
                _ => {}
            }
            write!(f, "{}", k)?;

            if i != self.rolls.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}

#[derive(Debug)]
pub enum RollModifier {
    Add(usize),
    Subtract(usize),
    Multiply(usize),
    Divide(usize),
    Explode(usize),
}

#[derive(Debug)]
pub struct RollExpression {
    pub faces: usize,
    pub count: usize,
    pub retention: RollRetention,
    pub modifiers: Vec<RollModifier>,
}

impl RollExpression {
    fn explodes_at(&self) -> Option<usize> {
        self.modifiers.iter().find_map(|m| match m {
            RollModifier::Explode(n) => {
                if *n <= self.faces && *n >= 1 {
                    Some(*n)
                } else {
                    panic!("Cannot explode above {}", n)
                }
            }
            _ => None,
        })
    }
}

pub trait Roll {
    fn roll(&mut self) -> RollResult;
}

impl Roll for RollExpression {
    fn roll(&mut self) -> RollResult {
        let mut rng = rand::thread_rng();
        let mut rolls: Vec<RollItem> = Vec::with_capacity(self.count);

        let explode_at = self.explodes_at();

        for _ in 0..self.count {
            let mut value = rng.gen_range(1..=self.faces);
            rolls.push(RollItem {
                value,
                retained: true,
                quality: match value {
                    v if v == self.faces => RollQuality::Good,
                    1 => RollQuality::Bad,
                    _ => RollQuality::Regular,
                },
            });

            if let Some(n) = explode_at {
                while value >= n {
                    value = rng.gen_range(1..=self.faces);
                    rolls.push(RollItem {
                        value,
                        retained: true,
                        quality: match value {
                            v if v == self.faces => RollQuality::Good,
                            1 => RollQuality::Bad,
                            _ => RollQuality::Regular,
                        },
                    });
                }
            }
        }

        match self.retention {
            RollRetention::Highest(n) => {
                if n > self.count {
                    panic!("cannot remove that many");
                }
                let mut removals = rolls.iter().map(|d| d.value).collect::<Vec<usize>>();
                removals.sort();
                removals = removals
                    .into_iter()
                    .take(rolls.len() - n)
                    .collect::<Vec<usize>>();
                rolls.iter_mut().for_each(|i| {
                    if let Some(idx) = removals.iter().position(|j| *j == i.value) {
                        removals.remove(idx);
                        i.retained = false;
                    }
                });
            }
            RollRetention::Lowest(n) => {
                if n > self.count {
                    panic!("cannot remove that many");
                }
                let mut removals = rolls.iter().map(|d| d.value).collect::<Vec<usize>>();
                removals.sort();
                removals.reverse();
                removals = removals
                    .into_iter()
                    .take(rolls.len() - n)
                    .collect::<Vec<usize>>();
                rolls.iter_mut().for_each(|i| {
                    if let Some(n) = removals.iter().position(|n| *n == i.value) {
                        removals.remove(n);
                        i.retained = false;
                    }
                });
            }
            RollRetention::All => {}
        }

        let mut total: isize = rolls.iter().fold(0, |acc, curr| {
            if curr.retained {
                acc + curr.value as isize
            } else {
                acc
            }
        });

        for m in self.modifiers.iter() {
            match m {
                RollModifier::Add(n) => total += *n as isize,
                RollModifier::Subtract(n) => total -= *n as isize,
                RollModifier::Multiply(n) => total *= *n as isize,
                RollModifier::Divide(n) => total /= *n as isize,
                _ => {}
            }
        }

        let mod_str = self
            .modifiers
            .iter()
            .map(|m| match m {
                RollModifier::Add(n) => format!("+{}", n),
                RollModifier::Subtract(n) => format!("-{}", n),
                RollModifier::Multiply(n) => format!("x{}", n),
                RollModifier::Divide(n) => format!("/{}", n),
                RollModifier::Explode(n) => {
                    if *n != self.faces {
                        format!("!{}", n)
                    } else {
                        "!".to_string()
                    }
                }
            })
            .collect::<String>();

        let ret_str = match self.retention {
            RollRetention::All => String::new(),
            RollRetention::Highest(n) => format!("h{}", n),
            RollRetention::Lowest(n) => format!("l{}", n),
        };

        let input = format!("{}d{}{}{}", self.count, self.faces, ret_str, mod_str);

        RollResult {
            total,
            rolls,
            input,
        }
    }
}

pub trait Notation {
    fn parse_from_str(input: &str) -> Result<Vec<RollExpression>, Error>;
}
