# Error Context

Sometimes implementing a trait one has to return a specific error, while the code inside can return multiple various errors.

The problem becomes even more annoying when all the errors are defined in crates, that are not in control of the user and thus no `From` and nice looking `?`.

## Application level error

One of the sulutions is to create an app-level Error, that implements `From` for all the errors the app needs to deal with, and `Into` for the nasty trait function error (see above in the description).

Still one has to constantly use `map_err` in the code.

## ErrorContext is to eliminate `map_err`

```rust
trait ApiToImplement {
    fn important_fn() -> Result<String, ApiError>;
}

impl ApiToImplement for MyStruct {
    fn important_fn() -> Result<String, ApiError> {
        MyAppError::context(|| {
            method1()?;
            method2()?;
            Ok(method3())
        })
    }
}

```

Considering that `method1`, `method2`, `method3` all return different errors, the above looks much cleaner, then explicitely converting errors for each call:

```rust
method1().map_err(|e| /*conversion*/)?;
method2().map_err(|e| /*conversion*/)?;
method3().map_err(|e| /*conversion*/)
```

## Licence

Apache 2.0
