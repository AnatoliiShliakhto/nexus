/// A utility macro for clean and concise variable capture in closures,
/// specifically optimized for Rust UI frameworks like Dioxus and async runtimes.
///
/// It eliminates the "forest of clones" boilerplate often required when
/// moving multiple `Rc`-wrapped or `Clone`-able types (like `Signal`, `DesktopContext`,
/// or `Arc<T>`) into closures and async blocks.
///
/// ### Features
/// - **Sync & Async Support**: Handles both standard closures and `async move` blocks.
/// - **Variable Aliasing**: Rename variables on the fly using the `as` keyword.
/// - **Double-Clone Magic**: Automatically handles the nested cloning required for
///   `move || async move { ... }` patterns, ensuring the future remains `'static`.
/// - **Trailing Commas**: Flexible syntax that won't break during refactoring.
///
/// ### Examples
///
/// #### 1. Basic Synchronization
/// Capture variables or context handles for a simple event handler.
/// ```rust
/// let window = use_window();
/// let title = "title";
///
/// let on_click = enclose!(window, title => move |_| {
///     window.set_title(&format!("App - {title}"));
/// });
/// ```
///
/// #### 2. Renaming (Aliases)
/// Keep your closure logic concise by shortening long variable names.
/// ```rust
/// let state_provider = use_storage::<Config>("config", Config::default);
///
/// let on_init = enclose!(state_provider as sp => move || {
///     let current = sp.read();
///     info!("Loaded: {current:?}");
/// });
/// ```
///
/// #### 3. The "Double-Clone" Pattern (Async Handlers)
/// In Dioxus, async event handlers often require cloning variables twice:
/// once for the closure and once for the generated `Future`. This macro
/// handles both steps automatically.
/// ```rust
/// let auth = use_auth_service();
/// let nav = use_navigator();
///
/// let on_submit = enclose!(auth, nav => move |_| async move {
///     if let Ok(_) = auth.login("user", "pass").await {
///         nav.push("/dashboard");
///     }
/// });
/// ```
///
/// #### 4. Pure Async Blocks
/// Useful for `use_future` or `spawn` where no outer closure is present.
/// ```rust
/// let data_service = use_data_service();
///
/// use_future(enclose!(data_service => async move {
///     data_service.sync_all().await;
/// }));
/// ```
#[macro_export]
macro_rules! enclose {
    // 1. SYNC CLOSURES (No arguments)
    ( $( $v:ident $(as $a:ident)? ),+ $(,)? => move || $b:expr ) => {{
        $( $crate::enclose!(@clone $v $(as $a)?); )+
        move || $b
    }};

    // 2. SYNC CLOSURES (With arguments)
    ( $( $v:ident $(as $a:ident)? ),+ $(,)? => move |$($arg:pat_param),*| $b:expr ) => {{
        $( $crate::enclose!(@clone $v $(as $a)?); )+
        move |$($arg),*| $b
    }};

    // 3. ASYNC CLOSURES (Double Clone Magic)
    ( $( $v:ident $(as $a:ident)? ),+ $(,)? => move || async move $b:block ) => {{
        $( $crate::enclose!(@clone $v $(as $a)?); )+
        move || {
            $( $crate::enclose!(@clone $v $(as $a)?); )+
            async move $b
        }
    }};

    ( $( $v:ident $(as $a:ident)? ),+ $(,)? => move |$($arg:pat_param),*| async move $b:block ) => {{
        $( $crate::enclose!(@clone $v $(as $a)?); )+
        move |$($arg),*| {
            $( $crate::enclose!(@clone $v $(as $a)?); )+
            async move $b
        }
    }};

    // 4. PURE ASYNC BLOCKS (use_future / spawn)
    ( $( $v:ident $(as $a:ident)? ),+ $(,)? => async move $b:block ) => {{
        $( $crate::enclose!(@clone $v $(as $a)?); )+
        async move $b
    }};

    // INTERNAL RULES
    (@clone $v:ident) => {
        let $v = $v.clone();
    };
    (@clone $v:ident as $a:ident) => {
        let $a = $v.clone();
    };
}
