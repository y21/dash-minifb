use dash_vm::local::LocalScope;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::Function;
use dash_vm::value::function::FunctionKind;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::Value;

mod window;

fn import(cx: &mut CallContext) -> Result<Value, Value> {
    create_export_object(cx.scope)
}

dash_dlloader::dashdl!(import);

fn create_export_object(scope: &mut LocalScope) -> Result<Value, Value> {
    let exports = NamedObject::new(scope);
    let window_constructor = Function::new(scope, Some("Window".into()), FunctionKind::Native(window::constructor));
    let window_constructor = scope.register(window_constructor);
    exports.set_property(
        scope,
        "Window".into(),
        PropertyValue::static_default(Value::Object(window_constructor)),
    )?;

    Ok(Value::Object(scope.register(exports)))
}
