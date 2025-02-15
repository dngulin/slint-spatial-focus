use i_slint_core::api::Window;
use i_slint_core::input::KeyEventResult;
use i_slint_core::item_tree::ItemRc;
use i_slint_core::items::{FocusScope, Item, KeyEvent, TextInput};
use i_slint_core::window::WindowInner;

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
}

pub trait SpatialFocusExtensions {
    fn move_focus(&self, axis: SpatialAxis, dir: SpatialDirection) -> Option<()>;
}

impl SpatialFocusExtensions for Window {
    fn move_focus(&self, axis: SpatialAxis, dir: SpatialDirection) -> Option<()> {
        let window = self.inner();
        let focus_chain = get_focus_chain(window)?;

        let target_group = axis.into();
        let ctx = FocusMoveCtx { axis, dir };

        let mut idx = 1;
        while idx < focus_chain.len().saturating_sub(1) {
            if get_item_of_type_at(&focus_chain, target_group, idx).is_some() {
                let start_item = &focus_chain[idx - 1].item;
                if let Some(focus_item) = check_siblings_of(start_item, &ctx, window, axis) {
                    window.set_focus_item(&focus_item, true);
                    return Some(());
                }
            }
            idx += 1;
        }

        None
    }
}

#[derive(Clone)]
struct FocusChainNode {
    pub item: ItemRc,
    pub node_type: FocusChainNodeType,
}

impl FocusChainNode {
    pub fn new(item: ItemRc, node_type: FocusChainNodeType) -> Self {
        Self { item, node_type }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FocusChainNodeType {
    Focusable,
    Transient,
    VerticalGroup,
    HorizontalGroup,
}

fn get_node_type(item: &ItemRc, w: &WindowInner) -> FocusChainNodeType {
    if item.downcast::<TextInput>().is_some() {
        return FocusChainNodeType::Focusable;
    }

    if let Some(fs) = item.downcast::<FocusScope>() {
        let fs = fs.as_pin_ref();
        let mut e: KeyEvent = Default::default();

        if fs.enabled() {
            return FocusChainNodeType::Focusable;
        }

        e.text = "VerticalFocusGroup".into();
        if fs.key_event(&e, &w.window_adapter(), item) == KeyEventResult::EventAccepted {
            return FocusChainNodeType::VerticalGroup;
        }

        e.text = "HorizontalFocusGroup".into();
        if fs.key_event(&e, &w.window_adapter(), item) == KeyEventResult::EventAccepted {
            return FocusChainNodeType::HorizontalGroup;
        }
    }

    FocusChainNodeType::Transient
}

fn get_focus_chain(window: &WindowInner) -> Option<Vec<FocusChainNode>> {
    let focused_item = window.focus_item.try_borrow().ok()?.upgrade()?;

    let mut item = focused_item;
    let t = get_node_type(&item, window);
    let mut chain = vec![FocusChainNode::new(item.clone(), t)];

    while let Some(parent) = item.parent_item() {
        item = parent;
        let t = get_node_type(&item, window);
        chain.push(FocusChainNode::new(item.clone(), t));
    }

    Some(chain)
}

fn get_item_of_type_at(
    chain: &[FocusChainNode],
    t: FocusChainNodeType,
    idx: usize,
) -> Option<ItemRc> {
    let node = chain.get(idx)?;
    (node.node_type == t).then_some(node.item.clone())
}

fn check_siblings_of(
    start_item: &ItemRc,
    ctx: &FocusMoveCtx,
    window: &WindowInner,
    parent_axis: SpatialAxis,
) -> Option<ItemRc> {
    let mut item = start_item.clone();
    while let Some(next) = get_next_sibling_of(&item, ctx, parent_axis) {
        item = next;
        if let Some(target) = check_item(&item, ctx, window, parent_axis) {
            return Some(target);
        }
    }

    None
}

fn check_item(
    item: &ItemRc,
    ctx: &FocusMoveCtx,
    window: &WindowInner,
    parent_axis: SpatialAxis,
) -> Option<ItemRc> {
    if !item.is_visible() {
        return None;
    }

    let item_type = get_node_type(item, window);
    if item_type == FocusChainNodeType::Focusable {
        return Some(item.clone());
    }

    let item_axis = item_type.try_into().unwrap_or(parent_axis);
    if let Some(child) = get_first_child(&item, ctx, item_axis) {
        if let Some(target) = check_item(&child, ctx, window, item_axis) {
            return Some(target);
        }

        if let Some(target) = check_siblings_of(&child, ctx, window, item_axis) {
            return Some(target);
        }
    }

    None
}

fn get_first_child(item: &ItemRc, ctx: &FocusMoveCtx, scope_axis: SpatialAxis) -> Option<ItemRc> {
    let reverse = ctx.dir == SpatialDirection::Backward && ctx.axis == scope_axis;
    if reverse {
        item.last_child()
    } else {
        item.first_child()
    }
}

fn get_next_sibling_of(
    item: &ItemRc,
    ctx: &FocusMoveCtx,
    scope_axis: SpatialAxis,
) -> Option<ItemRc> {
    let reverse = ctx.dir == SpatialDirection::Backward && ctx.axis == scope_axis;
    if reverse {
        item.previous_sibling()
    } else {
        item.next_sibling()
    }
}

impl From<SpatialAxis> for FocusChainNodeType {
    fn from(value: SpatialAxis) -> Self {
        match value {
            SpatialAxis::Horizontal => FocusChainNodeType::HorizontalGroup,
            SpatialAxis::Vertical => FocusChainNodeType::VerticalGroup,
        }
    }
}

impl TryFrom<FocusChainNodeType> for SpatialAxis {
    type Error = ();

    fn try_from(value: FocusChainNodeType) -> Result<Self, Self::Error> {
        match value {
            FocusChainNodeType::VerticalGroup => Ok(SpatialAxis::Vertical),
            FocusChainNodeType::HorizontalGroup => Ok(SpatialAxis::Horizontal),
            _ => Err(()),
        }
    }
}

trait Inner {
    fn inner(&self) -> &WindowInner;
}

impl Inner for Window {
    fn inner(&self) -> &WindowInner {
        unsafe { std::mem::transmute(self) }
    }
}
