# Arrow Key Navigation For Slint UI

This library adds spatial navigation to Slint applications.

How to use it:

- Use the `SpatialFocusHandler` as a root item in your window
- Export the `SpatialFocus` global object
- Call the `slint_spatial_focus::init!` macro to initialize the library

For details see the [example](examples/traverse.rs).

## Limitations

- The library depends on the `i-slint-core`
  and should be used with the specific Slint version (`1.9.2` for now).
- Focus traversal algorithm searches enabled `FocusScope` and `TextInput` objects.
  So, it misses all native controls.
- Only float point `Coord` representation is supported