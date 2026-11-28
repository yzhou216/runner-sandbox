// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The operations of hint mode: the walk that finds the places that the
//! operator can see, and the one that reaches a widget by its bounds
//!
//! The walk follows what the operator can see, which iced's operations do not
//! tell, since a scrollable reports how far it has scrolled and then hands its
//! content out at the layout that it had before.

use std::{any::Any, mem};

use iced::{
    Point, Rectangle, Size, Task, Vector,
    advanced::widget::{
        Id, Operation, operate,
        operation::{Focusable, Outcome, Scrollable},
    },
};

use super::{Hidden, Site, Target, Window};

/// Where the walk stands with regard to what the operator can see
#[derive(Clone, Copy)]
struct Sight {
    /// Part of the window that the scrollables around the walk leave in view
    clip: Rectangle,
    /// How far the scrollables around the walk have moved their content
    shift: Vector,
    /// Number of hidden regions that the walk is inside
    hidden: usize,
}

impl Sight {
    /// Sight before any window is met, which no point lies outside of
    const WINDOW: Self = Self {
        clip: Rectangle::new(Point::new(-1.0e9, -1.0e9), Size::new(2.0e9, 2.0e9)),
        shift: Vector::ZERO,
        hidden: 0,
    };

    /// Returns the sight of the content of a scrollable that lies in `bounds`
    /// in the layout and has moved its content by `translation`
    fn within(self, bounds: Rectangle, translation: Vector) -> Self {
        Self {
            clip: self
                .clip
                .intersection(&(bounds - self.shift))
                .unwrap_or_default(),
            shift: self.shift + translation,
            ..self
        }
    }

    /// Returns whether the top left corner of a widget in `bounds`, which its
    /// label lies level with, is in view
    fn sees(self, bounds: Rectangle) -> bool {
        self.hidden == 0 && self.clip.contains(bounds.position() - self.shift)
    }
}

/// Walk that returns the places in sight, in the order of the walk
struct Visit {
    sight: Sight,
    /// Sight of the content of the scrollable that the walk has just met,
    /// which the next descent takes
    entering: Option<Sight>,
    found: Vec<Site>,
}

impl Operation<Vec<Site>> for Visit {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Vec<Site>>)) {
        match self.entering.take() {
            Some(inner) => {
                let outer = mem::replace(&mut self.sight, inner);
                operate(self);
                self.sight = outer;
            }
            None => operate(self),
        }
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        self.entering = Some(self.sight.within(bounds, translation));
    }

    fn custom(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Any) {
        if let Some(Hidden(hidden)) = state.downcast_ref() {
            self.sight.hidden = if *hidden {
                self.sight.hidden + 1
            } else {
                self.sight.hidden.saturating_sub(1)
            };
        } else if state.is::<Window>() {
            self.sight = self.sight.within(bounds, Vector::ZERO);
        } else if let Some(Target(act)) = state.downcast_ref::<Target>()
            && self.sight.sees(bounds)
        {
            self.found.push(Site {
                shown: bounds - self.sight.shift,
                at: bounds,
                act: act.clone(),
            });
        }
    }

    fn finish(&self) -> Outcome<Vec<Site>> {
        Outcome::Some(self.found.clone())
    }
}

/// Walk that has the widget in `bounds` take the keyboard and the others lose it
struct Reach(Rectangle);

impl<T> Operation<T> for Reach {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        if bounds == self.0 {
            state.focus();
        } else {
            state.unfocus();
        }
    }
}

pub fn places() -> Task<Vec<Site>> {
    operate(walk())
}

pub(super) fn walk() -> impl Operation<Vec<Site>> {
    Visit {
        sight: Sight::WINDOW,
        entering: None,
        found: Vec::new(),
    }
}

/// Gives the keyboard to the widget that lies in `at`, which a place keeps
pub fn reach<T: Send + 'static>(at: Rectangle) -> Task<T> {
    operate(Reach(at))
}
