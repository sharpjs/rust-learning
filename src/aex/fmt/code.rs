// Code Output Formatting
//
// This file is part of AEx.
// Copyright (C) 2017 Jeffrey Sharp
//
// AEx is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published
// by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
//
// AEx is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See
// the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with AEx.  If not, see <http://www.gnu.org/licenses/>.


// -----------------------------------------------------------------------------

pub trait ToStyled: Context {
    fn styled<'a>(&'a self, style: &'a Style<Self::Context>) -> Styled<'a, Self>;
}

impl<T> ToStyled for T where T: Context {
    #[inline(always)]
    fn styled<'a>(&'a self, style: &'a Style<Self::Context>) -> Styled<'a, Self> {
        //Styled::new(self, style)
        panic!()
    }
}

// -----------------------------------------------------------------------------


impl<'a, T> Styled<'a, T> where T: 'a + Context + ?Sized {

    /// Maps a `Styled<T>` to `Styled<U>` by applying a function to the
    /// contained node.
    pub fn map<F, U>(&self, f: F, prec: Prec) -> Styled<U>
    where F: FnOnce(&T) -> &U,
          U: Context<Context=T::Context> {
        Styled {
            node:  f(self.node),
            style: self.style,
            prec:  prec,
        }
    }
}

