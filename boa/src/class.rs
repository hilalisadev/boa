use crate::{
    builtins::{
        function::{BuiltInFunction, Function, FunctionFlags, NativeFunction},
        object::{GcObject, Object, ObjectData, PROTOTYPE},
        property::{Attribute, Property, PropertyKey},
        Value,
    },
    Interpreter, Result,
};
use gc::Trace;
use std::{any::Any, fmt::Debug};

/// This trait allows Rust types to be passed around as objects.
///
/// This is automatically implemented, when a type implements `Debug`, `Any` and `Trace`.
pub trait NativeObject: Debug + Any + Trace {
    /// Convert the Rust type which implements `NativeObject` to a `&dyn Any`.
    fn as_any(&self) -> &dyn Any;

    /// Convert the Rust type which implements `NativeObject` to a `&mut dyn Any`.
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<T: Any + Debug + Trace> NativeObject for T {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

/// Native class.
pub trait Class: NativeObject + Sized {
    /// The binding name of the object.
    const NAME: &'static str;
    /// The amount of arguments the class `constructor` takes, default is `0`.
    const LENGTH: usize = 0;
    /// The attibutes the class will be binded with, default is `writable`, `enumerable`, `configurable`.
    const ATTRIBUTE: Attribute = Attribute::all();

    /// The constructor of the class.
    fn constructor(this: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Self>;

    /// Initializes the internals and the methods of the class.
    fn init(class: &mut ClassBuilder<'_>) -> Result<()>;
}

/// This is a wrapper around `Class::constructor` that sets the internal data of a class.
///
/// This is automatically implemented, when a type implements `Class`.
pub trait ClassConstructor: Class {
    fn raw_constructor(this: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Value>
    where
        Self: Sized;
}

impl<T: Class> ClassConstructor for T {
    fn raw_constructor(this: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Value>
    where
        Self: Sized,
    {
        let object_instance = Self::constructor(this, args, ctx)?;
        this.set_data(ObjectData::NativeObject(Box::new(object_instance)));
        Ok(this.clone())
    }
}

/// Class builder which allows adding methods and static methods to the class.
#[derive(Debug)]
pub struct ClassBuilder<'context> {
    context: &'context mut Interpreter,
    object: GcObject,
    prototype: GcObject,
}

impl<'context> ClassBuilder<'context> {
    pub(crate) fn new<T>(context: &'context mut Interpreter) -> Self
    where
        T: ClassConstructor,
    {
        let global = context.global();

        let prototype = {
            let object_prototype = global.get_field("Object").get_field(PROTOTYPE);

            let object = Object::create(object_prototype);
            GcObject::new(object)
        };
        // Create the native function
        let function = Function::BuiltIn(
            BuiltInFunction(T::raw_constructor),
            FunctionFlags::CONSTRUCTABLE,
        );

        // Get reference to Function.prototype
        // Create the function object and point its instance prototype to Function.prototype
        let mut constructor =
            Object::function(function, global.get_field("Function").get_field(PROTOTYPE));

        let length = Property::data_descriptor(
            T::LENGTH.into(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        constructor.insert_property("length", length);

        let name = Property::data_descriptor(
            T::NAME.into(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        constructor.insert_property("name", name);

        let constructor = GcObject::new(constructor);

        prototype
            .borrow_mut()
            .insert_field("constructor", constructor.clone().into());

        constructor
            .borrow_mut()
            .insert_field(PROTOTYPE, prototype.clone().into());

        Self {
            context,
            object: constructor,
            prototype,
        }
    }

    pub(crate) fn build(self) -> GcObject {
        self.object
    }

    /// Add a method to the class.
    ///
    /// It is added to `prototype`.
    pub fn method<N>(&mut self, name: N, length: usize, function: NativeFunction)
    where
        N: Into<String>,
    {
        let name = name.into();
        let mut function = Object::function(
            Function::BuiltIn(function.into(), FunctionFlags::CALLABLE),
            self.context
                .global()
                .get_field("Function")
                .get_field("prototype"),
        );

        function.insert_field("length", Value::from(length));
        function.insert_field("name", Value::from(name.as_str()));

        self.prototype
            .borrow_mut()
            .insert_field(name, Value::from(function));
    }

    /// Add a static method to the class.
    ///
    /// It is added to class object itself.
    pub fn static_method<N>(&mut self, name: N, length: usize, function: NativeFunction)
    where
        N: Into<String>,
    {
        let name = name.into();
        let mut function = Object::function(
            Function::BuiltIn(function.into(), FunctionFlags::CALLABLE),
            self.context
                .global()
                .get_field("Function")
                .get_field("prototype"),
        );

        function.insert_field("length", Value::from(length));
        function.insert_field("name", Value::from(name.as_str()));

        self.object
            .borrow_mut()
            .insert_field(name, Value::from(function));
    }

    /// Add a property to the class, with the specified attribute.
    ///
    /// It is added to `prototype`.
    #[inline]
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute)
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        // We bitwise or (`|`) with `Attribute::default()` (`READONLY | NON_ENUMERABLE | PERMANENT`)
        // so we dont get an empty attribute.
        let property = Property::data_descriptor(value.into(), attribute | Attribute::default());
        self.prototype
            .borrow_mut()
            .insert_property(key.into(), property);
    }

    /// Add a static property to the class, with the specified attribute.
    ///
    /// It is added to class object itself.
    #[inline]
    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute)
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        // We bitwise or (`|`) with `Attribute::default()` (`READONLY | NON_ENUMERABLE | PERMANENT`)
        // so we dont get an empty attribute.
        let property = Property::data_descriptor(value.into(), attribute | Attribute::default());
        self.object
            .borrow_mut()
            .insert_property(key.into(), property);
    }

    pub fn context(&mut self) -> &'_ mut Interpreter {
        self.context
    }
}
