use std::cell::Cell;
use std::cell::RefCell;

use dash_vm::delegate;
use dash_vm::gc::trace::Trace;
use dash_vm::local::LocalScope;
use dash_vm::throw;
use dash_vm::value::arraybuffer::ArrayBuffer;
use dash_vm::value::function::native::CallContext;
use dash_vm::value::function::Function;
use dash_vm::value::function::FunctionKind;
use dash_vm::value::object::NamedObject;
use dash_vm::value::object::Object;
use dash_vm::value::object::PropertyValue;
use dash_vm::value::Value;
use minifb::WindowOptions;

type NativeWindow = RefCell<minifb::Window>;

#[derive(Debug)]
pub struct Window {
    inner: NativeWindow,
    object: NamedObject,
}

unsafe impl Trace for Window {
    fn trace(&self) {
        // Using let destructuring so we get an error when a field is added.
        #[allow(unused_variables)]
        let Window { inner, object } = self;
        object.trace();
    }
}

fn initialize_window_object(obj: &NamedObject, scope: &mut LocalScope) -> Result<(), Value> {
    let limit_update_rate = {
        let f = Function::new(
            scope,
            Some("limitUpdateRateMs".into()),
            FunctionKind::Native(limit_update_rate_ms),
        );
        scope.register(f)
    };
    obj.set_property(
        scope,
        "limitUpdateRateMs".into(),
        PropertyValue::static_default(Value::Object(limit_update_rate)),
    )?;

    let is_open = {
        let f = Function::new(scope, Some("isOpen".into()), FunctionKind::Native(is_open));
        scope.register(f)
    };
    obj.set_property(
        scope,
        "isOpen".into(),
        PropertyValue::static_default(Value::Object(is_open)),
    )?;

    let update_with_buffer = {
        let f = Function::new(
            scope,
            Some("updateWithBuffer".into()),
            FunctionKind::Native(update_with_buffer),
        );
        scope.register(f)
    };
    obj.set_property(
        scope,
        "updateWithBuffer".into(),
        PropertyValue::static_default(Value::Object(update_with_buffer)),
    )?;
    Ok(())
}

impl Window {
    pub fn new(
        scope: &mut LocalScope,
        name: &str,
        width: usize,
        height: usize,
        opts: minifb::WindowOptions,
    ) -> Result<Self, Value> {
        let object = NamedObject::new(scope);
        initialize_window_object(&object, scope)?;

        let inner = RefCell::new(minifb::Window::new(name, width, height, opts).unwrap());

        Ok(Self { inner, object })
    }
}

impl Object for Window {
    delegate!(
        object,
        get_property,
        get_property_descriptor,
        set_property,
        delete_property,
        set_prototype,
        get_prototype,
        as_any,
        apply,
        own_keys
    );
}

pub fn constructor(cx: CallContext) -> Result<Value, Value> {
    let window = Window::new(cx.scope, "", 500, 500, WindowOptions::default())?;

    Ok(Value::Object(cx.scope.register(window)))
}

fn downcast_this<'a>(scope: &mut LocalScope, this: &'a Value) -> Result<&'a Window, Value> {
    match this {
        Value::Object(obj) => match obj.as_any().downcast_ref::<Window>() {
            Some(window) => Ok(window),
            None => throw!(scope, "Not a window"),
        },
        _ => throw!(scope, "Not an object"),
    }
}

fn limit_update_rate_ms(cx: CallContext) -> Result<Value, Value> {
    let mut this = downcast_this(cx.scope, &cx.this)?.inner.borrow_mut();
    this.limit_update_rate(match cx.args.first() {
        Some(Value::Number(num)) => Some(std::time::Duration::from_millis(num.0 as u64)),
        None => None,
        _ => throw!(cx.scope, "Invalid update rate"),
    });
    Ok(Value::undefined())
}

fn is_open(cx: CallContext) -> Result<Value, Value> {
    let this = downcast_this(cx.scope, &cx.this)?.inner.borrow();
    Ok(Value::Boolean(this.is_open()))
}

fn update_with_buffer(cx: CallContext) -> Result<Value, Value> {
    let this = downcast_this(cx.scope, &cx.this)?;
    let mut args = cx.args.iter();
    let buf = match args.next() {
        Some(Value::Object(buf)) => match buf.as_any().downcast_ref::<ArrayBuffer>() {
            Some(buf) => buf,
            None => throw!(cx.scope, "Not an ArrayBuffer"),
        },
        _ => throw!(cx.scope, "Missing ArrayBuffer argument"),
    };
    let (width, height) = match (args.next(), args.next()) {
        (Some(Value::Number(width)), Some(Value::Number(height))) => (width.0 as usize, height.0 as usize),
        _ => throw!(cx.scope, "Width and height must be present and numbers"),
    };

    let buf = buf.storage();
    update_window_buffer(buf, width, height, &mut *this.inner.borrow_mut());

    Ok(Value::undefined())
}

fn update_window_buffer(buf: &[Cell<u8>], width: usize, height: usize, window: &mut minifb::Window) {
    // This is pretty sketchy but minifb only accepts a &[u32] but we only have a &[Cell<u8>] and we can't afford to copy the buffer every time
    // So, temporarily cast the &[Cell<u8>] to a &[u8] to a &[u32] to pass it to `window.update_with_buffer`
    // NOTE: No writes to the storage buffer are allowed here!
    let buf = unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), buf.len()) };
    let buf = bytemuck::cast_slice::<u8, u32>(buf);

    window.update_with_buffer(buf, width, height).unwrap();
}
