# Nexus I18n

A lightweight localization runtime for the Nexus workspace built on top of [Fluent](https://projectfluent.org/).

This crate provides:

- a fast translation API for runtime use
- locale parsing and normalization
- fallback handling
- generated typed translation keys
- bundle loading from generated FTL files

## Features

- **Fast runtime translation** using cached Fluent bundles
- **Locale normalization** via `unic-langid`
- **Fallback support** from regional locale to base language and default language
- **Generated translation constants** for type-safe access
- **Minimal allocations in hot paths**
- **WASM-friendly design**

## Crate layout

The crate is split into small modules for clarity and maintainability:

- `runtime` — public translation API
- `locale` — locale parsing and normalization
- `bundle` — bundle loading and lookup
- `generated` — auto-generated translation data and constants

## How it works

At build time, the workspace code generator collects all `i18n/*.ftl` files from workspace crates and generates:

- merged FTL bundles
- a `mod.rs` file with:
  - `BUNDLES`
  - typed translation constants under `tr::...`

At runtime, this crate loads the generated bundles once and keeps them in memory for fast lookup.

## Translation flow

When translating a key:

1. The requested locale is used as the first lookup candidate.
2. If no exact match is found, the primary language is tried.
3. If still unresolved, the default language is used.
4. If the key is missing everywhere, the key itself is returned.

## API

### `I18n`

The main translation facade.
```rust,ignore 
use nx_i18n::I18n;

let i18n = I18n::new(); 
let text = i18n.t("en", "database-auth-error");
```

### `Locale`

A normalized locale wrapper.

```rust,ignore
nx_i18n::Locale;

let locale = Locale::parse("uk-UA")?;
```

For hot paths, prefer parsing the locale once and reusing it:

```rust,ignore
let locale = Locale::parse("uk-UA")?; 
let text = i18n.translate(&locale, "database-auth-error", None);
```

### `LocaleError`

Returned when locale parsing fails.

## Translation keys

Generated translation keys are exposed through `tr`:

```rust,ignore
use nx_i18n::tr;

let key = tr::database::DATABASE_AUTH_ERROR;
```

This gives you type-safe access to translation keys and avoids stringly-typed references in application code.

## Examples

### Basic translation

```rust,ignore
use nx_i18n::I18n;

let i18n = I18n::new(); 
let text = i18n.t("en", "database-auth-error"); 
assert_eq!(text, "Access denied. Please check your credentials or permissions.");
```

### Using a parsed locale

```rust,ignore
use nx_i18n::I18n;

let i18n = I18n::new(); 
let text = i18n.t("en", "database-auth-error"); 
assert_eq!(text, "Access denied. Please check your credentials or permissions.");
```

### Checking available languages

```rust,ignore
use nx_i18n::I18n;

let langs = I18n::available_languages(); 
assert!(langs.contains(&"en"));
```

## Performance notes

This crate is designed to be efficient in repeated call sites:

- locale parsing is optional and can be done once
- loaded bundles are cached per thread
- fallback lookup does not allocate temporary collections
- translation keys are compile-time constants

If you are translating repeatedly in a render loop, prefer:


```rust,ignore
let locale = Locale::parse(current_lang)?; 
let text = i18n.translate(&locale, key, None);
```

instead of parsing the locale on every call.

## Development

The generated files are produced by the workspace `codegen` pipeline.  
If translations change, regenerate the bundles and constants from the workspace root.

## License

Copyright © 2026 Nexus Project. All rights reserved.