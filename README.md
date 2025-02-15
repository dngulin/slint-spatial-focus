# Slint Spatial Navigation Prototype

This is the prototype introducing spatial navigation to Slint UI.

It provides a set of components to enable spatial navigation:

- `SpatialFocusHandler` - a component to catch spatial navigation events.
  Should be the root element in the window.
- `VerticalFocusGroup`, `HorizontalFocusGroup` - wrappers around corresponding layout components.
  They are required to give hints (in a very hacky way) to the spatial focus traverse algorithms.

The library doesn't provide any wrappers for grid layout and doesn't support it for now.

It is also requires to be initialized on the rust side.
For details see the [focus traverse example](examples/traverse.rs).