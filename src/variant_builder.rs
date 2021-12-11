// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

use glib::{ffi, translate::*, ToVariant, Variant, VariantTy};
use std::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

pub struct VariantBuilder(ffi::GVariantBuilder);

impl VariantBuilder {
    pub fn new(ty: &VariantTy) -> Self {
        let mut builder: MaybeUninit<ffi::GVariantBuilder> = MaybeUninit::uninit();
        Self(unsafe {
            ffi::g_variant_builder_init(builder.as_mut_ptr(), ty.to_glib_none().0);
            builder.assume_init()
        })
    }
    pub fn end(self) -> Variant {
        let v = unsafe { self.end_unsafe() };
        std::mem::forget(self);
        v
    }
}

impl Drop for VariantBuilder {
    fn drop(&mut self) {
        unsafe { ffi::g_variant_builder_clear(self.as_ptr()) };
    }
}

pub trait VariantBuilderExt {
    fn as_ptr(&self) -> *mut ffi::GVariantBuilder;
    unsafe fn add<T: ToVariant>(&self, value: &T) {
        self.add(&value.to_variant());
    }
    unsafe fn add_value(&self, value: &Variant) {
        ffi::g_variant_builder_add_value(self.as_ptr(), value.to_glib_none().0);
    }
    fn open(&self, ty: &VariantTy) -> VariantBuilderContainer<'_> {
        unsafe { ffi::g_variant_builder_open(self.as_ptr(), ty.to_glib_none().0) };
        VariantBuilderContainer {
            inner: NonNull::new(self.as_ptr()).unwrap(),
            phantom: PhantomData,
        }
    }
    unsafe fn end_unsafe(&self) -> Variant {
        from_glib_none(ffi::g_variant_builder_end(self.as_ptr()))
    }
}

impl VariantBuilderExt for VariantBuilder {
    fn as_ptr(&self) -> *mut ffi::GVariantBuilder {
        &self.0 as *const _ as *mut _
    }
}

#[repr(transparent)]
pub struct VariantBuilderContainer<'t> {
    inner: NonNull<ffi::GVariantBuilder>,
    phantom: PhantomData<&'t ()>,
}

impl<'t> Drop for VariantBuilderContainer<'t> {
    fn drop(&mut self) {
        unsafe { ffi::g_variant_builder_close(self.inner.as_ptr()) };
    }
}

impl<'t> VariantBuilderExt for VariantBuilderContainer<'t> {
    fn as_ptr(&self) -> *mut ffi::GVariantBuilder {
        self.inner.as_ptr()
    }
}

glib::wrapper! {
    pub struct SharedVariantBuilder(Shared<ffi::GVariantBuilder>);

    match fn {
        ref => |ptr| ffi::g_variant_builder_ref(ptr),
        unref => |ptr| ffi::g_variant_builder_unref(ptr),
        type_ => || ffi::g_variant_builder_get_type(),
    }
}

impl SharedVariantBuilder {
    pub fn new(ty: &VariantTy) -> Self {
        unsafe { from_glib_full(ffi::g_variant_builder_new(ty.to_glib_none().0)) }
    }
    pub fn end(&self) -> Variant {
        unsafe {
            let v = self.end_unsafe();
            ffi::g_variant_builder_init(self.as_ptr(), v.type_().to_glib_none().0);
            v
        }
    }
}

impl VariantBuilderExt for SharedVariantBuilder {
    fn as_ptr(&self) -> *mut ffi::GVariantBuilder {
        self.0.to_glib_none().0
    }
}
