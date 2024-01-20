use std::fmt;
use std::fmt::{Display};
use std::ops::Deref;

use ansi::RESET;
use difference::Difference;
use style::{Style, Colour};


/// An `ANSIDisplay` includes a generic Display type and a `Style` to
/// display it.
#[derive(PartialEq, Debug, Clone)]
pub struct ANSIDisplay<'a, T: Display + ?Sized>
{
    style: Style,
    display: &'a T
}

impl<'a, T: Display + ?Sized> From<&'a T> for ANSIDisplay<'a, T> {
    fn from(input: &'a T) -> Self {
        Self {
            style:  Style::default(),
            display: input
        }
    }
}

impl<'a, T: Display + ?Sized> ANSIDisplay<'a, T> {

    /// Directly access the style
    pub fn style_ref(&self) -> &Style {
        &self.style
    }

    /// Directly access the style mutably
    pub fn style_ref_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<'a, T: Display + ?Sized> Deref for ANSIDisplay<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.display
    }
}


/// A set of `ANSIGenericString`s collected together, in order to be
/// written with a minimum of control characters.
#[derive(Debug, PartialEq)]
pub struct ANSIDisplaySlice<'a, T: Display + ?Sized>(pub &'a [ANSIDisplay<'a, T>]);

// ---- paint functions ----

impl Style {

    /// Paints the given text with this colour, returning an ANSI string.
    #[must_use]
    pub fn paint<T: Display + ?Sized>(self, input: &T) -> ANSIDisplay<T> {
        ANSIDisplay {
            display: input,
            style:  self,
        }
    }
}


impl Colour {

    /// Paints the given text with this colour, returning an ANSI string.
    /// This is a short-cut so you don’t have to use `Blue.normal()` just
    /// to get blue text.
    ///
    /// ```
    /// use ansi_term::Colour::Blue;
    /// println!("{}", Blue.paint("da ba dee"));
    /// ```
    #[must_use]
    pub fn paint<T: Display + ?Sized>(self, input: &T) -> ANSIDisplay<T> {
        ANSIDisplay {
            display: input,
            style:  self.normal(),
        }
    }
}


// ---- writers for individual ANSI strings ----

impl<'a, T: Display + ?Sized> Display for ANSIDisplay<'a,  T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.style.prefix())?;
        self.display.fmt(f)?;
        write!(f, "{}", self.style.suffix())
    }
}

// ---- writers for combined ANSI strings ----

impl<'a, T: Display + ?Sized> Display for ANSIDisplaySlice<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Difference::*;

        let first = match self.0.first() {
            None => return Ok(()),
            Some(f) => f,
        };

        write!(f, "{}", first.style.prefix())?;
        first.display.fmt(f)?;

        for window in self.0.windows(2) {
            match Difference::between(&window[0].style, &window[1].style) {
                ExtraStyles(style) => write!(f, "{}", style.prefix())?,
                Reset              => write!(f, "{}{}", RESET, window[1].style.prefix())?,
                NoDifference       => {/* Do nothing! */},
            }

            window[1].display.fmt(f)?;
        }

        // Write the final reset string after all of the ANSIStrings have been
        // written, *except* if the last one has no styles, because it would
        // have already been written by this point.
        if let Some(last) = self.0.last() {
            if !last.style.is_plain() {
                write!(f, "{}", RESET)?;
            }
        }

        Ok(())
    }
}

// ---- tests ----

#[cfg(test)]
mod tests {
    pub use super::super::ANSIDisplaySlice;
    pub use style::Style;

    #[test]
    fn no_control_codes_for_plain() {
        let one = Style::default().paint("one");
        let two = Style::default().paint("two");
        let output = format!("{}", ANSIDisplaySlice( &[ one, two ] ));
        assert_eq!(&*output, "onetwo");
    }
}
