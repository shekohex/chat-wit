use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::preview2::{
    command, Table, WasiCtx, WasiCtxBuilder, WasiView,
};

bindgen!({
    path: "../crates/sha256/wit",
    async: true,
});

struct MyState {
    name: String,
    wasi: WasiCtx,
    table: Table,
}

impl WasiView for MyState {
    fn table(&self) -> &Table { &self.table }

    fn table_mut(&mut self) -> &mut Table { &mut self.table }

    fn ctx(&self) -> &WasiCtx { &self.wasi }

    fn ctx_mut(&mut self) -> &mut WasiCtx { &mut self.wasi }
}

#[tokio::main]
async fn main() -> wasmtime::Result<()> {
    // Configure an `Engine` and compile the `Component` that is being run for
    // the application.
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    let engine = Engine::new(&config)?;
    let component = Component::from_file(
        &engine,
        "./target/wasm32-wasi/debug/msg_sha256.wasm",
    )?;

    // Instantiation of bindings always happens through a `Linker`.
    // Configuration of the linker is done through a generated `add_to_linker`
    // method on the bindings structure.
    //
    // Note that the closure provided here is a projection from `T` in
    // `Store<T>` to `&mut U` where `U` implements the `HelloWorldImports`
    // trait. In this case the `T`, `MyState`, is stored directly in the
    // structure so no projection is necessary here.
    let mut linker = Linker::new(&engine);
    command::add_to_linker(&mut linker)?;

    // As with the core wasm API of Wasmtime instantiation occurs within a
    // `Store`. The bindings structure contains an `instantiate` method which
    // takes the store, component, and linker. This returns the `bindings`
    // structure which is an instance of `HelloWorld` and supports typed access
    // to the exports of the component.
    let wasi = WasiCtxBuilder::new().inherit_stdio().build();
    let table = Table::new();
    let mut store = Store::new(
        &engine,
        MyState {
            name: "me".to_string(),
            wasi,
            table,
        },
    );
    let (bindings, _) =
        Example::instantiate_async(&mut store, &component, &linker).await?;

    // Here our `run` function doesn't take any parameters for the component,
    // but in the Wasmtime embedding API the first argument is always a `Store`.
    let res = bindings.call_hello_world(&mut store).await?;
    eprintln!("{}", res);
    Ok(())
}
