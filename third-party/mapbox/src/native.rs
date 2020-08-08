/*
 * Terra Mach
 * Copyright [2020] Terra Mach Authors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>
 */

pub trait NativeDrop {
    fn drop(&mut self);
}

#[repr(transparent)]
pub struct Handle<T: NativeDrop>(*mut T);

impl<T: NativeDrop> Handle<T> {
    pub fn from_ptr(instance: *mut T) -> Self {
        debug_assert!(!instance.is_null());
        Handle(instance)
    }

    pub unsafe fn new_native(f: unsafe extern "C" fn(*mut T)) -> Self {
        Self::new(|this| f(this))
    }

    pub fn new<F>(f: F) -> Self where F: FnOnce(*mut T) {
        let mut instance = std::mem::MaybeUninit::uninit();
        f(instance.as_mut_ptr());
        unsafe {
            Self::from_ptr(Box::into_raw(Box::new(instance.assume_init())))
        }
    }
}

impl<T: NativeDrop> AsRef<T> for Handle<T> {
    fn as_ref(&self) -> &T {
        self.native()
    }
}

impl<T: NativeDrop> AsMut<T> for Handle<T> {
    fn as_mut(&mut self) -> &mut T {
        self.native_mut()
    }
}

impl<T: NativeDrop> Drop for Handle<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.0).drop();
        }
    }
}

impl<T> PartialEq for Handle<T> where T: NativeDrop + PartialEq<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || self.native().eq((*other).native())
    }
}

pub trait NativeCopy<T> {
    fn copy_to(&self, dst: *mut T);
}

impl<T> Clone for Handle<T> where T: NativeDrop + NativeCopy<T> {
    fn clone(&self) -> Self {
        Self::new(|this| self.native().copy_to(this))
    }
}

pub trait NativeAccess<T> {
    fn native(&self) -> &T;
    fn native_mut(&mut self) -> &mut T;
    unsafe fn native_mut_force(&self) -> *mut T {
        self.native() as *const T as *mut T
    }
}

impl<T: NativeDrop> NativeAccess<T> for Handle<T> {
    fn native(&self) -> &T {
        unsafe { &*self.0 }
    }

    fn native_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0 }
    }
}

pub trait IntoPtr<T> {
    fn into_ptr(self) -> *mut T;
}

impl<T: NativeDrop> IntoPtr<T> for Handle<T> {
    fn into_ptr(self) -> *mut T {
        let instance = self.0;
        std::mem::forget(self);
        instance
    }
}

pub trait PtrOrNull<T> {
    fn ptr_or_null(&self) -> *const T;
}

impl<T: NativeDrop, N: NativeAccess<T>> PtrOrNull<T> for Option<&N> {
    fn ptr_or_null(&self) -> *const T {
        match self {
            None => std::ptr::null(),
            Some(handle) => handle.native(),
        }
    }
}

pub trait MutPtrOrNull<T> {
    fn ptr_mut_or_null(&mut self) -> *mut T;
}

impl<T: NativeDrop, N: NativeAccess<T>> MutPtrOrNull<T> for Option<&mut N> {
    fn ptr_mut_or_null(&mut self) -> *mut T {
        match self {
            None => std::ptr::null_mut(),
            Some(handle) => handle.native_mut(),
        }
    }
}
