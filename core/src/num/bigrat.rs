use crate::num::biguint::BigUint;
use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::{Add, Mul, Neg, Sub};

mod sign {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Sign {
        Positive,
        Negative,
    }

    impl Sign {
        pub fn flip(self) -> Self {
            match self {
                Self::Positive => Self::Negative,
                Self::Negative => Self::Positive,
            }
        }

        pub fn sign_of_product(a: Self, b: Self) -> Self {
            match (a, b) {
                (Sign::Positive, Sign::Positive) => Sign::Positive,
                (Sign::Positive, Sign::Negative) => Sign::Negative,
                (Sign::Negative, Sign::Positive) => Sign::Negative,
                (Sign::Negative, Sign::Negative) => Sign::Positive,
            }
        }
    }
}

use sign::Sign;

#[derive(Clone)]
pub struct BigRat {
    sign: Sign,
    num: BigUint,
    den: BigUint,
    exact: bool,
}

impl Ord for BigRat {
    fn cmp(&self, other: &BigRat) -> Ordering {
        let diff = self - other;
        if diff.num == 0.into() {
            Ordering::Equal
        } else if diff.sign == Sign::Positive {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for BigRat {
    fn partial_cmp(&self, other: &BigRat) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BigRat {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for BigRat {}

impl BigRat {
    /// compute a + b
    fn add_internal(self, rhs: BigRat) -> BigRat {
        // a + b == -((-a) + (-b))
        if self.sign == Sign::Negative {
            return -((-self).add_internal(-rhs));
        }

        assert_eq!(self.sign, Sign::Positive);

        if self.den == rhs.den {
            if rhs.sign == Sign::Negative && self.num < rhs.num {
                BigRat {
                    sign: Sign::Negative,
                    num: rhs.num - self.num,
                    den: self.den,
                    exact: self.exact && rhs.exact,
                }
            } else {
                BigRat {
                    sign: Sign::Positive,
                    num: if rhs.sign == Sign::Positive {
                        self.num + rhs.num
                    } else {
                        self.num - rhs.num
                    },
                    den: self.den,
                    exact: self.exact && rhs.exact,
                }
            }
        } else {
            let new_denominator = BigUint::lcm(self.den.clone(), rhs.den.clone());
            let a = self.num * new_denominator.clone() / self.den;
            let b = rhs.num * new_denominator.clone() / rhs.den;
            if rhs.sign == Sign::Negative && a < b {
                BigRat {
                    sign: Sign::Positive,
                    num: b - a,
                    den: new_denominator,
                    exact: self.exact && rhs.exact,
                }
            } else {
                BigRat {
                    sign: Sign::Positive,
                    num: if rhs.sign == Sign::Positive {
                        a + b
                    } else {
                        a - b
                    },
                    den: new_denominator,
                    exact: self.exact && rhs.exact,
                }
            }
        }
    }
}

impl Add for BigRat {
    type Output = BigRat;

    fn add(self, rhs: BigRat) -> BigRat {
        self.add_internal(rhs)
    }
}

impl Neg for BigRat {
    type Output = BigRat;

    fn neg(self) -> BigRat {
        BigRat {
            sign: self.sign.flip(),
            num: self.num,
            den: self.den,
            exact: self.exact,
        }
    }
}

impl Neg for &BigRat {
    type Output = BigRat;

    fn neg(self) -> BigRat {
        -self.clone()
    }
}

impl Sub for BigRat {
    type Output = BigRat;

    fn sub(self, rhs: BigRat) -> BigRat {
        self.add_internal(-rhs)
    }
}

impl Sub for &BigRat {
    type Output = BigRat;

    fn sub(self, rhs: &BigRat) -> BigRat {
        self.clone().add_internal(-rhs.clone())
    }
}

impl Mul for BigRat {
    type Output = BigRat;

    fn mul(self, rhs: BigRat) -> BigRat {
        #[allow(clippy::suspicious_arithmetic_impl)]
        BigRat {
            sign: Sign::sign_of_product(self.sign, rhs.sign),
            num: self.num * rhs.num,
            den: self.den * rhs.den,
            exact: self.exact && rhs.exact,
        }
    }
}

impl From<u64> for BigRat {
    fn from(i: u64) -> Self {
        BigRat {
            sign: Sign::Positive,
            num: i.into(),
            den: 1.into(),
            exact: true,
        }
    }
}

impl From<i32> for BigRat {
    fn from(i: i32) -> Self {
        use std::convert::TryInto;

        if let Ok(j) = TryInto::<u64>::try_into(i) {
            BigRat {
                sign: Sign::Positive,
                num: j.into(),
                den: 1.into(),
                exact: true,
            }
        } else {
            let j: u64 = (-i).try_into().unwrap();
            BigRat {
                sign: Sign::Negative,
                num: j.into(),
                den: 1.into(),
                exact: true,
            }
        }
    }
}

impl BigRat {
    fn simplify(mut self) -> BigRat {
        if self.den == 1.into() {
            return self;
        }
        let gcd = BigUint::gcd(self.num.clone(), self.den.clone());
        self.num = self.num / gcd.clone();
        self.den = self.den / gcd;
        self
    }

    pub fn div(self, rhs: BigRat) -> Result<BigRat, String> {
        if rhs.num == 0.into() {
            return Err("Attempt to divide by zero".to_string());
        }
        #[allow(clippy::suspicious_arithmetic_impl)]
        Ok(BigRat {
            sign: Sign::sign_of_product(self.sign, rhs.sign),
            num: self.num * rhs.den,
            den: self.den * rhs.num,
            exact: self.exact && rhs.exact,
        })
    }

    pub fn pow(mut self, mut rhs: BigRat) -> Result<BigRat, String> {
        self = self.simplify();
        rhs = rhs.simplify();
        if rhs.den != 1.into() {
            return Err("Non-integer exponents not currently supported".to_string());
        }
        if rhs.sign == Sign::Negative {
            // a^-b => 1/a^b
            rhs.sign = Sign::Positive;
            return Ok(BigRat::from(1).div(self.pow(rhs)?)?);
        }
        Ok(BigRat {
            sign: Sign::Positive,
            num: BigUint::pow(self.num, rhs.num.clone())?,
            den: BigUint::pow(self.den, rhs.num)?,
            exact: self.exact && rhs.exact,
        })
    }

    // test if this fraction has a terminating representation
    // e.g. in base 10: 1/4 = 0.25, but not 1/3
    fn terminates_in_base(&self, base: BigUint) -> bool {
        let mut x = self.clone();
        let base = BigRat {
            sign: Sign::Positive,
            num: base,
            den: 1.into(),
            exact: true,
        };
        loop {
            let old_den = x.den.clone();
            x = (x * base.clone()).simplify();
            let new_den = x.den.clone();
            if new_den == old_den {
                break;
            }
        }
        x.den == 1.into()
    }

    // This method is dangerous!! Use this method only when the number has *not* been
    // simplified or otherwise changed.
    pub fn add_decimal_digit(&mut self, digit: u64) {
        self.num = self.num.clone() * 10.into() + digit.into();
        self.den = self.den.clone() * 10.into();
    }

    pub fn approx_pi() -> BigRat {
        BigRat {
            sign: Sign::Positive,
            num: BigUint::from(1068966896),
            den: BigUint::from(340262731),
            exact: false,
        }
    }
}

impl Display for BigRat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if !self.exact {
            write!(f, "approx. ")?;
        }
        let mut x = self.clone().simplify();
        let negative = x.sign == Sign::Negative;
        if negative {
            x.sign = Sign::Positive;
            write!(f, "-")?;
        };
        if x.den == 1.into() {
            write!(f, "{}", x.num)?;
        } else {
            let terminating = x.terminates_in_base(10.into());
            if !terminating && x.exact {
                write!(f, "{}/{}, approx. ", x.num, x.den)?;
                if negative {
                    write!(f, "-")?;
                }
            }
            let num_trailing_digits_to_print = if terminating { std::usize::MAX } else { 10 };
            let integer_part = x.num.clone() / x.den.clone();
            write!(f, "{}.", integer_part)?;
            let integer_as_rational = BigRat {
                sign: Sign::Positive,
                num: integer_part,
                den: 1.into(),
                exact: x.exact,
            };
            let mut remaining_fraction = x.clone() - integer_as_rational;
            let mut i = 0;
            while remaining_fraction.num > 0.into() && i < num_trailing_digits_to_print {
                remaining_fraction = (remaining_fraction * 10.into()).simplify();
                let digit = remaining_fraction.num.clone() / remaining_fraction.den.clone();
                write!(f, "{}", digit)?;
                remaining_fraction = remaining_fraction
                    - BigRat {
                        sign: Sign::Positive,
                        num: digit,
                        den: 1.into(),
                        exact: x.exact,
                    };
                i += 1;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for BigRat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BigRat;

    #[test]
    fn test_bigrat_from() {
        BigRat::from(2);
        BigRat::from(0);
        BigRat::from(-5);
    }

    #[test]
    fn test_addition() {
        eprintln!("{:?}", "yay");
        assert_eq!(BigRat::from(2) + BigRat::from(2), BigRat::from(4));
    }
}
