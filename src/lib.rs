use i_slint_core::api::Window;
use i_slint_core::item_tree::ItemRc;
use i_slint_core::items::{FocusScope, TextInput};
use i_slint_core::lengths::LogicalRect;
use i_slint_core::window::WindowInner;
use i_slint_core::Coord;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpatialAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpatialDirection {
    Forward,
    Backward,
}

struct FocusMoveCtx {
    pub axis: SpatialAxis,
    pub dir: SpatialDirection,
    pub focused_rect: LogicalRect,
}

pub trait SpatialFocusExtensions {
    fn move_focus(&self, axis: SpatialAxis, dir: SpatialDirection) -> Option<()>;
}

impl SpatialFocusExtensions for Window {
    fn move_focus(&self, axis: SpatialAxis, dir: SpatialDirection) -> Option<()> {
        let window = self.inner();
        let focused_item = window.focus_item.try_borrow().ok()?.upgrade()?;

        let focus_chain = get_hierarchy_chain(&focused_item);
        let focused_rect = get_rect(&focused_item);

        let ctx = FocusMoveCtx {
            axis,
            dir,
            focused_rect,
        };
        let mut idx = 1;

        while idx < focus_chain.len() {
            let item = &focus_chain[idx - 1];
            if let Some(focus_target) = find_focusable_sibling_of(item, &ctx) {
                window.set_focus_item(&focus_target, true);
                return Some(());
            }
            idx += 1;
        }

        None
    }
}

fn get_rect(item: &ItemRc) -> LogicalRect {
    let local_rect = item.geometry();
    let global_pos = item.map_to_window(local_rect.origin);

    LogicalRect::new(global_pos, local_rect.size)
}

fn is_focusable(item: &ItemRc) -> bool {
    if item.downcast::<TextInput>().is_some() {
        return true;
    }

    if let Some(fs) = item.downcast::<FocusScope>() {
        if fs.as_pin_ref().enabled() {
            return true;
        }
    }

    false
}

fn get_hierarchy_chain(start_item: &ItemRc) -> Vec<ItemRc> {
    let mut item = start_item.clone();
    let mut chain = vec![item.clone()];

    while let Some(parent) = item.parent_item() {
        item = parent;
        chain.push(item.clone());
    }

    chain
}

fn find_focusable_sibling_of(item: &ItemRc, ctx: &FocusMoveCtx) -> Option<ItemRc> {
    let parent = item.parent_item()?;

    let mut siblings = Vec::new();
    let mut visitor = |i: &ItemRc| {
        if i == item || !i.is_visible() {
            return TraversalOp::Skip;
        }

        if is_focusable(i) {
            siblings.push(i.clone());
            return TraversalOp::Skip;
        }

        TraversalOp::Continue
    };
    visit_children(&parent, &mut visitor);

    let candidates: Vec<(ItemRc, LogicalRect)> = siblings
        .iter()
        .map(|i| (i.clone(), get_rect(i)))
        .filter(|(_, r)| is_focus_target(r, ctx))
        .collect();

    let first = candidates.first()?;

    let mut curr_i = first.0.clone();
    let mut curr_d = distance(&first.1, ctx);
    let mut curr_od = ort_distance(&first.1, ctx);

    for (i, r) in &candidates[1..] {
        let d = distance(r, ctx);
        let od = ort_distance(r, ctx);

        if (d - curr_d).abs() <= TOLERANCE {
            if od < curr_od {
                curr_od = od;
                curr_i = i.clone();
            }
        } else if d < curr_d {
            curr_d = d;
            curr_od = od;
            curr_i = i.clone();
        }
    }

    Some(curr_i)
}

enum TraversalOp {
    Continue,
    Skip,
}

fn visit_children<F: FnMut(&ItemRc) -> TraversalOp>(item: &ItemRc, process: &mut F) {
    if let Some(child) = item.first_child() {
        let op = process(&child);
        match op {
            TraversalOp::Continue => {
                visit_children(&child, process);
            }
            TraversalOp::Skip => {}
        }

        let mut sibling = child.clone();
        while let Some(next_sibling) = sibling.next_sibling() {
            sibling = next_sibling;

            let op = process(&sibling);
            match op {
                TraversalOp::Continue => {
                    visit_children(&sibling, process);
                }
                TraversalOp::Skip => {}
            }
        }
    }
}

const TOLERANCE: Coord = 0.001;

fn is_focus_target(r: &LogicalRect, ctx: &FocusMoveCtx) -> bool {
    let f = ctx.focused_rect;
    match (ctx.axis, ctx.dir) {
        (SpatialAxis::Horizontal, SpatialDirection::Backward) => {
            r.origin.x + r.width() - TOLERANCE <= f.origin.x
        }
        (SpatialAxis::Horizontal, SpatialDirection::Forward) => {
            r.origin.x + TOLERANCE >= f.origin.x + f.width()
        }
        (SpatialAxis::Vertical, SpatialDirection::Backward) => {
            r.origin.y + r.height() - TOLERANCE <= f.origin.y
        }
        (SpatialAxis::Vertical, SpatialDirection::Forward) => {
            r.origin.y + TOLERANCE >= f.origin.y + f.height()
        }
    }
}

fn distance(r: &LogicalRect, ctx: &FocusMoveCtx) -> Coord {
    let f = ctx.focused_rect;
    let d = match (ctx.axis, ctx.dir) {
        (SpatialAxis::Horizontal, SpatialDirection::Backward) => {
            (r.origin.x + r.width()) - f.origin.x
        }
        (SpatialAxis::Horizontal, SpatialDirection::Forward) => {
            r.origin.x - (f.origin.x + f.width())
        }
        (SpatialAxis::Vertical, SpatialDirection::Backward) => {
            (r.origin.y + r.height()) - f.origin.y
        }
        (SpatialAxis::Vertical, SpatialDirection::Forward) => {
            r.origin.y - (f.origin.y + f.height())
        }
    };

    d.abs()
}

fn ort_distance(r: &LogicalRect, ctx: &FocusMoveCtx) -> Coord {
    let f = ctx.focused_rect;
    let (a, b) = match ctx.axis {
        SpatialAxis::Horizontal => {
            let a = (f.origin.y, f.origin.y + f.height());
            let b = (r.origin.y, r.origin.y + r.height());
            (a, b)
        }
        SpatialAxis::Vertical => {
            let a = (f.origin.x, f.origin.x + f.width());
            let b = (r.origin.x, r.origin.x + r.width());
            (a, b)
        }
    };

    if are_intersected(&a, &b) {
        return 0.0;
    }

    let ca = a.0 + (a.1 - a.0) / 2.0;
    let cb = b.0 + (b.1 - b.0) / 2.0;

    (ca - cb).abs()
}

fn are_intersected(a: &(Coord, Coord), b: &(Coord, Coord)) -> bool {
    let p1 = a.0 - b.1; // min(a.0, a.1) - max(b.0, b.1)
    let p2 = a.1 - b.0; // max(a.0, a.1) - min(b.0, b.1)
    p1 < 0.0 && p2 > 0.0 // Origin is inside the Minkowski difference, so segments are intersected
}

trait Inner {
    fn inner(&self) -> &WindowInner;
}

impl Inner for Window {
    fn inner(&self) -> &WindowInner {
        unsafe { std::mem::transmute(self) }
    }
}
