// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result, bail, ensure};

use super::expression::{Arithmetic, Comparison, Condition, Expression};
use crate::sexp::Sexp;

/// Parser of expressions in which each of `names` stands for the input at the
/// same position.
pub struct Parser<'a> {
    names: &'a [&'a str],
}

impl<'a> Parser<'a> {
    pub fn new(names: &'a [&'a str]) -> Self {
        Self { names }
    }

    /// Number, name, `if`, `cond`, or arithmetic such as `(+ a b)`.
    pub fn expression(&self, sexp: &Sexp) -> Result<Expression> {
        match sexp {
            Sexp::Number(number) if number.is_finite() => Ok(Expression::Number(*number)),
            Sexp::Number(_) => bail!("expected a finite number, found {sexp}"),
            Sexp::Symbol(name) => self.input(name),
            Sexp::List(items) => match items.split_first() {
                Some((Sexp::Symbol(operator), operands)) => self.operation(operator, operands),
                _ => bail!("expected (operator operand ...), found {sexp}"),
            },
            Sexp::String(_) => bail!("expected a number, found {sexp}"),
        }
    }

    /// Comparison such as `(< a b)`, `missing?`, `not`, `and`, or `or`.
    pub fn condition(&self, sexp: &Sexp) -> Result<Condition> {
        let Some((Sexp::Symbol(operator), operands)) =
            sexp.as_list().and_then(<[Sexp]>::split_first)
        else {
            bail!("expected a condition such as (< a b), found {sexp}");
        };
        match operator.as_str() {
            "missing?" => {
                let [operand] = operands else {
                    bail!("`missing?` expects one operand");
                };
                Ok(Condition::Missing(Box::new(self.expression(operand)?)))
            }
            "not" => {
                let [condition] = operands else {
                    bail!("`not` expects one operand");
                };
                Ok(Condition::Not(Box::new(self.condition(condition)?)))
            }
            "and" => Ok(Condition::And(self.conditions(operands)?)),
            "or" => Ok(Condition::Or(self.conditions(operands)?)),
            _ => {
                let comparison = comparison(operator)
                    .with_context(|| format!("unknown condition `{operator}`"))?;
                let [left, right] = operands else {
                    bail!("`{operator}` expects two operands");
                };
                Ok(Condition::Comparison(
                    comparison,
                    Box::new(self.expression(left)?),
                    Box::new(self.expression(right)?),
                ))
            }
        }
    }

    fn input(&self, name: &str) -> Result<Expression> {
        self.names
            .iter()
            .position(|bound| *bound == name)
            .map(Expression::Input)
            .with_context(|| {
                format!("unbound name `{name}`; list it in (pixelwise (name ...) ...)")
            })
    }

    fn operation(&self, operator: &str, operands: &[Sexp]) -> Result<Expression> {
        match operator {
            "if" => {
                let [condition, consequent, alternative] = operands else {
                    bail!("`if` expects a condition, a consequent, and an alternative");
                };
                Ok(Expression::Conditional(
                    vec![(self.condition(condition)?, self.expression(consequent)?)],
                    Box::new(self.expression(alternative)?),
                ))
            }
            "cond" => self.cond(operands),
            _ => {
                let arithmetic = arithmetic(operator)
                    .with_context(|| format!("unknown operator `{operator}`"))?;
                ensure!(
                    operands.len() >= 2,
                    "`{operator}` expects at least two operands"
                );
                let operands = operands
                    .iter()
                    .map(|operand| self.expression(operand))
                    .collect::<Result<_>>()?;
                Ok(Expression::Arithmetic(arithmetic, operands))
            }
        }
    }

    /// `(cond (condition value) ... (else value))`.
    fn cond(&self, clauses: &[Sexp]) -> Result<Expression> {
        let Some((last, clauses)) = clauses.split_last() else {
            bail!("`cond` expects at least an else clause");
        };
        let alternative = match last.as_list() {
            Some([Sexp::Symbol(keyword), value]) if keyword == "else" => self.expression(value)?,
            _ => bail!("expected (else value) as the last clause of `cond`, found {last}"),
        };
        let clauses = clauses
            .iter()
            .map(|clause| {
                let Some([condition, value]) = clause.as_list() else {
                    bail!("expected a clause (condition value), found {clause}");
                };
                Ok((self.condition(condition)?, self.expression(value)?))
            })
            .collect::<Result<_>>()?;
        Ok(Expression::Conditional(clauses, Box::new(alternative)))
    }

    fn conditions(&self, operands: &[Sexp]) -> Result<Vec<Condition>> {
        operands
            .iter()
            .map(|operand| self.condition(operand))
            .collect()
    }
}

fn arithmetic(operator: &str) -> Option<Arithmetic> {
    Some(match operator {
        "+" => Arithmetic::Add,
        "-" => Arithmetic::Subtract,
        "*" => Arithmetic::Multiply,
        "/" => Arithmetic::Divide,
        "min" => Arithmetic::Minimum,
        "max" => Arithmetic::Maximum,
        _ => return None,
    })
}

fn comparison(operator: &str) -> Option<Comparison> {
    Some(match operator {
        "<" => Comparison::Less,
        "<=" => Comparison::LessOrEqual,
        ">" => Comparison::Greater,
        ">=" => Comparison::GreaterOrEqual,
        "=" => Comparison::Equal,
        _ => return None,
    })
}
