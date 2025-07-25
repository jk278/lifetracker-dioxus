# Programmatic Navigation

Sometimes we want our application to navigate to another page without having the
user click on a link. This is called programmatic navigation.

## Using a Navigator

We can get a navigator with the [`navigator`] function which returns a [`Navigator`].

We can use the [`Navigator`] to trigger four different kinds of navigation:

- `push` will navigate to the target. It works like a regular anchor tag.
- `replace` works like `push`, except that it replaces the current history entry
  instead of adding a new one. This means the prior page cannot be restored with the browser's back button.
- `Go back` works like the browser's back button.
- `Go forward` works like the browser's forward button.

```rust
{{#include ../docs-router/src/doc_examples/untested_06/navigator.rs:nav}}
```

You might have noticed that, like [`Link`], the [`Navigator`]s `push` and
`replace` functions take a [`NavigationTarget`]. This means we can use either
`Internal`, or `External` targets.

## External Navigation Targets

Unlike a [`Link`], the [`Navigator`] cannot rely on the browser (or webview) to
handle navigation to external targets via a generated anchor element.

This means, that under certain conditions, navigation to external targets can
fail.


[`Link`]: https://docs.rs/dioxus-router/latest/dioxus_router/components/fn.Link.html
[`NavigationTarget`]: https://docs.rs/dioxus-router/latest/dioxus_router/navigation/enum.NavigationTarget.html
[`Navigator`]: https://docs.rs/dioxus-router/latest/dioxus_router/prelude/struct.Navigator.html
