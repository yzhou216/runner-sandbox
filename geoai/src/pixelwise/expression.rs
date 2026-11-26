// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Expressions evaluated at every pixel.
//!
//! Any value may be missing, and a missing operand makes the result missing.
//! An arithmetic result that is not a finite number, such as a quotient by
//! zero, is missing too.  Only [`Condition::Missing`] observes a missing
//! value without propagating it, and [`Condition::And`] and [`Condition::Or`]
//! stop at the first operand that decides them, so a guard written first
//! protects the tests after it.

/// Numeric expression.
#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    /// Finite number.
    Number(f64),
    /// Value of the input at this position among the bindings of `pixelwise`.
    Input(usize),
    /// Operator applied from left to right across two or more operands.
    Arithmetic(Arithmetic, Vec<Expression>),
    /// Value of the first clause whose condition holds, or else of the
    /// alternative, as `cond` and `if` choose.
    Conditional(Vec<(Condition, Expression)>, Box<Expression>),
}

/// Boolean expression.
#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    Comparison(Comparison, Box<Expression>, Box<Expression>),
    Missing(Box<Expression>),
    Not(Box<Condition>),
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
    Minimum,
    Maximum,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
}

impl Expression {
    /// Value given the value of each input by position, or `None` if missing.
    pub fn evaluate(&self, input: &impl Fn(usize) -> Option<f64>) -> Option<f64> {
        match self {
            Self::Number(number) => Some(*number),
            Self::Input(index) => input(*index),
            Self::Arithmetic(operator, operands) => operands
                .iter()
                .map(|operand| operand.evaluate(input))
                .reduce(|left, right| operator.apply(left?, right?))
                .flatten(),
            Self::Conditional(clauses, alternative) => {
                let chosen = clauses
                    .iter()
                    .map(|(condition, value)| (condition.evaluate(input), value))
                    .find(|(holds, _)| *holds != Some(false));
                match chosen {
                    Some((Some(true), value)) => value.evaluate(input),
                    // Condition that is missing, which makes the result missing
                    Some(_) => None,
                    None => alternative.evaluate(input),
                }
            }
        }
    }
}

impl Condition {
    /// Truth given the value of each input by position, or `None` if missing.
    pub fn evaluate(&self, input: &impl Fn(usize) -> Option<f64>) -> Option<bool> {
        match self {
            Self::Comparison(comparison, left, right) => {
                Some(comparison.apply(left.evaluate(input)?, right.evaluate(input)?))
            }
            Self::Missing(operand) => Some(operand.evaluate(input).is_none()),
            Self::Not(condition) => condition.evaluate(input).map(|holds| !holds),
            Self::And(conditions) => conditions
                .iter()
                .map(|condition| condition.evaluate(input))
                .find(|holds| *holds != Some(true))
                .unwrap_or(Some(true)),
            Self::Or(conditions) => conditions
                .iter()
                .map(|condition| condition.evaluate(input))
                .find(|holds| *holds != Some(false))
                .unwrap_or(Some(false)),
        }
    }
}

impl Arithmetic {
    /// Result of the operator, unless it is not a finite number.
    fn apply(self, left: f64, right: f64) -> Option<f64> {
        let result = match self {
            Self::Add => left + right,
            Self::Subtract => left - right,
            Self::Multiply => left * right,
            Self::Divide => left / right,
            Self::Minimum => left.min(right),
            Self::Maximum => left.max(right),
        };
        Some(result).filter(|result| result.is_finite())
    }
}

impl Comparison {
    fn apply(self, left: f64, right: f64) -> bool {
        match self {
            Self::Less => left < right,
            Self::LessOrEqual => left <= right,
            Self::Greater => left > right,
            Self::GreaterOrEqual => left >= right,
            Self::Equal => left == right,
        }
    }
}
